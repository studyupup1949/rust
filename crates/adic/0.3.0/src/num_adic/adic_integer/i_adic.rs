use std::iter::repeat;
use itertools::Itertools;
use num::{BigInt, Zero};
use crate::{adic_valid, AdicError, AdicSign, ZAdicValuation};
use super::{AdicInteger, UAdic};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Adic that represents an signed integer ([`iadic_pos`](crate::iadic_pos), [`iadic_neg`](crate::iadic_neg))
///
/// An [`AdicInteger`](crate::AdicInteger).
/// This can represent any real, signed integer.
/// The struct holds a finite list of digits, from the "zero" digit place to the max.
/// After this, it holds either all zeros or all (p-1)s, aka -1 in F_p.
/// This second-most basic Adic struct, has a finite number of digits and a sign.
/// With this, you can represent exactly the real integers.
///
/// ```
/// # use adic::{AdicInteger, IAdic};
/// assert_eq!("4321._5", IAdic::new_pos(5, vec![1, 2, 3, 4]).to_string());
/// let two = IAdic::new_pos(5, vec![2]);
/// assert_eq!(2, two.i32_value());
/// let neg_two = IAdic::new_neg(5, vec![3]);
/// assert_eq!("(4)3._5", neg_two.to_string());
/// assert_eq!(-2, neg_two.i32_value());
/// assert!((two + neg_two).is_zero());
/// ```
///
/// This representation EXACTLY matches the real number base-p digits for a positive real integer.
/// `2 = 2._5, 123 (base 5) = 123._5`
/// You can perform the same arithmetic on these numbers.
/// However, as a signed number, we can also represent negative numbers.
/// `-2 = (4)3._5 = ...4443._5, -123 (base 5) = (4)322._5`
///
/// These numbers can be subtracted but not divided.
/// Instead, look to rationals, [`RAdic`](crate::RAdic), or if they can be approximate, [`ZAdic`](crate::ZAdic).
///
/// [`ZAdic`](crate::ZAdic) internally uses `IAdic` for the exact case.
pub struct IAdic {
    /// Adic prime
    pub (super) p: u32,
    /// p-1
    pub (super) pm1: u32,
    /// Adic digits, each 0 to p-1
    pub (super) d: Vec<u32>,
    /// Positive (trailing zeros) or Negative (trailing p-1)
    pub (super) sign: AdicSign,
}


impl IAdic {

    /// Create an adic number with the given digits and sign
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub fn new(p: u32, mut init_digits: Vec<u32>, sign: AdicSign) -> Self {

        adic_valid::validate_p(p);
        adic_valid::validate_digits_mod_p(p, &init_digits);

        let bad_digit = sign.mod_p(p);

        // Truncate zeros/(p-1)s so there should never be a trailing digit for an IAdic
        while init_digits.last().is_some_and(|d| *d == bad_digit) {
            init_digits.pop();
        }

        Self {
            p,
            pm1: p-1,
            d: init_digits,
            sign,
        }

    }

    /// Create a positive adic number with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub fn new_pos(p: u32, init_digits: Vec<u32>) -> Self {
        Self::new(p, init_digits, AdicSign::Pos)
    }

    /// Create a negative adic number with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub fn new_neg(p: u32, init_digits: Vec<u32>) -> Self {
        Self::new(p, init_digits, AdicSign::Neg)
    }

    /// Consume `IAdic` and get the real absolute value `UAdic`
    ///
    /// ```
    /// # use adic::{iadic_pos, iadic_neg, uadic};
    /// let i = iadic_pos!(5, [1, 2, 3, 4, 0]);
    /// assert_eq!(uadic!(5, [1, 2, 3, 4]), i.into_abs());
    /// let i = iadic_neg!(5, [1, 2, 3, 4, 0]);
    /// assert_eq!(uadic!(5, [4, 2, 1, 0, 4]), i.into_abs());
    /// ```
    pub fn into_abs(self) -> UAdic {
        match self.sign {
            AdicSign::Pos => UAdic::new(self.p(), self.d),
            AdicSign::Neg => UAdic::new(self.p(), (-self).d),
        }
    }

    /// Real absolute value `UAdic`
    ///
    /// ```
    /// # use adic::{iadic_pos, iadic_neg, uadic};
    /// let i = iadic_pos!(5, [1, 2, 3, 4, 0]);
    /// assert_eq!(uadic!(5, [1, 2, 3, 4]), i.abs());
    /// let i = iadic_neg!(5, [1, 2, 3, 4, 0]);
    /// assert_eq!(uadic!(5, [4, 2, 1, 0, 4]), i.abs());
    /// ```
    pub fn abs(&self) -> UAdic {
        match self.sign {
            AdicSign::Pos => UAdic::new(self.p(), self.clone().d),
            AdicSign::Neg => UAdic::new(self.p(), (-self.clone()).d),
        }
    }

    /// Real sign
    ///
    /// ```
    /// # use adic::{AdicSign, iadic_pos, iadic_neg};
    /// assert_eq!(AdicSign::Pos, iadic_pos!(5, [1, 2, 3, 4, 0]).sgn());
    /// assert_eq!(AdicSign::Neg, iadic_neg!(5, [1, 2, 3, 4, 0]).sgn());
    /// ```
    pub fn sgn(&self) -> AdicSign {
        self.sign
    }

    /// Number of non-trailing digits
    ///
    /// ```
    /// # use adic::{iadic_pos, iadic_neg};
    /// assert_eq!(2, iadic_pos!(5, [2, 4, 0]).num_non_trailing());
    /// assert_eq!(1, iadic_neg!(5, [2, 4, 4]).num_non_trailing());
    /// ```
    pub fn num_non_trailing(&self) -> usize {
        self.d.len()
    }

    /// Trailing digit for `IAdic`, either 0 or (p-1)
    ///
    /// ```
    /// # use adic::{iadic_pos, iadic_neg};
    /// assert_eq!(0, iadic_pos!(5, [4, 2]).trailing_digit());
    /// assert_eq!(4, iadic_neg!(5, [4, 2]).trailing_digit());
    /// ```
    pub fn trailing_digit(&self) -> u32 {
        self.sign.mod_p(self.p)
    }

    /// The natural number value of the number, e.g. 5-adic 123 is 25+10+3=38
    ///
    /// Warning: This can overflow; use [`signed_bigint_value`](IAdic::signed_bigint_value) if unsure
    ///
    /// # Panics
    /// Panics if u32 -> i32 conversion fails
    ///
    /// ```
    /// # use adic::{iadic_neg, iadic_pos};
    /// assert_eq!(38, iadic_pos!(5, [3, 2, 1]).i32_value());
    /// assert_eq!(-38, iadic_neg!(5, [2, 2, 3]).i32_value());
    /// ```
    pub fn i32_value(&self) -> i32 {
        let abs_int = i32::try_from(self.abs().u32_value()).expect("i32_value u32 -> i32 conversion");
        abs_int * i32::from(self.sgn())
    }

    /// The bigint representation for the natural number value of the number ([`i32_value`](`Self::i32_value`))
    ///
    /// ```
    /// # use num::BigInt;
    /// # use adic::{iadic_neg, iadic_pos};
    /// assert_eq!(BigInt::from(38), iadic_pos!(5, [3, 2, 1]).signed_bigint_value());
    /// assert_eq!(BigInt::from(-38), iadic_neg!(5, [2, 2, 3]).signed_bigint_value());
    /// ```
    pub fn signed_bigint_value(&self) -> BigInt {
        BigInt::from(self.abs().bigint_value()) * i32::from(self.sgn())
    }

}


impl AdicInteger for IAdic {
    fn zero(p: u32) -> Self {
        Self::new_pos(p, vec![])
    }
    fn one(p: u32) -> Self {
        Self::new_pos(p, vec![1])
    }
    fn p(&self) -> u32 {
        self.p
    }
    fn num_digits(&self) -> ZAdicValuation {
        match self.sign {
            AdicSign::Pos => ZAdicValuation::Finite(self.d.len()),
            AdicSign::Neg => ZAdicValuation::PosInf,
        }
    }
    fn digit(&self, n: usize) -> Result<u32, AdicError> {
        Ok(self.d.get(n).copied().unwrap_or(match self.sign {
            AdicSign::Pos => 0,
            AdicSign::Neg => self.pm1
        }))
    }
    fn digits(&self) -> impl Iterator<Item=&u32> {
        // Returns infinite iterator if num_digits PosInf and finite else
        fn inner_iter<'a>(pm1: &'a u32, sign: AdicSign, digits: &'a [u32]) -> Box<dyn Iterator<Item=&'a u32> + 'a> {
            match sign {
                AdicSign::Pos => Box::new(digits.iter()),
                AdicSign::Neg => Box::new(digits.iter().chain(repeat(pm1))),
            }
        }
        inner_iter(&self.pm1, self.sign, &self.d)
    }
    fn into_digits(self) -> impl Iterator<Item=u32> {
        // Returns infinite iterator if num_digits PosInf and finite else
        fn inner_into_iter(pm1: u32, sign: AdicSign, digits: Vec<u32>) -> Box<dyn Iterator<Item=u32>> {
            match sign {
                AdicSign::Pos => Box::new(digits.into_iter()),
                AdicSign::Neg => Box::new(digits.into_iter().chain(repeat(pm1))),
            }
        }
        inner_into_iter(self.pm1, self.sign, self.d)
    }
    fn is_zero(&self) -> bool {
        matches!(self.sign, AdicSign::Pos) && self.digits().all(Zero::is_zero)
    }
    fn digit_str(&self) -> String {
        match self.sign {
            AdicSign::Pos => {
                // Finite digits
                self.d.iter().join("").chars().rev().collect::<String>()
            },
            AdicSign::Neg => {
                // "Infinite" digits, show (p-1) and then the finite part
                let pm1_symbol = self.pm1.to_string();
                let digits = self.d.iter().join("").chars().rev().collect::<String>();
                format!("({pm1_symbol}){digits}")
            },
        }
    }
    fn into_split(self, n: usize) -> (UAdic, Self) {
        if self.num_non_trailing() > n {
            let (r, q) = UAdic::new(self.p, self.d).into_split(n);
            (r, Self::new(self.p, q.into_digits_vec(), self.sign))
        } else {
            let trail = self.trailing_digit();
            let rem = self.d.into_iter().chain(repeat(trail)).take(n).collect();
            (UAdic::new(self.p, rem), Self::new(self.p, vec![], self.sign))
        }
    }
    fn certainty(&self) -> ZAdicValuation {
        ZAdicValuation::PosInf
    }
}



#[cfg(test)]
mod tests {
    use num::{rational::Ratio, traits::Pow};
    use crate::{iadic_pos, iadic_neg, uadic, zadic_approx, ZAdic, ZAdicValuation, ZAdicVariety};
    use super::{AdicInteger, IAdic};

    use crate::num_adic::test_util::i::*;


    #[test]
    fn strips_repeats() {
        let strips_zeros = iadic_pos!(5, [2, 0, 0, 0, 0]);
        assert_eq!(iadic_pos!(5, [2]), strips_zeros);
        let strips_fours = iadic_neg!(5, [2, 4, 4, 4, 4]);
        assert_eq!(iadic_neg!(5, [2]), strips_fours);
        assert_eq!(one_twenty_five().certainty(), ZAdicValuation::PosInf);
    }

    #[test]
    fn abs() {
        assert_eq!(uadic!(5, [1, 2, 3]), iadic_pos!(5, [1, 2, 3]).into_abs());
        assert_eq!(uadic!(5, [4, 2, 1]), iadic_neg!(5, [1, 2, 3]).into_abs());
        assert_eq!(uadic!(5, []), zero().into_abs());
        assert_eq!(uadic!(5, [1]), one().into_abs());
        assert_eq!(uadic!(5, [1]), neg_one().into_abs());
        assert_eq!(uadic!(5, [2]), two().into_abs());
        assert_eq!(uadic!(5, [2]), neg_two().into_abs());
        assert_eq!(uadic!(5, [0, 1]), five().into_abs());
        assert_eq!(uadic!(5, [0, 1]), neg_five().into_abs());
        assert_eq!(uadic!(5, [0, 2]), neg_ten().into_abs());
    }

    #[test]
    fn i32_value() {
        assert_eq!(1, iadic_pos!(5, [1]).i32_value());
        assert_eq!(2, iadic_pos!(5, [2]).i32_value());
        assert_eq!(6, iadic_pos!(5, [1, 1]).i32_value());
        assert_eq!(126, iadic_pos!(5, [1, 0, 0, 1]).i32_value());
        assert_eq!(124, iadic_pos!(5, [4, 4, 4]).i32_value());
    }

    #[test]
    fn i_adic_norm() {
        assert_eq!(ZAdicValuation::PosInf, zero().valuation());
        assert_eq!(Ratio::ZERO, zero().norm());
        assert_eq!(ZAdicValuation::Finite(0), neg_one().valuation());
        assert_eq!(Ratio::new(1, 1), neg_one().norm());
        assert_eq!(ZAdicValuation::Finite(0), neg_two().valuation());
        assert_eq!(Ratio::new(1, 1), neg_two().norm());
        assert_eq!(ZAdicValuation::Finite(1), neg_five().valuation());
        assert_eq!(Ratio::new(1, 5), neg_five().norm());
        assert_eq!(ZAdicValuation::Finite(0), neg_six().valuation());
        assert_eq!(Ratio::new(1, 1), neg_six().norm());
        assert_eq!(ZAdicValuation::Finite(2), neg_twenty_five().valuation());
        assert_eq!(Ratio::new(1, 25), neg_twenty_five().norm());
        assert_eq!(ZAdicValuation::Finite(3), neg_one_twenty_five().valuation());
        assert_eq!(Ratio::new(1, 125), neg_one_twenty_five().norm());
        assert_eq!(ZAdicValuation::Finite(0), neg_one_twenty_six().valuation());
        assert_eq!(Ratio::new(1, 1), neg_one_twenty_six().norm());
    }

    #[test]
    fn nth_root() {

        let check = |p: u32, a: &IAdic, n: u32, precision: usize, roots: Vec<ZAdic>| {
            // Check each root powers to match a to at least precision digits
            for root in &roots {
                assert_eq!(a.truncation(precision), root.pow(n).into_truncation_to_uadic().unwrap());
            }
            // Check roots match the output of nth_root
            assert_eq!(Ok(ZAdicVariety::new(p, roots)), a.nth_root(n, precision));
        };

        check(2, &iadic_pos!(2, [1, 0, 0, 0, 1]), 2, 6, vec![
            zadic_approx!(2, 6, [1, 0, 0, 1, 0, 1]),
            zadic_approx!(2, 6, [1, 1, 1, 0, 1, 0]),
        ]);

        check(5, &iadic_pos!(5, [1]), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);

        check(5, &iadic_pos!(5, [2]), 2, 6, vec![]);

        check(7, &iadic_pos!(7, [2]), 2, 6, vec![
            zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2]),
            zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4]),
        ]);

    }

}
