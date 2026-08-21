//! Adic utility methods

use std::collections::BTreeMap;
use num_prime::nt_funcs::factorize;


/// Euler totient function of num
///
/// ```
/// # use adic::util::totient;
/// assert_eq!(2*3, totient(3*3));
/// assert_eq!(2*4, totient(3*5));
/// ```
pub fn totient(num: u32) -> u32 {
    factorize(num).into_iter().map(
        |(pa, pa_num)| pa.pow((pa_num as u32) - 1) * (pa - 1)
    ).product()
}


/// Euler totient function of prod(nums)
///
/// ```
/// # use adic::util::totient_many;
/// assert_eq!(2*3, totient_many(&[3, 3]));
/// assert_eq!(2*4, totient_many(&[3, 5]));
/// ```
pub fn totient_many(nums: &[u32]) -> u32 {
    let all_factor_map = nums.iter().fold(BTreeMap::new(), |mut factor_map, num| {
        factorize(*num).into_iter().for_each(|(pm, pm_num)| {
            match factor_map.get_mut(&pm) {
                Some(pn_num) => { *pn_num += pm_num; },
                None => { factor_map.insert(pm, pm_num); }
            }
        });
        factor_map
    });
    all_factor_map.into_iter().map(
        |(pa, pa_num)| pa.pow((pa_num as u32) - 1) * (pa - 1)
    ).product()
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
