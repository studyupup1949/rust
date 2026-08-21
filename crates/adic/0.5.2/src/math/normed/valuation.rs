use std::{
    cmp::Ordering,
    fmt::{Debug, Display, Formatter, Result as fmtResult},
    hash::Hash,
    ops,
};
use num::Zero;
use super::ValuationRing;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Represents valuations of ultrametric numbers
///
/// In the digital representation,
///  this is the number of digits between decimal point and first nonzero digit,
///  possibly negative.
/// This struct can also be used to represent certainty.
/// E.g.
///
/// ```
/// # use adic::{normed::{UltraNormed, Valuation}, traits::{AdicPrimitive, HasApproximateDigits}, QAdic, UAdic, ZAdic};
/// let z = ZAdic::new_approx(5, 6, vec![0, 0, 3, 1, 2, 4]);
/// assert_eq!(Valuation::Finite(2), z.valuation());
/// assert_eq!(Valuation::Finite(6), z.certainty());
/// assert_eq!(Valuation::Finite(0), ZAdic::empty(5).valuation());
/// assert_eq!(Valuation::Finite(0), ZAdic::empty(5).certainty());
/// assert_eq!(Valuation::PosInf, UAdic::new(5, vec![]).valuation());
/// assert_eq!(Valuation::PosInf, UAdic::new(5, vec![]).certainty());
/// assert_eq!(Valuation::Finite(0), QAdic::new(UAdic::new(5, vec![1, 2]), 0).valuation());
/// assert_eq!(Valuation::Finite(2), QAdic::new(UAdic::new(5, vec![1, 2]), 2).valuation());
/// assert_eq!(Valuation::Finite(-2), QAdic::new(UAdic::new(5, vec![1, 2]), -2).valuation());
/// assert_eq!(Valuation::Finite(-1), QAdic::new(UAdic::new(5, vec![0, 2]), -2).valuation());
/// assert_eq!(Valuation::PosInf, QAdic::new(UAdic::zero(5), -2).valuation());
/// ```
pub enum Valuation<F>
where F: ValuationRing {
    /// Positive infinity, e.g. for the size of zero
    PosInf,
    /// Finite valuation
    Finite(F),
}

impl<F> Valuation<F>
where F: ValuationRing {

    /// Return finite value if `Finite` and `None` if `PosInf`
    pub fn finite(&self) -> Option<F> {
        if let Self::Finite(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    /// Is the valuation finite
    pub fn is_finite(&self) -> bool {
        matches!(self, Self::Finite(_))
    }

    /// Convert from one valuation to another
    ///
    /// # Errors
    /// Error if the conversion attempt fails
    pub fn convert<G>(self) -> Result<Valuation<G>, G::Error>
    where G: ValuationRing + TryFrom<F> {
        match self {
            Valuation::PosInf => Ok(Valuation::PosInf),
            Valuation::Finite(fval) => Ok(Valuation::Finite(fval.try_into()?)),
        }
    }

}


impl<F> From<F> for Valuation<F>
where F: ValuationRing {
    fn from(value: F) -> Self {
        Valuation::Finite(value)
    }
}


impl<F> Zero for Valuation<F>
where F: ValuationRing {
    fn zero() -> Self {
        Self::Finite(F::zero())
    }
    fn is_zero(&self) -> bool {
        *self == Self::Finite(F::zero())
    }
}


impl<F> Ord for Valuation<F>
where F: ValuationRing {
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

impl<F> PartialOrd for Valuation<F>
where F: ValuationRing {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


impl<F> Display for Valuation<F>
where F: ValuationRing + Display {
    fn fmt(&self, f: &mut Formatter) -> fmtResult {
        match self {
            Self::PosInf => write!(f, "inf"),
            Self::Finite(v) => write!(f, "{v}")
        }
    }
}


impl<F> Default for Valuation<F>
where F: ValuationRing + Default {
    fn default() -> Self {
        Self::Finite(F::default())
    }
}


impl<F> ops::Add for Valuation<F>
where F: ValuationRing {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        if let (Self::Finite(sv), Self::Finite(rv)) = (self, rhs) {
            Self::Finite(sv + rv)
        } else {
            Self::PosInf
        }
    }
}

impl<F> ops::Mul for Valuation<F>
where F: ValuationRing {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        if let (Self::Finite(sv), Self::Finite(rv)) = (self, rhs) {
            Self::Finite(sv * rv)
        } else {
            Self::PosInf
        }
    }
}

impl<F> ops::Neg for Valuation<F>
where F: ValuationRing + ops::Neg<Output=F> {
    type Output = Option<Self>;
    fn neg(self) -> Self::Output {
        if let Self::Finite(v) = self {
            Some(Self::Finite(-v))
        } else {
            None
        }
    }
}

impl<F> ops::Sub for Valuation<F>
where F: ValuationRing + ops::Sub<Output=F> {
    type Output = Option<Self>;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Finite(sv), Self::Finite(rv)) => Some(Self::Finite(sv - rv)),
            (Self::PosInf, Self::Finite(_)) => Some(Self::PosInf),
            _ => None,
        }
    }
}

impl<F> ops::Div for Valuation<F>
where F: ValuationRing + ops::Div<Output=F> {
    type Output = Option<Self>;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Finite(sv), Self::Finite(rv)) => Some(Self::Finite(sv / rv)),
            (Self::PosInf, Self::Finite(rv)) if Self::Finite(rv) > Self::zero() => Some(Self::PosInf),
            _ => None,
        }
    }
}
