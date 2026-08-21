pub type ComplexPolar32 = ComplexPolar<f32>;
pub type ComplexPolar64 = ComplexPolar<f64>;
use crate::traits::Number;
use core::fmt;

use super::Complex as Rectangular;
use core::ops::*;

/// Creates a complex number in polar form.
#[inline(always)]
#[must_use]
pub const fn complex_polar<FT>(abs: FT, arg: FT) -> ComplexPolar<FT> {
    ComplexPolar::new(abs, arg)
}

/// A complex number in polar form.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct ComplexPolar<FT> {
    pub abs: FT,
    pub arg: FT,
}

impl<FT> ComplexPolar<FT> {
    /// Creates a complex number.
    pub const fn new(abs: FT, arg: FT) -> Self {
        Self { abs, arg }
    }
}

impl<FT: Number> ComplexPolar<FT> {
    pub const ZERO: Self = Self::new(FT::ZERO, FT::ZERO);
    pub const ONE: Self = Self::new(FT::ONE, FT::ZERO);

    /// Computes the conjugate.                        
    pub fn conjugate(self) -> Self {
        Self::new(self.abs, -self.arg)
    }

    /// Computes the real component.
    pub fn re(self) -> FT {
        self.abs * self.arg.cos()
    }

    /// Computes the imaginary component.
    pub fn im(self) -> FT {
        self.abs * self.arg.sin()
    }

    /// Computes the squared absolute value.
    pub fn abs_sq(self) -> FT {
        self.abs * self.abs
    }

    /// Computes the reciprocal.
    pub fn recip(self) -> Self {
        Self::new(self.abs.recip(), -self.arg)
    }

    /// Computes the principle square root.
    pub fn sqrt(self) -> Self {
        let two = FT::ONE + FT::ONE;
        Self::new(self.abs.sqrt(), self.arg / two)
    }

    /// Convert to rectangular form.
    pub fn to_rectangular(self) -> Rectangular<FT> {
        let (sin, cos) = self.arg.sin_cos();
        Rectangular::new(cos, sin) * self.abs
    }

    /// Computes `e^self` where `e` is the base of the natural logarithm.
    pub fn exp(self) -> Self {
        self.to_rectangular().exp()
    }

    /// Computes `2^self`.
    pub fn exp2(self) -> Self {
        self.to_rectangular().exp2()
    }

    /// Computes the principle natural logarithm.
    pub fn ln(self) -> Rectangular<FT> {
        Rectangular::new(self.abs.ln(), self.arg)
    }

    /// Computes the principle logarithm in base 2.
    pub fn log2(self) -> Rectangular<FT> {
        self.ln() / FT::LN_2()
    }

    /// Computes the principle logarithm in base 10.
    pub fn log10(self) -> Rectangular<FT> {
        self.ln() / FT::LN_10()
    }

    /// Raises `self` to a floating point power.
    pub fn powf(self, x: FT) -> Self {
        if x < FT::ZERO && self.abs == FT::ZERO {
            return Self::ZERO;
        }
        Self::new(self.abs.powf(x), self.arg * x)
    }

    /// Raises `self` to an integer power.
    pub fn powi(self, n: i32) -> Self {
        if n < 0 && self.abs == FT::ZERO {
            return Self::ZERO;
        }
        Self::new(self.abs.powi(n), self.arg * FT::from_i32(n))
    }

    /// Normalizes the absolute value and the argument into the range `[0, ∞)` and `(-π, +π]` respectively.
    pub fn normalize(mut self) -> Self {
        self.arg = self.arg.rem_euclid(&FT::TAU());
        if self.abs < FT::ZERO {
            self.abs = -self.abs;
            if self.arg <= FT::ZERO {
                self.arg += FT::PI();
            } else {
                self.arg -= FT::PI();
            }
        } else if self.arg > FT::PI() {
            self.arg -= FT::TAU();
        } else if self.arg <= -FT::PI() {
            self.arg += FT::TAU();
        }
        self
    }
}

impl<FT: Number> Mul for ComplexPolar<FT> {
    type Output = Self;
    fn mul(mut self, other: Self) -> Self {
        self *= other;
        self
    }
}

impl<FT: Number> Mul<FT> for ComplexPolar<FT> {
    type Output = Self;
    fn mul(mut self, re: FT) -> Self::Output {
        self *= re;
        self
    }
}

impl<FT: Number> MulAssign for ComplexPolar<FT> {
    fn mul_assign(&mut self, other: Self) {
        self.abs *= other.abs;
        self.arg += other.arg;
    }
}

impl<FT: Number> MulAssign<FT> for ComplexPolar<FT> {
    fn mul_assign(&mut self, re: FT) {
        self.abs *= re;
    }
}

impl<FT: Number> Div for ComplexPolar<FT> {
    type Output = Self;
    fn div(mut self, other: Self) -> Self {
        self /= other;
        self
    }
}

impl<FT: Number> Div<FT> for ComplexPolar<FT> {
    type Output = Self;
    fn div(mut self, re: FT) -> Self {
        self /= re;
        self
    }
}

impl<FT: Number> DivAssign for ComplexPolar<FT> {
    fn div_assign(&mut self, other: Self) {
        *self *= other.recip();
    }
}

impl<FT: Number> DivAssign<FT> for ComplexPolar<FT> {
    fn div_assign(&mut self, re: FT) {
        self.abs /= re;
    }
}

impl<FT: Number> Neg for ComplexPolar<FT> {
    type Output = Self;
    fn neg(mut self) -> Self {
        self.abs = -self.abs;
        self
    }
}

impl<FT: Number + fmt::Display> fmt::Display for ComplexPolar<FT> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn fmt_x<FT: fmt::Display>(f: &mut fmt::Formatter, x: FT) -> fmt::Result {
            if let Some(p) = f.precision() {
                write!(f, "{x:.*}", p)
            } else {
                write!(f, "{x}")
            }
        }
        let pi_radians = self.arg / FT::PI();
        fmt_x(f, self.abs)?;
        if pi_radians == FT::ZERO || self.abs == FT::ZERO {
            Ok(())
        } else if pi_radians == FT::ONE {
            write!(f, "e^iπ")
        } else {
            write!(f, "e^")?;
            fmt_x(f, pi_radians)?;
            write!(f, "iπ")
        }
    }
}

impl<FT: Number> From<FT> for ComplexPolar<FT> {
    fn from(value: FT) -> Self {
        Self::new(value, FT::ZERO)
    }
}

#[cfg(feature = "approx")]
use approx::{AbsDiffEq, RelativeEq, UlpsEq};

#[cfg(feature = "approx")]
impl<FT: AbsDiffEq + Copy> AbsDiffEq for ComplexPolar<FT>
where
    <FT as AbsDiffEq>::Epsilon: Copy,
{
    type Epsilon = <FT as AbsDiffEq>::Epsilon;
    fn default_epsilon() -> Self::Epsilon {
        FT::default_epsilon()
    }
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        FT::abs_diff_eq(&self.abs, &other.abs, epsilon)
            && FT::abs_diff_eq(&self.arg, &other.arg, epsilon)
    }
}

#[cfg(feature = "approx")]
impl<FT: RelativeEq + Copy> RelativeEq for ComplexPolar<FT>
where
    <FT as AbsDiffEq>::Epsilon: Copy,
{
    fn default_max_relative() -> Self::Epsilon {
        FT::default_max_relative()
    }
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        FT::relative_eq(&self.abs, &other.abs, epsilon, max_relative)
            && FT::relative_eq(&self.arg, &other.arg, epsilon, max_relative)
    }
}

#[cfg(feature = "approx")]
impl<FT: UlpsEq + Copy> UlpsEq for ComplexPolar<FT>
where
    <FT as AbsDiffEq>::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        FT::default_max_ulps()
    }
    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        FT::ulps_eq(&self.abs, &other.abs, epsilon, max_ulps)
            && FT::ulps_eq(&self.arg, &other.arg, epsilon, max_ulps)
    }
}
