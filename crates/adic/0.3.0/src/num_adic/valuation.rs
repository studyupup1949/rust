use std::{cmp::Ordering, fmt};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Represents valuations of adic integers
///
/// In the digital representation,
///  this is the number of digits between decimal point and first nonzero digit.
/// This struct can also be used to represent certainty.
/// E.g.
/// ```
/// # use adic::{zadic_approx, zadic_exact_pos, AdicInteger, ZAdic, ZAdicValuation};
/// let z = zadic_approx!(5, 6, [0, 0, 3, 1, 2, 4]);
/// assert_eq!(ZAdicValuation::Finite(2), z.valuation());
/// assert_eq!(ZAdicValuation::Finite(6), z.certainty());
/// assert_eq!(ZAdicValuation::Finite(0), zadic_approx!(5, 0, []).valuation());
/// assert_eq!(ZAdicValuation::Finite(0), zadic_approx!(5, 0, []).certainty());
/// assert_eq!(ZAdicValuation::PosInf, zadic_exact_pos!(5, []).valuation());
/// assert_eq!(ZAdicValuation::PosInf, zadic_exact_pos!(5, []).certainty());
/// ```
pub enum ZAdicValuation {
    /// Positive infinity, e.g. for zero
    PosInf,
    /// Finite integer
    Finite(usize),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Represents valuations of adic numbers
///
/// In the digital representation,
///  this is the number of digits between decimal point and first nonzero digit,
///  possibly negative.
///
/// ```
/// # use adic::{qadic, uadic, QAdicValuation};
/// let q = qadic!(uadic!(5, [1, 2]), 0);
/// assert_eq!(QAdicValuation::Finite(0), q.valuation());
/// let q = qadic!(uadic!(5, [1, 2]), 2);
/// assert_eq!(QAdicValuation::Finite(2), q.valuation());
/// let q = qadic!(uadic!(5, [1, 2]), -2);
/// assert_eq!(QAdicValuation::Finite(-2), q.valuation());
/// let q = qadic!(uadic!(5, [0, 2]), -2);
/// assert_eq!(QAdicValuation::Finite(-1), q.valuation());
/// let q = qadic!(uadic!(5, []), -2);
/// assert_eq!(QAdicValuation::PosInf, q.valuation());
/// ```
pub enum QAdicValuation {
    /// Positive infinity, e.g. for zero
    PosInf,
    /// Finite number
    Finite(isize),
}




impl Ord for ZAdicValuation {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::PosInf, Self::PosInf) => Ordering::Equal,
            (Self::PosInf, Self::Finite(_)) => Ordering::Greater,
            (Self::Finite(_), Self::PosInf) => Ordering::Less,
            (Self::Finite(sv), Self::Finite(ov)) => {
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
        if let (Self::Finite(sv), Self::Finite(rv)) = (self, rhs) {
            Self::Finite(sv + rv)
        } else {
            Self::PosInf
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
        Self::Finite(0)
    }
}


impl Ord for QAdicValuation {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::PosInf, Self::PosInf) => Ordering::Equal,
            (Self::PosInf, Self::Finite(_)) => Ordering::Greater,
            (Self::Finite(_), Self::PosInf) => Ordering::Less,
            (Self::Finite(sv), Self::Finite(ov)) => {
                sv.cmp(ov)
            },
        }
    }
}

impl PartialOrd for QAdicValuation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


impl std::ops::Add for QAdicValuation {
    type Output = QAdicValuation;
    fn add(self, rhs: Self) -> Self::Output {
        if let (Self::Finite(sv), Self::Finite(rv)) = (self, rhs) {
            Self::Finite(sv + rv)
        } else {
            Self::PosInf
        }
    }
}


// Note: Should we have NegInf so we don't have Option<Self> here?

impl std::ops::Neg for QAdicValuation {
    type Output = Option<Self>;
    fn neg(self) -> Self::Output {
        if let Self::Finite(v) = self {
            Some(Self::Finite(-v))
        } else {
            None
        }
    }
}


impl std::ops::Sub for QAdicValuation {
    type Output = Option<Self>;
    fn sub(self, rhs: Self) -> Self::Output {
        Some(self + (-rhs)?)
    }
}


impl fmt::Display for QAdicValuation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PosInf => write!(f, "inf"),
            Self::Finite(v) => write!(f, "{v}")
        }
    }
}


impl Default for QAdicValuation {
    fn default() -> Self {
        Self::Finite(0)
    }
}


impl From<ZAdicValuation> for QAdicValuation {
    fn from(value: ZAdicValuation) -> Self {
        match value {
            ZAdicValuation::PosInf => QAdicValuation::PosInf,
            ZAdicValuation::Finite(zval) => QAdicValuation::Finite(
                zval.try_into().expect("valuation usize -> isize conversion")
            ),
        }
    }
}
