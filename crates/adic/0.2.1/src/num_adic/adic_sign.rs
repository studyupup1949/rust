#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Real sign of adic number
pub enum AdicSign {
    /// Positive
    Pos,
    /// Negative
    Neg
}

impl AdicSign {
    /// Return 0 if positive and p-1 if negative
    pub fn mod_p(&self, p: u32) -> u32 {
        match self {
            Self::Pos => 0,
            Self::Neg => p-1,
        }
    }
}

impl std::ops::Mul for AdicSign {
    type Output = AdicSign;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (AdicSign::Pos, AdicSign::Pos) | (AdicSign::Neg, AdicSign::Neg) => AdicSign::Pos,
            (AdicSign::Pos, AdicSign::Neg) | (AdicSign::Neg, AdicSign::Pos) => AdicSign::Neg,
        }
    }
}

impl From<AdicSign> for i32 {
    fn from(other: AdicSign) -> i32 {
        match other {
            AdicSign::Pos => 1,
            AdicSign::Neg => -1,
        }
    }
}
