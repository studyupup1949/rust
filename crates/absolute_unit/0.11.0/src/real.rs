use crate::{Angle, Radians, radians};
use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

/// An f64 wrapper that disallows NaN and is therefor orderable.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct Real(pub(crate) f64);

impl Real {
    pub const ZERO: Self = Self(0f64);

    #[inline]
    pub const fn new(v: f64) -> Self {
        Self(v).with_check()
    }

    #[inline]
    pub const fn check(&self) {
        assert!(!self.0.is_nan(), "value is nan");
        assert!(self.0.is_finite(), "value is inf");
    }

    #[inline]
    pub const fn with_check(self) -> Self {
        self.check();
        self
    }

    #[inline]
    pub fn ln(self) -> Self {
        Self(self.0.ln())
    }

    #[inline]
    pub const fn abs(self) -> Self {
        Self(self.0.abs())
    }

    #[inline]
    pub fn powf(self, v: f64) -> Self {
        Self(self.0.powf(v))
    }

    #[inline]
    pub fn sqrt(self) -> Self {
        Self(self.0.sqrt())
    }

    #[inline]
    pub fn asin(self) -> Angle<Radians> {
        radians!(self.into_inner().asin())
    }

    #[inline]
    pub fn acos(self) -> Angle<Radians> {
        radians!(self.into_inner().acos())
    }

    #[inline]
    pub fn atan2(self, other: Real) -> Angle<Radians> {
        radians!(self.0.atan2(other.0))
    }

    // Returns the number rounded towards 0
    #[inline]
    pub const fn floor(self) -> Self {
        Self(self.0.floor())
    }

    // Returns the number rounded towards negative infinity
    #[inline]
    pub const fn trunc(self) -> Self {
        Self(self.0.trunc())
    }

    // Returns the number rounded away from 0
    #[inline]
    pub const fn ceil(self) -> Self {
        Self(self.0.ceil())
    }

    // Returns the number rounded towards positive infinity
    // floor:trunc::ceil:untrunc
    #[inline]
    pub const fn untrunc(self) -> Self {
        Self(self.0.trunc() + 1.)
    }

    #[inline]
    pub const fn round(self) -> Self {
        Self(self.0.round())
    }

    #[inline]
    pub const fn signum(self) -> Real {
        Self(self.0.signum())
    }

    #[inline]
    pub const fn into_inner(self) -> f64 {
        self.0
    }
}

impl Display for Real {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Hash for Real {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // SAFETY: Real asserts non-NaN and non-Inf at construction.
        state.write(&self.0.to_le_bytes())
    }
}

impl PartialEq for Real {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Real {}

impl PartialOrd for Real {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
        // self.0.partial_cmp(&other.0)
    }
}

impl Ord for Real {
    fn cmp(&self, other: &Self) -> Ordering {
        // We know that neither `self` nor `other` are NaN or Inf.
        // Treat +/-0 as the same.
        if self.0 < other.0 {
            Ordering::Less
        } else if self.0 > other.0 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl Neg for Real {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Add for Real {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Real {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Real {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Real {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul for Real {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl MulAssign for Real {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl Div for Real {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0).with_check()
    }
}

impl DivAssign for Real {
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
        self.check();
    }
}
