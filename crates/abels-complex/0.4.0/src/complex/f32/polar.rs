use core::f32::consts::{FRAC_PI_2, PI};
use core::ops::*;
#[cfg(feature = "libm")]
use num_traits::real::Real;

type Polar = crate::complex::polar::ComplexPolar<FT>;
type FT = f32;

impl Mul<Polar> for FT {
    type Output = Polar;
    fn mul(self, mut other: Self::Output) -> Self::Output {
        other *= self;
        other
    }
}

impl Div<Polar> for FT {
    type Output = Polar;
    fn div(self, other: Self::Output) -> Self::Output {
        self * other.recip()
    }
}

impl Polar {
    pub const I: Self = Self::new(1.0, FRAC_PI_2);
    pub const NEG_ONE: Self = Self::new(1.0, PI);
    pub const NEG_I: Self = Self::new(1.0, -FRAC_PI_2);
}

#[cfg(feature = "rand")]
impl rand::distr::Distribution<Polar> for rand::distr::StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Polar {
        Polar::new(self.sample(rng), rng.random_range((-PI).next_up()..=PI))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::*;
    use core::f32::consts::{E, TAU};
    use rand::{
        Rng, SeedableRng,
        distr::{Distribution, StandardUniform, Uniform, uniform::*},
        rngs::StdRng,
    };

    type Rectangular = crate::complex::rectangular::Complex<FT>;

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
        for z0 in random_samples::<Polar>() {
            for z1 in random_samples::<Polar>() {
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
            assert_eq!(z0 * Polar::ONE, z0);
            assert_eq!((z0 * Polar::ZERO).abs, 0.0);
            assert_eq!((z0 * 0.0).abs, 0.0);
        }
    }

    #[test]
    fn division() {
        for z0 in random_samples::<Polar>() {
            for z1 in random_samples::<Polar>() {
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
            assert_ulps_eq!(z0 / z0, Polar::ONE);
            assert_eq!((Polar::ZERO / z0).abs, 0.0);
        }
    }

    #[test]
    fn negation() {
        assert_eq!((-Polar::ONE).normalize(), Polar::NEG_ONE);
        assert_eq!((-Polar::I).normalize(), Polar::NEG_I);
        assert_eq!((-Polar::NEG_ONE).normalize(), Polar::ONE);
        assert_ulps_eq!((-Polar::NEG_I).normalize(), Polar::I);
    }

    #[test]
    fn reciprocal() {
        for z in random_samples::<Polar>() {
            assert_ulps_eq!(z.recip(), 1.0 / z);
            assert_ulps_eq!(z * z.recip(), Polar::ONE);
        }
        assert_eq!(Polar::ONE.recip(), Polar::ONE);
        assert_eq!(Polar::I.recip(), Polar::NEG_I);
        assert_eq!(Polar::NEG_ONE.recip().normalize(), Polar::NEG_ONE);
        assert_eq!(Polar::NEG_I.recip(), Polar::I);
    }

    #[test]
    fn sqrt() {
        for z in random_samples::<Polar>() {
            assert_eq!(z.sqrt().abs, z.abs.sqrt());
            assert_eq!(z.sqrt().arg, z.arg / 2.0);
        }
        assert_eq!(Polar::ONE.sqrt(), Polar::ONE);
        assert_eq!(Polar::NEG_ONE.sqrt(), Polar::I);
        assert_eq!(Polar::ONE.sqrt(), Polar::ONE);
    }

    #[test]
    fn abs() {
        for z in random_samples::<Polar>() {
            assert_eq!(z.abs_sq(), z.abs * z.abs);
        }
        assert_eq!(Polar::ONE.abs, 1.0);
        assert_eq!(Polar::I.abs, 1.0);
        assert_eq!(Polar::NEG_ONE.abs, 1.0);
        assert_eq!(Polar::NEG_I.abs, 1.0);
    }

    #[test]
    fn conjugate() {
        for z in random_samples::<Polar>() {
            assert_eq!(z.conjugate().re(), z.re());
            assert_eq!(z.conjugate().im(), -z.im());
            assert_eq!(z.conjugate().conjugate(), z);
        }
        assert_ulps_eq!(Polar::ONE.conjugate().normalize(), Polar::ONE);
        assert_ulps_eq!(Polar::I.conjugate().normalize(), Polar::NEG_I);
        assert_ulps_eq!(Polar::NEG_ONE.conjugate().normalize(), Polar::NEG_ONE);
        assert_ulps_eq!(Polar::NEG_I.conjugate().normalize(), Polar::I);
    }

    #[test]
    fn arg() {
        assert_eq!(Polar::ONE.arg, 0.0);
        assert_eq!(Polar::I.arg, FRAC_PI_2);
        assert_eq!(Polar::NEG_ONE.arg, PI);
        assert_eq!(Polar::NEG_I.arg, -FRAC_PI_2);
    }

    #[test]
    fn exp() {
        for z in random_samples::<Polar>() {
            assert_eq!(z.exp().abs, z.re().exp());
            assert_eq!(z.exp().arg, z.im());
            assert_ulps_eq!(z.exp().ln(), z.to_rectangular());
        }
        assert_ulps_eq!(Polar::ONE.exp(), Polar::new(E, 0.0));
        assert_ulps_eq!(Polar::I.exp(), Polar::new(1.0, 1.0));
        assert_ulps_eq!(Polar::NEG_ONE.exp(), Polar::new(E.recip(), 0.0));
        assert_ulps_eq!(Polar::NEG_I.exp(), Polar::new(1.0, -1.0));
    }

    #[test]
    fn log() {
        for z in random_samples::<Polar>() {
            assert_eq!(z.ln().re, z.abs.ln());
            assert_eq!(z.ln().im, z.arg);
            assert_ulps_eq!(z.ln().exp(), z);
        }
        assert_eq!(Polar::ONE.ln(), Rectangular::ZERO);
        assert_eq!(Polar::I.ln(), Rectangular::I * FRAC_PI_2);
        assert_eq!(Polar::NEG_ONE.ln(), Rectangular::I * PI);
        assert_eq!(Polar::NEG_I.ln(), Rectangular::I * -FRAC_PI_2);

        assert_ulps_eq!(Polar::new(E, 0.0).ln(), Rectangular::ONE);
        assert_ulps_eq!(Polar::new(2.0, 0.0).log2(), Rectangular::ONE);
        assert_ulps_eq!(Polar::new(10.0, 0.0).log10(), Rectangular::ONE);
    }

    #[test]
    fn powi() {
        for z in random_samples::<Polar>() {
            assert_eq!(z.powi(0), Polar::ONE);
            assert_eq!(z.powi(1), z);
            for n in random_samples::<i32>() {
                assert_eq!(z.powi(n).abs, z.abs.powi(n));
                assert_eq!(z.powi(n).arg, z.arg * n as FT);
            }
        }
        for n in random_samples::<i32>() {
            assert_eq!(Polar::ZERO.powi(n.abs()), Polar::ZERO);
            assert_eq!(Polar::ONE.powi(n), Polar::ONE);
        }
    }

    #[test]
    fn powf() {
        for z in random_samples::<Polar>() {
            assert_eq!(z.powf(0.0), Polar::ONE);
            assert_eq!(z.powf(1.0), z);
            for n in random_samples::<i32>() {
                let x = n as FT * 0.01;
                assert_eq!(z.powf(x).abs, z.abs.powf(x));
                assert_eq!(z.powf(x).arg, z.arg * x);
            }
        }
        for n in random_samples::<i32>() {
            let x = n as FT * 0.01;
            assert_eq!(Polar::ZERO.powf(x.abs()), Polar::ZERO);
            assert_eq!(Polar::ONE.powf(x), Polar::ONE);
        }
    }

    #[test]
    fn normalize() {
        for z in random_samples::<Polar>() {
            for n in uniform_samples::<i32>(-99, 99) {
                let w = Polar::new(z.abs, z.arg + n as FT * TAU);
                assert_ulps_eq!(z, w.normalize(), epsilon = 2000.0 * FT::EPSILON);

                assert_ulps_eq!(
                    Polar::new(-z.abs, z.arg).normalize(),
                    Polar::new(z.abs, z.arg + PI).normalize(),
                    epsilon = 2000.0 * FT::EPSILON
                );
            }
        }
    }
}
