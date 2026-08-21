#![allow(non_snake_case)]

use super::{Font, Glyph, GlyphId};
use crate::charstring::{Context, State, Value, type1, type2};
use crate::error::{FontError, ParseResult};
use crate::tables::*;
use crate::{IResult, Input};
use nom::{
    Parser,
    bytes::complete::take,
    combinator::map,
    multi::count,
    number::complete::{be_i16, be_i32, be_u8, be_u16, be_u24, be_u32},
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::once;
use std::rc::Rc;
use tiny_skia::{Path, Transform};
use tracing::{debug, trace};

pub fn parse(data: &[u8], idx: u32) -> Result<Font, FontError> {
    let cff = t!(read_cff(data));
    let slot = t!(cff.slot(idx));
    let font = t!(slot.parse_font());
    Ok(font)
}

fn read_cff(data: &[u8]) -> Result<Cff<'_>, FontError> {
    let i = data;
    let (i, major) = be_u8(i)?;
    require_eq!(major, 1);
    let (i, minor) = be_u8(i)?;

    let (i, hdrSize) = be_u8(i)?;

    debug!("CFF Table: v. {}.{} {} bytes", major, minor, hdrSize);
    let (i, _offSize) = be_u8(i)?;
    require!(hdrSize >= 4);
    let (i, _) = take(hdrSize - 4)(i)?;

    let (i, _name_index) = index(i)?;
    let (i, dict_index) = index(i)?;
    let (i, string_index) = index(i)?;
    let (_, subroutines) = index(i)?;

    Ok(Cff {
        data,
        dict_index,
        string_index,
        subroutines,
    })
}
fn bias(num: usize) -> i32 {
    if num < 1240 {
        107
    } else if num < 33900 {
        1131
    } else {
        32768
    }
}
type Dict = HashMap<Operator, Vec<Value>>;
type Index<'a> = Vec<&'a [u8]>;

struct Cff<'a> {
    data: &'a [u8],
    dict_index: Index<'a>,
    string_index: Index<'a>,
    subroutines: Index<'a>,
}

struct CffSlot<'a> {
    cff: Cff<'a>,
    top_dict: Dict,
    private_dict: Vec<Rc<Dict>>,
    char_strings: Index<'a>,
    subrs: Vec<Rc<Index<'a>>>,
    num_glyphs: usize,
    font_matrix: Transform,
}

fn parse_font_matrix(arr: &[Value]) -> Option<Transform> {
    if arr.len() != 6 {
        return None;
    }
    let a = arr[0].to_float();
    let b = arr[1].to_float();
    let c = arr[2].to_float();
    let d = arr[3].to_float();
    let e = arr[4].to_float();
    let f = arr[5].to_float();
    Some(Transform::from_row(a, b, e, c, d, f))
}

impl<'a> Cff<'a> {
    fn slot(self, idx: u32) -> Result<CffSlot<'a>, FontError> {
        let data = self
            .dict_index
            .get(idx as usize)
            .ok_or(FontError::NoSuchSlot)?;
        let top_dict = dict(data)?;

        let offset = get!(top_dict, &Operator::CharStrings, 0).to_int()? as usize;
        let (_, char_strings) = index(slice!(self.data, offset..))?;

        // num glyphs includes glyph 0 (.notdef)
        let num_glyphs = char_strings.len() as usize;

        let mut font_matrix = top_dict
            .get(&Operator::FontMatrix)
            .and_then(|a| parse_font_matrix(&a));

        // retrieve Font Dicts if it exists.
        let mut private_dict_list = vec![];
        let mut subrs_list = vec![];
        if let Some(fdarray_entry) = top_dict.get(&Operator::FDArray) {
            let fdarray_offset = get!(fdarray_entry, 0).to_int()? as usize;
            let (_, fdarray_data_list) = index(offset!(self.data, fdarray_offset))?;
            for fdarray_data in &fdarray_data_list {
                let fdarray_dict = dict(&fdarray_data)?;
                if let Some(ma) = fdarray_dict
                    .get(&Operator::FontMatrix)
                    .and_then(|a| parse_font_matrix(&a))
                {
                    font_matrix = Some(ma);
                }
                let private_dict_entry = get!(fdarray_dict, &Operator::Private);
                let (private_dict, subrs) = self.private_dict_and_subrs(&private_dict_entry)?;
                private_dict_list.push(Rc::new(private_dict));
                subrs_list.push(Rc::new(subrs));
            }
        }
        let mut private_dict = vec![];
        let mut subrs = vec![];
        if let Some(fdselect_entry) = top_dict.get(&Operator::FDSelect) {
            let fdselect_offset = get!(fdselect_entry, 0).to_int()? as usize;
            let fd_indices = fd_select(offset!(self.data, fdselect_offset), num_glyphs)?;
            for fd_idx in fd_indices {
                private_dict.push(Rc::clone(&private_dict_list[fd_idx]));
                subrs.push(Rc::clone(&subrs_list[fd_idx]));
            }
        } else {
            let private_dict_entry = top_dict
                .get(&Operator::Private)
                .expect("no private dict entry");

            let (private_dict_global, subrs_global) =
                self.private_dict_and_subrs(&private_dict_entry)?;
            let private_dict_global = Rc::new(private_dict_global);
            let subrs_global = Rc::new(subrs_global);
            for _ in 0..num_glyphs {
                private_dict.push(Rc::clone(&private_dict_global));
                subrs.push(Rc::clone(&subrs_global));
            }
        }

        let offset = get!(top_dict, &Operator::CharStrings, 0).to_int()? as usize;
        let (_, char_strings) = index(slice!(self.data, offset..))?;

        // num glyphs includes glyph 0 (.notdef)
        let num_glyphs = char_strings.len() as usize;

        Ok(CffSlot {
            cff: self,
            top_dict,
            private_dict,
            char_strings,
            subrs,
            num_glyphs,
            font_matrix: font_matrix.unwrap_or(Transform::from_scale(0.001, 0.001)),
        })
    }

    fn private_dict_and_subrs(
        &self,
        private_dict_entry: &[Value],
    ) -> Result<(Dict, Index<'a>), FontError> {
        let private_dict_size = get!(private_dict_entry, 0).to_int()? as usize;
        let private_dict_offset = get!(private_dict_entry, 1).to_int()? as usize;
        let private_dict_data = get!(
            self.data,
            private_dict_offset..private_dict_offset + private_dict_size
        );
        let private_dict = dict(private_dict_data)?;
        //info!("private dict: {:?}", private_dict);

        let (_, subrs) = private_dict
            .get(&Operator::Subrs)
            .and_then(|arr| arr.get(0))
            .map(|item| {
                let private_subroutines_offset = item.to_int()? as usize;
                index(offset!(
                    self.data,
                    private_dict_offset + private_subroutines_offset
                ))
            })
            .transpose()?
            .unwrap_or_default();

        Ok((private_dict, subrs))
    }
}

fn fd_select(data: &[u8], num_glyphs: usize) -> Result<Vec<usize>, FontError> {
    let (data, fmt) = be_u8(data)?;
    match fmt {
        0 => count(map(be_u8, |i| i as usize), num_glyphs)
            .parse(data)
            .map(|(_, o)| o)
            .map_err(FontError::from),
        3 => {
            let (data, nranges) = map(be_u16, |i| i as usize).parse(data)?;
            let (data, range3) = count(range3_record, nranges).parse(data)?;
            let (_, sentinel) = map(be_u16, |i| i as usize).parse(data)?;
            let mut indexes = vec![0; sentinel];
            let mut stop = sentinel;
            if get!(range3, 0).0 != 0 {
                error!("the first range must have a first GID of 0")
            }
            for (first, fd) in range3.into_iter().rev() {
                get!(mut indexes, first..stop).fill(fd);
                stop = first;
            }
            Ok(indexes)
        }
        _ => error!("invalid FDSelect format: {}", fmt),
    }
}

fn range3_record(data: Input) -> IResult<(usize, usize)> {
    let (data, first) = map(be_u16, |f| f as usize).parse(data)?;
    let (data, fd) = map(be_u8, |f| f as usize).parse(data)?;
    Ok((data, (first, fd)))
}

impl<'a> CffSlot<'a> {
    fn font_matrix(&self) -> Transform {
        self.font_matrix
    }
    // -> (outline, width, lsb)
    fn outlines(
        &self,
    ) -> Result<impl Iterator<Item = Result<(Option<Path>, f32, f32), FontError>> + '_, FontError>
    {
        let n = self
            .top_dict
            .get(&Operator::CharstringType)
            .map(|v| get!(v, 0).to_int())
            .transpose()?
            .unwrap_or(2);
        let char_string_type = match n {
            1 => CharstringType::Type1,
            2 => CharstringType::Type2,
            _ => panic!("invalid charstring type"),
        };

        let global_subr_bias = match char_string_type {
            CharstringType::Type2 => bias(self.cff.subroutines.len() as usize),
            CharstringType::Type1 => 0,
        };

        // build glyphs
        let mut state = State::new();
        Ok(self.char_strings.iter().enumerate().map(move |(id, data)| {
            trace!("charstring for glyph {}", id);
            let subr_bias = match char_string_type {
                CharstringType::Type2 => bias(self.subrs[id].len()),
                CharstringType::Type1 => 0,
            };
            let context = Context {
                subr_bias,
                subrs: self.subrs[id].as_slice(),
                global_subrs: self.cff.subroutines.as_slice(),
                global_subr_bias,
            };
            match char_string_type {
                CharstringType::Type1 => {
                    t!(type1::charstring(data, &context, &mut state));
                }
                CharstringType::Type2 => {
                    t!(type2::charstring(data, &context, &mut state));
                }
            }
            let default_width = self.private_dict[id]
                .get(&Operator::DefaultWidthX)
                .map(|a| Ok(get!(a, 0).to_float()))
                .transpose()?
                .unwrap_or(0.);
            let nominal_width = self.private_dict[id]
                .get(&Operator::NominalWidthX)
                .map(|a| Ok(get!(a, 0).to_float()))
                .transpose()?
                .unwrap_or(0.);

            trace!(
                "glyph {} {:?} {:?}",
                id, state.char_width, state.delta_width
            );
            let width = match (state.char_width, state.delta_width) {
                (Some(w), None) => w,
                (None, None) => default_width,
                (None, Some(delta)) => delta + nominal_width,
                (Some(_), Some(_)) => panic!("BUG: both char_width and delta_width set"),
            };
            let lsb = state.lsb.unwrap_or_default();
            let path = state.take_path();
            state.clear();
            Ok((path, width, lsb))
        }))
    }

    fn parse_font(&self) -> Result<Font, FontError> {
        let glyph_name = |sid: SID| {
            if let Some(name) = STANDARD_STRINGS.get(sid as usize) {
                return Ok(*name);
            }
            if let Some(data) = self
                .cff
                .string_index
                .get(sid as usize - STANDARD_STRINGS.len())
            {
                return std::str::from_utf8(data)
                    .map_err(|_| FontError::Other(format!("invalid glyph name {:?}", data)));
            }
            Err(FontError::Other(format!(
                "SID out of bounds {} > {} standard strings + {} font string index entries",
                sid,
                STANDARD_STRINGS.len(),
                self.cff.string_index.len()
            )))
        };
        let charset_offset: usize = self
            .top_dict
            .get(&Operator::Charset)
            .map(|v| get!(v, 0).to_int().map(|v| v as usize))
            .transpose()?
            .unwrap_or(0);
        let sids: Cow<[SID]> = match charset_offset {
            0 => ISO_ADOBE_CHARSET[..].into(),
            1 => EXPERT_CHARSET[..].into(),
            2 => EXPERT_SUBSET_CHARSET[..].into(),
            offset => {
                let charset = charset(get!(self.cff.data, offset..), self.num_glyphs)
                    .map(|(_, o)| o)
                    .map_err(FontError::from)?;

                // index = gid - 1 -> sid
                match charset {
                    Charset::Continous(sids) => sids,
                    Charset::Ranges(ranges) => ranges
                        .into_iter()
                        .flat_map(|(sid, num)| sid..sid + num + 1)
                        .collect(),
                }
                .into()
            }
        };

        // sid -> glyph_name
        let encoding = once(0)
            .chain(sids.iter().cloned())
            .map(|sid| glyph_name(sid).unwrap_or_default().to_owned())
            .collect::<Vec<_>>();
        let glyph_names: HashMap<_, _> = once(0)
            .chain(sids.iter().cloned())
            .enumerate()
            .filter_map(|(gid, _)| Some((encoding[gid].clone(), GlyphId(gid as u32))))
            .collect();
        let glyphs: Vec<_> = self
            .outlines()?
            .map(|r| r.map(|(outline, _, _)| Glyph { path: outline }))
            .collect::<Result<_, _>>()?;

        Ok(Font {
            font_matrix: self.font_matrix(),
            encoding,
            glyph_names,
            glyphs,
        })
    }
}

fn dict(mut input: &[u8]) -> Result<HashMap<Operator, Vec<Value>>, FontError> {
    let mut map = HashMap::new();
    while input.len() > 0 {
        let mut args = Vec::new();
        while let Ok((i, arg)) = value(input) {
            args.push(arg);
            input = i;
        }
        let (i, key) = operator(input)?;
        map.insert(key, args);

        input = i;
    }

    Ok(map)
}

enum CharstringType {
    Type1,
    Type2,
}

fn index(i: &[u8]) -> ParseResult<'_, Vec<&[u8]>> {
    let (i, n) = map(be_u16, |n| n as usize).parse(i)?;
    debug!("n={}", n);
    if n != 0 {
        let (i, offSize) = t!(be_u8(i));
        let (i, offsets) = t!(count(map(|i| offset(offSize)(i), |o| o - 1), n + 1).parse(i));
        let (i, data) = t!(take(offsets[n])(i));

        let items = offsets
            .windows(2)
            .map(|w| Ok(slice!(data, w[0] as usize..w[1] as usize)))
            .collect::<Result<_, _>>()?;
        Ok((i, items))
    } else {
        Ok((i, vec![]))
    }
}

fn offset(size: u8) -> impl Fn(&[u8]) -> IResult<u32> {
    move |i| {
        Ok(match size {
            1 => map(be_u8, |n| n as u32).parse(i)?,
            2 => map(be_u16, |n| n as u32).parse(i)?,
            3 => be_u24(i)?,
            4 => be_u32(i)?,
            n => panic!("{n}"),
        })
    }
}

fn float(data: &[u8]) -> ParseResult<'_, f32> {
    let mut pos = 0;
    let mut next_nibble = || -> u8 {
        let nibble = (data[pos / 2] >> (4 * (1 - (pos & 1)) as u8)) & 0xf;
        pos += 1;
        nibble
    };

    let mut is_negaive = false;
    let mut num_digits = 0;
    let mut n: u128 = 0;
    let mut p: i32 = 0;
    let mut power_negative = false;
    let mut decimal_point = None;
    loop {
        match next_nibble() {
            d @ 0..=9 => {
                n = 10 * n + d as u128;
                num_digits += 1;
            }
            0xa => decimal_point = Some(num_digits),
            b @ 0xb | b @ 0xc => {
                // positive 10^x
                power_negative = b == 0xc;
                loop {
                    match next_nibble() {
                        d @ 0..=9 => p = 10 * p + d as i32,
                        0xf => break,
                        b => key!(b),
                    }
                }
            }
            0xd => reserved!(0xd),
            0xe => is_negaive = true,
            0xf => break,
            _ => unreachable!(),
        }
    }

    let mut value = n as f32;
    if is_negaive {
        value = -value;
    }
    let mut power = 0;
    if let Some(dp) = decimal_point {
        power += dp - num_digits;
    }
    if p != 0 {
        if power_negative {
            p *= -1;
        }
        power += p;
    }
    if power != 0 {
        value *= 10.0f32.powi(power);
    }
    Ok((&data[(pos + 1) / 2..], value))
}

fn value(input: &[u8]) -> ParseResult<'_, Value> {
    let (i, b0) = be_u8(input)?;

    Ok(match b0 {
        22..=27 => reserved!(b0),
        28 => map(be_i16, |n| n.into()).parse(i)?,
        29 => map(be_i32, |n| n.into()).parse(i)?,
        30 => float(i).map(|(i, f)| (i, f.into()))?,
        31 => reserved!(b0),
        b0 @ 32..=246 => (i, (b0 as i32 - 139).into()),
        b0 @ 247..=250 => map(be_u8, |b1| {
            ((b0 as i32 - 247) * 256 + b1 as i32 + 108).into()
        })
        .parse(i)?,
        b0 @ 251..=254 => map(be_u8, |b1| {
            (-(b0 as i32 - 251) * 256 - b1 as i32 - 108).into()
        })
        .parse(i)?,
        255 => reserved!(b0),
        b0 => reserved!(b0),
    })
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Hash)]
enum Operator {
    Version,
    Notice,
    Copyleft,
    FullName,
    FamilyName,
    Weight,
    IsFixedPitch,
    ItalicAngle,
    UnderlinePosition,
    UnderlineThickness,
    PaintType,
    CharstringType,
    FontMatrix,
    UniqueID,
    FontBBox,
    StrokeWidth,
    XUID,
    Charset,
    Encoding,
    CharStrings,
    Private,
    SyntheticBase,
    PostScript,
    BaseFontName,
    BaseFontBlend,
    ROS,
    CIDFontVersion,
    CIDFontRevision,
    CIDFontType,
    CIDCount,
    UIDBase,
    FDArray,
    FDSelect,
    FontName,

    BlueValues,
    OtherBlues,
    FamilyBlues,
    FamilyOtherBlues,
    BlueScale,
    BlueShift,
    BlueFuzz,
    StdHW,
    StdVW,
    StemSnapH,
    StemSnapV,
    ForceBold,
    LanguageGroup,
    ExpansionFactor,
    InitialRandomSeed,
    Subrs,
    DefaultWidthX,
    NominalWidthX,
    BCD,

    Reserved,
}

fn operator(input: &[u8]) -> ParseResult<'_, Operator> {
    use Operator::*;

    let (i, b) = be_u8(input)?;
    let (i, v) = match b {
        0 => (i, Version),
        1 => (i, Notice),
        2 => (i, FullName),
        3 => (i, FamilyName),
        4 => (i, Weight),
        5 => (i, FontBBox),
        6 => (i, BlueValues),
        7 => (i, OtherBlues),
        8 => (i, FamilyBlues),
        9 => (i, FamilyOtherBlues),
        10 => (i, StdHW),
        11 => (i, StdVW),
        12 => {
            let (i, b) = be_u8(i)?;
            match b {
                0 => (i, Copyleft),
                1 => (i, IsFixedPitch),
                2 => (i, ItalicAngle),
                3 => (i, UnderlinePosition),
                4 => (i, UnderlineThickness),
                5 => (i, PaintType),
                6 => (i, CharstringType),
                7 => (i, FontMatrix),
                8 => (i, StrokeWidth),
                9 => (i, BlueScale),
                10 => (i, BlueShift),
                11 => (i, BlueFuzz),
                12 => (i, StemSnapH),
                13 => (i, StemSnapV),
                14 => (i, ForceBold),
                15 | 16 => (i, Reserved),
                17 => (i, LanguageGroup),
                18 => (i, ExpansionFactor),
                19 => (i, InitialRandomSeed),
                20 => (i, SyntheticBase),
                21 => (i, PostScript),
                22 => (i, BaseFontName),
                23 => (i, BaseFontBlend),
                24..=29 => (i, Reserved),
                30 => (i, ROS),
                31 => (i, CIDFontVersion),
                32 => (i, CIDFontRevision),
                33 => (i, CIDFontType),
                34 => (i, CIDCount),
                35 => (i, UIDBase),
                36 => (i, FDArray),
                37 => (i, FDSelect),
                38 => (i, FontName),
                255 => (i, Reserved),
                n => {
                    key!(n);
                }
            }
        }
        13 => (i, UniqueID),
        14 => (i, XUID),
        15 => (i, Charset),
        16 => (i, Encoding),
        17 => (i, CharStrings),
        18 => (i, Private),
        19 => (i, Subrs),
        20 => (i, DefaultWidthX),
        21 => (i, NominalWidthX),
        22..=27 => (i, Reserved),
        30 => (i, BCD),
        31 => (i, Reserved),
        255 => (i, Reserved),
        n => {
            key!(n);
        }
    };
    Ok((i, v))
}

#[derive(Debug)]
enum Charset {
    Continous(Vec<SID>),
    Ranges(Vec<(SID, u16)>), // start, num-1
}

fn ranges<'a, F>(
    count_parser: F,
    num_glyphs: usize,
) -> impl Fn(&'a [u8]) -> IResult<'a, Vec<(SID, u16)>>
where
    F: Fn(&'a [u8]) -> IResult<'a, u16>,
{
    move |mut input: &[u8]| {
        let mut total = 0;
        let mut vec = Vec::new();
        loop {
            let (i, sid) = be_u16(input)?;
            let (i, n) = count_parser(i)?;
            vec.push((sid, n));

            total += n as usize + 1;
            input = i;

            if total >= num_glyphs - 1 {
                break;
            }
        }
        Ok((input, vec))
    }
}
fn charset(i: &[u8], num_glyphs: usize) -> IResult<'_, Charset> {
    let (i, format) = be_u8(i)?;

    match format {
        0 => map(count(be_u16, num_glyphs as usize - 1), |a| {
            Charset::Continous(a)
        })
        .parse(i),
        1 => map(
            ranges(|i| map(be_u8, |n| n as u16).parse(i), num_glyphs),
            |r| Charset::Ranges(r),
        )
        .parse(i),
        2 => map(ranges(be_u16, num_glyphs), |r| Charset::Ranges(r)).parse(i),
        _ => panic!("invalid charset format"),
    }
}
