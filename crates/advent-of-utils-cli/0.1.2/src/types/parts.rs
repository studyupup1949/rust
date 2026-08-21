use super::Display;
use crate::error::AocError;

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum Parts {
    Part1 = 1,
    Part2 = 2,
}

impl Display for Parts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Part1 => write!(f, "Part 1"),
            Self::Part2 => write!(f, "Part 2"),
        }
    }
}

impl TryFrom<u8> for Parts {
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Parts::new(value)
    }

    type Error = AocError;
}

impl Parts {
    pub fn new(num: u8) -> Result<Self, AocError> {
        match num {
            1 => Ok(Parts::Part1),
            2 => Ok(Parts::Part2),
            num => Err(AocError::InvalidPart(num)),
        }
    }
    pub fn as_number(&self) -> u8 {
        *self as u8
    }
}
