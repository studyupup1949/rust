use core::f32::consts::{FRAC_PI_2, LN_2, LN_10, PI, TAU};
use core::fmt;
use core::ops::*;
use core::write;
#[cfg(feature = "libm")]
use num_traits::real::Real;

use crate::rectangular::*;

/// Creates a complex number in polar form.
#[inline(always)]
#[must_use]
pub const fn complex_polar(abs: f32, arg: f32) -> ComplexPolar {
    ComplexPolar::new(abs, arg)
}

/// A complex number in polar form.
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct ComplexPolar {
    pub abs: f32,
    pub arg: f32,
}

impl ComplexPolar {
    pub const ZERO: Self = complex_polar(0.0, 0.0);
    pub const ONE: Self = complex_polar(1.0, 0.0);
    pub const I: Self = complex_polar(1.0, FRAC_PI_2);
    pub const NEG_ONE: Self = complex_polar(1.0, PI);
    pub const NEG_I: Self = complex_polar(1.0, -FRAC_PI_2);

    /// Creates a complex number.
    pub const fn new(abs: f32, arg: f32) -> Self {
        Self { abs, arg }
    }

    /// Computes the conjugate.
    pub const fn conjugate(self) -> Self {
        complex_polar(self.abs, -self.arg)
    }

    /// Computes the real component.
    pub fn re(self) -> f32 {
        self.abs * self.arg.cos()
    }

    /// Computes the imaginary component.
    pub fn im(self) -> f32 {
        self.abs * self.arg.sin()
    }

    /// Computes the squared absolute value.
    pub const fn abs_sq(self) -> f32 {
        self.abs * self.abs
    }

    /// Computes the reciprocal.
    pub fn recip(self) -> Self {
        complex_polar(self.abs.recip(), -self.arg)
    }

    /// Computes the principle square root.
    pub fn sqrt(self) -> Self {
        complex_polar(self.abs.sqrt(), self.arg / 2.0)
    }

    /// Convert to rectangular form.
    pub fn to_rectangular(self) -> Complex {
        let (sin, cos) = self.arg.sin_cos();
        self.abs * complex(cos, sin)
    }

    /// Computes `e^self` where `e` is the base of the natural logarithm.
    pub fn exp(self) -> Self {
        self.to_rectangular().exp()
    }

    /// Computes the principle natural logarithm.
    pub fn ln(self) -> Complex {
        complex(self.abs.ln(), self.arg)
    }

    /// Computes the principle logarithm in base 2.
    pub fn log2(self) -> Complex {
        self.ln() / LN_2
    }

    /// Computes the principle logarithm in base 10.
    pub fn log10(self) -> Complex {
        self.ln() / LN_10
    }

    /// Raises `self` to a floating point power.
    pub fn powf(self, x: f32) -> Self {
        if x < 0.0 && self.abs == 0.0 {
            return ComplexPolar::ZERO;
        }
        complex_polar(self.abs.powf(x), self.arg * x)
    }

    /// Raises `self` to an integer power.
    pub fn powi(self, n: i32) -> Self {
        if n < 0 && self.abs == 0.0 {
            return ComplexPolar::ZERO;
        }
        complex_polar(self.abs.powi(n), self.arg * n as f32)
    }

    /// Normalizes the absolute value and the argument into the range `[0, ∞)` and `(-π, +π]` respectively.
    pub fn normalize(mut self) -> Self {
        #[cfg(feature = "libm")]
        {
            self.arg = num_traits::Euclid::rem_euclid(&self.arg, &TAU);
        }
        #[cfg(not(feature = "libm"))]
        {
            self.arg = self.arg.rem_euclid(TAU);
        }
        if self.abs < 0.0 {
            self.abs = -self.abs;
            if self.arg <= 0.0 {
                self.arg += PI;
            } else {
                self.arg -= PI;
            }
        } else {
            if self.arg > PI {
                self.arg -= TAU;
            } else if self.arg <= -PI {
                self.arg += TAU;
            }
        }
        self
    }
}

impl Mul for ComplexPolar {
    type Output = Self;
    fn mul(mut self, other: Self) -> Self::Output {
        self *= other;
        self
    }
}

impl Mul<f32> for ComplexPolar {
    type Output = Self;
    fn mul(mut self, re: f32) -> Self::Output {
        self *= re;
        self
    }
}

impl Mul<ComplexPolar> for f32 {
    type Output = ComplexPolar;
    fn mul(self, mut other: ComplexPolar) -> Self::Output {
        other *= self;
        other
    }
}

impl MulAssign for ComplexPolar {
    fn mul_assign(&mut self, other: Self) {
        self.abs *= other.abs;
        self.arg += other.arg;
    }
}

impl MulAssign<f32> for ComplexPolar {
    fn mul_assign(&mut self, re: f32) {
        self.abs *= re;
    }
}

impl Div for ComplexPolar {
    type Output = Self;
    fn div(mut self, other: Self) -> Self::Output {
        self /= other;
        self
    }
}

impl Div<f32> for ComplexPolar {
    type Output = Self;
    fn div(mut self, re: f32) -> Self::Output {
        self /= re;
        self
    }
}

impl Div<ComplexPolar> for f32 {
    type Output = ComplexPolar;
    fn div(self, other: ComplexPolar) -> Self::Output {
        self * other.recip()
    }
}

impl DivAssign for ComplexPolar {
    fn div_assign(&mut self, other: Self) {
        *self *= other.recip();
    }
}

impl DivAssign<f32> for ComplexPolar {
    fn div_assign(&mut self, re: f32) {
        self.abs /= re;
    }
}

impl Neg for ComplexPolar {
    type Output = Self;
    fn neg(mut self) -> Self::Output {
        self.abs = -self.abs;
        self
    }
}

impl fmt::Display for ComplexPolar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn fmt_x(f: &mut fmt::Formatter, x: f32) -> fmt::Result {
            if let Some(p) = f.precision() {
                write!(f, "{x:.*}", p)
            } else {
                write!(f, "{x}")
            }
        }
        let pi_radians = self.arg / PI;
        fmt_x(f, self.abs)?;
        if pi_radians == 0.0 || self.abs == 0.0 {
            Ok(())
        } else if pi_radians == 1.0 {
            write!(f, "e^iπ")
        } else {
            write!(f, "e^")?;
            fmt_x(f, pi_radians)?;
            write!(f, "iπ")
        }
    }
}

#[cfg(feature = "rand")]
impl rand::distr::Distribution<ComplexPolar> for rand::distr::StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> ComplexPolar {
        complex_polar(self.sample(rng), rng.random_range((-PI).next_up()..=PI))
    }
}

#[cfg(feature = "approx")]
use approx::{AbsDiffEq, RelativeEq, UlpsEq};

#[cfg(feature = "approx")]
impl AbsDiffEq for ComplexPolar {
    type Epsilon = <f32 as AbsDiffEq>::Epsilon;
    fn default_epsilon() -> Self::Epsilon {
        f32::default_epsilon()
    }
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        f32::abs_diff_eq(&self.abs, &other.abs, epsilon)
            && f32::abs_diff_eq(&self.arg, &other.arg, epsilon)
    }
}

#[cfg(feature = "approx")]
impl RelativeEq for ComplexPolar {
    fn default_max_relative() -> Self::Epsilon {
        f32::default_max_relative()
    }
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        f32::relative_eq(&self.abs, &other.abs, epsilon, max_relative)
            && f32::relative_eq(&self.arg, &other.arg, epsilon, max_relative)
    }
}

#[cfg(feature = "approx")]
impl UlpsEq for ComplexPolar {
    fn default_max_ulps() -> u32 {
        f32::default_max_ulps()
    }
    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        f32::ulps_eq(&self.abs, &other.abs, epsilon, max_ulps)
            && f32::ulps_eq(&self.arg, &other.arg, epsilon, max_ulps)
    }
}

impl From<f32> for ComplexPolar {
    fn from(value: f32) -> Self {
        complex_polar(value, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;
    use core::f32::consts::{E, FRAC_PI_2, PI};
    use rand::{
        Rng, SeedableRng,
        distr::{Distribution, StandardUniform, Uniform, uniform::*},
        rngs::StdRng,
    };

    const NUM_SAMPLES: usize = 100;

    fn random_samples<T>() -> impl core::iter::Iterator<Item = T>
    where
        StandardUniform: Distribution<T>,
    {
        StdRng::seed_from_u64(21)
            .sample_iter(StandardUniform)
            .take(NUM_SAMPLES)
    }

    fn uniform_samples<T>(low: T, high: T) -> impl core::iter::Iterator<Item = T>
    where
        T: SampleUniform,
    {
        StdRng::seed_from_u64(21)
            .sample_iter(Uniform::new(low, high).unwrap())
            .take(NUM_SAMPLES)
    }

    #[test]
    fn multiplication() {
        for z0 in random_samples::<ComplexPolar>() {
            for z1 in random_samples::<ComplexPolar>() {
                let z = z0 * z1;
                assert_eq!(z.abs, z0.abs * z1.abs);
                assert_eq!(z.arg, z0.arg + z1.arg);

                let z = z0 * z1.re();
                assert_eq!(z.normalize().abs, z0.abs * z1.re().abs());
                assert_eq!(z.arg, z0.arg);

                let z = z0.re() * z1;
                assert_eq!(z.normalize().abs, z0.re().abs() * z1.abs);
                assert_eq!(z.arg, z1.arg);

                let mut z = z0;
                z *= z1;
                assert_eq!(z, z0 * z1);

                let mut z = z0;
                z *= z1.re();
                assert_eq!(z, z0 * z1.re());
            }
            assert_eq!(z0 * ComplexPolar::ONE, z0);
            assert_eq!((z0 * ComplexPolar::ZERO).abs, 0.0);
            assert_eq!((z0 * 0.0).abs, 0.0);
        }
    }

    #[test]
    fn division() {
        for z0 in random_samples::<ComplexPolar>() {
            for z1 in random_samples::<ComplexPolar>() {
                let z = z0 / z1;
                assert_ulps_eq!(z.abs, z0.abs / z1.abs);
                assert_ulps_eq!(z.arg, z0.arg - z1.arg);

                let z = z0 / z1.re();
                assert_ulps_eq!(z.normalize().abs, z0.abs / z1.re().abs());
                assert_ulps_eq!(z.arg, z0.arg);

                let z = z0.re() / z1;
                assert_ulps_eq!(z.normalize().abs, z0.re().abs() / z1.abs);
                assert_ulps_eq!(z.arg, -z1.arg);

                let mut z = z0;
                z /= z1;
                assert_eq!(z, z0 / z1);

                let mut z = z0;
                z /= z1.re();
                assert_eq!(z, z0 / z1.re());
            }
            assert_ulps_eq!(z0 / z0, ComplexPolar::ONE);
            assert_eq!((ComplexPolar::ZERO / z0).abs, 0.0);
        }
    }

    #[test]
    fn negation() {
        assert_eq!((-ComplexPolar::ONE).normalize(), ComplexPolar::NEG_ONE);
        assert_eq!((-ComplexPolar::I).normalize(), ComplexPolar::NEG_I);
        assert_eq!((-ComplexPolar::NEG_ONE).normalize(), ComplexPolar::ONE);
        assert_ulps_eq!((-ComplexPolar::NEG_I).normalize(), ComplexPolar::I);
    }

    #[test]
    fn reciprocal() {
        for z in random_samples::<ComplexPolar>() {
            assert_ulps_eq!(z.recip(), 1.0 / z);
            assert_ulps_eq!(z * z.recip(), ComplexPolar::ONE);
        }
        assert_eq!(ComplexPolar::ONE.recip(), ComplexPolar::ONE);
        assert_eq!(ComplexPolar::I.recip(), ComplexPolar::NEG_I);
        assert_eq!(
            ComplexPolar::NEG_ONE.recip().normalize(),
            ComplexPolar::NEG_ONE
        );
        assert_eq!(ComplexPolar::NEG_I.recip(), ComplexPolar::I);
    }

    #[test]
    fn sqrt() {
        for z in random_samples::<ComplexPolar>() {
            assert_eq!(z.sqrt().abs, z.abs.sqrt());
            assert_eq!(z.sqrt().arg, z.arg / 2.0);
        }
        assert_eq!(ComplexPolar::ONE.sqrt(), ComplexPolar::ONE);
        assert_eq!(ComplexPolar::NEG_ONE.sqrt(), ComplexPolar::I);
        assert_eq!(ComplexPolar::ONE.sqrt(), ComplexPolar::ONE);
    }

    #[test]
    fn abs() {
        for z in random_samples::<ComplexPolar>() {
            assert_eq!(z.abs_sq(), z.abs * z.abs);
        }
        assert_eq!(ComplexPolar::ONE.abs, 1.0);
        assert_eq!(ComplexPolar::I.abs, 1.0);
        assert_eq!(ComplexPolar::NEG_ONE.abs, 1.0);
        assert_eq!(ComplexPolar::NEG_I.abs, 1.0);
    }

    #[test]
    fn conjugate() {
        for z in random_samples::<ComplexPolar>() {
            assert_eq!(z.conjugate().re(), z.re());
            assert_eq!(z.conjugate().im(), -z.im());
            assert_eq!(z.conjugate().conjugate(), z);
        }
        assert_ulps_eq!(ComplexPolar::ONE.conjugate().normalize(), ComplexPolar::ONE);
        assert_ulps_eq!(ComplexPolar::I.conjugate().normalize(), ComplexPolar::NEG_I);
        assert_ulps_eq!(
            ComplexPolar::NEG_ONE.conjugate().normalize(),
            ComplexPolar::NEG_ONE
        );
        assert_ulps_eq!(ComplexPolar::NEG_I.conjugate().normalize(), ComplexPolar::I);
    }

    #[test]
    fn arg() {
        assert_eq!(ComplexPolar::ONE.arg, 0.0);
        assert_eq!(ComplexPolar::I.arg, FRAC_PI_2);
        assert_eq!(ComplexPolar::NEG_ONE.arg, PI);
        assert_eq!(ComplexPolar::NEG_I.arg, -FRAC_PI_2);
    }

    #[test]
    fn exp() {
        for z in random_samples::<ComplexPolar>() {
            assert_eq!(z.exp().abs, z.re().exp());
            assert_eq!(z.exp().arg, z.im());
            assert_ulps_eq!(z.exp().ln(), z.to_rectangular());
        }
        assert_ulps_eq!(ComplexPolar::ONE.exp(), complex_polar(E, 0.0));
        assert_ulps_eq!(ComplexPolar::I.exp(), complex_polar(1.0, 1.0));
        assert_ulps_eq!(ComplexPolar::NEG_ONE.exp(), complex_polar(E.recip(), 0.0));
        assert_ulps_eq!(ComplexPolar::NEG_I.exp(), complex_polar(1.0, -1.0));
    }

    #[test]
    fn log() {
        for z in random_samples::<ComplexPolar>() {
            assert_eq!(z.ln().re, z.abs.ln());
            assert_eq!(z.ln().im, z.arg);
            assert_ulps_eq!(z.ln().exp(), z);
        }
        assert_eq!(ComplexPolar::ONE.ln(), Complex::ZERO);
        assert_eq!(ComplexPolar::I.ln(), Complex::I * FRAC_PI_2);
        assert_eq!(ComplexPolar::NEG_ONE.ln(), Complex::I * PI);
        assert_eq!(ComplexPolar::NEG_I.ln(), Complex::I * -FRAC_PI_2);

        assert_ulps_eq!(complex_polar(E, 0.0).ln(), Complex::ONE);
        assert_ulps_eq!(complex_polar(2.0, 0.0).log2(), Complex::ONE);
        assert_ulps_eq!(complex_polar(10.0, 0.0).log10(), Complex::ONE);
    }

    #[test]
    fn powi() {
        for z in random_samples::<ComplexPolar>() {
            assert_eq!(z.powi(0), ComplexPolar::ONE);
            assert_eq!(z.powi(1), z);
            for n in random_samples::<i32>() {
                assert_eq!(z.powi(n).abs, z.abs.powi(n));
                assert_eq!(z.powi(n).arg, z.arg * n as f32);
            }
        }
        for n in random_samples::<i32>() {
            assert_eq!(ComplexPolar::ZERO.powi(n.abs()), ComplexPolar::ZERO);
            assert_eq!(ComplexPolar::ONE.powi(n), ComplexPolar::ONE);
        }
    }

    #[test]
    fn powf() {
        for z in random_samples::<ComplexPolar>() {
            assert_eq!(z.powf(0.0), ComplexPolar::ONE);
            assert_eq!(z.powf(1.0), z);
            for n in random_samples::<i32>() {
                let x = n as f32 * 0.01;
                assert_eq!(z.powf(x).abs, z.abs.powf(x));
                assert_eq!(z.powf(x).arg, z.arg * x);
            }
        }
        for n in random_samples::<i32>() {
            let x = n as f32 * 0.01;
            assert_eq!(ComplexPolar::ZERO.powf(x.abs()), ComplexPolar::ZERO);
            assert_eq!(ComplexPolar::ONE.powf(x), ComplexPolar::ONE);
        }
    }

    #[test]
    fn normalize() {
        for z in random_samples::<ComplexPolar>() {
            for n in uniform_samples::<i32>(-99, 99) {
                let w = complex_polar(z.abs, z.arg + n as f32 * TAU);
                assert_ulps_eq!(z, w.normalize(), epsilon = 2000.0 * f32::EPSILON);

                assert_ulps_eq!(
                    complex_polar(-z.abs, z.arg).normalize(),
                    complex_polar(z.abs, z.arg + PI).normalize(),
                    epsilon = 2000.0 * f32::EPSILON
                );
            }
        }
    }

    // #[test]
    // fn format_display() {
    //     for n in random_samples::<i32>() {
    //         let x = n as f32 * 0.01;
    //
    //         let z: ComplexPolar = x.into();
    //         assert_eq!(format!("{}", z), format!("{}", x));
    //         assert_eq!(format!("{:.2}", z), format!("{:.2}", x));
    //     }
    //     assert_eq!(format!("{}", ComplexPolar::ZERO), "0");
    //     assert_eq!(format!("{:+}", ComplexPolar::ZERO), "+0");
    //     assert_eq!(format!("{:+}", -ComplexPolar::ZERO), "-0");
    //     assert_eq!(format!("{}", ComplexPolar::ONE), "1");
    //     assert_eq!(format!("{}", ComplexPolar::NEG_ONE), "1e^iπ");
    //     assert_eq!(format!("{}", ComplexPolar::I), "1e^0.5iπ");
    //     assert_eq!(format!("{}", ComplexPolar::NEG_I), "1e^-0.5iπ");
    //
    //     assert_eq!(format!("{}", 2.0 * ComplexPolar::I), "2e^0.5iπ");
    //     assert_eq!(format!("{}", 2.0 * ComplexPolar::NEG_I), "2e^-0.5iπ");
    //
    //     assert_eq!(format!("{:.2}", ComplexPolar::I), "1.00e^0.50iπ");
    // }
}
