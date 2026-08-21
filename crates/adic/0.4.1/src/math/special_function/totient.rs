//! Adic utility methods

use std::{
    iter::Product,
    ops::Sub,
};
use num::{integer::lcm, Integer, One, Unsigned};
use num_prime::{
    detail::{PrimalityBase, PrimalityRefBase},
    nt_funcs::factorize,
};
use crate::{Composite, Prime, PrimePower};


/// Euler totient function of num
///
/// # Panics
/// Panics if usize -> u32 conversion fails
///
/// ```
/// # use adic::special_function::totient;
/// assert_eq!(2*3, totient(3u32*3u32));
/// assert_eq!(2*4, totient(3u32*5u32));
/// let num_28875: u32 = 3*5*5*5*7*11;
/// assert_eq!(2*4*5*5*6*10, totient(num_28875));
/// ```
pub fn totient<I>(num: I) -> I
where I: Clone + Sub<I> + Product + Integer + Unsigned + One + PrimalityBase,
for<'r> &'r I: PrimalityRefBase<I> {
    factorize(num).into_iter().map(|(pa, pa_num)| {
        let pa_num_us = u32::try_from(pa_num).expect("totient conversion usize -> u32");
        pa.clone().pow(pa_num_us - 1) * (pa - I::one())
    }).product()
}


/// Carmichael function of num
///
/// # Panics
/// Panics if usize -> u32 conversion fails
///
/// ```
/// # use adic::special_function::{carmichael, totient};
/// assert_eq!(12, carmichael(26u32));
/// assert_eq!(12, totient(26u32));
/// assert_eq!(10, carmichael(33u32));
/// assert_eq!(20, totient(33u32));
/// ```
pub fn carmichael<I>(num: I) -> I
where I: Clone + From<u32> + Sub<I> + Product + Integer + Unsigned + One + PrimalityBase,
for<'r> &'r I: PrimalityRefBase<I> {
    factorize(num).into_iter().map(|(pa, pa_num)| {
        // car(n) = tot(n) if n=1,2,4, or (p>2)^k
        //  = 1/2 tot(n) if n=2^k
        //  = lcm(car(n1), car(n2), ...), where n = n1*n2*... and ni rel prime to nj
        let pa_num_us = u32::try_from(pa_num).expect("carmichael conversion usize -> u32");
        if pa == I::from(2) {
            match(pa_num_us) {
                0 | 1 => I::one(),
                2 => I::from(2),
                k => pa.pow(k-2)
            }
        } else {
            pa.clone().pow(pa_num_us - 1) * (pa - I::one())
        }
    }).fold(I::one(), lcm)
}


/// Carmichael function of the separate powers of `c^cpower`, in an iterator
pub fn carmichael_iter(c: &Composite, cpower: u32) -> impl Iterator<Item = Composite> + use<'_> {
    // car(n) = tot(n) if n=1,2,4, or (p>2)^k
    //  = 1/2 tot(n) if n=2^k
    //  = lcm(car(n1), car(n2), ...), where n = n1*n2*... and ni rel prime to nj
    c.prime_powers().map(move |pp| {
        let p = pp.p();
        let k = cpower * pp.power();
        let is_two = (p == 2.into());
        if is_two && [0, 1].contains(&k) {
            Composite::one()
        } else if is_two && k == 2 {
            Prime::from(2).into()
        } else if is_two {
            PrimePower::from((p, k-2)).into()
        } else {
            let pm1_factors = factorize(u32::from(p) - 1).into_iter().map(|(p, power)| {
                let pow32 = u32::try_from(power).expect("usize -> u32 conversion");
                PrimePower::from((p, pow32))
            });
            let p_factors = [PrimePower::from((p, k-1))];
            Composite::new(pm1_factors.chain(p_factors))
        }
    })
}



#[cfg(test)]
mod test {
    use super::{totient, carmichael};

    #[test]
    fn test_totient() {

        assert_eq!(1, totient(1u32));
        assert_eq!(1, totient(2u32));
        assert_eq!(2, totient(3u32));
        assert_eq!(2, totient(4u32));
        assert_eq!(4, totient(5u32));
        assert_eq!(2, totient(6u32));
        assert_eq!(6, totient(7u32));
        assert_eq!(4, totient(8u32));
        assert_eq!(6, totient(9u32));
        assert_eq!(4, totient(10u32));
        assert_eq!(4, totient(12u32));
        assert_eq!(8, totient(15u32));

    }


    #[test]
    fn test_carmichael() {

        assert_eq!(1, carmichael(1u32));
        assert_eq!(1, carmichael(2u32));
        assert_eq!(2, carmichael(3u32));
        assert_eq!(2, carmichael(4u32));
        assert_eq!(4, carmichael(5u32));
        assert_eq!(2, carmichael(6u32));
        assert_eq!(6, carmichael(7u32));
        assert_eq!(2, carmichael(8u32));
        assert_eq!(6, carmichael(9u32));
        assert_eq!(4, carmichael(10u32));
        assert_eq!(2, carmichael(12u32));
        assert_eq!(4, carmichael(15u32));

    }

}
