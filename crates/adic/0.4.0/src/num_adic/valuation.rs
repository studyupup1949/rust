use std::{
    cmp::Ordering,
    fmt,
    hash::Hash,
    ops,
};
use num::{rational::Ratio, Zero};
use crate::{AdicError, AdicResult};


/// A ring that can represent a finite valuation
pub trait AdicValuationRing: Clone + Copy + PartialEq + Eq + Hash + Ord
    + Zero
    + ops::Add<Output = Self> + ops::Mul<Output = Self>
    + ops::Sub<Output = Self> + ops::Div<Output = Self> {
    /// Convert `usize` to `AdicValuationRing`
    ///
    /// # Errors
    /// Error if val conversion fails, e.g. usize -> isize
    fn try_from_usize(val: usize) -> AdicResult<Self>;
    /// Convert `AdicValuationRing` to `usize`
    ///
    /// # Errors
    /// Error if val conversion fails, e.g. isize -> usize
    fn try_into_usize(self) -> AdicResult<usize>;
}
impl AdicValuationRing for usize {
    fn try_from_usize(val: usize) -> AdicResult<Self> {
        Ok(val)
    }
    fn try_into_usize(self) -> AdicResult<usize> {
        Ok(self)
    }
}
impl AdicValuationRing for isize {
    fn try_from_usize(val: usize) -> AdicResult<Self> {
        Ok(val.try_into()?)
    }
    fn try_into_usize(self) -> AdicResult<usize> {
        Ok(self.try_into()?)
    }
}
impl AdicValuationRing for Ratio<usize> {
    fn try_from_usize(val: usize) -> AdicResult<Self> {
        Ok(Ratio::from_integer(val))
    }
    fn try_into_usize(self) -> AdicResult<usize> {
        if self.is_integer() {
            Ok(self.to_integer())
        } else {
            Err(AdicError::BadConversion)
        }
    }
}
impl AdicValuationRing for Ratio<isize> {
    fn try_from_usize(val: usize) -> AdicResult<Self> {
        Ok(Ratio::from_integer(val.try_into()?))
    }
    fn try_into_usize(self) -> AdicResult<usize> {
        if self.is_integer() {
            Ok(self.to_integer().try_into()?)
        } else {
            Err(AdicError::BadConversion)
        }
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Represents valuations of adic numbers
///
/// In the digital representation,
///  this is the number of digits between decimal point and first nonzero digit,
///  possibly negative.
/// This struct can also be used to represent certainty.
/// E.g.
///
/// ```
/// # use adic::{qadic, uadic, zadic_approx, zadic_exact_pos, AdicApproximate, AdicSized, AdicValuation, ZAdic};
/// let z = zadic_approx!(5, 6, [0, 0, 3, 1, 2, 4]);
/// assert_eq!(AdicValuation::Finite(2), z.valuation());
/// assert_eq!(AdicValuation::Finite(6), z.certainty());
/// assert_eq!(AdicValuation::Finite(0), zadic_approx!(5, 0, []).valuation());
/// assert_eq!(AdicValuation::Finite(0), zadic_approx!(5, 0, []).certainty());
/// assert_eq!(AdicValuation::PosInf, zadic_exact_pos!(5, []).valuation());
/// assert_eq!(AdicValuation::PosInf, zadic_exact_pos!(5, []).certainty());
/// assert_eq!(AdicValuation::Finite(0), qadic!(uadic!(5, [1, 2]), 0).valuation());
/// assert_eq!(AdicValuation::Finite(2), qadic!(uadic!(5, [1, 2]), 2).valuation());
/// assert_eq!(AdicValuation::Finite(-2), qadic!(uadic!(5, [1, 2]), -2).valuation());
/// assert_eq!(AdicValuation::Finite(-1), qadic!(uadic!(5, [0, 2]), -2).valuation());
/// assert_eq!(AdicValuation::PosInf, qadic!(uadic!(5, []), -2).valuation());
/// ```
pub enum AdicValuation<F>
where F: AdicValuationRing {
    /// Positive infinity, e.g. for the size of zero
    PosInf,
    /// Finite valuation
    Finite(F),
}

impl<F> AdicValuation<F>
where F: AdicValuationRing {

    /// Return finite value if `Finite` and `None` if `PosInf`
    pub fn finite(&self) -> Option<F> {
        if let Self::Finite(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    /// Convert from one valuation to another
    ///
    /// # Errors
    /// Error if the conversion attempt fails
    pub fn convert<G, E>(self) -> AdicResult<AdicValuation<G>>
    where G: AdicValuationRing + TryFrom<F, Error=E>, AdicError: From<E> {
        match self {
            AdicValuation::PosInf => Ok(AdicValuation::PosInf),
            AdicValuation::Finite(fval) => Ok(AdicValuation::Finite(fval.try_into()?)),
        }
    }

}


impl<F> From<F> for AdicValuation<F>
where F: AdicValuationRing {
    fn from(value: F) -> Self {
        AdicValuation::Finite(value)
    }
}


impl<F> Zero for AdicValuation<F>
where F: AdicValuationRing {
    fn zero() -> Self {
        Self::Finite(F::zero())
    }
    fn is_zero(&self) -> bool {
        *self == Self::Finite(F::zero())
    }
}


impl<F> Ord for AdicValuation<F>
where F: AdicValuationRing {
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

impl<F> PartialOrd for AdicValuation<F>
where F: AdicValuationRing {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


impl<F> fmt::Display for AdicValuation<F>
where F: AdicValuationRing + fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PosInf => write!(f, "inf"),
            Self::Finite(v) => write!(f, "{v}")
        }
    }
}


impl<F> Default for AdicValuation<F>
where F: AdicValuationRing + Default {
    fn default() -> Self {
        Self::Finite(F::default())
    }
}


impl<F> ops::Add for AdicValuation<F>
where F: AdicValuationRing {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        if let (Self::Finite(sv), Self::Finite(rv)) = (self, rhs) {
            Self::Finite(sv + rv)
        } else {
            Self::PosInf
        }
    }
}

impl<F> ops::Mul for AdicValuation<F>
where F: AdicValuationRing {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        if let (Self::Finite(sv), Self::Finite(rv)) = (self, rhs) {
            Self::Finite(sv * rv)
        } else {
            Self::PosInf
        }
    }
}

impl<F> ops::Neg for AdicValuation<F>
where F: AdicValuationRing + ops::Neg<Output=F> {
    type Output = Option<Self>;
    fn neg(self) -> Self::Output {
        if let Self::Finite(v) = self {
            Some(Self::Finite(-v))
        } else {
            None
        }
    }
}

impl<F> ops::Sub for AdicValuation<F>
where F: AdicValuationRing + ops::Sub<Output=F> {
    type Output = Option<Self>;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Finite(sv), Self::Finite(rv)) => Some(Self::Finite(sv - rv)),
            (Self::PosInf, Self::Finite(_)) => Some(Self::PosInf),
            _ => None,
        }
    }
}

impl<F> ops::Div for AdicValuation<F>
where F: AdicValuationRing + ops::Div<Output=F> {
    type Output = Option<Self>;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Finite(sv), Self::Finite(rv)) => Some(Self::Finite(sv / rv)),
            (Self::PosInf, Self::Finite(rv)) if Self::Finite(rv) > Self::zero() => Some(Self::PosInf),
            _ => None,
        }
    }
}


/// Represents valuations of adic integers
pub type ZAdicValuation = AdicValuation<usize>;

/// Represents valuations of adic fractions
pub type QAdicValuation = AdicValuation<isize>;
