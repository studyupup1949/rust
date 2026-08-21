use super::polar::*;
use core::f32::consts::{LN_2, LN_10};
use core::fmt;
use core::ops::*;
use core::write;
#[cfg(feature = "libm")]
use num_traits::real::Real;

type Polar = DualPolar;

/// Creates a dual number in rectangular form.
#[inline(always)]
#[must_use]
pub const fn dual(re: f32, im: f32) -> Dual {
    Dual::new(re, im)
}

/// A dual number in rectangular form.
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Dual {
    pub re: f32,
    pub im: f32,
}

impl Dual {
    pub const ZERO: Self = dual(0.0, 0.0);
    pub const ONE: Self = dual(1.0, 0.0);
    pub const NEG_ONE: Self = dual(-1.0, 0.0);
    pub const I: Self = dual(0.0, 1.0);
    pub const NEG_I: Self = dual(0.0, -1.0);

    /// Creates a dual number.
    pub const fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }

    /// Computes the conjugate.
    pub const fn conjugate(self) -> Self {
        dual(self.re, -self.im)
    }

    /// Computes the absolute value.
    pub fn abs(self) -> f32 {
        self.re.abs()
    }

    /// Computes the squared absolute value.
    pub fn abs_sq(self) -> f32 {
        self.re * self.re
    }

    /// Computes the argument in the range `(-∞, +∞)`.
    pub fn arg(self) -> f32 {
        if self.re == 0.0 {
            return f32::MAX;
        }
        self.im / self.re
    }

    /// Computes the reciprocal.
    pub fn recip(self) -> Self {
        self.conjugate() / self.abs_sq()
    }

    /// Computes the principle square root.
    pub fn sqrt(self) -> Self {
        let abs = self.abs();
        dual(
            (0.5 * (abs + self.re)).sqrt(),
            (0.5 * (abs - self.re)).sqrt().copysign(self.im),
        )
    }

    /// Convert to polar form.
    pub fn to_polar(self) -> Polar {
        dual_polar(self.abs(), self.arg())
    }

    /// Computes `e^self` where `e` is the base of the natural logarithm.
    pub fn exp(self) -> Polar {
        dual_polar(self.re.exp(), self.im)
    }

    /// Computes the principle natural logarithm.
    pub fn ln(self) -> Self {
        self.to_polar().ln()
    }

    /// Computes the principle logarithm in base 2.
    pub fn log2(self) -> Self {
        self.ln() / LN_2
    }

    /// Computes the principle logarithm in base 10.
    pub fn log10(self) -> Self {
        self.ln() / LN_10
    }

    /// Raises `self` to an integer power.
    pub fn powi(self, n: i32) -> Polar {
        self.to_polar().powi(n)
    }

    /// Raises `self` to a floating point power.
    pub fn powf(self, x: f32) -> Polar {
        self.to_polar().powf(x)
    }

    /// Computes the euclidian distance between two points.
    pub fn distance(self, other: Self) -> f32 {
        (self - other).abs()
    }

    /// Computes the squared euclidian distance between two points.
    pub fn distance_squared(self, other: Self) -> f32 {
        (self - other).abs_sq()
    }

    /// Casts to a glam::Vec2.
    #[cfg(feature = "glam")]
    pub fn as_vec2(self) -> glam::Vec2 {
        glam::vec2(self.re, self.im)
    }
}

impl Add for Dual {
    type Output = Self;
    fn add(mut self, other: Self) -> Self::Output {
        self += other;
        self
    }
}

impl Add<f32> for Dual {
    type Output = Self;
    fn add(mut self, re: f32) -> Self::Output {
        self += re;
        self
    }
}

impl Add<Dual> for f32 {
    type Output = Dual;
    fn add(self, mut z: Dual) -> Self::Output {
        z += self;
        z
    }
}

impl AddAssign for Dual {
    fn add_assign(&mut self, other: Self) {
        self.re += other.re;
        self.im += other.im;
    }
}

impl AddAssign<f32> for Dual {
    fn add_assign(&mut self, re: f32) {
        self.re += re;
    }
}

impl Sub for Dual {
    type Output = Self;
    fn sub(mut self, other: Self) -> Self::Output {
        self -= other;
        self
    }
}

impl Sub<f32> for Dual {
    type Output = Self;
    fn sub(mut self, re: f32) -> Self::Output {
        self -= re;
        self
    }
}

impl Sub<Dual> for f32 {
    type Output = Dual;
    fn sub(self, z: Dual) -> Self::Output {
        dual(self - z.re, -z.im)
    }
}

impl SubAssign for Dual {
    fn sub_assign(&mut self, other: Self) {
        self.re -= other.re;
        self.im -= other.im;
    }
}

impl SubAssign<f32> for Dual {
    fn sub_assign(&mut self, re: f32) {
        self.re -= re;
    }
}

impl Mul for Dual {
    type Output = Self;
    fn mul(mut self, other: Self) -> Self::Output {
        self *= other;
        self
    }
}

impl Mul<f32> for Dual {
    type Output = Self;
    fn mul(mut self, re: f32) -> Self::Output {
        self *= re;
        self
    }
}

impl Mul<Dual> for f32 {
    type Output = Dual;
    fn mul(self, mut other: Dual) -> Self::Output {
        other *= self;
        other
    }
}

impl MulAssign for Dual {
    fn mul_assign(&mut self, other: Self) {
        let re = self.re * other.re;
        self.im = self.re * other.im + self.im * other.re;
        self.re = re;
    }
}

impl MulAssign<f32> for Dual {
    fn mul_assign(&mut self, re: f32) {
        self.re *= re;
        self.im *= re;
    }
}

impl Div for Dual {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        self * other.recip()
    }
}

impl Div<f32> for Dual {
    type Output = Self;
    fn div(mut self, re: f32) -> Self::Output {
        self /= re;
        self
    }
}

impl Div<Dual> for f32 {
    type Output = Dual;
    fn div(self, other: Dual) -> Self::Output {
        self * other.recip()
    }
}

impl DivAssign for Dual {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

impl DivAssign<f32> for Dual {
    fn div_assign(&mut self, re: f32) {
        self.re /= re;
        self.im /= re;
    }
}

impl Neg for Dual {
    type Output = Self;
    fn neg(self) -> Self::Output {
        dual(-self.re, -self.im)
    }
}

impl fmt::Display for Dual {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn fmt_x(f: &mut fmt::Formatter, x: f32, sign_plus: bool) -> fmt::Result {
            match (f.precision(), sign_plus) {
                (None, false) => write!(f, "{}", x),
                (None, true) => write!(f, "{:+}", x),
                (Some(p), false) => write!(f, "{:.*}", p, x),
                (Some(p), true) => write!(f, "{:+.*}", p, x),
            }
        }
        match (self.re, self.im, f.sign_plus()) {
            (re, 0.0, sp) => fmt_x(f, re, sp),
            (0.0, 1.0, false) => write!(f, "i"),
            (0.0, 1.0, true) => write!(f, "+i"),
            (0.0, -1.0, _) => write!(f, "-i"),
            (0.0, im, sp) => {
                fmt_x(f, im, sp)?;
                write!(f, "i")
            }
            (re, 1.0, sp) => {
                fmt_x(f, re, sp)?;
                write!(f, "+i")
            }
            (re, -1.0, sp) => {
                fmt_x(f, re, sp)?;
                write!(f, "-i")
            }
            (re, im, sp) => {
                fmt_x(f, re, sp)?;
                fmt_x(f, im, true)?;
                write!(f, "i")
            }
        }
    }
}

#[cfg(feature = "rand")]
impl rand::distr::Distribution<Dual> for rand::distr::StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Dual {
        rng.sample::<Polar, _>(self).to_rectangular()
    }
}

#[cfg(feature = "approx")]
use approx::{AbsDiffEq, RelativeEq, UlpsEq};

#[cfg(feature = "approx")]
impl AbsDiffEq for Dual {
    type Epsilon = <f32 as AbsDiffEq>::Epsilon;
    fn default_epsilon() -> Self::Epsilon {
        f32::default_epsilon()
    }
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        f32::abs_diff_eq(&self.re, &other.re, epsilon)
            && f32::abs_diff_eq(&self.im, &other.im, epsilon)
    }
}

#[cfg(feature = "approx")]
impl RelativeEq for Dual {
    fn default_max_relative() -> Self::Epsilon {
        f32::default_max_relative()
    }
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        f32::relative_eq(&self.re, &other.re, epsilon, max_relative)
            && f32::relative_eq(&self.im, &other.im, epsilon, max_relative)
    }
}

#[cfg(feature = "approx")]
impl UlpsEq for Dual {
    fn default_max_ulps() -> u32 {
        f32::default_max_ulps()
    }
    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        f32::ulps_eq(&self.re, &other.re, epsilon, max_ulps)
            && f32::ulps_eq(&self.im, &other.im, epsilon, max_ulps)
    }
}

impl From<f32> for Dual {
    fn from(value: f32) -> Self {
        dual(value, 0.0)
    }
}

#[cfg(feature = "glam")]
impl From<glam::Vec2> for Dual {
    fn from(v: glam::Vec2) -> Self {
        dual(v.x, v.y)
    }
}

#[cfg(feature = "glam")]
impl From<Dual> for glam::Vec2 {
    fn from(z: Dual) -> Self {
        z.as_vec2()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;
    use core::f32::consts::{E, FRAC_PI_2, PI};
    use core::iter::Iterator;
    use rand::{
        Rng, SeedableRng,
        distr::{Distribution, StandardUniform},
        rngs::StdRng,
    };

    const NUM_SAMPLES: usize = 100;

    fn random_samples<T>() -> impl Iterator<Item = T>
    where
        StandardUniform: Distribution<T>,
    {
        StdRng::seed_from_u64(21)
            .sample_iter(StandardUniform)
            .take(NUM_SAMPLES)
    }

    #[test]
    fn addition() {
        for z0 in random_samples::<Dual>() {
            for z1 in random_samples::<Dual>() {
                let z = z0 + z1;
                assert_eq!(z.re, z0.re + z1.re);
                assert_eq!(z.im, z0.im + z1.im);

                let z = z0 + z1.re;
                assert_eq!(z.re, z0.re + z1.re);
                assert_eq!(z.im, z0.im);

                let z = z0.re + z1;
                assert_eq!(z.re, z0.re + z1.re);
                assert_eq!(z.im, z1.im);

                let mut z = z0;
                z += z1;
                assert_eq!(z, z0 + z1);

                let mut z = z0;
                z += z1.re;
                assert_eq!(z, z0 + z1.re);
            }
            assert_eq!(z0 + Dual::ZERO, z0);
        }
    }

    #[test]
    fn subtraction() {
        for z0 in random_samples::<Dual>() {
            for z1 in random_samples::<Dual>() {
                let z = z0 - z1;
                assert_eq!(z.re, z0.re - z1.re);
                assert_eq!(z.im, z0.im - z1.im);

                let z = z0 - z1.re;
                assert_eq!(z.re, z0.re - z1.re);
                assert_eq!(z.im, z0.im);

                let z = z0.re - z1;
                assert_eq!(z.re, z0.re - z1.re);
                assert_eq!(z.im, -z1.im);

                let mut z = z0;
                z -= z1;
                assert_eq!(z, z0 - z1);

                let mut z = z0;
                z -= z1.re;
                assert_eq!(z, z0 - z1.re);
            }
            assert_eq!(z0 - z0, Dual::ZERO);
            assert_eq!(z0 - Dual::ZERO, z0);
        }
    }

    #[test]
    fn multiplication() {
        for z0 in random_samples::<Dual>() {
            for z1 in random_samples::<Dual>() {
                let z = z0 * z1;
                assert_ulps_eq!(z.abs(), z0.abs() * z1.abs());
                assert_ulps_eq!(
                    z.arg().sin(),
                    (z0.arg() + z1.arg()).sin(),
                    epsilon = 4.0 * f32::EPSILON
                );

                let z = z0 * z1.re;
                assert_eq!(z.re, z0.re * z1.re);
                assert_eq!(z.im, z0.im * z1.re);

                let z = z0.re * z1;
                assert_eq!(z.re, z0.re * z1.re);
                assert_eq!(z.im, z0.re * z1.im);

                let mut z = z0;
                z *= z1;
                assert_eq!(z, z0 * z1);

                let mut z = z0;
                z *= z1.re;
                assert_eq!(z, z0 * z1.re);
            }
            assert_eq!(z0 * Dual::ONE, z0);
            assert_eq!(z0 * Dual::ZERO, Dual::ZERO);
            assert_eq!(z0 * 0.0, Dual::ZERO);
        }
    }

    #[test]
    fn division() {
        for z0 in random_samples::<Dual>() {
            for z1 in random_samples::<Dual>() {
                let z = z0 / z1;
                assert_relative_eq!(
                    z.abs(),
                    z0.abs() / z1.abs(),
                    max_relative = 3.0 * f32::EPSILON
                );
                assert_ulps_eq!(
                    z.arg().sin(),
                    (z0.arg() - z1.arg()).sin(),
                    epsilon = 4.0 * f32::EPSILON
                );

                let z = z0 / z1.re;
                assert_eq!(z.re, z0.re / z1.re);
                assert_eq!(z.im, z0.im / z1.re);

                let z = z0.re / z1;
                assert_ulps_eq!(z.abs(), z0.re.abs() / z1.abs());
                assert_ulps_eq!(
                    z.arg().sin(),
                    (-z0.re.signum() * z1.arg()).sin(),
                    epsilon = 2.0 * f32::EPSILON
                );

                let mut z = z0;
                z /= z1;
                assert_eq!(z, z0 / z1);

                let mut z = z0;
                z /= z1.re;
                assert_eq!(z, z0 / z1.re);
            }
            assert_ulps_eq!(z0 / z0, Dual::ONE);
            assert_eq!(Dual::ZERO / z0, Dual::ZERO);
        }
    }

    #[test]
    fn negation() {
        for z in random_samples::<Dual>() {
            assert_eq!(-z, dual(-z.re, -z.im));
        }
        assert_eq!(-Dual::ONE, Dual::NEG_ONE);
        assert_eq!(-Dual::I, Dual::NEG_I);
        assert_eq!(-Dual::NEG_ONE, Dual::ONE);
        assert_eq!(-Dual::NEG_I, Dual::I);
    }

    #[test]
    fn reciprocal() {
        for z in random_samples::<Dual>() {
            assert_eq!(z.recip(), 1.0 / z);
            assert_ulps_eq!(z * z.recip(), Dual::ONE);
        }
        assert_eq!(Dual::ONE.recip(), Dual::ONE);
        assert_eq!(Dual::I.recip(), Dual::NEG_I);
        assert_eq!(Dual::NEG_ONE.recip(), Dual::NEG_ONE);
        assert_eq!(Dual::NEG_I.recip(), Dual::I);
    }

    #[test]
    fn sqrt() {
        for z in random_samples::<Dual>() {
            assert_ulps_eq!(z.sqrt().abs(), z.abs().sqrt());
            assert_ulps_eq!(
                z.sqrt().arg(),
                z.arg() / 2.0,
                epsilon = 1400.0 * f32::EPSILON
            );
        }
        assert_eq!(Dual::ONE.sqrt(), Dual::ONE);
        assert_eq!(Dual::NEG_ONE.sqrt(), Dual::I);
        assert_eq!(dual(0.0, 2.0).sqrt(), dual(1.0, 1.0));
        assert_eq!(dual(0.0, -2.0).sqrt(), dual(1.0, -1.0));
    }

    #[test]
    fn abs() {
        for z in random_samples::<Dual>() {
            assert_ulps_eq!(z.abs_sq(), z.abs() * z.abs());
            assert_eq!(z.abs_sq(), z.re * z.re);
        }
        assert_eq!(Dual::ONE.abs(), 1.0);
        assert_eq!(Dual::I.abs(), 0.0);
        assert_eq!(Dual::NEG_ONE.abs(), 1.0);
        assert_eq!(Dual::NEG_I.abs(), 0.0);
        assert_eq!(dual(1.0, 1.0).abs(), 1.0);
        assert_eq!(dual(-1.0, 1.0).abs(), 1.0);
        assert_eq!(dual(-1.0, -1.0).abs(), 1.0);
        assert_eq!(dual(1.0, -1.0).abs(), 1.0);
    }

    #[test]
    fn conjugate() {
        for z in random_samples::<Dual>() {
            assert_eq!(z.conjugate().re, z.re);
            assert_eq!(z.conjugate().im, -z.im);
            assert_eq!(z.conjugate().conjugate(), z);
        }
        assert_eq!(Dual::ONE.conjugate(), Dual::ONE);
        assert_eq!(Dual::I.conjugate(), Dual::NEG_I);
        assert_eq!(Dual::NEG_ONE.conjugate(), Dual::NEG_ONE);
        assert_eq!(Dual::NEG_I.conjugate(), Dual::I);
    }

    #[test]
    fn arg() {
        assert_eq!(Dual::ONE.arg(), 0.0);
        assert_eq!(Dual::I.arg(), FRAC_PI_2);
        assert_eq!(Dual::NEG_ONE.arg(), PI);
        assert_eq!(Dual::NEG_I.arg(), -FRAC_PI_2);
    }

    #[test]
    fn exp() {
        for z in random_samples::<Dual>() {
            assert_eq!(z.exp().abs, z.re.exp());
            assert_eq!(z.exp().arg, z.im);
            assert_ulps_eq!(z.exp().ln(), z);
        }
        assert_eq!(Dual::ONE.exp(), dual_polar(E, 0.0));
        assert_eq!(Dual::I.exp(), dual_polar(1.0, 1.0));
        assert_eq!(Dual::NEG_ONE.exp(), dual_polar(E.recip(), 0.0));
        assert_eq!(Dual::NEG_I.exp(), dual_polar(1.0, -1.0));
    }

    #[test]
    fn log() {
        for z in random_samples::<Dual>() {
            assert_eq!(z.ln().re, z.abs().ln());
            assert_eq!(z.ln().im, z.arg());
            assert_ulps_eq!(z.ln().exp(), z.to_polar());
        }
        assert_eq!(Dual::ONE.ln(), Dual::ZERO);
        assert_eq!(Dual::I.ln(), Dual::I * FRAC_PI_2);
        assert_eq!(Dual::NEG_ONE.ln(), Dual::I * PI);
        assert_eq!(Dual::NEG_I.ln(), Dual::I * -FRAC_PI_2);

        assert_ulps_eq!(dual(E, 0.0).ln(), Dual::ONE);
        assert_ulps_eq!(dual(2.0, 0.0).log2(), Dual::ONE);
        assert_ulps_eq!(dual(10.0, 0.0).log10(), Dual::ONE);
    }

    #[test]
    fn powi() {
        for z in random_samples::<Dual>() {
            assert_eq!(z.powi(0), Polar::ONE);
            assert_eq!(z.powi(1), z.to_polar());
            for n in random_samples::<i32>() {
                assert_eq!(z.powi(n).abs, z.abs().powi(n));
                assert_eq!(z.powi(n).arg, z.arg() * n as f32);
            }
        }
        for n in random_samples::<i32>() {
            assert_eq!(Dual::ZERO.powi(n.abs()), Polar::ZERO);
            assert_eq!(Dual::ONE.powi(n), Polar::ONE);
        }
    }

    #[test]
    fn powf() {
        for z in random_samples::<Dual>() {
            assert_eq!(z.powf(0.0), Polar::ONE);
            assert_eq!(z.powf(1.0), z.to_polar());
            for n in random_samples::<i32>() {
                let x = n as f32 * 0.01;
                assert_eq!(z.powf(x).abs, z.abs().powf(x));
                assert_eq!(z.powf(x).arg, z.arg() * x);
            }
        }
        for n in random_samples::<i32>() {
            let x = n as f32 * 0.01;
            assert_eq!(Dual::ZERO.powf(x.abs()), Polar::ZERO);
            assert_eq!(Dual::ONE.powf(x), Polar::ONE);
        }
    }

    // #[test]
    // fn format_display() {
    //     for n in random_samples::<i32>() {
    //         let x = n as f32 * 0.01;
    //
    //         let z: Complex = x.into();
    //         assert_eq!(format!("{}", z), format!("{}", x));
    //         assert_eq!(format!("{:+}", z), format!("{:+}", x));
    //
    //         let z = complex(0.0, x);
    //         assert_eq!(format!("{}", z), format!("{}i", x));
    //         assert_eq!(format!("{:+}", z), format!("{:+}i", x));
    //     }
    //     assert_eq!(format!("{}", Complex::ZERO), "0");
    //     assert_eq!(format!("{}", Complex::ONE), "1");
    //     assert_eq!(format!("{}", Complex::NEG_ONE), "-1");
    //     assert_eq!(format!("{}", Complex::I), "i");
    //     assert_eq!(format!("{}", Complex::NEG_I), "-i");
    //
    //     assert_eq!(format!("{:+}", Complex::ZERO), "+0");
    //     assert_eq!(format!("{:+}", -Complex::ZERO), "-0");
    //     assert_eq!(format!("{:+}", Complex::ONE), "+1");
    //     assert_eq!(format!("{}", 2.0 * Complex::I), "2i");
    //     assert_eq!(format!("{}", 2.0 * Complex::NEG_I), "-2i");
    //
    //     assert_eq!(format!("{:+}", Complex::I), "+i");
    //     assert_eq!(format!("{:+}", 2.0 * Complex::I), "+2i");
    //
    //     assert_eq!(format!("{}", complex(1.0, 1.0)), "1+i");
    //     assert_eq!(format!("{}", complex(-1.0, 1.0)), "-1+i");
    //     assert_eq!(format!("{}", complex(-1.0, -1.0)), "-1-i");
    //     assert_eq!(format!("{}", complex(1.0, -1.0)), "1-i");
    //
    //     assert_eq!(format!("{}", complex(2.0, 2.0)), "2+2i");
    //     assert_eq!(format!("{}", complex(-2.0, 2.0)), "-2+2i");
    //     assert_eq!(format!("{}", complex(-2.0, -2.0)), "-2-2i");
    //     assert_eq!(format!("{}", complex(2.0, -2.0)), "2-2i");
    //
    //     assert_eq!(format!("{}", complex(1.234, 1.234)), "1.234+1.234i");
    //     assert_eq!(format!("{:.2}", complex(1.234, 1.234)), "1.23+1.23i");
    //     assert_eq!(format!("{:+}", complex(1.234, 1.234)), "+1.234+1.234i");
    //     assert_eq!(format!("{:+.2}", complex(1.234, 1.234)), "+1.23+1.23i");
    // }
}
