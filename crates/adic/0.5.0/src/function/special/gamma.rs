use num::Zero;

use crate::{
    normed::{UltraNormed, Valuation},
    traits::{AdicPrimitive, CanApproximate, HasApproximateDigits, HasDigits},
    ZAdic,
};


/// Provides the gamma function for adic numbers
pub (crate) fn naive_gamma<T>(input: T) -> ZAdic
where T: Into<ZAdic> + HasDigits {
    let z: ZAdic = input.into();
    let p = z.p();

    // Note: This is a naive implementation, especially because it only works for finite, unsigned numbers

    assert!(z.has_finite_digits(), "Input value does not have finite digits!");

    let one = match z.certainty() {
        Valuation::PosInf => ZAdic::one(p),
        Valuation::Finite(certainty) => ZAdic::one(p).approximation(certainty),
    };

    if z.is_approx_zero() { return one.clone() }

    let mut i = z.clone();
    std::iter::repeat_with(|| {
        i = i.clone() - one.clone();
        i.clone()
    })
        .take_while(|i| !i.is_approx_zero())
        .fold(
            -one.clone(),
            |acc, i| {
                if i.valuation().is_zero() {
                    acc * -i
                } else {
                    acc * -one.clone()
                }
            }
        )
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        iter::{once, repeat},
    };
    use num::Rational32;

    use crate::{
        traits::{AdicInteger, CanApproximate, PrimedFrom, TryPrimedFrom},
        EAdic, IAdic, RAdic, ZAdic,
    };
    use super::naive_gamma;

    #[test]
    fn simple() {
        let z = ZAdic::new_approx(5, 6, vec![1, 1]);
        let gamma_z = naive_gamma(z);
        assert_eq!(gamma_z, ZAdic::new_approx(5, 6, vec![4, 4]));
    }

    #[test]
    fn special_values() {
        let certainty = 6;
        let expected = [1, -1, 1];

        for p in [2, 3, 5, 7, 11, 13] {
            for (n, gamma_n) in expected.into_iter().enumerate()
                .map(|(n, gamma_n)| (u32::try_from(n).unwrap(), gamma_n)) {
                assert_eq!(
                    naive_gamma(IAdic::primed_from(p, n)),
                    ZAdic::primed_from(p, gamma_n)
                );
                assert_eq!(
                    naive_gamma(ZAdic::primed_from(p, n).approximation(certainty)),
                    ZAdic::primed_from(p, gamma_n).approximation(certainty)
                );
                assert_eq!(
                    naive_gamma(ZAdic::primed_from(p, n)),
                    ZAdic::primed_from(p, gamma_n)
                );
            }
        }
    }

    #[test]
    fn natural_zadic_values() {
        let certainty = 6;
        let expected = HashMap::from([
            (2, [-1, 3, -3, 15, -15, 105, -105]),
            (3, [-2, 2, -8, 40, -40, 280, -2240]),
            (5, [-2, 6, -24, 24, -144, 1008, -8064]),
            (7, [-2, 6, -24, 120, -720, 720, -5760]),
            (11, [-2, 6, -24, 120, -720, 5040, -40320]),
            (13, [-2, 6, -24, 120, -720, 5040, -40320]),
        ]);

        for &p in expected.keys() {
            for (n, gamma_n) in expected.get(&p).unwrap().into_iter().enumerate()
                .map(|(n, &gamma_n)| (u32::try_from(n + 3 /* skip special values */).unwrap(), gamma_n)) {
                assert_eq!(
                    naive_gamma(IAdic::primed_from(p, n)),
                    ZAdic::primed_from(p, gamma_n)
                );
                assert_eq!(
                    naive_gamma(ZAdic::primed_from(p, n).approximation(certainty)),
                    ZAdic::primed_from(p, gamma_n).approximation(certainty)
                );
                assert_eq!(
                    naive_gamma(ZAdic::primed_from(p, n)),
                    ZAdic::primed_from(p, gamma_n)
                );
            }
        }
    }

    #[test]
    fn rational_zadic_values() {
        // gamma(1/4)
        let z = ZAdic::new_approx(5, 6, vec![4, 3, 3, 3, 3, 3]);
        let gamma_z = ZAdic::new_approx(5, 6, vec![1, 4, 0, 0, 3, 0]);
        assert_eq!(
            naive_gamma(z),
            gamma_z
        );

        // gamma (1/4)^2 = -2 + sqrt(-1)
        let sqrt_neg_one = ZAdic::primed_from(5, -1).nth_root(2, 6).unwrap().into_roots().skip(1).next().unwrap();
        let two = ZAdic::primed_from(5, 2).approximation(6);
        assert_eq!(gamma_z.clone() * gamma_z.clone(), sqrt_neg_one - two);
    }

    #[test]
    #[should_panic]
    fn panic_negative_input() {
        let z = IAdic::primed_from(5, -1);
        naive_gamma(z);
    }

    #[test]
    #[should_panic]
    fn panic_rational_input() {
        let z = RAdic::try_primed_from(5, Rational32::new(1, 4)).unwrap();
        naive_gamma(z);
    }

    #[test]
    #[ignore = "for reporting"]
    fn report_gamma_values_table() {

        // Create a table of integer values for the p-adic gamma function for various values of p

        certainty_check();

        let primes = [2, 3, 5, 7, 11, 13];
        let max_n = 10;

        iadic_table(&primes, max_n);

        println!();
        println!();

        zadic_table(&primes, max_n);

        fn iadic_table(primes: &[u32], max_n: u32) {
            let width = naive_gamma(EAdic::primed_from(2, max_n - 1)).to_string().len() + 2;

            print!("{:>width$}", "p");
            for n in 0..max_n {
                let label = format!("gamma({n})");
                print!("{:>width$}", label);
            }
            println!();

            for &p in primes.into_iter() {
                print!("{:>width$}", p);

                for n in 0..max_n {
                    let val = naive_gamma(EAdic::primed_from(p, n));
                    print!("{:>width$}", val.to_string());
                }

                println!();
            }
        }

        fn zadic_table(primes: &[u32], max_n: u32) {
            let width = 16;
            let certainty = width - 8;

            print!("{:>width$}", "p");
            for n in 0..max_n {
                let label = format!("gamma({n})");
                print!("{:>width$}", label);
            }
            println!();

            for &p in primes.into_iter() {
                print!("{:>width$}", p);

                for n in 0..max_n {
                    let val = naive_gamma(ZAdic::primed_from(p, n).approximation(certainty));
                    print!("{:>width$}", val.to_string());
                }

                println!();
            }
        }

        fn certainty_check() {
            let overall_runtime = std::time::Instant::now();
            for certainty in 2..10 {
                let z = ZAdic::new_approx(5, certainty,
                    once(4)
                    .chain(repeat(3))
                    .take(certainty)
                    .collect());

                let function_runtime = std::time::Instant::now();
                println!("gamma({}) is {}",
                    z.to_string(),
                    naive_gamma(z)
                );
                println!("Elapsed time for certainty {}: {}ms", certainty, function_runtime.elapsed().as_millis());
            }

            println!("Total elapsed time: {}s", overall_runtime.elapsed().as_secs());
        }
    }

}
