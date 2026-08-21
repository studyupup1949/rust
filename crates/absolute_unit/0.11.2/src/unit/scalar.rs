use crate::{Angle, DynamicUnits, Quantity, Radians, Real, radians};
use approx::AbsDiffEq;
use std::{
    fmt::{Display, Formatter},
    ops::{Add, Div, Mul, Neg, Sub},
};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct Scalar(pub(crate) Real);

impl Scalar {
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
    pub fn untrunc(self) -> Self {
        Self(self.0.trunc() + Real(1.0))
    }

    #[inline]
    pub fn round(self) -> Self {
        Self(self.0.round())
    }

    #[inline]
    pub fn into_real(self) -> Real {
        self.0
    }

    #[inline]
    pub fn into_inner(self) -> f64 {
        self.0.into_inner()
    }

    #[inline]
    pub fn as_dyn(&self) -> DynamicUnits {
        DynamicUnits::new0o0(self.0)
    }

    #[inline]
    pub fn f32(&self) -> f32 {
        self.into_inner() as f32
    }
}

impl Quantity for Scalar {
    fn f64(&self) -> f64 {
        self.into_inner()
    }
}

impl From<DynamicUnits> for Scalar {
    fn from(v: DynamicUnits) -> Self {
        let f = v.real();
        v.assert_units_empty();
        Self(f)
    }
}

impl Display for Scalar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Neg for Scalar {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(self.0.neg())
    }
}

impl Add<Scalar> for Scalar {
    type Output = Scalar;
    fn add(self, rhs: Scalar) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<Scalar> for Scalar {
    type Output = Scalar;
    fn sub(self, rhs: Scalar) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul<Scalar> for Scalar {
    type Output = Scalar;

    fn mul(self, rhs: Scalar) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Div<Scalar> for Scalar {
    type Output = Scalar;

    fn div(self, rhs: Scalar) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl From<f64> for Scalar {
    fn from(v: f64) -> Self {
        Self(Real::new(v))
    }
}

impl From<&f64> for Scalar {
    fn from(v: &f64) -> Self {
        Self(Real::new(*v))
    }
}

impl From<f32> for Scalar {
    fn from(v: f32) -> Self {
        Self(Real::new(v as f64))
    }
}

impl From<&f32> for Scalar {
    fn from(v: &f32) -> Self {
        Self(Real::new(*v as f64))
    }
}

impl From<Real> for Scalar {
    fn from(v: Real) -> Self {
        Self(v)
    }
}

impl From<usize> for Scalar {
    fn from(v: usize) -> Self {
        Self(Real::new(v as f64))
    }
}

impl From<i32> for Scalar {
    fn from(v: i32) -> Self {
        Self(Real(v as f64))
    }
}

impl From<u32> for Scalar {
    fn from(v: u32) -> Self {
        Self(Real(v as f64))
    }
}

impl AbsDiffEq for Scalar {
    type Epsilon = f64;

    fn default_epsilon() -> Self::Epsilon {
        <f64 as AbsDiffEq>::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.0.0.abs_diff_eq(&other.0.0, epsilon)
    }
}

#[macro_export]
macro_rules! scalar {
    ($num:expr) => {
        $crate::Scalar::from($num)
    };
}
