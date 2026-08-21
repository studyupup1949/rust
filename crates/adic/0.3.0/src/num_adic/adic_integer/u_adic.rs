use itertools::Itertools;
use num::BigUint;
use crate::{adic_valid, AdicError, ZAdicValuation};
use super::AdicInteger;


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
/// assert_eq!(2, two.u32_value());
/// let five = UAdic::new(5, vec![0, 1]);
/// assert_eq!(7, (two.clone() + five.clone()).u32_value());
/// assert_eq!(10, (two.clone() * five.clone()).u32_value());
/// ```
///
/// This representation EXACTLY matches the (base-p) digits for a non-negative real integer.
/// `2 = 2._5, 123 (base 5) = 123._5`
/// You can perform the same arithmetic on these numbers.
/// However, just like an unsigned number, these numbers do not subtract or divide well.
/// Instead, look to signed numbers for subtraction, [`IAdic`](crate::IAdic),
///  or full adic NUMBERS for division, [`QAdic`](crate::QAdic).
///
/// Many calculations truncate `AdicInteger`s to `UAdic`s in order to perform simple calculations.
pub struct UAdic {
    /// Adic prime
    pub (super) p: u32,
    /// Adic digits, each 0 to p-1
    pub (super) d: Vec<u32>,
}


impl UAdic {

    /// Create an adic number with the given digits; truncates if zero
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub fn new(p: u32, mut init_digits: Vec<u32>) -> Self {

        adic_valid::validate_p(p);
        adic_valid::validate_digits_mod_p(p, &init_digits);

        // Truncate zeros so there should never be leading zeros for a UAdic
        while let Some(0) = init_digits.last() {
            init_digits.pop();
        }

        Self {
            p,
            d: init_digits,
        }

    }

    /// Number of digits in `UAdic`, avoiding possibility of infinity from [`num_digits`](Self::num_digits) method
    pub fn finite_num_digits(&self) -> usize {
        self.d.len()
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
    ///
    /// # Panics
    /// Panics if digit is outside of [0, p)
    pub fn push_digit(&mut self, digit: u32) {

        adic_valid::validate_digit_mod_p(self.p, digit);

        if digit != 0 { self.d.push(digit) }

    }

    /// Push digits onto the end of the number; truncates if zero
    ///
    /// # Panics
    /// Panics if digits are outside of [0, p)
    pub fn extend_digits(&mut self, additional: &[u32]) {

        adic_valid::validate_digits_mod_p(self.p, additional);

        self.d.extend(additional);

        // Truncate zeros so there should never be leading zeros for a UAdic
        while let Some(0) = self.d.last() {
            self.d.pop();
        }

    }

    /// The (pseudo-)remainder and (pseudo-)quotient with respect to (p^n - 1).
    /// The caveat is that (p^n-1) itself will return (p^n-1) remainder and 0 quotient.
    /// This method is used in [`RAdic`](crate::RAdic) multiplication.
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
    /// Warning: This can overflow; use [`bigint_value`](Self::bigint_value) if unsure
    ///
    /// ```
    /// # use adic::uadic;
    /// assert_eq!(38, uadic!(5, [3, 2, 1]).u32_value());
    /// ```
    pub fn u32_value(&self) -> u32 {
        self.digits()
            .zip(0..)
            .map(|(d, k)| *d * self.p.pow(k))
            .sum()
    }

    /// The bigint representation for the natural number value of the number ([`u32_value`](`Self::u32_value`))
    ///
    /// ```
    /// # use num::BigUint;
    /// # use adic::uadic;
    /// assert_eq!(BigUint::from(38u32), uadic!(5, [3, 2, 1]).bigint_value());
    /// ```
    pub fn bigint_value(&self) -> BigUint {
        self.digits()
            .zip(0..)
            .map(|(d, k)| BigUint::from(*d) * BigUint::from(self.p).pow(k))
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
        ZAdicValuation::Finite(self.d.len())
    }
    fn digit(&self, n: usize) -> Result<u32, AdicError> {
        Ok(self.d.get(n).copied().unwrap_or(0))
    }
    fn digits(&self) -> impl Iterator<Item=&u32> {
        self.d.iter()
    }
    fn into_digits(self) -> impl Iterator<Item=u32> {
        self.d.into_iter()
    }
    fn digit_str(&self) -> String {
        self.digits().join("").chars().rev().collect::<String>()
    }
    fn into_split(self, n: usize) -> (UAdic, Self) {
        let mut remainder = Vec::with_capacity(n);
        let mut quotient = Vec::with_capacity(self.finite_num_digits());
        for (i, d) in self.d.into_iter().enumerate() {
            if i < n {
                remainder.push(d);
            } else {
                quotient.push(d);
            }
        }
        (UAdic::new(self.p, remainder), Self::new(self.p, quotient))
    }
    fn certainty(&self) -> ZAdicValuation {
        ZAdicValuation::PosInf
    }
}



#[cfg(test)]
mod tests {
    use num::{rational::Ratio, traits::Pow};
    use crate::{uadic, zadic_approx, ZAdic, ZAdicValuation, ZAdicVariety};
    use super::{AdicInteger, UAdic};

    use crate::num_adic::test_util::u::*;


    #[test]
    fn strips_zeros() {
        let strips_zeros = uadic!(5, [2, 0, 0, 0, 0]);
        assert_eq!(uadic!(5, [2]), strips_zeros);
        assert_eq!(one_twenty_five().certainty(), ZAdicValuation::PosInf);
    }

    #[test]
    fn u32_value() {
        assert_eq!(1, uadic!(5, [1]).u32_value());
        assert_eq!(2, uadic!(5, [2]).u32_value());
        assert_eq!(6, uadic!(5, [1, 1]).u32_value());
        assert_eq!(126, uadic!(5, [1, 0, 0, 1]).u32_value());
        assert_eq!(124, uadic!(5, [4, 4, 4]).u32_value());
    }

    #[test]
    fn truncate() {
        let u = uadic!(5, [1, 1, 1, 1, 1]);
        assert_eq!(5, u.d.len());
        assert_eq!(781, u.u32_value());
        let t = u.truncation(4);
        assert_eq!(4, t.d.len());
        assert_eq!(156, t.u32_value());
        let t = u.truncation(3);
        assert_eq!(3, t.d.len());
        assert_eq!(31, t.u32_value());
        let t = u.truncation(2);
        assert_eq!(2, t.d.len());
        assert_eq!(6, t.u32_value());
        let t = u.truncation(1);
        assert_eq!(1, t.d.len());
        assert_eq!(1, t.u32_value());
        let t = u.truncation(6);
        assert_eq!(5, t.d.len());
        assert_eq!(781, t.u32_value());
        let t = u.truncation(0);
        assert_eq!(0, t.d.len());
        assert_eq!(0, t.u32_value());
        assert!(t.is_zero());
    }

    #[test]
    fn pseudo_pn_minus_1_rem_quot() {
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
    fn u_adic_norm() {
        assert_eq!(ZAdicValuation::PosInf, zero().valuation());
        assert_eq!(Ratio::ZERO, zero().norm());
        assert_eq!(ZAdicValuation::Finite(0), one().valuation());
        assert_eq!(Ratio::new(1, 1), one().norm());
        assert_eq!(ZAdicValuation::Finite(0), two().valuation());
        assert_eq!(Ratio::new(1, 1), two().norm());
        assert_eq!(ZAdicValuation::Finite(1), five().valuation());
        assert_eq!(Ratio::new(1, 5), five().norm());
        assert_eq!(ZAdicValuation::Finite(0), six().valuation());
        assert_eq!(Ratio::new(1, 1), six().norm());
        assert_eq!(ZAdicValuation::Finite(2), twenty_five().valuation());
        assert_eq!(Ratio::new(1, 25), twenty_five().norm());
        assert_eq!(ZAdicValuation::Finite(3), one_twenty_five().valuation());
        assert_eq!(Ratio::new(1, 125), one_twenty_five().norm());
        assert_eq!(ZAdicValuation::Finite(0), one_twenty_six().valuation());
        assert_eq!(Ratio::new(1, 1), one_twenty_six().norm());
    }

    #[test]
    fn nth_root() {

        let check = |p: u32, a: &UAdic, n: u32, precision: usize, roots: Vec<ZAdic>| {
            // Check each root powers to match a to at least precision digits
            for root in &roots {
                assert_eq!(a.truncation(precision), root.pow(n).into_truncation_to_uadic().unwrap());
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

}
