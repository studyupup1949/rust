use super::rectangular::*;
use core::f32::consts::{FRAC_PI_2, LN_2, LN_10, PI, TAU};
use core::fmt;
use core::ops::*;
use core::write;
#[cfg(feature = "libm")]
use num_traits::real::Real;

type Rectangular = Dual;

/// Creates a dual number in polar form.
#[inline(always)]
#[must_use]
pub const fn dual_polar(abs: f32, arg: f32) -> DualPolar {
    DualPolar::new(abs, arg)
}

/// A dual number in polar form.
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct DualPolar {
    pub abs: f32,
    pub arg: f32,
}

impl DualPolar {
    pub const ZERO: Self = dual_polar(0.0, 0.0);
    pub const ONE: Self = dual_polar(1.0, 0.0);
    pub const I: Self = dual_polar(1.0, FRAC_PI_2);
    pub const NEG_ONE: Self = dual_polar(1.0, PI);
    pub const NEG_I: Self = dual_polar(1.0, -FRAC_PI_2);

    /// Creates a dual number.
    pub const fn new(abs: f32, arg: f32) -> Self {
        Self { abs, arg }
    }

    /// Computes the conjugate.
    pub const fn conjugate(self) -> Self {
        dual_polar(self.abs, -self.arg)
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
        dual_polar(self.abs.recip(), -self.arg)
    }

    /// Computes the principle square root.
    pub fn sqrt(self) -> Self {
        dual_polar(self.abs.sqrt(), self.arg / 2.0)
    }

    /// Convert to rectangular form.
    pub fn to_rectangular(self) -> Rectangular {
        let a = self.abs;
        if a == 0.0 {
            return Rectangular::ZERO;
        }
        let b = self.arg / a;
        dual(a, b)
    }

    /// Computes `e^self` where `e` is the base of the natural logarithm.
    pub fn exp(self) -> Self {
        self.to_rectangular().exp()
    }

    /// Computes the principle natural logarithm.
    pub fn ln(self) -> Rectangular {
        dual(self.abs.ln(), self.arg)
    }

    /// Computes the principle logarithm in base 2.
    pub fn log2(self) -> Rectangular {
        self.ln() / LN_2
    }

    /// Computes the principle logarithm in base 10.
    pub fn log10(self) -> Rectangular {
        self.ln() / LN_10
    }

    /// Raises `self` to a floating point power.
    pub fn powf(self, x: f32) -> Self {
        // self.re.powf(x) * Rectangular::new(1.0, (self.im * x) / self.re);
        if x < 0.0 && self.abs == 0.0 {
            return DualPolar::ZERO;
        }
        dual_polar(self.abs.powf(x), self.arg * x)
    }

    /// Raises `self` to an integer power.
    pub fn powi(self, n: i32) -> Self {
        if n < 0 && self.abs == 0.0 {
            return DualPolar::ZERO;
        }
        dual_polar(self.abs.powi(n), self.arg * n as f32)
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

impl Mul for DualPolar {
    type Output = Self;
    fn mul(mut self, other: Self) -> Self::Output {
        self *= other;
        self
    }
}

impl Mul<f32> for DualPolar {
    type Output = Self;
    fn mul(mut self, re: f32) -> Self::Output {
        self *= re;
        self
    }
}

impl Mul<DualPolar> for f32 {
    type Output = DualPolar;
    fn mul(self, mut other: DualPolar) -> Self::Output {
        other *= self;
        other
    }
}

impl MulAssign for DualPolar {
    fn mul_assign(&mut self, other: Self) {
        self.abs *= other.abs;
        self.arg += other.arg;
    }
}

impl MulAssign<f32> for DualPolar {
    fn mul_assign(&mut self, re: f32) {
        self.abs *= re;
    }
}

impl Div for DualPolar {
    type Output = Self;
    fn div(mut self, other: Self) -> Self::Output {
        self /= other;
        self
    }
}

impl Div<f32> for DualPolar {
    type Output = Self;
    fn div(mut self, re: f32) -> Self::Output {
        self /= re;
        self
    }
}

impl Div<DualPolar> for f32 {
    type Output = DualPolar;
    fn div(self, other: DualPolar) -> Self::Output {
        self * other.recip()
    }
}

impl DivAssign for DualPolar {
    fn div_assign(&mut self, other: Self) {
        *self *= other.recip();
    }
}

impl DivAssign<f32> for DualPolar {
    fn div_assign(&mut self, re: f32) {
        self.abs /= re;
    }
}

impl Neg for DualPolar {
    type Output = Self;
    fn neg(mut self) -> Self::Output {
        self.abs = -self.abs;
        self
    }
}

impl fmt::Display for DualPolar {
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
impl rand::distr::Distribution<DualPolar> for rand::distr::StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> DualPolar {
        dual_polar(self.sample(rng), rng.random_range((-PI).next_up()..=PI))
    }
}

#[cfg(feature = "approx")]
use approx::{AbsDiffEq, RelativeEq, UlpsEq};

#[cfg(feature = "approx")]
impl AbsDiffEq for DualPolar {
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
impl RelativeEq for DualPolar {
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
impl UlpsEq for DualPolar {
    fn default_max_ulps() -> u32 {
        f32::default_max_ulps()
    }
    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        f32::ulps_eq(&self.abs, &other.abs, epsilon, max_ulps)
            && f32::ulps_eq(&self.arg, &other.arg, epsilon, max_ulps)
    }
}

impl From<f32> for DualPolar {
    fn from(value: f32) -> Self {
        dual_polar(value, 0.0)
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
        for z0 in random_samples::<DualPolar>() {
            for z1 in random_samples::<DualPolar>() {
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
            assert_eq!(z0 * DualPolar::ONE, z0);
            assert_eq!((z0 * DualPolar::ZERO).abs, 0.0);
            assert_eq!((z0 * 0.0).abs, 0.0);
        }
    }

    #[test]
    fn division() {
        for z0 in random_samples::<DualPolar>() {
            for z1 in random_samples::<DualPolar>() {
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
            assert_ulps_eq!(z0 / z0, DualPolar::ONE);
            assert_eq!((DualPolar::ZERO / z0).abs, 0.0);
        }
    }

    #[test]
    fn negation() {
        assert_eq!((-DualPolar::ONE).normalize(), DualPolar::NEG_ONE);
        assert_eq!((-DualPolar::I).normalize(), DualPolar::NEG_I);
        assert_eq!((-DualPolar::NEG_ONE).normalize(), DualPolar::ONE);
        assert_ulps_eq!((-DualPolar::NEG_I).normalize(), DualPolar::I);
    }

    #[test]
    fn reciprocal() {
        for z in random_samples::<DualPolar>() {
            assert_ulps_eq!(z.recip(), 1.0 / z);
            assert_ulps_eq!(z * z.recip(), DualPolar::ONE);
        }
        assert_eq!(DualPolar::ONE.recip(), DualPolar::ONE);
        assert_eq!(DualPolar::I.recip(), DualPolar::NEG_I);
        assert_eq!(DualPolar::NEG_ONE.recip().normalize(), DualPolar::NEG_ONE);
        assert_eq!(DualPolar::NEG_I.recip(), DualPolar::I);
    }

    #[test]
    fn sqrt() {
        for z in random_samples::<DualPolar>() {
            assert_eq!(z.sqrt().abs, z.abs.sqrt());
            assert_eq!(z.sqrt().arg, z.arg / 2.0);
        }
        assert_eq!(DualPolar::ONE.sqrt(), DualPolar::ONE);
        assert_eq!(DualPolar::NEG_ONE.sqrt(), DualPolar::I);
        assert_eq!(DualPolar::ONE.sqrt(), DualPolar::ONE);
    }

    #[test]
    fn abs() {
        for z in random_samples::<DualPolar>() {
            assert_eq!(z.abs_sq(), z.abs * z.abs);
        }
        assert_eq!(DualPolar::ONE.abs, 1.0);
        assert_eq!(DualPolar::I.abs, 1.0);
        assert_eq!(DualPolar::NEG_ONE.abs, 1.0);
        assert_eq!(DualPolar::NEG_I.abs, 1.0);
    }

    #[test]
    fn conjugate() {
        for z in random_samples::<DualPolar>() {
            assert_eq!(z.conjugate().re(), z.re());
            assert_eq!(z.conjugate().im(), -z.im());
            assert_eq!(z.conjugate().conjugate(), z);
        }
        assert_ulps_eq!(DualPolar::ONE.conjugate().normalize(), DualPolar::ONE);
        assert_ulps_eq!(DualPolar::I.conjugate().normalize(), DualPolar::NEG_I);
        assert_ulps_eq!(
            DualPolar::NEG_ONE.conjugate().normalize(),
            DualPolar::NEG_ONE
        );
        assert_ulps_eq!(DualPolar::NEG_I.conjugate().normalize(), DualPolar::I);
    }

    #[test]
    fn arg() {
        assert_eq!(DualPolar::ONE.arg, 0.0);
        assert_eq!(DualPolar::I.arg, FRAC_PI_2);
        assert_eq!(DualPolar::NEG_ONE.arg, PI);
        assert_eq!(DualPolar::NEG_I.arg, -FRAC_PI_2);
    }

    #[test]
    fn exp() {
        for z in random_samples::<DualPolar>() {
            assert_eq!(z.exp().abs, z.re().exp());
            assert_eq!(z.exp().arg, z.im());
            assert_ulps_eq!(z.exp().ln(), z.to_rectangular());
        }
        assert_ulps_eq!(DualPolar::ONE.exp(), dual_polar(E, 0.0));
        assert_ulps_eq!(DualPolar::I.exp(), dual_polar(1.0, 1.0));
        assert_ulps_eq!(DualPolar::NEG_ONE.exp(), dual_polar(E.recip(), 0.0));
        assert_ulps_eq!(DualPolar::NEG_I.exp(), dual_polar(1.0, -1.0));
    }

    #[test]
    fn log() {
        for z in random_samples::<DualPolar>() {
            assert_eq!(z.ln().re, z.abs.ln());
            assert_eq!(z.ln().im, z.arg);
            assert_ulps_eq!(z.ln().exp(), z);
        }
        assert_eq!(DualPolar::ONE.ln(), Rectangular::ZERO);
        assert_eq!(DualPolar::I.ln(), Rectangular::I * FRAC_PI_2);
        assert_eq!(DualPolar::NEG_ONE.ln(), Rectangular::I * PI);
        assert_eq!(DualPolar::NEG_I.ln(), Rectangular::I * -FRAC_PI_2);

        assert_ulps_eq!(dual_polar(E, 0.0).ln(), Rectangular::ONE);
        assert_ulps_eq!(dual_polar(2.0, 0.0).log2(), Rectangular::ONE);
        assert_ulps_eq!(dual_polar(10.0, 0.0).log10(), Rectangular::ONE);
    }

    #[test]
    fn powi() {
        for z in random_samples::<DualPolar>() {
            assert_eq!(z.powi(0), DualPolar::ONE);
            assert_eq!(z.powi(1), z);
            for n in random_samples::<i32>() {
                assert_eq!(z.powi(n).abs, z.abs.powi(n));
                assert_eq!(z.powi(n).arg, z.arg * n as f32);
            }
        }
        for n in random_samples::<i32>() {
            assert_eq!(DualPolar::ZERO.powi(n.abs()), DualPolar::ZERO);
            assert_eq!(DualPolar::ONE.powi(n), DualPolar::ONE);
        }
    }

    #[test]
    fn powf() {
        for z in random_samples::<DualPolar>() {
            assert_eq!(z.powf(0.0), DualPolar::ONE);
            assert_eq!(z.powf(1.0), z);
            for n in random_samples::<i32>() {
                let x = n as f32 * 0.01;
                assert_eq!(z.powf(x).abs, z.abs.powf(x));
                assert_eq!(z.powf(x).arg, z.arg * x);
            }
        }
        for n in random_samples::<i32>() {
            let x = n as f32 * 0.01;
            assert_eq!(DualPolar::ZERO.powf(x.abs()), DualPolar::ZERO);
            assert_eq!(DualPolar::ONE.powf(x), DualPolar::ONE);
        }
    }

    #[test]
    fn normalize() {
        for z in random_samples::<DualPolar>() {
            for n in uniform_samples::<i32>(-99, 99) {
                let w = dual_polar(z.abs, z.arg + n as f32 * TAU);
                assert_ulps_eq!(z, w.normalize(), epsilon = 2000.0 * f32::EPSILON);

                assert_ulps_eq!(
                    dual_polar(-z.abs, z.arg).normalize(),
                    dual_polar(z.abs, z.arg + PI).normalize(),
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
