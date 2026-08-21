use std::{
    fmt,
    iter::repeat,
};
use itertools::Itertools;
use num::{traits::Pow, BigInt};
use num_prime::nt_funcs::is_prime;
use crate::AdicError;
use super::{AdicInteger, ZAdicValuation};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Adic that represents an unsigned integer ([`uadic`](crate::uadic))
///
/// An [`AdicInteger`](crate::AdicInteger).
/// The struct holds a finite list of digits, from the "zero" digit place to the max.
/// This is the most basic Adic struct, just consisting of a finite number of digits.
/// With this, you can represent exactly the non-negative real integers.
///
/// ```
/// # use adic::{AdicInteger, UAdic};
/// assert_eq!("4321._5", UAdic::new(5, vec![1, 2, 3, 4]).to_string());
/// let two = UAdic::new(5, vec![2]);
/// assert_eq!(2, two.integer_value());
/// let five = UAdic::new(5, vec![0, 1]);
/// assert_eq!(7, (two.clone() + five.clone()).integer_value());
/// assert_eq!(10, (two.clone() * five.clone()).integer_value());
/// ```
///
/// This representation EXACTLY matches the (base-p) digits for a non-negative real integer.
/// `2 = 2._5, 123 = 123._5`
/// You can perform the same arithmetic on these numbers.
/// However, just like an unsigned number, these numbers do not subtract or divide well.
/// Instead, look to rationals, [`RAdic`](crate::RAdic), or if they can be approximate, [`ZAdic`](crate::ZAdic).
///
/// Many calculations truncate `AdicInteger`s to `UAdic`s in order to perform simple calculations.
pub struct UAdic {
    /// Adic prime
    p: u32,
    /// Adic digits, each 0 to p-1
    d: Vec<u32>,
}


impl UAdic {

    /// Create an adic number with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime
    pub fn new(p: u32, mut init_digits: Vec<u32>) -> Self {

        assert!(is_prime(&p, None).probably());

        // Truncate zeros so there should never be leading zeros for a UAdic
        while let Some(0) = init_digits.last() {
            init_digits.pop();
        }

        Self {
            p,
            d: init_digits,
        }

    }

    /// Create adic number associated with (unsigned) integer n
    pub fn from_integer(p: u32, mut n: u32) -> Self {
        let mut digits = vec![];
        while n != 0 {
            digits.push(n % p);
            n = n / p;
        }
        Self::new(p, digits)
    }

    /// Consume `UAdic` and get the digits vector
    ///
    /// ```
    /// # use adic::uadic;
    /// let u = uadic!(5, [1, 2, 3, 4, 0]);
    /// assert_eq!(vec![1, 2, 3, 4], u.into_digits_vec());
    /// ```
    pub fn into_digits_vec(self) -> Vec<u32> {
        self.into_digits().collect()
    }

    /// Push another digit onto the end of the number; truncates if zero
    pub fn push_digit(&mut self, digit: u32) {
        if digit != 0 { self.d.push(digit) }
    }

    /// Push digits onto the end of the number; truncates if zero
    pub fn extend_digits<I>(&mut self, additional: I)
    where I: Iterator<Item=u32> {

        self.d.extend(additional);

        // Truncate zeros so there should never be leading zeros for a UAdic
        while let Some(0) = self.d.last() {
            self.d.pop();
        }

    }

    /// Split adic into digits [0, n) and [n, ...).
    /// This splits the number into remainder and quotient.
    ///
    /// `a = (a % p^n) + p^n * (a // p^n)`
    ///
    /// ```
    /// # use adic::uadic;
    /// let u = uadic!(5, [1, 2, 3, 4]);
    /// let (r, q) = u.split(2);
    /// assert_eq!(uadic!(5, [1, 2]), r);
    /// assert_eq!(uadic!(5, [3, 4]), q);
    /// ```
    pub fn split(&self, n: usize) -> (Self, Self) {
        let (remainder, quotient) = match self.d.split_at_checked(n) {
            None => (self.d.clone(), vec![]),
            Some(digits) => (digits.0.to_vec(), digits.1.to_vec()),
        };
        (
            Self::new(self.p, remainder),
            Self::new(self.p, quotient),
        )
    }

    #[must_use]
    /// Divide an adic number by p^n.
    /// This can be thought of as the quotient a // p^n
    ///
    /// ```
    /// # use adic::uadic;
    /// let u = uadic!(5, [1, 2, 3, 4]);
    /// let q = u.int_div(2);
    /// assert_eq!(uadic!(5, [3, 4]), q);
    /// ```
    pub fn int_div(&self, n: usize) -> Self {
        Self::new(self.p, match self.d.split_at_checked(n) {
            None => vec![],
            Some(digits) => digits.1.to_vec(),
        })
    }

    /// The (pseudo-)remainder and (pseudo-)quotient with respect to (p^n - 1)
    ///
    /// The caveat is that (p^n-1) itself will return (p^n-1) remainder and 0 quotient
    ///
    /// ```
    /// # use adic::{uadic, AdicInteger, UAdic};
    /// let u = uadic!(5, [1, 2, 3, 4]);
    /// let (r, q) = u.pseudo_pn_minus_1_rem_quot(2);
    /// assert_eq!(uadic!(5, [0, 2]), r);
    /// assert_eq!(uadic!(5, [4, 4]), q);
    /// let u = uadic!(5, [4, 4]);
    /// assert_eq!((u.clone(), UAdic::zero(5)), u.pseudo_pn_minus_1_rem_quot(2));
    /// ```
    pub fn pseudo_pn_minus_1_rem_quot(self, n: usize) -> (Self, Self) {

        // As a note, there's a trick for calculating quotient and mod of a UAdic with respect to p^n-1
        // You split at n, the (p^n) quotient is a partial quotient
        //  and you ADD the quotient to the remainder for a trial remainder
        // If the trial remainder grows bigger than p^n, you repeat, accumulating quotients
        // Once the trial remainder is below p^n, your accumulated quotient is (almost!) the quotient
        // The big caveat is that p^n-1 is left untouched at the end, instead of giving mod -> 0

        let (mut acc_quotient, mut trial_remainder) = (UAdic::zero(self.p), self);
        while trial_remainder.d.len() > n {
            let (new_remainder, new_quotient) = trial_remainder.split(n);
            acc_quotient = acc_quotient + new_quotient.clone();
            trial_remainder = new_remainder + new_quotient;
        }
        (trial_remainder, acc_quotient)

    }

    /// The natural number value of the number, e.g. 5-adic 123 is 25+10+3=38
    ///
    /// Warning: This can overflow; use [`UAdic::big_integer_value`] if unsure
    ///
    /// ```
    /// # use adic::uadic;
    /// assert_eq!(38, uadic!(5, [3, 2, 1]).integer_value());
    /// ```
    pub fn integer_value(&self) -> u32 {
        self.digits()
            .enumerate()
            .map(|(k, d)| *d * self.p.pow(k as u32))
            .sum()
    }

    /// The bigint representation for the natural number value of the number
    ///
    /// ```
    /// # use num::BigInt;
    /// # use adic::uadic;
    /// assert_eq!(BigInt::from(38), uadic!(5, [3, 2, 1]).big_integer_value());
    /// ```
    pub fn big_integer_value(&self) -> BigInt {
        self.digits()
            .enumerate()
            .map(|(k, d)| BigInt::from(*d) * BigInt::from(self.p).pow(k as u32))
            .sum()
    }

}


impl AdicInteger for UAdic {
    fn zero(p: u32) -> Self {
        Self::new(p, vec![])
    }
    fn one(p: u32) -> Self {
        Self::new(p, vec![1])
    }
    fn p(&self) -> u32 {
        self.p
    }
    fn num_digits(&self) -> ZAdicValuation {
        ZAdicValuation::Finite(self.d.len() as u32)
    }
    fn digit(&self, n: u32) -> Result<u32, AdicError> {
        Ok(self.d.get(n as usize).copied().unwrap_or(0))
    }
    fn digits(&self) -> impl Iterator<Item=&u32> {
        self.d.iter()
    }
    fn into_digits(self) -> impl Iterator<Item=u32> {
        self.d.into_iter()
    }
    fn certainty(&self) -> ZAdicValuation {
        ZAdicValuation::PosInf
    }
}


impl std::ops::Add for UAdic {
    type Output = UAdic;
    fn add(self, rhs: Self) -> Self::Output {

        // Add the like digits one-by-one
        // Then reduce each to the [0, p) range and handle the carry

        assert!(self.p == rhs.p, "{:?}", AdicError::MixedCharacteristic);
        let p = self.p;

        let ls = self.d.len();
        let lr = rhs.d.len();

        let (base_digits, adding_digits) = if ls >= lr {
            (self.d, rhs.d)
        } else {
            (rhs.d, self.d)
        };

        let mut summed_digits = Vec::with_capacity(std::cmp::max(ls, lr) + 1);
        summed_digits.extend(
            base_digits
                .into_iter()
                .zip(adding_digits.into_iter().chain(repeat(0)))
                .map(|(d1, d2)| d1 + d2)
                .collect::<Vec<_>>()
        );

        let mut carry = 0;
        for digit in &mut summed_digits {
            let bigger_digit = *digit + carry;
            *digit = bigger_digit % p;
            carry = bigger_digit / p;
        }
        while carry > 0 {
            summed_digits.push(carry % p);
            carry = carry / p;
        }

        Self::new(p, summed_digits)

    }
}


impl std::ops::Mul for UAdic {
    type Output = UAdic;
    fn mul(self, rhs: Self) -> Self::Output {

        // Turn rhs around and "drag it across" longer to create the multiplied digits one-by-one
        // Then reduce each to the [0, p) range and handle the carry

        assert!(self.p == rhs.p, "{:?}", AdicError::MixedCharacteristic);
        let p = self.p;

        let ls = self.d.len();
        let lr = rhs.d.len();
        if ls + lr == 0 {
            return UAdic::zero(p);
        }
        let lt = ls + lr - 1;

        // Performance critical here!
        let self_digits = self.d.clone();
        let mut rev_rhs_digits = rhs.d.clone();
        rev_rhs_digits.reverse();
        let mut summed_digits = Vec::with_capacity(lt + 1);
        summed_digits.extend((0..lt).map(|digit_place| {
            let (self_skip, rhs_skip) = if (digit_place >= lr) {
                (digit_place + 1 - lr, 0)
            } else {
                (0, lr - digit_place - 1)
            };
            let mut d = 0;
            for (&ds, &dr) in self_digits[self_skip..].iter().zip(rev_rhs_digits[rhs_skip..].iter()) {
                d += ds * dr;
            }
            d
        }));

        let mut carry = 0;
        for digit in &mut summed_digits {
            let bigger_digit = *digit + carry;
            *digit = bigger_digit % p;
            carry = bigger_digit / p;
        }
        while carry > 0 {
            summed_digits.push(carry % p);
            carry = carry / p;
        }

        Self::new(p, summed_digits)

    }
}


impl Pow<u32> for &UAdic {
    type Output = UAdic;
    fn pow(self, power: u32) -> Self::Output {
        repeat(
            self.clone()
        ).take(
            power as usize
        ).reduce(
            |acc, e| acc * e
        ).unwrap_or(
            UAdic::one(self.p)
        )
    }
}


impl fmt::Display for UAdic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p = self.p;
        let digits = if self.d.is_empty() {
            "0".to_string()
        } else {
            self.digits().join("").chars().rev().collect::<String>()
        };
        write!(f, "{digits}._{p}")
    }
}


#[cfg(test)]
mod tests {
    use itertools::{Itertools, repeat_n};
    use num::{traits::Pow, Rational32};
    use crate::{uadic, zadic_approx, ZAdic, ZAdicValuation, ZAdicVariety};
    use super::{AdicInteger, UAdic};

    fn zero() -> UAdic { uadic!(5, []) }
    fn one() -> UAdic { uadic!(5, [1]) }
    fn two() -> UAdic { uadic!(5, [2]) }
    fn three() -> UAdic { uadic!(5, [3]) }
    fn four() -> UAdic { uadic!(5, [4]) }
    fn five() -> UAdic { uadic!(5, [0, 1]) }
    fn six() -> UAdic { uadic!(5, [1, 1]) }
    fn eight() -> UAdic { uadic!(5, [3, 1]) }
    fn ten() -> UAdic { uadic!(5, [0, 2]) }
    fn twenty_four() -> UAdic { uadic!(5, [4, 4]) }
    fn twenty_five() -> UAdic { uadic!(5, [0, 0, 1]) }
    fn twelve() -> UAdic { uadic!(5, [2, 2]) }
    fn one_fifty_six() -> UAdic { uadic!(5, [1, 1, 1, 1]) }
    fn six_twenty_four() -> UAdic { uadic!(5, [4, 4, 4, 4]) }
    fn app_neg_one() -> UAdic { uadic!(5, [4, 4, 4, 4]) }
    fn app_neg_two() -> UAdic { uadic!(5, [3, 4, 4, 4]) }
    fn app_neg_three() -> UAdic { uadic!(5, [2, 4, 4, 4]) }
    fn _app_neg_four() -> UAdic { uadic!(5, [1, 4, 4, 4]) }
    fn app_neg_five() -> UAdic { uadic!(5, [0, 4, 4, 4]) }
    fn app_neg_ten() -> UAdic { uadic!(5, [0, 3, 4, 4]) }
    fn one_twenty_five() -> UAdic { uadic!(5, [0, 0, 0, 1]) }
    fn one_twenty_six() -> UAdic { uadic!(5, [1, 0, 0, 1]) }


    #[test]
    fn strips_zeros() {
        let strips_zeros = uadic!(5, [2, 0, 0, 0, 0]);
        assert_eq!(uadic!(5, [2]), strips_zeros);
        assert_eq!(one_twenty_five().certainty(), ZAdicValuation::PosInf);
    }

    #[test]
    fn test_add_u_adic() {
        assert_eq!(two(), one() + one());
        assert_eq!(three(), two() + one());
        assert_eq!(five(), two() + three());
        let neg_one_plus_neg_one = app_neg_one() + app_neg_one();
        assert_eq!(uadic!(5, [3, 4, 4, 4, 1]), neg_one_plus_neg_one);
        assert_eq!(app_neg_two(), neg_one_plus_neg_one.into_truncation(4));
        let neg_two_plus_neg_three = app_neg_two() + app_neg_three();
        assert_eq!(uadic!(5, [0, 4, 4, 4, 1]), neg_two_plus_neg_three);
        assert_eq!(app_neg_five(), neg_two_plus_neg_three.into_truncation(4));
        let neg_five_plus_neg_five = app_neg_five() + app_neg_five();
        assert_eq!(uadic!(5, [0, 3, 4, 4, 1]), neg_five_plus_neg_five);
        assert_eq!(app_neg_ten(), neg_five_plus_neg_five.into_truncation(4));
        let two_plus_neg_two = two() + app_neg_two();
        assert_eq!(uadic!(5, [0, 0, 0, 0, 1]), two_plus_neg_two);
        assert_eq!(zero(), two_plus_neg_two.into_truncation(4));
        let four_plus_one_grows = uadic!(5, [4]) + uadic!(5, [1]);
        assert_eq!(uadic!(5, [0, 1]), four_plus_one_grows);
    }

    #[test]
    fn test_mul_u_adic() {
        assert_eq!(one(), one() * one());
        assert_eq!(two(), two() * one());
        assert_eq!(six(), two() * three());
        let neg_one_mul_neg_one = app_neg_one() * app_neg_one();
        assert_eq!(uadic!(5, [1, 0, 0, 0, 3, 4, 4, 4]), neg_one_mul_neg_one);
        assert_eq!(one(), neg_one_mul_neg_one.into_truncation(4));
        let neg_two_mul_neg_three = app_neg_two() * app_neg_three();
        assert_eq!(uadic!(5, [1, 1, 0, 0, 0, 4, 4, 4]), neg_two_mul_neg_three);
        assert_eq!(six(), neg_two_mul_neg_three.into_truncation(4));
        assert_eq!(zero(), zero() * two());
        assert_eq!(zero(), zero() * app_neg_two());
        let truncates_zeros = uadic!(5, [2, 0, 0, 0, 0]) * uadic!(5, [3, 0, 0, 0, 0]);
        assert_eq!(uadic!(5, [1, 1]), truncates_zeros);
        assert_eq!(ten(), five() * two());
        assert_eq!(twenty_five(), five() * five());
    }

    #[test]
    fn test_pow_u_adic() {
        assert_eq!(zero(), zero().pow(2));
        assert_eq!(zero(), zero().pow(3));
        assert_eq!(one(), one().pow(2));
        assert_eq!(one(), one().pow(3));
        assert_eq!(four(), two().pow(2));
        assert_eq!(eight(), two().pow(3));
        assert_eq!(twenty_five(), five().pow(2));
        assert_eq!(one(), app_neg_two().pow(0));
        assert_eq!(app_neg_one(), app_neg_one().pow(1));
        assert_eq!(uadic!(5, [1, 0, 0, 0, 3, 4, 4, 4]), app_neg_one().pow(2));
        assert_eq!(uadic!(5, [4, 0, 0, 0, 1, 4, 4, 4]), app_neg_two().pow(2));
    }

    #[test]
    fn test_u_adic_ops_many() {
        // Test addition and multiplication over many integers using integer_value
        let p = 5;
        let n1 = 3;
        let n2 = 2;
        let firsts = repeat_n(0..p, n1).multi_cartesian_product().map(
            |digits| UAdic::new(p, digits[0..n1].to_vec())
        );
        let seconds = repeat_n(0..p, n2).multi_cartesian_product().map(
            |digits| UAdic::new(p, digits[0..n2].to_vec())
        );
        for (first, second) in firsts.cartesian_product(seconds) {
            let first_val = first.integer_value();
            let second_val = second.integer_value();
            let sum_val = (first.clone() + second.clone()).integer_value();
            let prod_val = (first.clone() * second.clone()).integer_value();
            assert_eq!(first_val + second_val, sum_val);
            assert_eq!(first_val * second_val, prod_val);
        }
    }

    #[test]
    #[should_panic]
    fn test_non_prime() {
        let _ = uadic!(6, [2]);
    }

    #[test]
    #[should_panic]
    fn test_mixed_characteristic() {
        let _ = uadic!(5, [1]) + uadic!(7, [1]);
    }

    #[test]
    fn test_integer_value() {
        assert_eq!(1, uadic!(5, [1]).integer_value());
        assert_eq!(2, uadic!(5, [2]).integer_value());
        assert_eq!(6, uadic!(5, [1, 1]).integer_value());
        assert_eq!(126, uadic!(5, [1, 0, 0, 1]).integer_value());
        assert_eq!(124, uadic!(5, [4, 4, 4]).integer_value());
    }

    #[test]
    fn test_truncate() {
        let u = uadic!(5, [1, 1, 1, 1, 1]);
        assert_eq!(5, u.d.len());
        assert_eq!(781, u.integer_value());
        let t = u.truncation(4);
        assert_eq!(4, t.d.len());
        assert_eq!(156, t.integer_value());
        let t = u.truncation(3);
        assert_eq!(3, t.d.len());
        assert_eq!(31, t.integer_value());
        let t = u.truncation(2);
        assert_eq!(2, t.d.len());
        assert_eq!(6, t.integer_value());
        let t = u.truncation(1);
        assert_eq!(1, t.d.len());
        assert_eq!(1, t.integer_value());
        let t = u.truncation(6);
        assert_eq!(5, t.d.len());
        assert_eq!(781, t.integer_value());
        let t = u.truncation(0);
        assert_eq!(0, t.d.len());
        assert_eq!(0, t.integer_value());
        assert!(t.is_zero());
    }

    #[test]
    fn test_pseudo_pn_minus_1_rem_quot() {
        let check = |r: &UAdic, q: &UAdic, a: &UAdic, n: usize| {
            assert_eq!((r.clone(), q.clone()), a.clone().pseudo_pn_minus_1_rem_quot(n));
        };
        check(&zero(), &zero(), &zero(), 0);
        check(&zero(), &zero(), &zero(), 1);
        check(&zero(), &zero(), &zero(), 2);
        for a in [&one(), &two(), &three(), &four()] {
            check(a, &zero(), a, 1);
            check(a, &zero(), a, 2);
        }
        check(&one(), &one(), &five(), 1);
        check(&five(), &zero(), &five(), 2);
        check(&twelve(), &six(), &one_fifty_six(), 2);
        check(&twenty_four(), &twenty_five(), &six_twenty_four(), 2);
    }

    #[test]
    fn test_u_adic_norm() {
        assert_eq!(ZAdicValuation::PosInf, zero().valuation());
        assert_eq!(Rational32::ZERO, zero().norm());
        assert_eq!(ZAdicValuation::Finite(0), one().valuation());
        assert_eq!(Rational32::new(1, 1), one().norm());
        assert_eq!(ZAdicValuation::Finite(0), two().valuation());
        assert_eq!(Rational32::new(1, 1), two().norm());
        assert_eq!(ZAdicValuation::Finite(1), five().valuation());
        assert_eq!(Rational32::new(1, 5), five().norm());
        assert_eq!(ZAdicValuation::Finite(0), six().valuation());
        assert_eq!(Rational32::new(1, 1), six().norm());
        assert_eq!(ZAdicValuation::Finite(2), twenty_five().valuation());
        assert_eq!(Rational32::new(1, 25), twenty_five().norm());
        assert_eq!(ZAdicValuation::Finite(3), one_twenty_five().valuation());
        assert_eq!(Rational32::new(1, 125), one_twenty_five().norm());
        assert_eq!(ZAdicValuation::Finite(0), one_twenty_six().valuation());
        assert_eq!(Rational32::new(1, 1), one_twenty_six().norm());
    }

    #[test]
    fn test_nth_root() {

        let check = |p: u32, a: &UAdic, n: u32, precision: u32, roots: Vec<ZAdic>| {
            // Check each root powers to match a to at least precision digits
            for root in &roots {
                assert_eq!(a.truncation(precision as usize), root.pow(n).into_truncation_to_uadic().unwrap());
            }
            // Check roots match the output of nth_root
            assert_eq!(Ok(ZAdicVariety::new(p, roots)), a.nth_root(n, precision));
        };

        check(2, &uadic!(2, [1, 0, 0, 0, 1]), 2, 6, vec![
            zadic_approx!(2, 6, [1, 0, 0, 1, 0, 1]),
            zadic_approx!(2, 6, [1, 1, 1, 0, 1, 0]),
        ]);

        check(5, &uadic!(5, [1]), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);

        check(5, &uadic!(5, [2]), 2, 6, vec![]);

        check(7, &uadic!(7, [2]), 2, 6, vec![
            zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2]),
            zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4]),
        ]);

    }

    #[test]
    fn test_display() {
        assert_eq!("0._5", zero().to_string());
        assert_eq!("1._5", one().to_string());
        assert_eq!("2._5", two().to_string());
        assert_eq!("3._5", three().to_string());
        assert_eq!("4._5", four().to_string());
        assert_eq!("10._5", five().to_string());
        assert_eq!("11._5", six().to_string());
        assert_eq!("20._5", ten().to_string());
        assert_eq!("44._5", twenty_four().to_string());
        assert_eq!("100._5", twenty_five().to_string());
        assert_eq!("22._5", twelve().to_string());
        assert_eq!("1111._5", one_fifty_six().to_string());
        assert_eq!("4444._5", six_twenty_four().to_string());
        assert_eq!("1000._5", one_twenty_five().to_string());
        assert_eq!("1001._5", one_twenty_six().to_string());
    }

}
