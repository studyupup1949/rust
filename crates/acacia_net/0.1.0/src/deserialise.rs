use std::{
    borrow::Cow,
    io::{Error as IoError, Read},
};

use byteorder::{ReadBytesExt, BE};

use super::types::{VarInt, VarLong, Position};

#[derive(Debug, Fail)]
pub enum ParseError {
    #[fail(display = "The packet was not long enough: {} more bytes needed", _0)]
    MoreNeeded(usize),
    #[fail(display = "Invalid Packet ID: {}", _0)]
    InvalidPacketId(i32),
    #[fail(display = "{} is not a discriminant of enum {}", _1, _0)]
    InvalidEnumDiscriminant(&'static str, i32),
    #[fail(display = "The packet was not long enough")]
    UnexpectedEof,
    #[fail(display = "VarInt too long")]
    VarIntTooLong,
    #[fail(display = "VarLong too long")]
    VarLongTooLong,
    #[fail(display = "String too long")]
    StringTooLong,
    #[fail(display = "String had negative length")]
    StringLengthNegative,
    #[fail(display = "String wasn't UTF-8")]
    StringNotUtf8,
    #[fail(display = "Bool wasn't 0 or 1")]
    InvalidBool,
    #[fail(display = "NBT parsing error: {}", _0)]
    NbtError(#[cause] nbt::Error),
    #[fail(display = "IO error: {}", _0)]
    IoError(#[cause] IoError),
}

impl From<IoError> for ParseError {
    fn from(io_error: IoError) -> Self {
        ParseError::IoError(io_error)
    }
}

impl From<nbt::Error> for ParseError {
    fn from(nbt_error: nbt::Error) -> Self {
        match nbt_error {
            nbt::Error::IoError(e) => ParseError::IoError(e),
            e => ParseError::NbtError(e),
        }
    }
}

pub(crate) trait Deserialise<Meta>: Sized {
    fn deserialise<R: Read>(reader: R, meta: Meta) -> Result<Self, ParseError>;
}

impl Deserialise<()> for nbt::Value {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let value = nbt::from_reader(&mut reader)?;
        Ok(value)
    }
}

impl Deserialise<()> for VarInt {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let mut ret = 0i32;
        let mut i = 0;
        let mut byte = [0];
        while let Ok(1) = reader.read(&mut byte) {
            let byte = byte[0];
            ret |= ((byte & 0b0111_1111) as i32) << (7 * i);
            i += 1;
            if byte & 0b1000_0000 == 0 {
                return Ok(VarInt(ret));
            }
            if i > 5 {
                return Err(ParseError::VarIntTooLong);
            }
        }
        Err(ParseError::UnexpectedEof)
    }
}

impl Deserialise<()> for VarLong {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let mut ret = 0i64;
        let mut i = 0;
        let mut byte = [0];
        while let Ok(1) = reader.read(&mut byte) {
            let byte = byte[0];
            ret |= ((byte & 0b0111_1111) as i64) << (7 * i);
            i += 1;
            if byte & 0b1000_0000 == 0 {
                return Ok(VarLong(ret));
            }
            if i > 10 {
                return Err(ParseError::VarLongTooLong);
            }
        }
        Err(ParseError::UnexpectedEof)
    }
}

impl Deserialise<()> for Position {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let v = u64::deserialise(&mut reader, ())?;
        let mut x = (v >> 38) as i64;
        let mut y = ((v >> 26) & 0xFFF) as i64;
        let mut z = (v & 0x3FF_FFFF) as i64;
        if x >= 2^25 { x -= 1 << 26 }
        if y >= 2^11 { y -= 1 << 12 }
        if z >= 2^25 { z -= 1 << 26 }
        Ok(Position(x, y, z))
    }
}

impl Deserialise<()> for Cow<'_, str> {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let VarInt(len) = VarInt::deserialise(&mut reader, ())?;
        if len < 0 {
            return Err(ParseError::StringLengthNegative);
        } else if len > 32767 {
            return Err(ParseError::StringTooLong);
        }

        let mut buf = vec![0; len as usize];
        reader.read_exact(&mut buf)?;
        let ret = String::from_utf8(buf).map_err(|_| ParseError::StringNotUtf8)?.into();
        Ok(ret)
    }
}

impl<T: Deserialise<()> + Clone> Deserialise<usize> for Cow<'_, [T]> {
    fn deserialise<R: Read>(mut reader: R, len: usize) -> Result<Self, ParseError> {
        let mut ret = Vec::<T>::with_capacity(len);
        for _ in 0..len {
            ret.push(T::deserialise(&mut reader, ())?);
        }
        Ok(ret.into())
    }
}

impl<T: Deserialise<()> + Clone> Deserialise<()> for Cow<'_, [T]> {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let mut ret = Vec::<T>::new();
        while let Ok(v) = T::deserialise(&mut reader, ()) {
            ret.push(v);
        }
        Ok(ret.into())
    }
}

impl<M: Copy, T: Deserialise<M> + Clone> Deserialise<(usize, M)> for Cow<'_, [T]> {
    fn deserialise<R: Read>(mut reader: R, (len, inner_meta): (usize, M)) -> Result<Self, ParseError> {
        let mut ret = Vec::<T>::with_capacity(len);
        for _ in 0..len {
            ret.push(T::deserialise(&mut reader, inner_meta)?);
        }
        Ok(ret.into())
    }
}

pub struct NoLengthHint;

impl<M: Copy, T: Deserialise<M> + Clone> Deserialise<(NoLengthHint, M)> for Cow<'_, [T]> {
    fn deserialise<R: Read>(mut reader: R, (_, inner_meta): (NoLengthHint, M)) -> Result<Self, ParseError> {
        let mut ret = Vec::<T>::new();
        while let Ok(v) = T::deserialise(&mut reader, inner_meta) {
            ret.push(v);
        }
        Ok(ret.into())
    }
}

impl<M, T: Deserialise<M>> Deserialise<(bool, M)> for Option<T> {
    fn deserialise<R: Read>(mut reader: R, (meta, inner_meta): (bool, M)) -> Result<Self, ParseError> {
        if meta {
            Ok(Some(T::deserialise(&mut reader, inner_meta)?))
        } else {
            Ok(None)
        }
    }
}

impl<T: Deserialise<()>> Deserialise<bool> for Option<T> {
    fn deserialise<R: Read>(mut reader: R, meta: bool) -> Result<Self, ParseError> {
        if meta {
            Ok(Some(T::deserialise(&mut reader, ())?))
        } else {
            Ok(None)
        }
    }
}

impl Deserialise<()> for bool {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let val = reader.read_u8()?;
        let ret = if val == 1 {
            true
        } else if val == 0 {
            false
        } else {
            return Err(ParseError::InvalidBool);
        };
        Ok(ret)
    }
}

impl Deserialise<()> for u8 {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let ret = reader.read_u8()?;
        Ok(ret)
    }
}

impl Deserialise<()> for i8 {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let ret = reader.read_i8()?;
        Ok(ret)
    }
}

impl Deserialise<()> for u16 {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let ret = reader.read_u16::<BE>()?;
        Ok(ret)
    }
}

impl Deserialise<()> for i16 {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let ret = reader.read_i16::<BE>()?;
        Ok(ret)
    }
}

impl Deserialise<()> for u32 {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let ret = reader.read_u32::<BE>()?;
        Ok(ret)
    }
}

impl Deserialise<()> for i32 {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let ret = reader.read_i32::<BE>()?;
        Ok(ret)
    }
}

impl Deserialise<()> for u64 {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let ret = reader.read_u64::<BE>()?;
        Ok(ret)
    }
}

impl Deserialise<()> for i64 {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let ret = reader.read_i64::<BE>()?;
        Ok(ret)
    }
}

impl Deserialise<()> for u128 {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let ret = reader.read_u128::<BE>()?;
        Ok(ret)
    }
}

impl Deserialise<()> for i128 {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let ret = reader.read_i128::<BE>()?;
        Ok(ret)
    }
}

impl Deserialise<()> for f32 {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let ret = reader.read_f32::<BE>()?;
        Ok(ret)
    }
}

impl Deserialise<()> for f64 {
    fn deserialise<R: Read>(mut reader: R, _meta: ()) -> Result<Self, ParseError> {
        let ret = reader.read_f64::<BE>()?;
        Ok(ret)
    }
}
