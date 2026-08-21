#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::{error, fmt, ops::Deref};

/// An RGB color.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color {
    /// The red component of the color.
    pub r: u8,
    /// The blue component of the color.
    pub g: u8,
    /// The green component of the color.
    pub b: u8,
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}

impl PartialEq<(u8, u8, u8)> for Color {
    fn eq(&self, &rhs: &(u8, u8, u8)) -> bool {
        (self.r, self.g, self.b) == rhs
    }
}

/// A palette of colors created from a vector or from parsing the bytes of a `.act` file.
#[derive(Debug, PartialEq)]
pub struct Palette {
    colors: Vec<Color>,
}

impl Palette {
    /// Creates a new palette from preset colors.
    pub fn new(colors: Vec<Color>) -> Self {
        Self { colors }
    }

    /// Parses a palette from the bytes of an act file.
    ///
    /// The expected format is a series of RGB triplets, followed by padding until the third-to-last byte,
    /// which is the number of triplets. The final two bytes are also padding.
    pub fn from_act(bytes: &[u8]) -> Result<Self, ParseError> {
        const BYTES_PER_COLOR: usize = 3;
        const COUNT_OFFSET_FROM_END: usize = 3;

        let count = *bytes
            .get(bytes.len() - COUNT_OFFSET_FROM_END)
            .ok_or(ParseError)?;

        let colors = bytes
            .chunks_exact(BYTES_PER_COLOR)
            .take(count as _)
            .map(|chunk| {
                Ok(Color {
                    r: *chunk.first().ok_or(ParseError)?,
                    g: *chunk.get(1).ok_or(ParseError)?,
                    b: *chunk.get(2).ok_or(ParseError)?,
                })
            })
            .collect::<Result<Vec<_>, ParseError>>()?;

        if colors.len() as u8 != count {
            return Err(ParseError);
        }

        Ok(Self { colors })
    }
}

impl fmt::Display for Palette {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "JASC-PAL\n0100")?;
        writeln!(f, "{}", self.colors.len())?;

        for Color { r, g, b } in &self.colors {
            writeln!(f, "{r} {g} {b}")?;
        }

        Ok(())
    }
}

impl Deref for Palette {
    type Target = [Color];

    fn deref(&self) -> &Self::Target {
        &self.colors
    }
}

/// An error encountered when the bytes of a `.act` file cannot be parsed.
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unprocessable bytes")
    }
}

impl error::Error for ParseError {}
