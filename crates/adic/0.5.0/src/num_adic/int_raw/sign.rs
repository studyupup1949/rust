use crate::divisible::{Divisible, Prime};


#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Real sign of adic number.
/// Only applicable to exact rational numbers, since inexact or irrational numbers have no definable sign.
/// Private, to avoid confusion
pub (super) enum Sign {
    /// Positive
    Pos,
    /// Negative
    Neg
}

impl Sign {
    /// Return 0 if positive and p-1 if negative
    pub (super) fn mod_p(self, p: Prime) -> u32 {
        match self {
            Self::Pos => 0,
            Self::Neg => p.m1(),
        }
    }
}

impl std::ops::Mul for Sign {
    type Output = Sign;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Sign::Pos, Sign::Pos) | (Sign::Neg, Sign::Neg) => Sign::Pos,
            (Sign::Pos, Sign::Neg) | (Sign::Neg, Sign::Pos) => Sign::Neg,
        }
    }
}

impl std::iter::Product for Sign {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::Pos, |acc, x| acc * x)
    }
}


impl From<Sign> for i32 {
    fn from(other: Sign) -> i32 {
        match other {
            Sign::Pos => 1,
            Sign::Neg => -1,
        }
    }
}
