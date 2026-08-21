use crate::Variable;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

macro_rules! impl_math_fn {
    ($method:ident) => {
        #[inline]
        fn $method(self) -> Self {
            Self::$method(self)
        }
    };

    ($method:ident, $param:ty) => {
        #[inline]
        fn $method(self, param: $param) -> Self {
            Self::$method(self, param)
        }
    };
}

pub trait ScalarLike:
    Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Add<f64, Output = Self>
    + Sub<f64, Output = Self>
    + Mul<f64, Output = Self>
    + Div<f64, Output = Self>
    + AddAssign<Self>
    + SubAssign<Self>
    + MulAssign<Self>
    + DivAssign<Self>
    + AddAssign<f64>
    + SubAssign<f64>
    + MulAssign<f64>
    + DivAssign<f64>
    + Sized
    + Clone
    + Copy
{
    #[must_use]
    fn sin(self) -> Self;
    #[must_use]
    fn cos(self) -> Self;
    #[must_use]
    fn tan(self) -> Self;
    #[must_use]
    fn sinh(self) -> Self;
    #[must_use]
    fn cosh(self) -> Self;
    #[must_use]
    fn tanh(self) -> Self;

    #[must_use]
    fn ln(self) -> Self;
    #[must_use]
    fn log(self, base: f64) -> Self;
    #[must_use]
    fn log2(self) -> Self;
    #[must_use]
    fn log10(self) -> Self;
    #[must_use]
    fn exp(self) -> Self;
    #[must_use]
    fn exp2(self) -> Self;
    #[must_use]
    fn powf(self, exponent: f64) -> Self;
    #[must_use]
    fn powi(self, exponent: i32) -> Self;

    #[must_use]
    fn sqrt(self) -> Self;
    #[must_use]
    fn cbrt(self) -> Self;
    #[must_use]
    fn recip(self) -> Self;
    #[must_use]
    fn abs(self) -> Self;

    #[must_use]
    fn asin(self) -> Self;
    #[must_use]
    fn acos(self) -> Self;
    #[must_use]
    fn atan(self) -> Self;
    #[must_use]
    fn asinh(self) -> Self;
    #[must_use]
    fn acosh(self) -> Self;
    #[must_use]
    fn atanh(self) -> Self;

    #[must_use]
    fn hypot(self, other: Self) -> Self;
}

macro_rules! impl_scalar_like {
    ($type:ty) => {
        impl ScalarLike for $type {
            impl_math_fn!(sin);
            impl_math_fn!(cos);
            impl_math_fn!(tan);
            impl_math_fn!(sinh);
            impl_math_fn!(cosh);
            impl_math_fn!(tanh);
            impl_math_fn!(ln);
            impl_math_fn!(log, f64);
            impl_math_fn!(log2);
            impl_math_fn!(log10);
            impl_math_fn!(exp);
            impl_math_fn!(exp2);
            impl_math_fn!(powi, i32);
            impl_math_fn!(powf, f64);
            impl_math_fn!(sqrt);
            impl_math_fn!(cbrt);
            impl_math_fn!(recip);
            impl_math_fn!(abs);
            impl_math_fn!(asin);
            impl_math_fn!(acos);
            impl_math_fn!(atan);
            impl_math_fn!(asinh);
            impl_math_fn!(acosh);
            impl_math_fn!(atanh);
            impl_math_fn!(hypot, Self);
        }
    };
}

impl_scalar_like!(f64);
impl_scalar_like!(Variable<'_>);
