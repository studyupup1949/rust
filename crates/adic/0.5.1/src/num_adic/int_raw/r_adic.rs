#![allow(dead_code)]

use std::{
    collections::VecDeque,
    iter::repeat_n,
};
use itertools::Itertools;
use num::{
    traits::Pow,
    BigInt, BigRational, BigUint, One, Rational32,
};
use crate::{
    error::{validate_digits_mod_p, AdicError, AdicResult},
    divisible::{Divisible, Prime},
    traits::{AdicInteger, AdicPrimitive, HasDigitDisplay},
};
use super::UAdic;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Adic that represents integers and rationals
pub (crate) struct RAdic {
    /// Adic prime
    pub (super) p: Prime,
    /// Adic digits, each 0 to p-1
    pub (super) fix_d: Vec<u32>,
    /// Repeating digits, each 0 to p-1
    pub (super) rep_d: Vec<u32>,
}


impl RAdic {

    /// Create an adic number with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub fn new<P>(p: P, fix_d: Vec<u32>, rep_d: Vec<u32>) -> Self
    where P: Into<Prime> {

        let p = p.into();
        validate_digits_mod_p(p, &fix_d);
        validate_digits_mod_p(p, &rep_d);

        Self {
            p,
            fix_d,
            rep_d,
        }.normalize_integer_and_repeats()

    }


    /// Fixed digits for this adic, from one's place to p to p^2, etc.
    pub fn into_fixed_digits(self) -> impl Iterator<Item=u32> {
        self.fix_d.into_iter()
    }

    /// Fixed digits for this adic, from one's place to p to p^2, etc.
    pub fn fixed_digits(&self) -> impl Iterator<Item=u32> {
        self.fix_d.clone().into_iter()
    }

    /// Repeat digits for this adic, from one's place to p to p^2, etc.
    pub fn into_repeat_digits(self) -> impl Iterator<Item=u32> {
        self.rep_d.into_iter()
    }

    /// Repeat digits for this adic, from one's place to p to p^2, etc.
    pub fn repeat_digits(&self) -> impl Iterator<Item=u32> {
        self.rep_d.clone().into_iter()
    }

    /// Constructor helper
    /// Check for:
    ///
    /// 1. the end of finite digits matches repeats
    /// 2. repeats has a shorter period
    /// 3. repeats are just zeros
    fn normalize_integer_and_repeats(self) -> RAdic {

        let p = self.p();
        let mut finite_integer_digits = self.fix_d;
        let repeat_len = self.rep_d.len();

        // If repeats are all zero, just trim fix_d and return
        if self.rep_d.iter().all(|d| *d == 0) {

            // Truncate zeros
            while let Some(0) = finite_integer_digits.last() {
                finite_integer_digits.pop();
            }
            Self {
                p,
                fix_d: finite_integer_digits,
                rep_d: vec![],
            }

        } else {

            // If the end of finite integer matches repeats, move it
            let mut repeat_deque = VecDeque::from(self.rep_d);
            while let (Some(int_digit), Some(repeat_digit)) = (
                finite_integer_digits.last(), repeat_deque.back()
            ) {
                if int_digit == repeat_digit {
                    finite_integer_digits.pop();
                    let digit = repeat_deque.pop_back().unwrap();
                    repeat_deque.push_front(digit);
                } else {
                    break;
                }
            }

            // If repeats has a smaller period, reduce to that cycle
            let mut repeats = Vec::with_capacity(repeat_len);
            let mut repeats_checking_staged = repeats.iter().cycle();
            let mut staged = vec![];
            for repeat in repeat_deque {
                staged.push(repeat);
                // If staged is not the same as what's in repeat, move it in
                if repeats_checking_staged.next().is_none_or(|next_rep| repeat != *next_rep) {
                    repeats.append(&mut staged);
                    repeats_checking_staged = repeats.iter().cycle();
                }
            }

            // We can discard staged iff its size is a multiple of repeats
            if staged.len() % repeats.len() != 0  {
                repeats.append(&mut staged);
            }

            Self {
                p,
                fix_d: finite_integer_digits,
                rep_d: repeats,
            }

        }

    }

    /// The rational number value of the number, e.g. 5-adic ...111 is -1/4
    ///
    /// Warning: This can easily overflow; use [`big_rational_value`](Self::big_rational_value) if unsure
    pub fn rational_value(&self) -> AdicResult<Rational32> {

        let finite_val = UAdic::new(self.p, self.fix_d.clone()).u32_value()?;
        let finite_val = i32::try_from(finite_val)?;

        // sum (d * p^k)
        let mut numerator = 0u32;
        for (d, k) in repeat_n(&0, self.fix_d.len()).chain(self.rep_d.iter()).zip(0..) {
            let trial = u32::from(self.p).checked_pow(k).and_then(|pn| d.checked_mul(pn)).and_then(|n| numerator.checked_add(n));
            if let Some(t) = trial {
                numerator = t;
            } else {
                Err(AdicError::TryFromIntError)?;
            }
        }
        let numerator = i32::try_from(numerator)?;

        // p^n-1
        let denominator: u32 = if self.rep_d.is_empty() {
            1
        } else {
            let rep_us = u32::try_from(self.rep_d.len())?;
            u32::from(self.p()).checked_pow(rep_us).map(|pn| pn - 1).ok_or(AdicError::TryFromIntError)?
        };
        let denominator = i32::try_from(denominator)?;

        let real_numer = finite_val.checked_mul(denominator).and_then(|n| n.checked_sub(numerator)).ok_or(AdicError::TryFromIntError)?;
        Ok(Rational32::new(real_numer, denominator))

    }

    /// The big rational representation for the rational number value of the number ([`rational_value`](Self::rational_value))
    ///
    /// # Panics
    /// Panics if usize -> u32 conversion fails
    pub fn big_rational_value(&self) -> BigRational {
        let finite_val = BigInt::from(UAdic::new(self.p, self.fix_d.clone()).bigint_value());
        let numerator: BigInt = repeat_n(&0, self.fix_d.len()).chain(self.rep_d.iter())
            .zip(0u32..)
            .map(|(d, k)| BigInt::from(*d) * BigInt::from(BigUint::from(self.p()).pow(k)))
            .sum();
        let denominator = if self.rep_d.is_empty() {
            BigInt::one()
        } else {
            let rep_us = u32::try_from(self.rep_d.len()).expect("big_rational_value usize -> u32 conversion");
            BigInt::from(BigUint::from(self.p()).pow(rep_us)) - BigInt::one()
        };
        BigRational::new(finite_val * denominator.clone() - numerator, denominator)
    }

}


impl AdicPrimitive for RAdic {

    fn p(&self) -> Prime {
        self.p
    }
    fn zero<P>(p: P) -> Self
    where P: Into<Prime> {
        Self::new(p, vec![], vec![])
    }
    fn one<P>(p: P) -> Self
    where P: Into<Prime> {
        Self::new(p, vec![1], vec![])
    }

}


impl AdicInteger for RAdic { }
impl HasDigitDisplay for RAdic {
    type DigitDisplay = String;
    fn digit_display(&self) -> String {
        if self.rep_d.is_empty() {
            return UAdic::new(self.p, self.fix_d.clone()).digit_display()
        }
        let p = self.p();
        let rep_digits = self.rep_d.iter().map(|d| p.display_digit(*d)).collect::<Vec<_>>();
        let rep_digits = rep_digits.into_iter().rev().join("");
        let fix_digits = self.fix_d.iter().map(|d| p.display_digit(*d)).collect::<Vec<_>>();
        let fix_digits = fix_digits.into_iter().rev().join("");
        format!("({rep_digits}){fix_digits}")
    }

}



#[cfg(test)]
mod tests {
    use num::{rational::Ratio, traits::Pow, BigInt, BigRational, Rational32};
    use crate::{
        error::AdicError,
        normed::{Normed, UltraNormed, Valuation},
        traits::{AdicPrimitive, CanApproximate, CanTruncate, HasApproximateDigits, HasDigits, PrimedFrom, TryPrimedFrom},
        Variety, ZAdic,
    };
    use super::{AdicInteger, RAdic};

    use crate::num_adic::test_util::e::*;


    #[test]
    fn converted_doctests() {

        // Top level 1
        assert_eq!("(23)41._5", RAdic::new(5, vec![1, 4], vec![3, 2]).to_string());
        let neg_one = RAdic::new(5, vec![], vec![4]);
        assert_eq!("(4)._5", neg_one.to_string());
        assert_eq!(Ok(Rational32::new(-1, 1)), neg_one.rational_value());
        assert_eq!(RAdic::zero(5), RAdic::one(5) + neg_one);

        // Top level 2
        let neg_1_4 = RAdic::new(5, vec![], vec![1]);
        assert_eq!(Ratio::new(1, 1), (-neg_1_4.clone()).norm());
        assert_eq!(Ratio::new(1, 5), (RAdic::new(5, vec![1], vec![]) - neg_1_4.clone()).norm());
        assert_eq!(Ratio::new(1, 25), (RAdic::new(5, vec![1, 1], vec![]) - neg_1_4.clone()).norm());
        assert_eq!(Ratio::new(1, 125), (RAdic::new(5, vec![1, 1, 1], vec![]) - neg_1_4.clone()).norm());
        assert_eq!(Ratio::new(1, 625), (RAdic::new(5, vec![1, 1, 1, 1], vec![]) - neg_1_4.clone()).norm());

        // into_fixed_digits
        assert_eq!(vec![2, 1], RAdic::new(5, vec![2, 1], vec![3, 4]).into_fixed_digits().collect::<Vec<_>>());

        // fixed_digits
        assert_eq!(vec![2, 1], RAdic::new(5, vec![2, 1], vec![3, 4]).fixed_digits().collect::<Vec<_>>());

        // into_repeat_digits
        assert_eq!(vec![3, 4], RAdic::new(5, vec![2, 1], vec![3, 4]).into_repeat_digits().collect::<Vec<_>>());

        // repeat_digits
        assert_eq!(vec![3, 4], RAdic::new(5, vec![2, 1], vec![3, 4]).repeat_digits().collect::<Vec<_>>());

        // rational_value
        assert_eq!(Ok(Rational32::new(-1, 4)), RAdic::new(5, vec![], vec![1]).rational_value());

        // big_rational_value
        assert_eq!(
            BigRational::new(BigInt::from(-1), BigInt::from(4)),
            RAdic::new(5, vec![], vec![1]).big_rational_value()
        );

    }


    #[test]
    fn r_adic() {
        assert_eq!(uadic!(5, [1, 1, 1]), neg_1_4().into_truncation(3));
        assert_eq!(uadic!(5, [1, 1, 1, 1, 1, 1]), neg_1_4().into_truncation(6));
        assert_eq!(uadic!(5, [1, 1, 1, 1, 1, 1, 1, 1, 1]), neg_1_4().into_truncation(9));
        assert_eq!(eadic_rep!(5, [], [1]), eadic_rep!(5, [1], [1]));
        assert_eq!(eadic_rep!(5, [1], [2]), eadic_rep!(5, [1, 2], [2]));
        assert_eq!(eadic_rep!(5, [1], []), eadic_rep!(5, [1], [0, 0]));
        assert_eq!(eadic_rep!(5, [], [1, 0]), eadic_rep!(5, [1], [0, 1]));
        assert_eq!(eadic_rep!(5, [1, 0, 1], []), RAdic::primed_from(5, 26).into());
        assert_eq!(eadic_rep!(5, [4, 4, 3], [4]), RAdic::primed_from(5, -26).into());
        assert_eq!(twenty_five().certainty(), Valuation::PosInf);
        assert_eq!(pos_17_6().certainty(), Valuation::PosInf);
    }

    #[test]
    fn rational_value() {
        assert_eq!(
            Ok(Rational32::from_integer(1)),
            eadic_rep!(5, [1], []).rational_value()
        );
        assert_eq!(
            Ok(Rational32::from_integer(2)),
            eadic_rep!(5, [2], []).rational_value()
        );
        assert_eq!(
            Ok(Rational32::new(-1, 4)),
            eadic_rep!(5, [], [1]).rational_value()
        );
        assert_eq!(
            Ok(Rational32::new(23, 24)),
            eadic_rep!(5, [2], [0, 1]).rational_value()
        );
        assert_eq!(Ok(Rational32::new(-1, 3)), neg_1_3_2().rational_value());
        assert_eq!(Ok(Rational32::new(1, 9)), pos_1_9_2().rational_value());
        assert_eq!(Ok(Rational32::new(-8, 3)), neg_8_3_2().rational_value());
        assert_eq!(Ok(Rational32::new(64, 9)), pos_64_9_2().rational_value());
    }

    #[test]
    fn from_rational() {
        assert_eq!(
            eadic_rep!(5, [], [1]),
            RAdic::try_primed_from(5, Rational32::new(-1, 4)).unwrap().into()
        );
        assert_eq!(
            eadic_rep!(5, [0, 4], [3]),
            RAdic::try_primed_from(5, Rational32::new(5, 4)).unwrap().into()
        );
        assert_eq!(
            eadic_rep!(5, [], [2, 1, 4, 2, 3, 0]),
            RAdic::try_primed_from(5, Rational32::new(-1, 7)).unwrap().into()
        );
    }

    #[test]
    fn r_adic_unit_valuation() {
        let neg_5 = eadic_rep!(5, [1, 1, 0], [1]);
        assert_eq!(Some(neg_5.clone()), neg_5.unit());
    }

    #[test]
    fn r_adic_norm() {
        assert_eq!(Valuation::PosInf, zero().valuation());
        assert_eq!(Ratio::ZERO, zero().norm());
        assert_eq!(Valuation::Finite(0), one().valuation());
        assert_eq!(Ratio::new(1, 1), one().norm());
        assert_eq!(Valuation::Finite(1), five().valuation());
        assert_eq!(Ratio::new(1, 5), five().norm());
        assert_eq!(Valuation::Finite(0), neg_1_4().valuation());
        assert_eq!(Ratio::new(1, 1), neg_1_4().norm());
        assert_eq!(Valuation::Finite(1), neg_5_4().valuation());
        assert_eq!(Ratio::new(1, 5), neg_5_4().norm());
        assert_eq!(Valuation::Finite(0), neg_1_24().valuation());
        assert_eq!(Ratio::new(1, 1), neg_1_24().norm());
        assert_eq!(Valuation::Finite(1), neg_5_24().valuation());
        assert_eq!(Ratio::new(1, 5), neg_5_24().norm());
    }

    #[test]
    fn nth_root() {

        let check = |a: &RAdic, n: u32, precision: usize, roots: Vec<ZAdic>| {
            // Check each root powers to match a to at least precision digits
            for root in &roots {
                assert_eq!(a.approximation(precision), root.clone().pow(n));
            }
            // Check roots match the output of nth_root
            assert_eq!(Ok(Variety::new(roots)), a.nth_root(n, precision));
        };

        check(&RAdic::new(5, vec![1], vec![]), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);
        check(&RAdic::new(5, vec![1], vec![0, 0, 0, 0, 0, 1]), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);

        check(&RAdic::new(5, vec![2], vec![]), 2, 6, vec![]);
        check(&RAdic::new(5, vec![2], vec![0, 0, 0, 0, 0, 1]), 2, 6, vec![]);

        check(&RAdic::new(7, vec![2], vec![]), 2, 6, vec![
            zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2]),
            zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4]),
        ]);
        check(&RAdic::new(7, vec![2], vec![0, 0, 0, 0, 0, 1]), 2, 6, vec![
            zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2]),
            zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4]),
        ]);

        let zadic_pos_1_4 = ZAdic::new_approx(5, 6, pos_1_4().digits().take(6).collect());
        let zadic_neg_1_4 = ZAdic::new_approx(5, 6, neg_1_4().digits().take(6).collect());
        check(&RAdic::new(5, vec![1], vec![2, 3, 4, 0]), 2, 6, vec![zadic_neg_1_4, zadic_pos_1_4]);

        assert!(matches!(
            zadic_approx!(7, 4, [2]).nth_root(2, 6),
            Err(AdicError::InappropriatePrecision(_))
        ));

    }

}
