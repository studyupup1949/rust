#![allow(dead_code)]

use itertools::Itertools;
use num::BigInt;
use crate::{
    divisible::{Divisible, Prime},
    error::{validate_digits_mod_p, AdicResult},
    traits::{AdicInteger, AdicPrimitive, HasDigitDisplay},
};
use super::{Sign, UAdic};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Adic that represents a signed integer
pub (crate) struct IAdic {
    /// Adic prime
    pub (super) p: Prime,
    /// Adic digits, each 0 to p-1
    pub (super) d: Vec<u32>,
    /// Positive (trailing zeros) or Negative (trailing p-1)
    pub (super) sign: Sign,
}


impl IAdic {

    /// Create an adic number with the given digits and sign
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub (super) fn new<P>(p: P, mut init_digits: Vec<u32>, sign: Sign) -> Self
    where P: Into<Prime> {

        let p = p.into();
        validate_digits_mod_p(p, &init_digits);

        let bad_digit = sign.mod_p(p);

        // Truncate zeros/(p-1)s so there should never be a trailing digit for an IAdic
        while init_digits.last().is_some_and(|d| *d == bad_digit) {
            init_digits.pop();
        }

        Self {
            p,
            d: init_digits,
            sign,
        }

    }

    /// Create a positive adic number with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub (crate) fn new_pos<P>(p: P, init_digits: Vec<u32>) -> Self
    where P: Into<Prime> {
        Self::new(p, init_digits, Sign::Pos)
    }

    /// Create a negative adic number with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub (crate) fn new_neg<P>(p: P, init_digits: Vec<u32>) -> Self
    where P: Into<Prime> {
        Self::new(p, init_digits, Sign::Neg)
    }

    /// Consume `IAdic` and get the real absolute value `UAdic`
    pub fn into_abs(self) -> UAdic {
        match self.sign {
            Sign::Pos => UAdic::new(self.p(), self.d),
            Sign::Neg => UAdic::new(self.p(), (-self).d),
        }
    }

    /// Real absolute value `UAdic`
    pub fn abs(&self) -> UAdic {
        match self.sign {
            Sign::Pos => UAdic::new(self.p(), self.clone().d),
            Sign::Neg => UAdic::new(self.p(), (-self.clone()).d),
        }
    }

    /// Does the `IAdic` repesent a non-negative integer
    pub fn is_non_negative(&self) -> bool {
        matches!(self.sign, Sign::Pos)
    }

    /// Number of non-trailing digits
    pub fn num_non_trailing(&self) -> usize {
        self.d.len()
    }

    /// Trailing digit for `IAdic`, either 0 or (p-1)
    pub fn trailing_digit(&self) -> u32 {
        self.sign.mod_p(self.p)
    }

    /// The natural number value of the number, e.g. 5-adic 123 is 25+10+3=38
    ///
    /// Warning: This can overflow; use [`signed_bigint_value`](IAdic::signed_bigint_value) if unsure
    pub fn i32_value(&self) -> AdicResult<i32> {
        let abs_int = i32::try_from(self.abs().u32_value()?)?;
        Ok(abs_int * i32::from(self.sign))
    }

    /// The bigint representation for the natural number value of the number ([`i32_value`](`Self::i32_value`))
    pub fn signed_bigint_value(&self) -> BigInt {
        BigInt::from(self.abs().bigint_value()) * i32::from(self.sign)
    }

}


impl AdicPrimitive for IAdic {

    fn zero<P>(p: P) -> Self
    where P: Into<Prime> {
        Self::new_pos(p, vec![])
    }
    fn one<P>(p: P) -> Self
    where P: Into<Prime> {
        Self::new_pos(p, vec![1])
    }
    fn p(&self) -> Prime {
        self.p
    }

}


impl AdicInteger for IAdic { }
impl HasDigitDisplay for IAdic {
    type DigitDisplay = String;
    fn digit_display(&self) -> String {
        let p = self.p();
        let ds = self.d.iter().map(|d| p.display_digit(*d)).collect::<Vec<_>>();
        let digits = ds.into_iter().rev().join("");
        match self.sign {
            Sign::Pos => {
                // Finite digits
                digits
            },
            Sign::Neg => {
                // "Infinite" digits, show (p-1) and then the finite part
                let pm1_symbol = p.display_digit(self.p.m1());
                format!("({pm1_symbol}){digits}")
            },
        }
    }
}



#[cfg(test)]
mod tests {
    use num::{BigInt, rational::Ratio, traits::Pow};
    use crate::{
        error::AdicError,
        local_num::LocalZero,
        normed::{Normed, UltraNormed, Valuation},
        traits::{AdicPrimitive, CanApproximate, CanTruncate, HasApproximateDigits},
        Variety, ZAdic,
    };
    use super::{AdicInteger, IAdic};

    use crate::num_adic::test_util::e::*;


    #[test]
    fn converted_doctests() {

        // Top level
        assert_eq!("4321._5", IAdic::new_pos(5, vec![1, 2, 3, 4]).to_string());
        let two = IAdic::new_pos(5, vec![2]);
        assert_eq!(Ok(2), two.i32_value());
        let neg_two = IAdic::new_neg(5, vec![3]);
        assert_eq!("(4)3._5", neg_two.to_string());
        assert_eq!(Ok(-2), neg_two.i32_value());
        assert!((two + neg_two).is_local_zero());

        // into_abs
        let i = IAdic::new_pos(5, vec![1, 2, 3, 4, 0]);
        assert_eq!(uadic!(5, [1, 2, 3, 4]), i.into_abs());
        let i = IAdic::new_neg(5, vec![1, 2, 3, 4, 0]);
        assert_eq!(uadic!(5, [4, 2, 1, 0, 4]), i.into_abs());

        // abs
        let i = IAdic::new_pos(5, vec![1, 2, 3, 4, 0]);
        assert_eq!(uadic!(5, [1, 2, 3, 4]), i.abs());
        let i = IAdic::new_neg(5, vec![1, 2, 3, 4, 0]);
        assert_eq!(uadic!(5, [4, 2, 1, 0, 4]), i.abs());

        // num_non_trailing
        assert_eq!(2, IAdic::new_pos(5, vec![2, 4, 0]).num_non_trailing());
        assert_eq!(1, IAdic::new_neg(5, vec![2, 4, 4]).num_non_trailing());

        // trailing_digit
        assert_eq!(0, IAdic::new_pos(5, vec![4, 2]).trailing_digit());
        assert_eq!(4, IAdic::new_neg(5, vec![4, 2]).trailing_digit());

        // i32_value
        assert_eq!(Ok(38), IAdic::new_pos(5, vec![3, 2, 1]).i32_value());
        assert_eq!(Ok(-38), IAdic::new_neg(5, vec![2, 2, 3]).i32_value());

        // signed_bigint_value
        assert_eq!(BigInt::from(38), IAdic::new_pos(5, vec![3, 2, 1]).signed_bigint_value());
        assert_eq!(BigInt::from(-38), IAdic::new_neg(5, vec![2, 2, 3]).signed_bigint_value());

    }


    #[test]
    fn strips_repeats() {
        let strips_zeros = eadic!(5, [2, 0, 0, 0, 0]);
        assert_eq!(eadic!(5, [2]), strips_zeros);
        let strips_fours = eadic_neg!(5, [2, 4, 4, 4, 4]);
        assert_eq!(eadic_neg!(5, [2]), strips_fours);
        assert_eq!(one_twenty_five().certainty(), Valuation::PosInf);
    }

    #[test]
    fn split() {
        assert_eq!((uadic!(5, [1]), eadic!(5, [2, 3, 4])), eadic!(5, [1, 2, 3, 4]).into_split(1));
        assert_eq!((uadic!(5, []), eadic!(5, [1, 2, 3, 4])), eadic!(5, [1, 2, 3, 4]).into_split(0));
        assert_eq!((uadic!(5, [1, 2, 3, 4, 0, 0]), eadic!(5, [])), eadic!(5, [1, 2, 3, 4]).into_split(6));
        assert_eq!((uadic!(5, [1]), eadic_neg!(5, [2, 3, 4])), eadic_neg!(5, [1, 2, 3, 4]).into_split(1));
        assert_eq!((uadic!(5, []), eadic_neg!(5, [1, 2, 3, 4])), eadic_neg!(5, [1, 2, 3, 4]).into_split(0));
        assert_eq!((uadic!(5, [1, 2, 3, 4, 4, 4]), eadic_neg!(5, [])), eadic_neg!(5, [1, 2, 3, 4]).into_split(6));
        assert_eq!((uadic!(5, []), eadic_neg!(5, [1, 4, 0, 4, 0, 4, 0])), eadic_neg!(5, [1, 4, 0, 4, 0, 4, 0]).into_split(0));
    }

    #[test]
    fn abs() {
        assert_eq!(uadic!(5, [1, 2, 3]), IAdic::new_pos(5, vec![1, 2, 3]).into_abs());
        assert_eq!(uadic!(5, [4, 2, 1]), IAdic::new_neg(5, vec![1, 2, 3]).into_abs());
        assert_eq!(uadic!(5, []), IAdic::zero(5).into_abs());
        assert_eq!(uadic!(5, [1]), IAdic::one(5).into_abs());
        assert_eq!(uadic!(5, [1]), (-IAdic::one(5)).into_abs());
        assert_eq!(uadic!(5, [2]), IAdic::new_pos(5, vec![2]).into_abs());
        assert_eq!(uadic!(5, [2]), IAdic::new_neg(5, vec![3]).into_abs());
        assert_eq!(uadic!(5, [0, 1]), IAdic::new_pos(5, vec![0, 1]).into_abs());
        assert_eq!(uadic!(5, [0, 1]), IAdic::new_neg(5, vec![0]).into_abs());
        assert_eq!(uadic!(5, [0, 2]), IAdic::new_neg(5, vec![0, 3]).into_abs());
    }

    #[test]
    fn i32_value() {
        assert_eq!(Ok(1), eadic!(5, [1]).i32_value());
        assert_eq!(Ok(2), eadic!(5, [2]).i32_value());
        assert_eq!(Ok(6), eadic!(5, [1, 1]).i32_value());
        assert_eq!(Ok(126), eadic!(5, [1, 0, 0, 1]).i32_value());
        assert_eq!(Ok(124), eadic!(5, [4, 4, 4]).i32_value());
        assert_eq!(Err(AdicError::BadConversion), eadic_rep!(5, [4], [3]).i32_value());
    }

    #[test]
    fn i_adic_norm() {
        assert_eq!(Valuation::PosInf, zero().valuation());
        assert_eq!(Ratio::ZERO, zero().norm());
        assert_eq!(Valuation::Finite(0), neg_one().valuation());
        assert_eq!(Ratio::new(1, 1), neg_one().norm());
        assert_eq!(Valuation::Finite(0), neg_two().valuation());
        assert_eq!(Ratio::new(1, 1), neg_two().norm());
        assert_eq!(Valuation::Finite(1), neg_five().valuation());
        assert_eq!(Ratio::new(1, 5), neg_five().norm());
        assert_eq!(Valuation::Finite(0), neg_six().valuation());
        assert_eq!(Ratio::new(1, 1), neg_six().norm());
        assert_eq!(Valuation::Finite(2), neg_twenty_five().valuation());
        assert_eq!(Ratio::new(1, 25), neg_twenty_five().norm());
        assert_eq!(Valuation::Finite(3), neg_one_twenty_five().valuation());
        assert_eq!(Ratio::new(1, 125), neg_one_twenty_five().norm());
        assert_eq!(Valuation::Finite(0), neg_one_twenty_six().valuation());
        assert_eq!(Ratio::new(1, 1), neg_one_twenty_six().norm());
    }

    #[test]
    fn nth_root() {

        let check = |a: &IAdic, n: u32, precision: usize, roots: Vec<ZAdic>| {
            // Check each root powers to match a to at least precision digits
            for root in &roots {
                assert_eq!(a.approximation(precision), root.clone().pow(n));
            }
            // Check roots match the output of nth_root
            assert_eq!(Ok(Variety::new(roots)), a.nth_root(n, precision));
        };

        check(&IAdic::new_pos(2, vec![1, 0, 0, 0, 1]), 2, 6, vec![
            zadic_approx!(2, 6, [1, 0, 0, 1, 0, 1]),
            zadic_approx!(2, 6, [1, 1, 1, 0, 1, 0]),
        ]);

        check(&IAdic::new_pos(5, vec![1]), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);

        check(&IAdic::new_pos(5, vec![2]), 2, 6, vec![]);

        check(&IAdic::new_pos(7, vec![2]), 2, 6, vec![
            zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2]),
            zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4]),
        ]);

    }

}
