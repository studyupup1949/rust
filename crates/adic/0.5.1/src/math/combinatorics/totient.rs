//! Adic utility methods

use std::{
    iter::{Product, empty},
    ops::Sub,
};
use itertools::Either;
use num::{integer::lcm, Integer, One, Unsigned};
use machine_factor::factorize;
use crate::divisible::{Composite, Prime, PrimePower};


/// Prime power factorization, in `u64`
///
/// Note: Currently a lightly wrapped call to `machine-factor::factorize`.
pub fn prime_power_factors(num: u64) -> impl Iterator<Item = (u64, u8)> {
    if num == 0 {
        panic!("Cannot factor zero");
    } else if num == 1 {
        Either::Left(empty())
    } else {
        let factorization = factorize(num);
        Either::Right(factorization.factors.into_iter().zip(factorization.powers).take(factorization.len))
    }
}


/// Euler totient function
pub fn totient(num: u64) -> u64 {
    let pp_factors = prime_power_factors(num).map(|(pa, pa_num)| {
        let pa_num32 = u32::from(pa_num);
        pa.pow(pa_num32 - 1) * (pa - 1)
    });
    pp_factors.product()
}


/// Carmichael function of num
pub fn carmichael(num: u64) -> u64 {
    let carm_factors = prime_power_factors(num).map(|(pa, pa_num)| {
        // car(n) = tot(n) if n=1,2,4, or (p>2)^k
        //  = 1/2 tot(n) if n=2^k
        //  = lcm(car(n1), car(n2), ...), where n = n1*n2*... and ni rel prime to nj
        let pa_num32 = u32::from(pa_num);
        if pa == 2 {
            match(pa_num32) {
                0 | 1 => 1,
                2 => 2,
                k => pa.pow(k-2)
            }
        } else {
            pa.pow(pa_num32 - 1) * (pa - 1)
        }
    });
    carm_factors.fold(1, lcm)
}



#[cfg(test)]
mod test {
    use super::{carmichael, prime_power_factors, totient};

    #[test]
    fn test_prime_power_factorization() {

        assert_eq!(prime_power_factors(1).collect::<Vec<_>>(), vec![]);
        assert_eq!(prime_power_factors(2).collect::<Vec<_>>(), vec![(2, 1)]);
        assert_eq!(prime_power_factors(3*5*5*5*7*11).collect::<Vec<_>>(), vec![(3, 1), (5, 3), (7, 1), (11, 1)]);

    }

    #[test]
    #[should_panic]
    fn test_factorize_zero() {
        let _ = prime_power_factors(0);
    }

    #[test]
    fn test_totient() {

        assert_eq!(2*3, totient(3*3));
        assert_eq!(2*4, totient(3*5));
        assert_eq!(2*4*5*5*6*10, totient(3*5*5*5*7*11));

        assert_eq!(1, totient(1));
        assert_eq!(1, totient(2));
        assert_eq!(2, totient(3));
        assert_eq!(2, totient(4));
        assert_eq!(4, totient(5));
        assert_eq!(2, totient(6));
        assert_eq!(6, totient(7));
        assert_eq!(4, totient(8));
        assert_eq!(6, totient(9));
        assert_eq!(4, totient(10));
        assert_eq!(4, totient(12));
        assert_eq!(8, totient(15));

    }


    #[test]
    fn test_carmichael() {

        assert_eq!(12, carmichael(26));
        assert_eq!(12, totient(26));
        assert_eq!(10, carmichael(33));
        assert_eq!(20, totient(33));

        assert_eq!(1, carmichael(1));
        assert_eq!(1, carmichael(2));
        assert_eq!(2, carmichael(3));
        assert_eq!(2, carmichael(4));
        assert_eq!(4, carmichael(5));
        assert_eq!(2, carmichael(6));
        assert_eq!(6, carmichael(7));
        assert_eq!(2, carmichael(8));
        assert_eq!(6, carmichael(9));
        assert_eq!(4, carmichael(10));
        assert_eq!(2, carmichael(12));
        assert_eq!(4, carmichael(15));

    }

}
