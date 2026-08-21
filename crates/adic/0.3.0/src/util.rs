//! Adic utility methods

use std::collections::BTreeMap;
use num::integer::binomial;
use num_prime::nt_funcs::factorize;
use crate::{AdicInteger, UAdic};


/// Euler totient function of num
///
/// # Panics
/// Panics if usize -> u32 conversion fails
///
/// ```
/// # use adic::util::totient;
/// assert_eq!(2*3, totient(3*3));
/// assert_eq!(2*4, totient(3*5));
/// assert_eq!(2*4*5*5*6*10, totient(3*5*5*5*7*11));
/// ```
pub fn totient(num: u32) -> u32 {
    factorize(num).into_iter().map(|(pa, pa_num)| {
        let pa_num_us = u32::try_from(pa_num).expect("totient conversion usize -> u32");
        pa.pow(pa_num_us - 1) * (pa - 1)
    }).product()
}


/// Euler totient function of prod(nums)
///
/// # Panics
/// Panics if usize -> u32 conversion fails
///
/// ```
/// # use adic::util::totient_many;
/// assert_eq!(2*3, totient_many(&[3, 3]));
/// assert_eq!(2*4, totient_many(&[3, 5]));
/// assert_eq!(2*4*5*5*6*10, totient_many(&[3*5, 5*5*11, 7]));
/// ```
pub fn totient_many(nums: &[u32]) -> u32 {
    let all_factor_map = nums.iter().fold(BTreeMap::new(), |mut factor_map, num| {
        for (pm, pm_num) in factorize(*num) {
            match factor_map.get_mut(&pm) {
                Some(pn_num) => { *pn_num += pm_num; },
                None => { factor_map.insert(pm, pm_num); }
            }
        }
        factor_map
    });
    all_factor_map.into_iter().map(|(pa, pa_num)| {
        let pa_num_us = u32::try_from(pa_num).expect("totient conversion usize -> u32");
        pa.pow(pa_num_us - 1) * (pa - 1)
    }).product()
}


/// Binomial coefficients in `UAdic` form
///
/// ```
/// # use adic::{util::adic_binomial, AdicInteger, UAdic};
/// assert_eq!(UAdic::from_u32(3, 10), adic_binomial(3, 5, 2));
/// assert_eq!("101._3", adic_binomial(3, 5, 2).to_string());
/// assert_eq!(UAdic::from_u32(5, 10), adic_binomial(5, 5, 2));
/// assert_eq!("20._5", adic_binomial(5, 5, 2).to_string());
/// ```
pub fn adic_binomial(p: u32, n: u32, k: u32) -> UAdic {
    // TODO: we can make this better by not going through integers to get there
    // We can remove the chance of overflow this way
    UAdic::from_u32(p, binomial(n, k))
}


#[cfg(test)]
mod test {
    use super::{totient, totient_many};

    #[test]
    fn test_totient() {

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

        assert_eq!(totient(1), totient_many(&[1, 1]));
        assert_eq!(totient(10), totient_many(&[10, 1]));
        assert_eq!(totient(10), totient_many(&[5, 2]));
        assert_eq!(totient(10), totient_many(&[2, 5]));
        assert_eq!(totient(10), totient_many(&[1, 10]));
        assert_eq!(totient(9), totient_many(&[9, 1]));
        assert_eq!(totient(9), totient_many(&[3, 3]));
        assert_eq!(totient(9), totient_many(&[1, 9]));

    }

}
