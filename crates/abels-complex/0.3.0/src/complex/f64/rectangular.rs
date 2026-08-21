use core::f64::consts::{LN_2, LN_10};
use core::fmt;
use core::ops::*;
use core::write;
#[cfg(feature = "libm")]
use num_traits::real::Real;

type Polar = crate::complex::polar::ComplexPolar<FT>;
type Rectangular = crate::complex::rectangular::Complex<FT>;
type FT = f64;

impl Rectangular {
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const ONE: Self = Self::new(1.0, 0.0);
    pub const NEG_ONE: Self = Self::new(-1.0, 0.0);
    pub const I: Self = Self::new(0.0, 1.0);
    pub const NEG_I: Self = Self::new(0.0, -1.0);

    /// Computes the conjugate.
    pub const fn conjugate(self) -> Self {
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

    /// Computes the principle square root.
    pub fn sqrt(self) -> Self {
        let abs = self.abs();
        Self::new(
            (0.5 * (abs + self.re)).sqrt(),
            (0.5 * (abs - self.re)).sqrt().copysign(self.im),
        )
    }

    /// Convert to polar form.
    pub fn to_polar(self) -> Polar {
        Polar::new(self.abs(), self.arg())
    }

    /// Computes `e^self` where `e` is the base of the natural logarithm.
    pub fn exp(self) -> Polar {
        Polar::new(self.re.exp(), self.im)
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
    pub fn powf(self, x: FT) -> Polar {
        self.to_polar().powf(x)
    }

    /// Computes the euclidian distance between two points.
    pub fn distance(self, other: Self) -> FT {
        (self - other).abs()
    }

    /// Computes the squared euclidian distance between two points.
    pub fn distance_squared(self, other: Self) -> FT {
        (self - other).abs_sq()
    }

    /// Casts to a glam::Vec2.
    #[cfg(feature = "glam")]
    pub fn as_vec2(self) -> glam::Vec2 {
        glam::vec2(self.re as f32, self.im as f32)
    }

    /// Casts to a glam::DVec2.
    #[cfg(feature = "glam")]
    pub fn as_dvec2(self) -> glam::DVec2 {
        glam::dvec2(self.re, self.im)
    }
}

impl Add for Rectangular {
    type Output = Self;
    fn add(mut self, other: Self) -> Self::Output {
        self += other;
        self
    }
}

impl Add<FT> for Rectangular {
    type Output = Self;
    fn add(mut self, re: FT) -> Self::Output {
        self += re;
        self
    }
}

impl Add<Rectangular> for FT {
    type Output = Rectangular;
    fn add(self, mut z: Self::Output) -> Self::Output {
        z += self;
        z
    }
}

impl AddAssign for Rectangular {
    fn add_assign(&mut self, other: Self) {
        self.re += other.re;
        self.im += other.im;
    }
}

impl AddAssign<FT> for Rectangular {
    fn add_assign(&mut self, re: FT) {
        self.re += re;
    }
}

impl Sub for Rectangular {
    type Output = Self;
    fn sub(mut self, other: Self) -> Self::Output {
        self -= other;
        self
    }
}

impl Sub<FT> for Rectangular {
    type Output = Self;
    fn sub(mut self, re: FT) -> Self::Output {
        self -= re;
        self
    }
}

impl Sub<Rectangular> for FT {
    type Output = Rectangular;
    fn sub(self, z: Self::Output) -> Self::Output {
        Rectangular::new(self - z.re, -z.im)
    }
}

impl SubAssign for Rectangular {
    fn sub_assign(&mut self, other: Self) {
        self.re -= other.re;
        self.im -= other.im;
    }
}

impl SubAssign<FT> for Rectangular {
    fn sub_assign(&mut self, re: FT) {
        self.re -= re;
    }
}

impl Mul for Rectangular {
    type Output = Self;
    fn mul(mut self, other: Self) -> Self {
        self *= other;
        self
    }
}

impl Mul<FT> for Rectangular {
    type Output = Self;
    fn mul(mut self, re: FT) -> Self {
        self *= re;
        self
    }
}

impl Mul<Rectangular> for FT {
    type Output = Rectangular;
    fn mul(self, mut other: Self::Output) -> Self::Output {
        other *= self;
        other
    }
}

impl MulAssign for Rectangular {
    fn mul_assign(&mut self, other: Self) {
        let re = self.re * other.re - self.im * other.im;
        self.im = self.re * other.im + self.im * other.re;
        self.re = re;
    }
}

impl MulAssign<FT> for Rectangular {
    fn mul_assign(&mut self, re: FT) {
        self.re *= re;
        self.im *= re;
    }
}

impl Div for Rectangular {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        self * other.recip()
    }
}

impl Div<FT> for Rectangular {
    type Output = Self;
    fn div(mut self, re: FT) -> Self::Output {
        self /= re;
        self
    }
}

impl Div<Rectangular> for FT {
    type Output = Rectangular;
    fn div(self, other: Self::Output) -> Self::Output {
        self * other.recip()
    }
}

impl DivAssign for Rectangular {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

impl DivAssign<FT> for Rectangular {
    fn div_assign(&mut self, re: FT) {
        self.re /= re;
        self.im /= re;
    }
}

impl Neg for Rectangular {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.re, -self.im)
    }
}

impl fmt::Display for Rectangular {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn fmt_x(f: &mut fmt::Formatter, x: FT, sign_plus: bool) -> fmt::Result {
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
impl rand::distr::Distribution<Rectangular> for rand::distr::StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Rectangular {
        rng.sample::<Polar, _>(self).to_rectangular()
    }
}

#[cfg(feature = "approx")]
use approx::{AbsDiffEq, RelativeEq, UlpsEq};

#[cfg(feature = "approx")]
impl AbsDiffEq for Rectangular {
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
impl RelativeEq for Rectangular {
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
impl UlpsEq for Rectangular {
    fn default_max_ulps() -> u32 {
        FT::default_max_ulps()
    }
    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        FT::ulps_eq(&self.re, &other.re, epsilon, max_ulps)
            && FT::ulps_eq(&self.im, &other.im, epsilon, max_ulps)
    }
}

impl From<FT> for Rectangular {
    fn from(value: FT) -> Self {
        Self::new(value, 0.0)
    }
}

#[cfg(feature = "glam")]
impl From<glam::Vec2> for Rectangular {
    fn from(v: glam::Vec2) -> Self {
        Self::new(v.x as FT, v.y as FT)
    }
}

#[cfg(feature = "glam")]
impl From<glam::DVec2> for Rectangular {
    fn from(v: glam::DVec2) -> Self {
        Self::new(v.x, v.y)
    }
}

#[cfg(feature = "glam")]
impl From<Rectangular> for glam::Vec2 {
    fn from(z: Rectangular) -> Self {
        z.as_vec2()
    }
}

#[cfg(feature = "glam")]
impl From<Rectangular> for glam::DVec2 {
    fn from(z: Rectangular) -> Self {
        z.as_dvec2()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;
    use core::f64::consts::{E, FRAC_PI_2, PI, SQRT_2};
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
        for z0 in random_samples::<Rectangular>() {
            for z1 in random_samples::<Rectangular>() {
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
            assert_eq!(z0 + Rectangular::ZERO, z0);
        }
    }

    #[test]
    fn subtraction() {
        for z0 in random_samples::<Rectangular>() {
            for z1 in random_samples::<Rectangular>() {
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
            assert_eq!(z0 - z0, Rectangular::ZERO);
            assert_eq!(z0 - Rectangular::ZERO, z0);
        }
    }

    #[test]
    fn multiplication() {
        for z0 in random_samples::<Rectangular>() {
            for z1 in random_samples::<Rectangular>() {
                let z = z0 * z1;
                assert_ulps_eq!(z.abs(), z0.abs() * z1.abs());
                assert_ulps_eq!(
                    z.arg().sin(),
                    (z0.arg() + z1.arg()).sin(),
                    epsilon = 4.0 * FT::EPSILON
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
            assert_eq!(z0 * Rectangular::ONE, z0);
            assert_eq!(z0 * Rectangular::ZERO, Rectangular::ZERO);
            assert_eq!(z0 * 0.0, Rectangular::ZERO);
        }
    }

    #[test]
    fn division() {
        for z0 in random_samples::<Rectangular>() {
            for z1 in random_samples::<Rectangular>() {
                let z = z0 / z1;
                assert_relative_eq!(
                    z.abs(),
                    z0.abs() / z1.abs(),
                    max_relative = 3.0 * FT::EPSILON
                );
                assert_ulps_eq!(
                    z.arg().sin(),
                    (z0.arg() - z1.arg()).sin(),
                    epsilon = 4.0 * FT::EPSILON
                );

                let z = z0 / z1.re;
                assert_eq!(z.re, z0.re / z1.re);
                assert_eq!(z.im, z0.im / z1.re);

                let z = z0.re / z1;
                assert_ulps_eq!(z.abs(), z0.re.abs() / z1.abs());
                assert_ulps_eq!(
                    z.arg().sin(),
                    (-z0.re.signum() * z1.arg()).sin(),
                    epsilon = 2.0 * FT::EPSILON
                );

                let mut z = z0;
                z /= z1;
                assert_eq!(z, z0 / z1);

                let mut z = z0;
                z /= z1.re;
                assert_eq!(z, z0 / z1.re);
            }
            assert_ulps_eq!(z0 / z0, Rectangular::ONE);
            assert_eq!(Rectangular::ZERO / z0, Rectangular::ZERO);
        }
    }

    #[test]
    fn negation() {
        for z in random_samples::<Rectangular>() {
            assert_eq!(-z, Rectangular::new(-z.re, -z.im));
        }
        assert_eq!(-Rectangular::ONE, Rectangular::NEG_ONE);
        assert_eq!(-Rectangular::I, Rectangular::NEG_I);
        assert_eq!(-Rectangular::NEG_ONE, Rectangular::ONE);
        assert_eq!(-Rectangular::NEG_I, Rectangular::I);
    }

    #[test]
    fn reciprocal() {
        for z in random_samples::<Rectangular>() {
            assert_eq!(z.recip(), 1.0 / z);
            assert_ulps_eq!(z * z.recip(), Rectangular::ONE);
        }
        assert_eq!(Rectangular::ONE.recip(), Rectangular::ONE);
        assert_eq!(Rectangular::I.recip(), Rectangular::NEG_I);
        assert_eq!(Rectangular::NEG_ONE.recip(), Rectangular::NEG_ONE);
        assert_eq!(Rectangular::NEG_I.recip(), Rectangular::I);
    }

    #[test]
    fn sqrt() {
        for z in random_samples::<Rectangular>() {
            assert_ulps_eq!(z.sqrt().abs(), z.abs().sqrt());
            assert_ulps_eq!(
                z.sqrt().arg(),
                z.arg() / 2.0,
                epsilon = 1400.0 * FT::EPSILON
            );
        }
        assert_eq!(Rectangular::ONE.sqrt(), Rectangular::ONE);
        assert_eq!(Rectangular::NEG_ONE.sqrt(), Rectangular::I);
        assert_eq!(
            Rectangular::new(0.0, 2.0).sqrt(),
            Rectangular::new(1.0, 1.0)
        );
        assert_eq!(
            Rectangular::new(0.0, -2.0).sqrt(),
            Rectangular::new(1.0, -1.0)
        );
    }

    #[test]
    fn abs() {
        for z in random_samples::<Rectangular>() {
            assert_ulps_eq!(z.abs_sq(), z.abs() * z.abs());
            assert_eq!(z.abs_sq(), z.re * z.re + z.im * z.im);
        }
        assert_eq!(Rectangular::ONE.abs(), 1.0);
        assert_eq!(Rectangular::I.abs(), 1.0);
        assert_eq!(Rectangular::NEG_ONE.abs(), 1.0);
        assert_eq!(Rectangular::NEG_I.abs(), 1.0);
        assert_eq!(Rectangular::new(1.0, 1.0).abs(), SQRT_2);
        assert_eq!(Rectangular::new(-1.0, 1.0).abs(), SQRT_2);
        assert_eq!(Rectangular::new(-1.0, -1.0).abs(), SQRT_2);
        assert_eq!(Rectangular::new(1.0, -1.0).abs(), SQRT_2);
    }

    #[test]
    fn conjugate() {
        for z in random_samples::<Rectangular>() {
            assert_eq!(z.conjugate().re, z.re);
            assert_eq!(z.conjugate().im, -z.im);
            assert_eq!(z.conjugate().conjugate(), z);
        }
        assert_eq!(Rectangular::ONE.conjugate(), Rectangular::ONE);
        assert_eq!(Rectangular::I.conjugate(), Rectangular::NEG_I);
        assert_eq!(Rectangular::NEG_ONE.conjugate(), Rectangular::NEG_ONE);
        assert_eq!(Rectangular::NEG_I.conjugate(), Rectangular::I);
    }

    #[test]
    fn arg() {
        assert_eq!(Rectangular::ONE.arg(), 0.0);
        assert_eq!(Rectangular::I.arg(), FRAC_PI_2);
        assert_eq!(Rectangular::NEG_ONE.arg(), PI);
        assert_eq!(Rectangular::NEG_I.arg(), -FRAC_PI_2);
    }

    #[test]
    fn exp() {
        for z in random_samples::<Rectangular>() {
            assert_eq!(z.exp().abs, z.re.exp());
            assert_eq!(z.exp().arg, z.im);
            assert_ulps_eq!(z.exp().ln(), z);
        }
        assert_eq!(Rectangular::ONE.exp(), Polar::new(E, 0.0));
        assert_eq!(Rectangular::I.exp(), Polar::new(1.0, 1.0));
        assert_eq!(Rectangular::NEG_ONE.exp(), Polar::new(E.recip(), 0.0));
        assert_eq!(Rectangular::NEG_I.exp(), Polar::new(1.0, -1.0));
    }

    #[test]
    fn log() {
        for z in random_samples::<Rectangular>() {
            assert_eq!(z.ln().re, z.abs().ln());
            assert_eq!(z.ln().im, z.arg());
            assert_ulps_eq!(z.ln().exp(), z.to_polar());
        }
        assert_eq!(Rectangular::ONE.ln(), Rectangular::ZERO);
        assert_eq!(Rectangular::I.ln(), Rectangular::I * FRAC_PI_2);
        assert_eq!(Rectangular::NEG_ONE.ln(), Rectangular::I * PI);
        assert_eq!(Rectangular::NEG_I.ln(), Rectangular::I * -FRAC_PI_2);

        assert_ulps_eq!(Rectangular::new(E, 0.0).ln(), Rectangular::ONE);
        assert_ulps_eq!(Rectangular::new(2.0, 0.0).log2(), Rectangular::ONE);
        assert_ulps_eq!(Rectangular::new(10.0, 0.0).log10(), Rectangular::ONE);
    }

    #[test]
    fn powi() {
        for z in random_samples::<Rectangular>() {
            assert_eq!(z.powi(0), Polar::ONE);
            assert_eq!(z.powi(1), z.to_polar());
            for n in random_samples::<i32>() {
                assert_eq!(z.powi(n).abs, z.abs().powi(n));
                assert_eq!(z.powi(n).arg, z.arg() * n as FT);
            }
        }
        for n in random_samples::<i32>() {
            assert_eq!(Rectangular::ZERO.powi(n.abs()), Polar::ZERO);
            assert_eq!(Rectangular::ONE.powi(n), Polar::ONE);
        }
    }

    #[test]
    fn powf() {
        for z in random_samples::<Rectangular>() {
            assert_eq!(z.powf(0.0), Polar::ONE);
            assert_eq!(z.powf(1.0), z.to_polar());
            for n in random_samples::<i32>() {
                let x = n as FT * 0.01;
                assert_eq!(z.powf(x).abs, z.abs().powf(x));
                assert_eq!(z.powf(x).arg, z.arg() * x);
            }
        }
        for n in random_samples::<i32>() {
            let x = n as FT * 0.01;
            assert_eq!(Rectangular::ZERO.powf(x.abs()), Polar::ZERO);
            assert_eq!(Rectangular::ONE.powf(x), Polar::ONE);
        }
    }
}
