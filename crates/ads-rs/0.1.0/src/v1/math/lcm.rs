use crate::v1::math::gcd_many;

/// Finds the LCM (Least Common Multiple) for an array of elements.
/// # Examples
/// ```
/// use ads_rs::prelude::v1::math::lcm_many;
///
/// let res0 = lcm_many(&[42, 8, 144]);
/// let res1 = lcm_many(&[89, 144, 233, 377, 610]);
/// let res2 = lcm_many(&[25, 105, 235, 100]);
///
/// assert_eq!(24192, res0);
/// assert_eq!(686719856160, res1);
/// assert_eq!(12337500, res2);
/// ```
/// ## Corner cases
/// - LCM of an empty array equals 0.
/// - LCM of a single element array equals that element.
/// ```
/// use ads_rs::prelude::v1::math::lcm_many;
///
/// let res0 = lcm_many(&[]);
/// let res1 = lcm_many(&[25]);
///
/// assert_eq!(0, res0);
/// assert_eq!(25, res1);
/// ```
/// # Implementation details
/// - Stein's algorithm is used to find GCD (from [`gcd_many`]).
/// - Time complexity: O(K * N<sup>2</sup>) where:
///     - N - bits count in the biggest number.
///     - K - number's count
pub fn lcm_many(elems: &[u64]) -> u64 {
    if elems.is_empty() {
        return 0;
    }

    if elems.len() == 1 {
        return elems[0];
    }

    let gcd = gcd_many(elems);

    // GCD is zero only when all elements are zeros
    if gcd == 0 {
        return 0;
    }

    elems[1..].iter().fold(elems[0] / gcd, |acc, e| acc * (*e))
}

/// Finds an LCM (Least Common Multiple) for a pair of numbers.
/// # Examples
/// ```
/// use ads_rs::prelude::v1::math::lcm;
///
/// let res0 = lcm(42, 144);
/// let res1 = lcm(377, 610);
/// let res2 = lcm(105, 25);
///
/// assert_eq!(1008, res0);
/// assert_eq!(229970, res1);
/// assert_eq!(525, res2);
/// ```
/// ## Corner case
/// LCM of both zero numbers equals 0.
/// ```
/// use ads_rs::prelude::v1::math::lcm;
///
/// let res = lcm(0, 0);
///
/// assert_eq!(0, res);
/// ```
/// # Implementation details
/// - Stein's algorithm used (from [`gcd_many`]).
/// - Time complexity: O(N<sup>2</sup>) where N - number of bits in the biggest number.
#[inline]
pub fn lcm(lhs: u64, rhs: u64) -> u64 {
    lcm_many(&[lhs, rhs])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcm_many_works() {
        // arrange
        let test_suits = [
            // Empty array
            (vec![], 0),
            // Single element
            (vec![223], 223),
            // Relative prime numbers
            (vec![1, 2, 3, 4, 5], 120),
            // Regular case
            (vec![8, 24, 156, 36], 269568),
            // All zeros
            (vec![0, 0, 0, 0], 0),
        ];

        // act
        let result: Vec<u64> = test_suits.iter().map(|t| lcm_many(&t.0)).collect();

        // assert
        for i in 0..test_suits.len() {
            assert_eq!(test_suits[i].1, result[i]);
        }
    }
}
