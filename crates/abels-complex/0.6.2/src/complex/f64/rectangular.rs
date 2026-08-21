use core::fmt;
use core::ops::*;
use core::write;

#[allow(dead_code)]
type Polar = crate::complex::polar::ComplexPolar<FT>;
type Rectangular = crate::complex::rectangular::Complex<FT>;
type FT = f64;

impl Add<Rectangular> for FT {
    type Output = Rectangular;
    fn add(self, z: Self::Output) -> Self::Output {
        z + self
    }
}

impl Sub<Rectangular> for FT {
    type Output = Rectangular;
    fn sub(self, z: Self::Output) -> Self::Output {
        Rectangular::new(self - z.re, -z.im)
    }
}

impl Mul<Rectangular> for FT {
    type Output = Rectangular;
    fn mul(self, z: Self::Output) -> Self::Output {
        z * self
    }
}

impl Div<Rectangular> for FT {
    type Output = Rectangular;
    fn div(self, z: Self::Output) -> Self::Output {
        self * z.recip()
    }
}

impl Rectangular {
    pub const NEG_ONE: Self = Self::new(-1.0, 0.0);
    pub const NEG_I: Self = Self::new(0.0, -1.0);
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
        use rand::RngExt;
        rng.sample::<Polar, _>(self).to_rectangular()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;
    use core::f64::consts::{E, FRAC_PI_2, FRAC_PI_4, LN_2, PI, SQRT_2};
    use core::iter::Iterator;
    use rand::{
        RngExt, SeedableRng,
        distr::{Distribution, StandardUniform},
        rngs::StdRng,
    };

    const NUM_SAMPLES: usize = 100;
    const SEED: u64 = 21;

    fn random_samples<T>() -> impl Iterator<Item = T>
    where
        StandardUniform: Distribution<T>,
    {
        StdRng::seed_from_u64(SEED)
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
    fn expm1() {
        for z in random_samples::<Rectangular>() {
            assert_ulps_eq!(z.expm1(), z.exp().to_rectangular() - 1.0, max_ulps = 5);
        }
        assert_eq!(Rectangular::ZERO.expm1(), Rectangular::ZERO);
        assert_ulps_eq!(Rectangular::ONE.expm1(), Rectangular::new(E - 1.0, 0.0));
        assert_ulps_eq!(
            Rectangular::I.expm1(),
            Rectangular::I.exp().to_rectangular() - 1.0
        );

        let tiny = Rectangular::new(1e-16, 0.0);
        assert_eq!(tiny.exp().to_rectangular() - 1.0, Rectangular::ZERO);
        assert_ne!(tiny.expm1(), Rectangular::ZERO);
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
    fn ln_1p() {
        for z in random_samples::<Rectangular>() {
            // (z + 1).ln() is unreliable near the singularity at z = -1
            let z = Rectangular::new(z.re.max(-0.75), z.im);
            assert_ulps_eq!(z.ln_1p(), (z + 1.0).ln(), max_ulps = 2);
        }
        assert_eq!(Rectangular::ZERO.ln_1p(), Rectangular::ZERO);
        assert_ulps_eq!(Rectangular::ONE.ln_1p(), Rectangular::new(LN_2, 0.0));
        assert_ulps_eq!(
            Rectangular::I.ln_1p(),
            Rectangular::new(LN_2 / 2.0, FRAC_PI_4)
        );
        // Near zero: (z + 1).ln() loses all precision, ln_1p preserves it
        let tiny = Rectangular::new(1e-16, 0.0);
        assert_eq!((tiny + 1.0).ln(), Rectangular::ZERO); // 1.0 + 1e-16 == 1.0 in f64
        assert_ne!(tiny.ln_1p(), Rectangular::ZERO);
    }

    #[test]
    fn ln_branch() {
        use core::f64::consts::{LN_10, TAU};
        // k=0 matches principal value
        for z in random_samples::<Rectangular>() {
            assert_eq!(z.ln_branch(0), z.ln());
        }
        // ln_k(z) = ln(z) + 2πki
        for z in random_samples::<Rectangular>() {
            for k in -3_i32..=3 {
                assert_ulps_eq!(z.ln_branch(k), z.ln() + Rectangular::I * (k as FT * TAU));
            }
        }
        // log2_branch and log10_branch consistency
        for z in random_samples::<Rectangular>() {
            assert_ulps_eq!(z.log2_branch(1), z.ln_branch(1) / LN_2);
            assert_ulps_eq!(z.log10_branch(1), z.ln_branch(1) / LN_10);
        }
    }

    #[test]
    fn ln_1p_branch() {
        use core::f64::consts::TAU;
        // k=0 matches principal value
        for z in random_samples::<Rectangular>() {
            assert_eq!(z.ln_1p_branch(0), z.ln_1p());
        }
        // ln_1p_branch(k) = ln_1p + 2πki
        for z in random_samples::<Rectangular>() {
            for k in -3_i32..=3 {
                assert_ulps_eq!(
                    z.ln_1p_branch(k),
                    z.ln_1p() + Rectangular::I * (k as FT * TAU)
                );
            }
        }
    }

    #[test]
    fn sqrt_branch() {
        // k=0 matches principal sqrt (in polar form)
        for z in random_samples::<Rectangular>() {
            assert_eq!(z.sqrt_branch(0), z.to_polar().sqrt());
        }
        // k=1 gives the other root: abs same, arg offset by π
        for z in random_samples::<Rectangular>() {
            assert_ulps_eq!(z.sqrt_branch(1).abs, z.sqrt_branch(0).abs);
            assert_ulps_eq!(z.sqrt_branch(1).arg, z.sqrt_branch(0).arg + PI);
        }
    }

    #[test]
    fn nth_root() {
        use core::f64::consts::TAU;
        // k=0 consistent with powf(1/n)
        for z in random_samples::<Rectangular>() {
            let r = z.nth_root(3, 0);
            assert_ulps_eq!(r.abs, z.abs().powf(1.0 / 3.0));
            assert_ulps_eq!(r.arg, z.arg() / 3.0);
        }
        // Cube roots of unity
        let r0 = Rectangular::ONE.nth_root(3, 0);
        let r1 = Rectangular::ONE.nth_root(3, 1);
        let r2 = Rectangular::ONE.nth_root(3, 2);
        assert_ulps_eq!(r0, Polar::ONE);
        assert_ulps_eq!(r1.abs, 1.0);
        assert_ulps_eq!(r1.arg, TAU / 3.0);
        assert_ulps_eq!(r2.abs, 1.0);
        assert_ulps_eq!(r2.arg, TAU * 2.0 / 3.0);
    }

    #[test]
    fn pow_rational() {
        // p/q = 1/2, k=0: matches sqrt_branch(0)
        for z in random_samples::<Rectangular>() {
            assert_eq!(z.pow_rational(1, 2, 0), z.sqrt_branch(0));
        }
        // p/q = 2/3: 8^(2/3) = 4
        let z = Rectangular::new(8.0, 0.0);
        assert_ulps_eq!(z.pow_rational(2, 3, 0).abs, 4.0);
        // Consistent with polar
        for z in random_samples::<Rectangular>() {
            for k in 0..3_i32 {
                assert_eq!(z.pow_rational(1, 3, k), z.to_polar().pow_rational(1, 3, k));
            }
        }
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
