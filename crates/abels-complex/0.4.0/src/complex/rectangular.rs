use core::ops::*;
pub type Complex32 = Complex<f32>;
pub type Complex64 = Complex<f64>;
use crate::traits::Number;

use super::ComplexPolar as Polar;

/// Creates a complex number in rectangular form.
#[inline(always)]
#[must_use]
pub const fn complex<FT>(re: FT, im: FT) -> Complex<FT> {
    Complex::new(re, im)
}

/// A complex number in rectangular form.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[repr(C)]
pub struct Complex<FT> {
    pub re: FT,
    pub im: FT,
}
impl<FT> Complex<FT> {
    /// Creates a complex number.
    pub const fn new(re: FT, im: FT) -> Self {
        Self { re, im }
    }
}

impl<FT: Number> Complex<FT> {
    pub const ZERO: Self = Self::new(FT::ZERO, FT::ZERO);
    pub const ONE: Self = Self::new(FT::ONE, FT::ZERO);
    pub const I: Self = Self::new(FT::ZERO, FT::ONE);

    /// Computes the conjugate.
    pub fn conjugate(self) -> Self {
        Self::new(self.re, -self.im)
    }

    /// Computes the absolute value.
    pub fn abs(self) -> FT {
        self.abs_sq().sqrt()
    }

    /// Computes the squared absolute value.
    ///
    /// This is faster than `abs()` as it avoids a square root operation.
    pub fn abs_sq(self) -> FT {
        self.re * self.re + self.im * self.im
    }

    /// Computes the argument in the range `(-π, +π]`.
    pub fn arg(self) -> FT {
        self.im.atan2(self.re)
    }

    /// Computes the reciprocal.
    pub fn recip(self) -> Self {
        self.conjugate() / self.abs_sq()
    }

    /// Convert to polar form.
    pub fn to_polar(self) -> Polar<FT> {
        Polar::new(self.abs(), self.arg())
    }

    /// Computes `e^self` where `e` is the base of the natural logarithm.
    pub fn exp(self) -> Polar<FT> {
        Polar::new(self.re.exp(), self.im)
    }

    /// Computes the principle natural logarithm.
    pub fn ln(self) -> Self {
        self.to_polar().ln()
    }

    /// Computes the principle logarithm in base 2.
    pub fn log2(self) -> Self {
        self.ln() / FT::LN_2()
    }

    /// Computes the principle logarithm in base 10.
    pub fn log10(self) -> Self {
        self.ln() / FT::LN_10()
    }

    /// Raises `self` to an integer power.
    pub fn powi(self, n: i32) -> Polar<FT> {
        self.to_polar().powi(n)
    }

    /// Raises `self` to a floating point power.
    pub fn powf(self, x: FT) -> Polar<FT> {
        self.to_polar().powf(x)
    }

    /// Computes the principle square root.
    pub fn sqrt(self) -> Self {
        let two = FT::ONE + FT::ONE;
        let abs = self.abs();
        Self::new(
            ((abs + self.re) / two).sqrt(),
            ((abs - self.re) / two).sqrt().copysign(self.im),
        )
    }

    /// Computes the euclidian distance between two points.
    pub fn distance(self, other: Self) -> FT {
        (self - other).abs()
    }

    /// Computes the squared euclidian distance between two points.
    pub fn distance_squared(self, other: Self) -> FT {
        (self - other).abs_sq()
    }

    /// Computes the linear interpolation between two points based on the value `t`.
    pub fn lerp(self, other: Self, t: FT) -> Self {
        self + (other - self) * t
    }
}

impl<FT: Number> Add for Complex<FT> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Complex::new(self.re + other.re, self.im + other.im)
    }
}

impl<FT: Number> Add<FT> for Complex<FT> {
    type Output = Self;
    fn add(self, re: FT) -> Self::Output {
        Complex::new(self.re + re, self.im)
    }
}

impl<FT: Number> AddAssign for Complex<FT> {
    fn add_assign(&mut self, other: Self) {
        self.re += other.re;
        self.im += other.im;
    }
}

impl<FT: Number> AddAssign<FT> for Complex<FT> {
    fn add_assign(&mut self, re: FT) {
        self.re += re;
    }
}

impl<FT: Number> Sub for Complex<FT> {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Complex::new(self.re - other.re, self.im - other.im)
    }
}

impl<FT: Number> Sub<FT> for Complex<FT> {
    type Output = Self;
    fn sub(self, re: FT) -> Self::Output {
        Complex::new(self.re - re, self.im)
    }
}

impl<FT: Number> SubAssign for Complex<FT> {
    fn sub_assign(&mut self, other: Self) {
        self.re -= other.re;
        self.im -= other.im;
    }
}

impl<FT: Number> SubAssign<FT> for Complex<FT> {
    fn sub_assign(&mut self, re: FT) {
        self.re -= re;
    }
}

impl<FT: Number> Mul for Complex<FT> {
    type Output = Self;
    fn mul(mut self, other: Self) -> Self {
        self *= other;
        self
    }
}

impl<FT: Number> Mul<FT> for Complex<FT> {
    type Output = Self;
    fn mul(self, re: FT) -> Self {
        Complex::new(self.re * re, self.im * re)
    }
}

impl<FT: Number> MulAssign for Complex<FT> {
    fn mul_assign(&mut self, other: Self) {
        let re = self.re * other.re - self.im * other.im;
        self.im = self.re * other.im + self.im * other.re;
        self.re = re;
    }
}

impl<FT: Number> MulAssign<FT> for Complex<FT> {
    fn mul_assign(&mut self, re: FT) {
        self.re *= re;
        self.im *= re;
    }
}

impl<FT: Number> Div for Complex<FT> {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        self * other.recip()
    }
}

impl<FT: Number> Div<FT> for Complex<FT> {
    type Output = Self;
    fn div(self, re: FT) -> Self::Output {
        Complex::new(self.re / re, self.im / re)
    }
}

impl<FT: Number> DivAssign for Complex<FT> {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

impl<FT: Number> DivAssign<FT> for Complex<FT> {
    fn div_assign(&mut self, re: FT) {
        self.re /= re;
        self.im /= re;
    }
}

impl<FT: Number> Neg for Complex<FT> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.re, -self.im)
    }
}

impl<FT: Number> From<FT> for Complex<FT> {
    fn from(value: FT) -> Self {
        Self::new(value, FT::ZERO)
    }
}

#[cfg(feature = "approx")]
use approx::{AbsDiffEq, RelativeEq, UlpsEq};

#[cfg(feature = "approx")]
impl<FT: AbsDiffEq + Copy> AbsDiffEq for Complex<FT>
where
    <FT as AbsDiffEq>::Epsilon: Copy,
{
    type Epsilon = <FT as AbsDiffEq>::Epsilon;
    fn default_epsilon() -> Self::Epsilon {
        FT::default_epsilon()
    }
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        FT::abs_diff_eq(&self.re, &other.re, epsilon)
            && FT::abs_diff_eq(&self.im, &other.im, epsilon)
    }
}

#[cfg(feature = "approx")]
impl<FT: RelativeEq + Copy> RelativeEq for Complex<FT>
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
        FT::relative_eq(&self.re, &other.re, epsilon, max_relative)
            && FT::relative_eq(&self.im, &other.im, epsilon, max_relative)
    }
}

#[cfg(feature = "approx")]
impl<FT: UlpsEq + Copy> UlpsEq for Complex<FT>
where
    <FT as AbsDiffEq>::Epsilon: Copy,
{
    fn default_max_ulps() -> u32 {
        FT::default_max_ulps()
    }
    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        FT::ulps_eq(&self.re, &other.re, epsilon, max_ulps)
            && FT::ulps_eq(&self.im, &other.im, epsilon, max_ulps)
    }
}
