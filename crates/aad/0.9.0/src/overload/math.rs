use crate::{FloatLike, variable::Variable};
use num_traits::One;
use std::{
    cmp::Ordering,
    ops::{Div as _, Mul, Neg, Sub as _},
};

impl From<Variable<'_, f64>> for f64 {
    fn from(value: Variable<'_, f64>) -> Self {
        value.value
    }
}

impl From<Variable<'_, Variable<'_, f64>>> for f64 {
    fn from(value: Variable<'_, Variable<'_, f64>>) -> Self {
        value.value.value
    }
}

impl Variable<'_, f64> {
    #[inline]
    #[must_use]
    pub fn sin(self) -> Self {
        self.apply_unary_function(f64::sin, f64::cos)
    }

    #[inline]
    #[must_use]
    pub fn cos(self) -> Self {
        self.apply_unary_function(f64::cos, |x| -x.sin())
    }

    #[inline]
    #[must_use]
    pub fn tan(self) -> Self {
        self.apply_unary_function(f64::tan, |x| x.cos().powi(2).recip())
    }

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
        self.apply_scalar_function(f64::powf, |x, p| p.mul(x.powf(p.sub(f64::one()))), power)
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
        self.apply_unary_function(f64::sqrt, |x| x.sqrt().recip().div(f64::one() + f64::one()))
    }

    #[inline]
    #[must_use]
    pub fn cbrt(self) -> Self {
        self.apply_unary_function(f64::cbrt, |x| {
            x.powf(-(f64::from(2) / f64::from(3))) / f64::from(3)
        })
    }

    #[inline]
    #[must_use]
    pub fn recip(self) -> Self {
        self.apply_unary_function(f64::recip, |x| x.powi(2).recip().neg())
    }

    #[inline]
    #[must_use]
    pub fn exp2(self) -> Self {
        self.apply_unary_function(f64::exp2, |x| {
            f64::ln(f64::one() + f64::one()) * f64::exp2(x)
        })
    }

    #[inline]
    #[must_use]
    pub fn log2(self) -> Self {
        self.apply_unary_function(f64::log2, |x| {
            x.recip() * f64::ln(f64::one() + f64::one()).recip()
        })
    }

    #[inline]
    #[must_use]
    pub fn log10(self) -> Self {
        self.apply_unary_function(f64::log10, |x| x.recip() * 10.0_f64.ln().recip())
    }

    #[inline]
    #[must_use]
    pub fn hypot(self, other: Self) -> Self {
        self.apply_binary_function(&other, f64::hypot, |x, y| {
            let denom = x.hypot(y);
            (x / denom, y / denom)
        })
    }

    #[inline]
    #[must_use]
    pub fn abs(self) -> Self {
        self.apply_unary_function(f64::abs, f64::signum)
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
        self.apply_unary_function(f64::asin, |x| (f64::one() - x * x).sqrt().recip())
    }

    #[inline]
    #[must_use]
    pub fn acos(self) -> Self {
        self.apply_unary_function(f64::acos, |x| -(f64::one() - x * x).sqrt().recip())
    }

    #[inline]
    #[must_use]
    pub fn atan(self) -> Self {
        self.apply_unary_function(f64::atan, |x| (f64::one() + x * x).recip())
    }

    #[inline]
    #[must_use]
    pub fn asinh(self) -> Self {
        self.apply_unary_function(f64::asinh, |x| (x * x + f64::one()).sqrt().recip())
    }

    #[inline]
    #[must_use]
    pub fn acosh(self) -> Self {
        self.apply_unary_function(f64::acosh, |x| (x * x - f64::one()).sqrt().recip())
    }

    #[inline]
    #[must_use]
    pub fn atanh(self) -> Self {
        self.apply_unary_function(f64::atanh, |x| (f64::one() - x * x).recip())
    }
}

impl FloatLike<f64> for Variable<'_, f64> {
    #[inline]
    fn sin(self) -> Self {
        self.sin()
    }

    #[inline]
    fn cos(self) -> Self {
        self.cos()
    }

    #[inline]
    fn tan(self) -> Self {
        self.tan()
    }

    #[inline]
    fn sinh(self) -> Self {
        self.sinh()
    }

    #[inline]
    fn cosh(self) -> Self {
        self.cosh()
    }

    #[inline]
    fn tanh(self) -> Self {
        self.tanh()
    }

    #[inline]
    fn ln(self) -> Self {
        self.ln()
    }

    #[inline]
    fn log(self, base: f64) -> Self {
        self.log(base)
    }

    #[inline]
    fn log2(self) -> Self {
        self.log2()
    }

    #[inline]
    fn log10(self) -> Self {
        self.log10()
    }

    #[inline]
    fn exp(self) -> Self {
        self.exp()
    }

    #[inline]
    fn exp2(self) -> Self {
        self.exp2()
    }

    #[inline]
    fn powf(self, exponent: f64) -> Self {
        self.powf(exponent)
    }

    #[inline]
    fn powi(self, exponent: i32) -> Self {
        self.powi(exponent)
    }

    #[inline]
    fn sqrt(self) -> Self {
        self.sqrt()
    }

    #[inline]
    fn cbrt(self) -> Self {
        self.cbrt()
    }

    #[inline]
    fn recip(self) -> Self {
        self.recip()
    }

    #[inline]
    fn abs(self) -> Self {
        self.abs()
    }

    #[inline]
    fn asin(self) -> Self {
        self.asin()
    }

    #[inline]
    fn acos(self) -> Self {
        self.acos()
    }

    #[inline]
    fn atan(self) -> Self {
        self.atan()
    }

    #[inline]
    fn asinh(self) -> Self {
        self.asinh()
    }

    #[inline]
    fn acosh(self) -> Self {
        self.acosh()
    }

    #[inline]
    fn atanh(self) -> Self {
        self.atanh()
    }

    #[inline]
    fn hypot(self, other: Self) -> Self {
        self.hypot(other)
    }
}

impl Variable<'_, Variable<'_, f64>> {
    #[inline]
    #[must_use]
    pub fn sin(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::sin, Variable::<'_, f64>::cos)
    }

    #[inline]
    #[must_use]
    pub fn cos(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::cos, |x| -x.sin())
    }

    #[inline]
    #[must_use]
    pub fn tan(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::tan, |x| x.cos().powi(2).recip())
    }

    #[inline]
    #[must_use]
    pub fn ln(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::ln, Variable::<'_, f64>::recip)
    }

    #[inline]
    #[must_use]
    pub fn log(self, base: f64) -> Self {
        self.apply_scalar_function(
            Variable::<'_, f64>::log,
            |x, b| x.recip().mul(b.ln().recip()),
            base,
        )
    }

    #[inline]
    #[must_use]
    pub fn powf(self, power: f64) -> Self {
        self.apply_scalar_function(
            Variable::<'_, f64>::powf,
            |x, p| p.mul(x.powf(p.sub(f64::one()))),
            power,
        )
    }

    #[inline]
    #[must_use]
    pub fn powi(self, power: i32) -> Self {
        self.apply_scalar_function(
            Variable::<'_, f64>::powi,
            |x, p| f64::from(p) * x.powi(p - 1),
            power,
        )
    }

    #[inline]
    #[must_use]
    pub fn exp(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::exp, Variable::<'_, f64>::exp)
    }

    #[inline]
    #[must_use]
    pub fn sqrt(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::sqrt, |x| {
            x.sqrt().recip().div(f64::one() + f64::one())
        })
    }

    #[inline]
    #[must_use]
    pub fn cbrt(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::cbrt, |x| {
            x.powf(-(f64::from(2) / f64::from(3))) / f64::from(3)
        })
    }

    #[inline]
    #[must_use]
    pub fn recip(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::recip, |x| x.powi(2).recip().neg())
    }

    #[inline]
    #[must_use]
    pub fn exp2(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::exp2, |x| {
            Variable::<'_, f64>::ln(Variable::<'_, f64>::one() + Variable::<'_, f64>::one())
                * Variable::<'_, f64>::exp2(x)
        })
    }

    #[inline]
    #[must_use]
    pub fn log2(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::log2, |x| {
            x.recip()
                * Variable::<'_, f64>::ln(Variable::<'_, f64>::one() + Variable::<'_, f64>::one())
                    .recip()
        })
    }

    #[inline]
    #[must_use]
    pub fn log10(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::log10, |x| {
            x.recip() * 10.0_f64.ln().recip()
        })
    }

    #[inline]
    #[must_use]
    pub fn hypot(self, other: Self) -> Self {
        self.apply_binary_function(&other, Variable::<'_, f64>::hypot, |x, y| {
            let denom = x.hypot(y);
            (x / denom, y / denom)
        })
    }

    #[inline]
    #[must_use]
    pub fn abs(self) -> Self {
        todo!()
        // self.apply_unary_function(Variable::abs, Variable::signum)
    }

    #[inline]
    #[must_use]
    pub fn sinh(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::sinh, Variable::<'_, f64>::cosh)
    }

    #[inline]
    #[must_use]
    pub fn cosh(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::cosh, Variable::<'_, f64>::sinh)
    }

    #[inline]
    #[must_use]
    pub fn tanh(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::tanh, |x| {
            Variable::<'_, f64>::cosh(x).powi(2).recip()
        })
    }

    #[inline]
    #[must_use]
    pub fn asin(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::asin, |x| {
            (Variable::<'_, f64>::one() - x * x).sqrt().recip()
        })
    }

    #[inline]
    #[must_use]
    pub fn acos(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::acos, |x| {
            -(Variable::<'_, f64>::one() - x * x).sqrt().recip()
        })
    }

    #[inline]
    #[must_use]
    pub fn atan(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::atan, |x| {
            (Variable::<'_, f64>::one() + x * x).recip()
        })
    }

    #[inline]
    #[must_use]
    pub fn asinh(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::asinh, |x| {
            (x * x + Variable::<'_, f64>::one()).sqrt().recip()
        })
    }

    #[inline]
    #[must_use]
    pub fn acosh(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::acosh, |x| {
            (x * x - Variable::<'_, f64>::one()).sqrt().recip()
        })
    }

    #[inline]
    #[must_use]
    pub fn atanh(self) -> Self {
        self.apply_unary_function(Variable::<'_, f64>::atanh, |x| {
            (Variable::<'_, f64>::one() - x * x).recip()
        })
    }
}

impl FloatLike<f64> for Variable<'_, Variable<'_, f64>> {
    #[inline]
    fn sin(self) -> Self {
        self.sin()
    }

    #[inline]
    fn cos(self) -> Self {
        self.cos()
    }

    #[inline]
    fn tan(self) -> Self {
        self.tan()
    }

    #[inline]
    fn sinh(self) -> Self {
        self.sinh()
    }

    #[inline]
    fn cosh(self) -> Self {
        self.cosh()
    }

    #[inline]
    fn tanh(self) -> Self {
        self.tanh()
    }

    #[inline]
    fn ln(self) -> Self {
        self.ln()
    }

    #[inline]
    fn log(self, base: f64) -> Self {
        self.log(base)
    }

    #[inline]
    fn log2(self) -> Self {
        self.log2()
    }

    #[inline]
    fn log10(self) -> Self {
        self.log10()
    }

    #[inline]
    fn exp(self) -> Self {
        self.exp()
    }

    #[inline]
    fn exp2(self) -> Self {
        self.exp2()
    }

    #[inline]
    fn powf(self, exponent: f64) -> Self {
        self.powf(exponent)
    }

    #[inline]
    fn powi(self, exponent: i32) -> Self {
        self.powi(exponent)
    }

    #[inline]
    fn sqrt(self) -> Self {
        self.sqrt()
    }

    #[inline]
    fn cbrt(self) -> Self {
        self.cbrt()
    }

    #[inline]
    fn recip(self) -> Self {
        self.recip()
    }

    #[inline]
    fn abs(self) -> Self {
        self.abs()
    }

    #[inline]
    fn asin(self) -> Self {
        self.asin()
    }

    #[inline]
    fn acos(self) -> Self {
        self.acos()
    }

    #[inline]
    fn atan(self) -> Self {
        self.atan()
    }

    #[inline]
    fn asinh(self) -> Self {
        self.asinh()
    }

    #[inline]
    fn acosh(self) -> Self {
        self.acosh()
    }

    #[inline]
    fn atanh(self) -> Self {
        self.atanh()
    }

    #[inline]
    fn hypot(self, other: Self) -> Self {
        self.hypot(other)
    }
}

impl<T, F: PartialOrd<T>> PartialOrd<T> for Variable<'_, F>
where
    Self: PartialEq<T>,
{
    #[inline]
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.value.partial_cmp(other)
    }
}

impl<T, F: PartialEq<T>> PartialEq<T> for Variable<'_, F> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.value == *other
    }
}
