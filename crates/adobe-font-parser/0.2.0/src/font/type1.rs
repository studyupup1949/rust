use crate::charstring::{Context, State, type1};
use crate::postscript::{Decoder, RefItem, Vm};
use crate::{Font, FontError, Glyph, GlyphId};
use std::collections::HashMap;
use tiny_skia::Transform;
use tracing::debug;

pub fn parse(data: &[u8]) -> Result<Font, FontError> {
    let mut vm = Vm::new();
    vm.parse_and_exec(data)?;
    let (_font_name, font_dict) = expect!(vm.fonts().nth(0), "no font in vm");

    let private_dict = expect!(font_dict.get_dict("Private"), "no /Private dict");
    let len_iv = private_dict.get_int("lenIV").unwrap_or(4) as usize;

    let char_strings = expect!(font_dict.get_dict("CharStrings"), "no /CharStrings");

    let mut subrs = Vec::new();
    if let Some(arr) = private_dict.get_array("Subrs") {
        for item in arr.iter() {
            subrs.push(
                item.as_bytes()
                    .map(|data| Decoder::charstring().decode(data, len_iv)),
            );
        }
    }

    let context = Context {
        subr_bias: 0,
        subrs,
        global_subr_bias: 0,
        global_subrs: (),
    };

    let encoding = font_dict
        .get("Encoding")
        .expect("no /Encoding")
        .as_array()
        .expect("/Encoding is not an array");

    let mut glyphs = Vec::with_capacity(char_strings.len());
    let mut glyph_names = HashMap::new();
    let mut state = State::new();
    for (glyph_id, (glyph_name, item)) in char_strings.string_entries().enumerate() {
        let data = expect!(item.as_bytes(), "data is not bytes");

        let decoded = Decoder::charstring().decode(&data, len_iv);
        debug!("{glyph_name} decoded: {decoded:?}");

        if let Err(e) = type1::charstring(&decoded, &context, &mut state) {
            tracing::debug!("Failed to decode charstring for glyph {glyph_name}: {e:?}");
            continue;
        }

        glyph_names.insert(glyph_name.to_string(), GlyphId(glyph_id as u32));
        glyphs.push(Glyph {
            path: state.take_path(),
        });
        state.clear();
    }

    let encoding = encoding
        .iter()
        .map(|item| {
            let RefItem::Literal(name) = item else {
                return String::new();
            };
            if name == b".notdef" {
                return String::new();
            }
            std::str::from_utf8(name).unwrap_or("").to_string()
        })
        .collect::<Vec<_>>();

    let font_matrix = expect!(font_dict.get("FontMatrix"), "no FontMatrix");
    let font_matrix = expect!(font_matrix.as_array(), "FontMatrix not an array");
    require_eq!(font_matrix.len(), 6);
    let a = font_matrix.get(0).unwrap().as_f32().unwrap();
    let b = font_matrix.get(1).unwrap().as_f32().unwrap();
    let c = font_matrix.get(2).unwrap().as_f32().unwrap();
    let d = font_matrix.get(3).unwrap().as_f32().unwrap();
    let e = font_matrix.get(4).unwrap().as_f32().unwrap();
    let f = font_matrix.get(5).unwrap().as_f32().unwrap();

    Ok(Font {
        font_matrix: Transform::from_row(a, b, c, d, e, f),
        encoding,
        glyph_names,
        glyphs,
    })
}
