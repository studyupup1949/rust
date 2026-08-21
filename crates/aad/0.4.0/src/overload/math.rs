use crate::variable::Variable;
use std::ops::{Mul, Neg, Sub};

impl Variable<'_> {
    #[inline]
    #[must_use]
    pub fn ln(self) -> Self {
        self.apply_unary_function(f64::ln, f64::recip)
    }

    #[inline]
    #[must_use]
    pub fn log(self, base: f64) -> Self {
        self.apply_scalar_function(f64::log, |x, b| x.recip().mul(b.ln().recip()), base)
    }
    #[inline]
    #[must_use]
    pub fn powf(self, power: f64) -> Self {
        self.apply_scalar_function(f64::powf, |x, p| p.mul(x.powf(p.sub(1.0))), power)
    }

    #[inline]
    #[must_use]
    pub fn powi(self, power: i32) -> Self {
        self.apply_scalar_function(f64::powi, |x, p| f64::from(p) * x.powi(p - 1), power)
    }

    #[inline]
    #[must_use]
    pub fn exp(self) -> Self {
        self.apply_unary_function(f64::exp, f64::exp)
    }

    #[inline]
    #[must_use]
    pub fn sqrt(self) -> Self {
        self.apply_unary_function(f64::sqrt, |x| 0.5f64 * x.sqrt().recip())
    }

    #[inline]
    #[must_use]
    pub fn cbrt(self) -> Self {
        self.apply_unary_function(f64::cbrt, |x| x.powf(-2.0 / 3.0) / 3.0)
    }

    #[inline]
    #[must_use]
    pub fn recip(self) -> Self {
        self.apply_unary_function(f64::recip, |x| x.powi(2).recip().neg())
    }

    #[inline]
    #[must_use]
    pub fn exp2(self) -> Self {
        self.apply_unary_function(f64::exp2, |x| f64::ln(2.0) * f64::exp2(x))
    }

    #[inline]
    #[must_use]
    pub fn log2(self) -> Self {
        self.apply_unary_function(f64::log2, |x| x.recip() * f64::ln(2.0).recip())
    }

    #[inline]
    #[must_use]
    pub fn log10(self) -> Self {
        self.apply_unary_function(f64::log10, |x| x.recip() * f64::ln(10.0).recip())
    }

    #[inline]
    #[must_use]
    pub fn hypot(self, other: Self) -> Self {
        self.apply_binary_function(other, f64::hypot, |x, y| {
            let denom = x.hypot(y);
            (x / denom, y / denom)
        })
    }

    #[inline]
    #[must_use]
    pub fn abs(self) -> Self {
        self.apply_unary_function(f64::abs, f64::signum)
    }
}

// trigonometric functions
impl Variable<'_> {
    #[inline]
    #[must_use]
    pub fn sin(self) -> Self {
        self.apply_unary_function(f64::sin, f64::cos)
    }

    #[inline]
    #[must_use]
    pub fn cos(self) -> Self {
        self.apply_unary_function(f64::cos, |x| -f64::sin(x))
    }

    #[inline]
    #[must_use]
    pub fn tan(self) -> Self {
        self.apply_unary_function(f64::tan, |x| f64::cos(x).powi(2).recip())
    }

    #[inline]
    #[must_use]
    pub fn sinh(self) -> Self {
        self.apply_unary_function(f64::sinh, f64::cosh)
    }

    #[inline]
    #[must_use]
    pub fn cosh(self) -> Self {
        self.apply_unary_function(f64::cosh, f64::sinh)
    }

    #[inline]
    #[must_use]
    pub fn tanh(self) -> Self {
        self.apply_unary_function(f64::tanh, |x| f64::cosh(x).powi(2).recip())
    }

    #[inline]
    #[must_use]
    pub fn asin(self) -> Self {
        self.apply_unary_function(f64::asin, |x| (1.0 - x * x).sqrt().recip())
    }

    #[inline]
    #[must_use]
    pub fn acos(self) -> Self {
        self.apply_unary_function(f64::acos, |x| -(1.0 - x * x).sqrt().recip())
    }

    #[inline]
    #[must_use]
    pub fn atan(self) -> Self {
        self.apply_unary_function(f64::atan, |x| (1.0 + x * x).recip())
    }

    #[inline]
    #[must_use]
    pub fn asinh(self) -> Self {
        self.apply_unary_function(f64::asinh, |x| (x * x + 1.0).sqrt().recip())
    }

    #[inline]
    #[must_use]
    pub fn acosh(self) -> Self {
        self.apply_unary_function(f64::acosh, |x| (x * x - 1.0).sqrt().recip())
    }

    #[inline]
    #[must_use]
    pub fn atanh(self) -> Self {
        self.apply_unary_function(f64::atanh, |x| (1.0 - x * x).recip())
    }
}
