use std::iter::{once, repeat_n};
use crate::{AdicInteger, AdicNumber, AdicResult, AdicVariety, Prime, ZAdic};
use super::AdicPolynomial;


/// Calculate the roots of unity for given `p`.
/// These are the solutions of `x^2 = 1` if `p = 2` and `x^(p-1) = 1` if `p > 2`.
///
/// # Errors
/// Returns any errors that [`AdicInteger::nth_root`] returns
///
/// ```
/// # use adic::{roots_of_unity, zadic_variety};
/// assert_eq!(
///     Ok(zadic_variety!(2, 6, [[1, 0, 0, 0, 0, 0], [1, 1, 1, 1, 1, 1]])),
///     roots_of_unity(2, 6)
/// );
/// assert_eq!(
///     Ok(zadic_variety!(3, 6, [[1, 0, 0, 0, 0, 0], [2, 2, 2, 2, 2, 2]])),
///     roots_of_unity(3, 6)
/// );
/// assert_eq!(
///     Ok(zadic_variety!(5, 6, [
///         [1, 0, 0, 0, 0, 0],
///         [2, 1, 2, 1, 3, 4],
///         [3, 3, 2, 3, 1, 0],
///         [4, 4, 4, 4, 4 ,4],
///     ])),
///     roots_of_unity(5, 6)
/// );
/// ```
pub fn roots_of_unity<P>(p: P, precision: usize) -> AdicResult<AdicVariety<ZAdic>>
where P: Into<Prime> {
    let p = p.into();
    let n = if p.is_two() { 2 } else { u32::from(p)-1 };
    ZAdic::one(p).nth_root(n, precision)
}


/// Calculate the Teichmuller characters for given `p`.
/// These are the solutions of `x^p - x = 0`.
///
/// # Errors
/// Returns any errors that [`AdicInteger::nth_root`] returns
///
/// ```
/// # use adic::{teichmuller, zadic_variety};
/// assert_eq!(
///     Ok(zadic_variety!(2, 6, [
///         [0, 0, 0, 0, 0, 0],
///         [1, 0, 0, 0, 0, 0],
///     ])),
///     teichmuller(2, 6)
/// );
/// assert_eq!(
///     Ok(zadic_variety!(3, 6, [
///         [0, 0, 0, 0, 0, 0],
///         [1, 0, 0, 0, 0, 0],
///         [2, 2, 2, 2, 2, 2],
///     ])),
///     teichmuller(3, 6)
/// );
/// assert_eq!(
///     Ok(zadic_variety!(5, 6, [
///         [0, 0, 0, 0, 0, 0],
///         [1, 0, 0, 0, 0, 0],
///         [2, 1, 2, 1, 3, 4],
///         [3, 3, 2, 3, 1, 0],
///         [4, 4, 4, 4, 4 ,4],
///     ])),
///     teichmuller(5, 6)
/// );
/// ```
pub fn teichmuller<P>(p: P, precision: usize) -> AdicResult<AdicVariety<ZAdic>>
where P: Into<Prime> {
    let p = p.into();
    let zero = ZAdic::zero(p);
    let one = ZAdic::one(p);
    let pm2 = usize::try_from(p.m2()).expect("prime -> usize conversion");
    let coeffs = once(zero.clone())
        .chain(once(-one.clone()))
        .chain(repeat_n(zero.clone(), pm2))
        .chain(once(one.clone()))
        .collect::<Vec<_>>();
    let poly = AdicPolynomial::new(p, coeffs);
    poly.variety(precision)
}



#[cfg(test)]
mod test {

    use itertools::Itertools;
    use crate::{zadic_approx, zadic_variety};
    use super::{roots_of_unity, teichmuller};

    #[test]
    fn unity_roots() {

        let expected_unity = zadic_variety!(2, 6, [
            [1, 0, 0, 0, 0, 0],
            [1, 1, 1, 1, 1, 1],
        ]);
        let expected_teich = zadic_variety!(2, 6, [
            [0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0],
        ]);
        let actual_unity = roots_of_unity(2, 6).unwrap();
        let actual_teich = teichmuller(2, 6).unwrap();
        assert_eq!(expected_unity, actual_unity);
        assert_eq!(expected_teich, actual_teich);
        assert!(actual_unity.roots().try_len() == Ok(2));
        assert!(actual_unity.roots().contains(&zadic_approx![2, 6, [1, 0, 0, 0, 0, 0]]));
        assert!(actual_unity.roots().contains(&zadic_approx![2, 6, [1, 1, 1, 1, 1, 1]]));

        let expected_unity = zadic_variety!(3, 6, [
            [1, 0, 0, 0, 0, 0],
            [2, 2, 2, 2, 2, 2],
        ]);
        let expected_teich = zadic_variety!(3, 6, [
            [0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0],
            [2, 2, 2, 2, 2, 2],
        ]);
        let actual_unity = roots_of_unity(3, 6).unwrap();
        let actual_teich = teichmuller(3, 6).unwrap();
        assert_eq!(expected_unity, actual_unity);
        assert_eq!(expected_teich, actual_teich);
        assert!(actual_unity.roots().try_len() == Ok(2));
        assert!(actual_unity.roots().contains(&zadic_approx![3, 6, [1, 0, 0, 0, 0, 0]]));
        assert!(actual_unity.roots().contains(&zadic_approx![3, 6, [2, 2, 2, 2, 2, 2]]));

        let expected_unity = zadic_variety!(5, 6, [
            [1, 0, 0, 0, 0, 0],
            [2, 1, 2, 1, 3, 4],
            [3, 3, 2, 3, 1, 0],
            [4, 4, 4, 4, 4, 4],
        ]);
        let expected_teich = zadic_variety!(5, 6, [
            [0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0],
            [2, 1, 2, 1, 3, 4],
            [3, 3, 2, 3, 1, 0],
            [4, 4, 4, 4, 4, 4],
        ]);
        let actual_unity = roots_of_unity(5, 6).unwrap();
        let actual_teich = teichmuller(5, 6).unwrap();
        assert_eq!(expected_unity, actual_unity);
        assert_eq!(expected_teich, actual_teich);
        assert!(actual_unity.roots().try_len() == Ok(4));
        assert!(actual_unity.roots().contains(&zadic_approx![5, 6, [1, 0, 0, 0, 0, 0]]));
        assert!(actual_unity.roots().contains(&zadic_approx![5, 6, [4, 4, 4, 4, 4, 4]]));

        let expected_unity = zadic_variety!(7, 6, [
            [1, 0, 0, 0, 0, 0],
            [2, 4, 6, 3, 0, 2],
            [3, 4, 6, 3, 0, 2],
            [4, 2, 0, 3, 6, 4],
            [5, 2, 0, 3, 6, 4],
            [6, 6, 6, 6, 6, 6],
        ]);
        let expected_teich = zadic_variety!(7, 6, [
            [0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0],
            [2, 4, 6, 3, 0, 2],
            [3, 4, 6, 3, 0, 2],
            [4, 2, 0, 3, 6, 4],
            [5, 2, 0, 3, 6, 4],
            [6, 6, 6, 6, 6, 6],
        ]);
        let actual_unity = roots_of_unity(7, 6).unwrap();
        let actual_teich = teichmuller(7, 6).unwrap();
        assert_eq!(expected_unity, actual_unity);
        assert_eq!(expected_teich, actual_teich);
        assert!(actual_unity.roots().try_len() == Ok(6));
        assert!(actual_unity.roots().contains(&zadic_approx![7, 6, [1, 0, 0, 0, 0, 0]]));
        assert!(actual_unity.roots().contains(&zadic_approx![7, 6, [6, 6, 6, 6, 6, 6]]));

    }

}
