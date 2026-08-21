use crate::polar::*;
use core::fmt;
use core::ops::*;
use core::write;
#[cfg(feature = "libm")]
use num_traits::real::Real;

/// Creates a complex number in rectangular form.
#[inline(always)]
#[must_use]
pub const fn complex(re: f32, im: f32) -> Complex {
    Complex::new(re, im)
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Complex {
    pub re: f32,
    pub im: f32,
}

impl Complex {
    pub const ZERO: Self = complex(0.0, 0.0);
    pub const ONE: Self = complex(1.0, 0.0);
    pub const NEG_ONE: Self = complex(-1.0, 0.0);
    pub const I: Self = complex(0.0, 1.0);
    pub const NEG_I: Self = complex(0.0, -1.0);

    pub const fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }

    pub const fn conjugate(self) -> Self {
        complex(self.re, -self.im)
    }

    pub fn abs(self) -> f32 {
        self.abs_sq().sqrt()
    }

    pub fn abs_sq(self) -> f32 {
        self.re * self.re + self.im * self.im
    }

    pub fn arg(self) -> f32 {
        self.im.atan2(self.re)
    }

    pub fn recip(self) -> Self {
        self.conjugate() / self.abs_sq()
    }

    pub fn sqrt(self) -> Self {
        let abs = self.abs();
        complex(
            (0.5 * (abs + self.re)).sqrt(),
            (0.5 * (abs - self.re)).sqrt().copysign(self.im),
        )
    }

    pub fn to_polar(self) -> ComplexPolar {
        complex_polar(self.abs(), self.arg())
    }

    pub fn exp(self) -> ComplexPolar {
        complex_polar(self.re.exp(), self.im)
    }

    pub fn log(self) -> Self {
        self.to_polar().log()
    }

    pub fn powi(self, n: i32) -> ComplexPolar {
        self.to_polar().powi(n)
    }

    pub fn powf(self, x: f32) -> ComplexPolar {
        self.to_polar().powf(x)
    }

    pub fn distance(self, other: Self) -> f32 {
        (self - other).abs()
    }

    pub fn distance_squared(self, other: Self) -> f32 {
        (self - other).abs_sq()
    }
}

impl Add for Complex {
    type Output = Self;
    fn add(mut self, other: Self) -> Self::Output {
        self += other;
        self
    }
}

impl Add<f32> for Complex {
    type Output = Self;
    fn add(mut self, re: f32) -> Self::Output {
        self += re;
        self
    }
}

impl Add<Complex> for f32 {
    type Output = Complex;
    fn add(self, mut z: Complex) -> Self::Output {
        z += self;
        z
    }
}

impl AddAssign for Complex {
    fn add_assign(&mut self, other: Self) {
        self.re += other.re;
        self.im += other.im;
    }
}

impl AddAssign<f32> for Complex {
    fn add_assign(&mut self, re: f32) {
        self.re += re;
    }
}

impl Sub for Complex {
    type Output = Self;
    fn sub(mut self, other: Self) -> Self::Output {
        self -= other;
        self
    }
}

impl Sub<f32> for Complex {
    type Output = Self;
    fn sub(mut self, re: f32) -> Self::Output {
        self -= re;
        self
    }
}

impl Sub<Complex> for f32 {
    type Output = Complex;
    fn sub(self, z: Complex) -> Self::Output {
        complex(self - z.re, -z.im)
    }
}

impl SubAssign for Complex {
    fn sub_assign(&mut self, other: Self) {
        self.re -= other.re;
        self.im -= other.im;
    }
}

impl SubAssign<f32> for Complex {
    fn sub_assign(&mut self, re: f32) {
        self.re -= re;
    }
}

impl Mul for Complex {
    type Output = Self;
    fn mul(mut self, other: Self) -> Self::Output {
        self *= other;
        self
    }
}

impl Mul<f32> for Complex {
    type Output = Self;
    fn mul(mut self, re: f32) -> Self::Output {
        self *= re;
        self
    }
}

impl Mul<Complex> for f32 {
    type Output = Complex;
    fn mul(self, mut other: Complex) -> Self::Output {
        other *= self;
        other
    }
}

impl MulAssign for Complex {
    fn mul_assign(&mut self, other: Self) {
        let re = self.re * other.re - self.im * other.im;
        self.im = self.re * other.im + self.im * other.re;
        self.re = re;
    }
}

impl MulAssign<f32> for Complex {
    fn mul_assign(&mut self, re: f32) {
        self.re *= re;
        self.im *= re;
    }
}

impl Div for Complex {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        self * other.conjugate() / other.abs_sq()
    }
}

impl Div<f32> for Complex {
    type Output = Self;
    fn div(mut self, re: f32) -> Self::Output {
        self /= re;
        self
    }
}

impl Div<Complex> for f32 {
    type Output = Complex;
    fn div(self, other: Complex) -> Self::Output {
        self * other.conjugate() / other.abs_sq()
    }
}

impl DivAssign for Complex {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

impl DivAssign<f32> for Complex {
    fn div_assign(&mut self, re: f32) {
        self.re /= re;
        self.im /= re;
    }
}

impl Neg for Complex {
    type Output = Self;
    fn neg(self) -> Self::Output {
        complex(-self.re, -self.im)
    }
}

impl fmt::Display for Complex {
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
impl rand::distr::Distribution<Complex> for rand::distr::StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Complex {
        rng.sample::<ComplexPolar, _>(self).to_rectangular()
    }
}

#[cfg(feature = "approx")]
use approx::{AbsDiffEq, RelativeEq, UlpsEq};

#[cfg(feature = "approx")]
impl AbsDiffEq for Complex {
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
impl RelativeEq for Complex {
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
impl UlpsEq for Complex {
    fn default_max_ulps() -> u32 {
        f32::default_max_ulps()
    }
    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        f32::ulps_eq(&self.re, &other.re, epsilon, max_ulps)
            && f32::ulps_eq(&self.im, &other.im, epsilon, max_ulps)
    }
}

impl From<f32> for Complex {
    fn from(value: f32) -> Self {
        complex(value, 0.0)
    }
}

#[cfg(feature = "glam")]
impl From<glam::Vec2> for Complex {
    fn from(v: glam::Vec2) -> Self {
        complex(v.x, v.y)
    }
}

#[cfg(feature = "glam")]
impl From<Complex> for glam::Vec2 {
    fn from(z: Complex) -> Self {
        glam::vec2(z.re, z.im)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;
    use core::f32::consts::{E, FRAC_PI_2, PI, SQRT_2};
    use core::iter::Iterator;
    use rand::{
        distr::{Distribution, StandardUniform},
        rngs::StdRng,
        Rng, SeedableRng,
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
        for z0 in random_samples::<Complex>() {
            for z1 in random_samples::<Complex>() {
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
            assert_eq!(z0 + Complex::ZERO, z0);
        }
    }

    #[test]
    fn subtraction() {
        for z0 in random_samples::<Complex>() {
            for z1 in random_samples::<Complex>() {
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
            assert_eq!(z0 - z0, Complex::ZERO);
            assert_eq!(z0 - Complex::ZERO, z0);
        }
    }

    #[test]
    fn multiplication() {
        for z0 in random_samples::<Complex>() {
            for z1 in random_samples::<Complex>() {
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
            assert_eq!(z0 * Complex::ONE, z0);
            assert_eq!(z0 * Complex::ZERO, Complex::ZERO);
            assert_eq!(z0 * 0.0, Complex::ZERO);
        }
    }

    #[test]
    fn division() {
        for z0 in random_samples::<Complex>() {
            for z1 in random_samples::<Complex>() {
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
            assert_eq!(z0 / z0, Complex::ONE);
            assert_eq!(Complex::ZERO / z0, Complex::ZERO);
        }
    }

    #[test]
    fn negation() {
        for z in random_samples::<Complex>() {
            assert_eq!(-z, complex(-z.re, -z.im));
        }
        assert_eq!(-Complex::ONE, Complex::NEG_ONE);
        assert_eq!(-Complex::I, Complex::NEG_I);
        assert_eq!(-Complex::NEG_ONE, Complex::ONE);
        assert_eq!(-Complex::NEG_I, Complex::I);
    }

    #[test]
    fn reciprocal() {
        for z in random_samples::<Complex>() {
            assert_eq!(z.recip(), 1.0 / z);
            assert_ulps_eq!(z * z.recip(), Complex::ONE);
        }
        assert_eq!(Complex::ONE.recip(), Complex::ONE);
        assert_eq!(Complex::I.recip(), Complex::NEG_I);
        assert_eq!(Complex::NEG_ONE.recip(), Complex::NEG_ONE);
        assert_eq!(Complex::NEG_I.recip(), Complex::I);
    }

    #[test]
    fn sqrt() {
        for z in random_samples::<Complex>() {
            assert_ulps_eq!(z.sqrt().abs(), z.abs().sqrt());
            assert_ulps_eq!(
                z.sqrt().arg(),
                z.arg() / 2.0,
                epsilon = 1400.0 * f32::EPSILON
            );
        }
        assert_eq!(Complex::ONE.sqrt(), Complex::ONE);
        assert_eq!(Complex::NEG_ONE.sqrt(), Complex::I);
        assert_eq!(complex(0.0, 2.0).sqrt(), complex(1.0, 1.0));
        assert_eq!(complex(0.0, -2.0).sqrt(), complex(1.0, -1.0));
    }

    #[test]
    fn abs() {
        for z in random_samples::<Complex>() {
            assert_ulps_eq!(z.abs_sq(), z.abs() * z.abs());
            assert_eq!(z.abs_sq(), z.re * z.re + z.im * z.im);
        }
        assert_eq!(Complex::ONE.abs(), 1.0);
        assert_eq!(Complex::I.abs(), 1.0);
        assert_eq!(Complex::NEG_ONE.abs(), 1.0);
        assert_eq!(Complex::NEG_I.abs(), 1.0);
        assert_eq!(complex(1.0, 1.0).abs(), SQRT_2);
        assert_eq!(complex(-1.0, 1.0).abs(), SQRT_2);
        assert_eq!(complex(-1.0, -1.0).abs(), SQRT_2);
        assert_eq!(complex(1.0, -1.0).abs(), SQRT_2);
    }

    #[test]
    fn conjugate() {
        for z in random_samples::<Complex>() {
            assert_eq!(z.conjugate().re, z.re);
            assert_eq!(z.conjugate().im, -z.im);
            assert_eq!(z.conjugate().conjugate(), z);
        }
        assert_eq!(Complex::ONE.conjugate(), Complex::ONE);
        assert_eq!(Complex::I.conjugate(), Complex::NEG_I);
        assert_eq!(Complex::NEG_ONE.conjugate(), Complex::NEG_ONE);
        assert_eq!(Complex::NEG_I.conjugate(), Complex::I);
    }

    #[test]
    fn arg() {
        assert_eq!(Complex::ONE.arg(), 0.0);
        assert_eq!(Complex::I.arg(), FRAC_PI_2);
        assert_eq!(Complex::NEG_ONE.arg(), PI);
        assert_eq!(Complex::NEG_I.arg(), -FRAC_PI_2);
    }

    #[test]
    fn exp() {
        for z in random_samples::<Complex>() {
            assert_eq!(z.exp().abs, z.re.exp());
            assert_eq!(z.exp().arg, z.im);
            assert_ulps_eq!(z.exp().log(), z);
        }
        assert_eq!(Complex::ONE.exp(), complex_polar(E, 0.0));
        assert_eq!(Complex::I.exp(), complex_polar(1.0, 1.0));
        assert_eq!(Complex::NEG_ONE.exp(), complex_polar(E.recip(), 0.0));
        assert_eq!(Complex::NEG_I.exp(), complex_polar(1.0, -1.0));
    }

    #[test]
    fn log() {
        for z in random_samples::<Complex>() {
            assert_eq!(z.log().re, z.abs().ln());
            assert_eq!(z.log().im, z.arg());
            assert_ulps_eq!(z.log().exp(), z.to_polar());
        }
        assert_eq!(Complex::ONE.log(), Complex::ZERO);
        assert_eq!(Complex::I.log(), Complex::I * FRAC_PI_2);
        assert_eq!(Complex::NEG_ONE.log(), Complex::I * PI);
        assert_eq!(Complex::NEG_I.log(), Complex::I * -FRAC_PI_2);
    }

    #[test]
    fn powi() {
        for z in random_samples::<Complex>() {
            assert_eq!(z.powi(0), ComplexPolar::ONE);
            assert_eq!(z.powi(1), z.to_polar());
            for n in random_samples::<i32>() {
                assert_eq!(z.powi(n).abs, z.abs().powi(n));
                assert_eq!(z.powi(n).arg, z.arg() * n as f32);
            }
        }
        for n in random_samples::<i32>() {
            assert_eq!(Complex::ZERO.powi(n.abs()), ComplexPolar::ZERO);
            assert_eq!(Complex::ONE.powi(n), ComplexPolar::ONE);
        }
    }

    #[test]
    fn powf() {
        for z in random_samples::<Complex>() {
            assert_eq!(z.powf(0.0), ComplexPolar::ONE);
            assert_eq!(z.powf(1.0), z.to_polar());
            for n in random_samples::<i32>() {
                let x = n as f32 * 0.01;
                assert_eq!(z.powf(x).abs, z.abs().powf(x));
                assert_eq!(z.powf(x).arg, z.arg() * x);
            }
        }
        for n in random_samples::<i32>() {
            let x = n as f32 * 0.01;
            assert_eq!(Complex::ZERO.powf(x.abs()), ComplexPolar::ZERO);
            assert_eq!(Complex::ONE.powf(x), ComplexPolar::ONE);
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
