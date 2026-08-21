use std::{cmp::Ordering, fmt};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Represents valuations of adic numbers
///
/// In the digital representation,
///  this is the number of digits between decimal point and first nonzero digit.
/// This struct can also be used to represent certainty.
/// E.g.
/// ```
/// # use adic::{zadic_approx, zadic_exact, AdicInteger, ZAdic, ZAdicValuation};
/// let z = zadic_approx!(5, 6, [0, 0, 3, 1, 2, 4]);
/// assert_eq!(ZAdicValuation::Finite(2), z.valuation());
/// assert_eq!(ZAdicValuation::Finite(6), z.certainty());
/// assert_eq!(ZAdicValuation::Finite(0), zadic_approx!(5, 0, []).valuation());
/// assert_eq!(ZAdicValuation::Finite(0), zadic_approx!(5, 0, []).certainty());
/// assert_eq!(ZAdicValuation::PosInf, zadic_exact!(5, []).valuation());
/// assert_eq!(ZAdicValuation::PosInf, zadic_exact!(5, []).certainty());
/// ```
pub enum ZAdicValuation {
    /// Positive infinity, e.g. for zero
    PosInf,
    /// Finite integer
    Finite(u32),
}


use ZAdicValuation::{PosInf, Finite};


impl Ord for ZAdicValuation {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (PosInf, PosInf) => Ordering::Equal,
            (PosInf, Finite(_)) => Ordering::Greater,
            (Finite(_), PosInf) => Ordering::Less,
            (Finite(sv), Finite(ov)) => {
                sv.cmp(ov)
            },
        }
    }
}

impl PartialOrd for ZAdicValuation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::ops::Add for ZAdicValuation {
    type Output = ZAdicValuation;
    fn add(self, rhs: Self) -> Self::Output {
        if let (Finite(sv), Finite(rv)) = (self, rhs) {
            Finite(sv + rv)
        } else {
            PosInf
        }
    }
}


impl fmt::Display for ZAdicValuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PosInf => write!(f, "inf"),
            Self::Finite(v) => write!(f, "{v}")
        }
    }
}


impl Default for ZAdicValuation {
    fn default() -> Self {
        ZAdicValuation::Finite(0)
    }
}
