mod cis;
mod kind;
#[cfg(test)]
mod tests;

use core::{fmt, str};

#[cfg(feature = "serde")]
use serde_with::{DeserializeFromStr, SerializeDisplay};

pub use kind::*;

#[cfg_attr(feature = "serde", derive(DeserializeFromStr, SerializeDisplay))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Abler {
    #[default]
    Disable, 
    Enable,
}

include!(concat!(env!("OUT_DIR"), "/abler_set.rs"));

/// Type representing some toggle/switch(er)/flag/etc with various conversions suport.
impl Abler {
    pub const DEFAULT_STRICTNESS: bool = true;

    pub fn new<T: InAbler>(src: T) -> Result<Self, T::Error> {
        src.in_abler::<{Self::DEFAULT_STRICTNESS}>()
    }

    pub fn from_u32_relaxed(src: u32) -> Self {
        match src {
            0 => Self::Disable,
            _ => Self::Enable,
        }
    }

    pub fn try_from_u32_strict(src: u32) -> Result<Self, errors::FromU32> {
        Ok(match src {
            0 => Self::Disable,
            1 => Self::Enable,
            _ => return Err(errors::FromU32(src)),
        })
    }

    pub fn try_from_char(src: char, strict: bool) -> Result<Self, errors::FromChar> {
        if !src.is_ascii_digit() {
            return Err(errors::FromChar::from(src));
        }

        let numeric = u32::from(src) - u32::from('0');

        if strict {
            Self::try_from_u32_strict(numeric).map_err(errors::FromChar::from)
        } else {
            Ok(Self::from_u32_relaxed(numeric))
        }
    }

    pub fn try_from_str(src: &str, strict: bool) -> Result<Self, errors::FromStr> {
        if let Some(flag) = Self::ALIASES.get(&cis::Cis(src)) {
            Ok(flag.into())
        } else {
            let numeric: u32 = src.parse().map_err(errors::FromStr::map_parse_err(src))?;
            if strict {
                Self::try_from_u32_strict(numeric).map_err(errors::FromStr::U32)
            } else {
                Ok(Self::from_u32_relaxed(numeric))
            }
        }
    }

    pub fn display(self, kind: Kind) -> Display {
        Display { value: self, kind }
    }
}

impl fmt::Display for Abler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.into())
    }
}

impl From<bool> for Abler {
    fn from(value: bool) -> Self {
        value.then_some(Self::Enable).unwrap_or(Self::Disable)
    }
}

impl From<&bool> for Abler {
    fn from(value: &bool) -> Self {
        Self::from(*value)
    }
}

impl From<Abler> for u8 {
    fn from(value: Abler) -> Self {
        match value {
            Abler::Disable => 0,
            Abler::Enable => 1,
        }
    }
}

impl From<&Abler> for u8 {
    fn from(value: &Abler) -> Self {
        Self::from(*value)
    }
}

impl From<Abler> for i8 {
    fn from(value: Abler) -> Self {
        match value {
            Abler::Disable => 0,
            Abler::Enable => 1,
        }
    }
}

impl From<&Abler> for i8 {
    fn from(value: &Abler) -> Self {
        Self::from(*value)
    }
}

impl From<Abler> for &'static str {
    fn from(value: Abler) -> Self {
        value.display(Kind::default()).into()
    }
}

impl From<&Abler> for &'static str {
    fn from(value: &Abler) -> Self {
        Self::from(*value)
    }
}

impl str::FromStr for Abler {
    type Err = errors::FromStr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.in_abler::<{Self::DEFAULT_STRICTNESS}>()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Display {
    value: Abler,
    kind: Kind,
}

impl From<Display> for &'static str {
    fn from(value: Display) -> Self {
        Self::from(&value)
    }
}

impl From<&Display> for &'static str {
    fn from(value: &Display) -> Self {
        value.kind.names()[u8::from(value.value) as usize]
    }
}

impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.into())
    }
}

pub trait InAbler {
    type Error;

    fn in_abler<const STRICT: bool>(self) -> Result<Abler, Self::Error>;
}

impl InAbler for u32 {
    type Error = errors::FromU32;

    fn in_abler<const STRICT: bool>(self) -> Result<Abler, Self::Error> {
        if STRICT {
            Abler::try_from_u32_strict(self)
        } else {
            Ok(Abler::from_u32_relaxed(self))
        }
    }
}

impl InAbler for char {
    type Error = errors::FromChar;

    fn in_abler<const STRICT: bool>(self) -> Result<Abler, Self::Error> {
        Abler::try_from_char(self, STRICT)
    }
}

impl InAbler for &str {
    type Error = errors::FromStr;

    fn in_abler<const STRICT: bool>(self) -> Result<Abler, Self::Error> {
        Abler::try_from_str(self, STRICT)
    }
}

pub mod errors {
    use core::num;

    #[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
    #[error("failed converting strict Abler value from '{0}'")]
    pub struct FromU32(pub(super) u32);

    #[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
    pub enum FromChar {
        #[error("failed parsing Abler value from '{0}'")]
        Char(char),
        #[error(transparent)]
        U32(#[from] FromU32),
    }

    impl From<char> for FromChar {
        fn from(value: char) -> Self {
            Self::Char(value)
        }
    }

    #[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
    pub enum FromStr {
        #[error("failed parsing Abler value from '{0}'")]
        Text(String),
        #[error("failed parsing Abler value from '{value}': {source}")]
        Parse {
            source: num::ParseIntError,
            value: String
        },
        #[error(transparent)]
        U32(#[from] FromU32),
    }

    impl FromStr {
        pub(super) fn map_parse_err<I: Into<String>>(value: I) -> impl FnOnce(num::ParseIntError) -> Self {
            move |source: num::ParseIntError| -> Self { Self::Parse { source, value: value.into() }}
        }
    }
}