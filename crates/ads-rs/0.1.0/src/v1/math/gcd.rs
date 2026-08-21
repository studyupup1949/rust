use std::mem::{replace, swap};

/// Finds the GCD (Greatest Common Divisor) for an array of elements.
/// # Examples
/// ```
/// use ads_rs::prelude::v1::math::gcd_many;
///
/// let res0 = gcd_many(&[42, 8, 144]);
/// let res1 = gcd_many(&[89, 144, 233, 377, 610]);
/// let res2 = gcd_many(&[25, 105, 235, 100]);
///
/// assert_eq!(2, res0);
/// assert_eq!(1, res1);
/// assert_eq!(5, res2);
/// ```
/// ## Corner cases
/// - GCD of an empty array equals 0.
/// - GCD of a single element array equals that element.
/// ```
/// use ads_rs::prelude::v1::math::gcd_many;
///
/// let res0 = gcd_many(&[]);
/// let res1 = gcd_many(&[25]);
///
/// assert_eq!(0, res0);
/// assert_eq!(25, res1);
/// ```
/// # Implementation details
/// - Stein's algorithm is used.
/// - Time complexity: O(K * N<sup>2</sup>) where:
///     - N - bits count in the biggest number.
///     - K - number's count
pub fn gcd_many(elems: &[u64]) -> u64 {
    if elems.is_empty() {
        return 0;
    }

    if elems.len() == 1 {
        return elems[0];
    }

    elems.iter().fold(0, |acc, e| {
        let (mut lhs, mut rhs) = (acc, *e);

        if lhs == 0 || rhs == 0 {
            return lhs | rhs;
        }

        // find common factor of 2
        let shift = (lhs | rhs).trailing_zeros();

        // divide lhs and rhs by 2 until odd
        rhs >>= rhs.trailing_zeros();
        while lhs > 0 {
            lhs >>= lhs.trailing_zeros();

            if rhs > lhs {
                swap(&mut lhs, &mut rhs);
            }

            lhs -= rhs
        }

        rhs << shift
    })
}

/// Finds an extended GCD (Greatest Common Divisor) for a pair of numbers.  
/// "Extended" means that algorithm will return not only GCD, but two coefficients `x` and `y` such that the equality
///
/// x * lhs + y * rhs = gcd(lhs, rhs)
///
/// holds.
/// # Examples
/// ```
/// use ads_rs::prelude::v1::math::extended_gcd;
///
/// let res0 = extended_gcd(30, 20);
/// let res1 = extended_gcd(15, 35);
/// let res2 = extended_gcd(161, 28);
///
/// assert_eq!((10, 1, -1), res0);
/// assert_eq!((5, -2, 1), res1);
/// assert_eq!((7, -1, 6), res2);
/// ```
/// ## Corner case
/// - Result of `extended_gcd(0, 0)` equals tuple `(0, 1, 0)`.
/// - Negative numbers is not supported, but implementation allows it.
/// ```
/// use ads_rs::prelude::v1::math::extended_gcd;
///
/// let res = extended_gcd(0, 0);
///
/// assert_eq!((0, 1, 0), res);
/// ```
/// # Implementation details
/// - Euclid's algorithm used, because its extended version is faster than Stein's algorithm
/// - Time complexity is O(log<sub>2</sub>(min(lhs, rhs)))
pub fn extended_gcd(lhs: u64, rhs: u64) -> (u64, i64, i64) {
    let (mut x, mut y) = (1, 0);
    let (mut x1, mut y1, mut lhs1, mut rhs1) = (0i64, 1i64, lhs, rhs);

    while rhs1 > 0 {
        let q = lhs1 / rhs1;

        let new_x1 = x - (q as i64) * x1;
        x = replace(&mut x1, new_x1);

        let new_y1 = y - (q as i64) * y1;
        y = replace(&mut y1, new_y1);

        let new_rhs1 = lhs1 - q * rhs1;
        lhs1 = replace(&mut rhs1, new_rhs1);
    }

    (lhs1, x, y)
}

/// Finds an GCD (Greatest Common Divisor) for a pair of numbers.
/// # Examples
/// ```
/// use ads_rs::prelude::v1::math::gcd;
///
/// let res0 = gcd(42, 144);
/// let res1 = gcd(377, 610);
/// let res2 = gcd(105, 25);
///
/// assert_eq!(6, res0);
/// assert_eq!(1, res1);
/// assert_eq!(5, res2);
/// ```
/// ## Corner case
/// GCD of both zero numbers equals 0.
/// ```
/// use ads_rs::prelude::v1::math::gcd;
///
/// let res = gcd(0, 0);
///
/// assert_eq!(0, res);
/// ```
/// # Implementation details
/// - Stein's algorithm used (from [`gcd_many`]).
/// - Time complexity: O(N<sup>2</sup>) where N - number of bits in the biggest number.
#[inline]
pub fn gcd(lhs: u64, rhs: u64) -> u64 {
    gcd_many(&[lhs, rhs])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extended_gcd_works() {
        // arrange
        let test_suits = [
            // Zero rhs
            (123, 0, (123, 1, 0)),
            // Zero lhs
            (0, 123, (123, 0, 1)),
            // Regular case
            (2048, 48, (16, -1, 43)),
            // Relative prime
            (2052, 617, (1, 132, -439)),
            // Zero lhs and rhs
            (0, 0, (0, 1, 0)),
        ];

        // act
        let result: Vec<(u64, i64, i64)> =
            test_suits.iter().map(|t| extended_gcd(t.0, t.1)).collect();

        // assert
        for i in 0..test_suits.len() {
            assert_eq!(test_suits[i].2, result[i]);
        }
    }
    #[test]
    fn gcd_many_works() {
        // arrange
        let test_suits = [
            // Empty array
            (vec![], 0),
            // Single element
            (vec![223], 223),
            // Relative prime numbers
            (vec![1, 2, 3, 4, 5], 1),
            // Regular case
            (vec![8, 24, 156, 36], 4),
            // All zeros
            (vec![0, 0, 0, 0], 0),
        ];

        // act
        let result: Vec<u64> = test_suits.iter().map(|t| gcd_many(&t.0)).collect();

        // assert
        for i in 0..test_suits.len() {
            assert_eq!(test_suits[i].1, result[i]);
        }
    }
}
