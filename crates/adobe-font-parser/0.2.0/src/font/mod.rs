use crate::FontError;
use std::collections::HashMap;
use tiny_skia::{Path, Transform};

mod cff;
mod type1;

#[derive(Clone)]
pub struct Glyph {
    /// transform by font_matrix to scale it to 1em
    pub path: Option<Path>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GlyphId(pub u32);

#[derive(Clone)]
pub struct Font {
    pub font_matrix: Transform,
    pub encoding: Vec<String>,
    pub glyph_names: HashMap<String, GlyphId>,
    pub glyphs: Vec<Glyph>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontType {
    Type1,
    Cff,
    Unknown([u8; 4]),
}

pub fn font_type(data: &[u8]) -> FontType {
    let Some(magic) = data.get(0..4) else {
        return FontType::Unknown([0; 4]);
    };
    match magic {
        b"%!PS" | &[37, 33, _, _] => FontType::Type1,
        &[1, _, _, _] => FontType::Cff,
        magic => FontType::Unknown(magic.try_into().unwrap()),
    }
}

pub fn parse(data: &[u8]) -> Result<Font, FontError> {
    let ty = font_type(data);
    Ok(match ty {
        FontType::Type1 => type1::parse(data)?,
        FontType::Cff => cff::parse(data, 0)?,
        FontType::Unknown(magic) => return Err(FontError::UnknownMagic(magic)),
    })
}
