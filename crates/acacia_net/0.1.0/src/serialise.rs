use std::{
    borrow::Cow,
    io::{Error, ErrorKind, Write},
};

use byteorder::{BE, WriteBytesExt};

use crate::types::{VarInt, VarLong, Position};

pub(crate) trait Serialise {
    fn serialise<W: Write>(&self, writer: W) -> Result<(), Error>;
}

impl Serialise for nbt::Value {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        nbt::to_writer(&mut writer, self, None).map_err(Into::into)
    }
}

impl Serialise for VarInt {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        let mut val = self.0 as u32;   // include the sign bit in shifts
        loop {
            let mut byte = (val & 0b0111_1111) as u8;
            val >>= 7;
            if val != 0 {
                byte |= 0b1000_0000;
            }
            writer.write_u8(byte)?;
            if val == 0 { break; }
        }
        Ok(())
    }
}

impl Serialise for VarLong {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        let mut val = self.0 as u64;   // include the sign bit in shifts
        loop {
            let mut byte = (val & 0b0111_1111) as u8;
            val >>= 7;
            if val != 0 {
                byte |= 0b1000_0000;
            }
            writer.write_u8(byte)?;
            if val == 0 { break; }
        }
        Ok(())
    }
}

impl Serialise for Position {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        let Position(x, y, z) = self;
        (((x << 38) | ((y & 0xFFF) << 26) | (z & 0x3FF_FFFF)) as u64).serialise(&mut writer)?;
        Ok(())
    }
}

impl Serialise for Cow<'_, str> {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        if self.len() > 32767 * 4 {
            return Err(Error::from(ErrorKind::InvalidData));
        }
        VarInt(self.len() as i32).serialise(&mut writer)?;
        writer.write_all(self.as_bytes())?;
        Ok(())
    }
}

impl<T: Serialise> Serialise for Cow<'_, [T]>
        where [T]: ToOwned {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        for v in &**self {
            v.serialise(&mut writer)?;
        }
        Ok(())
    }
}

impl<T: Serialise> Serialise for Option<T> {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        match self {
            Some(v) => v.serialise(&mut writer),
            None => Ok(()),
        }
    }
}

impl Serialise for bool {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_u8(*self as u8)?;
        Ok(())
    }
}

impl Serialise for u8 {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_u8(*self)?;
        Ok(())
    }
}

impl Serialise for i8 {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_i8(*self)?;
        Ok(())
    }
}

impl Serialise for u16 {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_u16::<BE>(*self)?;
        Ok(())
    }
}

impl Serialise for i16 {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_i16::<BE>(*self)?;
        Ok(())
    }
}

impl Serialise for u32 {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_u32::<BE>(*self)?;
        Ok(())
    }
}

impl Serialise for i32 {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_i32::<BE>(*self)?;
        Ok(())
    }
}

impl Serialise for u64 {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_u64::<BE>(*self)?;
        Ok(())
    }
}

impl Serialise for i64 {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_i64::<BE>(*self)?;
        Ok(())
    }
}

impl Serialise for u128 {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_u128::<BE>(*self)?;
        Ok(())
    }
}

impl Serialise for i128 {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_i128::<BE>(*self)?;
        Ok(())
    }
}

impl Serialise for f32 {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_f32::<BE>(*self)?;
        Ok(())
    }
}

impl Serialise for f64 {
    fn serialise<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_f64::<BE>(*self)?;
        Ok(())
    }
}
