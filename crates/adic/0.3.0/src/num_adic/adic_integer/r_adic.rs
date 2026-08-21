use std::{
    collections::VecDeque,
    iter::repeat_n,
};
use itertools::Itertools;
use num::{
    BigInt, BigRational, One, Rational32
};
use crate::{adic_valid, AdicError, ZAdicValuation};
use super::{AdicInteger, UAdic};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Adic that represents integers and rationals ([`radic`](crate::radic))
///
/// An [`AdicInteger`](crate::AdicInteger).
/// The actual adic is a set of "finite" digits and then repeats digits after.
/// So
/// ```
/// # use num::Rational32;
/// # use adic::{AdicInteger, RAdic};
/// assert_eq!("(23)41._5", RAdic::new(5, vec![1, 4], vec![3, 2]).to_string());
/// let neg_one = RAdic::new(5, vec![], vec![4]);
/// assert_eq!("(4)._5", neg_one.to_string());
/// assert_eq!(Rational32::new(-1, 1), neg_one.rational_value());
/// assert_eq!(RAdic::zero(5), RAdic::one(5) + neg_one);
/// ```
///
/// Just like with real numbers, there is an adic digital representation for fractions.
/// In both cases, fractions are characterized by an infinite REPEATING sequence of digits.
/// For adics, these repeat to the left, larger and larger powers of p.
/// In this way, we would say that 5-adically:
///
/// `-1/4 = 1/(1-5) = (geometric series) = 1 + 5 + 5^2 + 5^3 + 5^4 + ... = ...11111._5`
///
/// This seems weird, but 5-adically, the number "5" is small, "10", "15", and "20" are equally small,
///  and "25" is even smaller.
/// This is a CONVERGENT series in the 5-adics, converging to the rational number -1/4.
/// You can see that subtracting -1/4 from more powers of 5 gets smaller and smaller with the 5-adic norm:
///
/// `1 - (-1/4) = 5/4; 6 - (-1/4) = 25/4; 31 - (-1/4) = 125/4; ...`
///
/// ```
/// # use num::rational::Ratio;
/// # use adic::{AdicInteger, radic};
/// let neg_1_4 = radic!(5, [], [1]);
/// assert_eq!(Ratio::new(1, 1), (-neg_1_4.clone()).norm());
/// assert_eq!(Ratio::new(1, 5), (radic!(5, [1], []) - neg_1_4.clone()).norm());
/// assert_eq!(Ratio::new(1, 25), (radic!(5, [1, 1], []) - neg_1_4.clone()).norm());
/// assert_eq!(Ratio::new(1, 125), (radic!(5, [1, 1, 1], []) - neg_1_4.clone()).norm());
/// assert_eq!(Ratio::new(1, 625), (radic!(5, [1, 1, 1, 1], []) - neg_1_4.clone()).norm());
/// ```
///
/// The powers of p in the numerator get larger, and the SIZE (norm) of the combined number gets smaller.
///
/// `RAdic` represents a rational number as an adic digital expansion.
/// Any rational number can be represented this way EXCEPT those with powers of p in the denominator.
/// (Said numbers are not integers and not "small"; see [`QAdic<RAdic>`](crate::QAdic) to represent these.)
/// Even negative numbers can be represented, without a negative sign symbol!
///
/// `-1 = ...44444._5`
///
/// `...44444._5 + 1 = ...44445._5 = ...44450._5 = ...44500._5 = ...45000._5 = ...`
///
/// <div class="warning">
///
/// A big caveat to this struct: multiplication is intensive.
/// When calculating fractions as sets of repeating digits, the fraction repeat gets big FAST.
/// This means while it is a nice struct for declaring simple adic integers,
///  it is often inefficient to do TOO much arithmetic with them.
/// Use a [`ZAdic`](crate::ZAdic) if you can afford to approximate.
/// You can also truncate to a [`UAdic`](crate::UAdic), if you don't mind it growing after truncation.
///
/// </div>
pub struct RAdic {
    /// Adic prime
    pub (super) p: u32,
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
    pub fn new(p: u32, fix_d: Vec<u32>, rep_d: Vec<u32>) -> Self {

        adic_valid::validate_p(p);
        adic_valid::validate_digits_mod_p(p, &fix_d);
        adic_valid::validate_digits_mod_p(p, &rep_d);

        Self {
            p,
            fix_d,
            rep_d,
        }.normalize_integer_and_repeats()

    }


    /// Fixed digits for this adic, from one's place to p to p^2, etc.
    ///
    /// ```
    /// # use adic::{radic, uadic, AdicInteger};
    /// assert_eq!(vec![2, 1], radic!(5, [2, 1], [3, 4]).into_fixed_digits().collect::<Vec<_>>());
    /// ```
    pub fn into_fixed_digits(self) -> impl Iterator<Item=u32> {
        self.fix_d.into_iter()
    }

    /// Fixed digits for this adic, from one's place to p to p^2, etc.
    ///
    /// ```
    /// # use adic::{radic, uadic, AdicInteger};
    /// assert_eq!(vec![2, 1], radic!(5, [2, 1], [3, 4]).fixed_digits().cloned().collect::<Vec<_>>());
    /// ```
    pub fn fixed_digits(&self) -> impl Iterator<Item=&u32> {
        self.fix_d.iter()
    }

    /// Repeat digits for this adic, from one's place to p to p^2, etc.
    ///
    /// ```
    /// # use adic::{radic, uadic, AdicInteger};
    /// assert_eq!(vec![3, 4], radic!(5, [2, 1], [3, 4]).into_repeat_digits().collect::<Vec<_>>());
    /// ```
    pub fn into_repeat_digits(self) -> impl Iterator<Item=u32> {
        self.rep_d.into_iter()
    }

    /// Repeat digits for this adic, from one's place to p to p^2, etc.
    ///
    /// ```
    /// # use adic::{radic, AdicInteger};
    /// assert_eq!(vec![3, 4], radic!(5, [2, 1], [3, 4]).repeat_digits().cloned().collect::<Vec<_>>());
    /// ```
    pub fn repeat_digits(&self) -> impl Iterator<Item=&u32> {
        self.rep_d.iter()
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
    ///
    /// # Panics
    /// Panics if primitive number conversions fail
    ///
    /// ```
    /// # use num::Rational32;
    /// # use adic::radic;
    /// assert_eq!(Rational32::new(-1, 4), radic!(5, [], [1]).rational_value());
    /// ```
    pub fn rational_value(&self) -> Rational32 {
        let finite_val = UAdic::new(self.p, self.fix_d.clone()).u32_value();
        let finite_val = i32::try_from(finite_val).expect("rational_value u32 -> i32 conversion");
        let numerator: u32 = repeat_n(&0, self.fix_d.len()).chain(self.rep_d.iter())
            .zip(0..)
            .map(|(d, k)| *d * self.p().pow(k))
            .sum();
        let numerator = i32::try_from(numerator).expect("rational_value u32 -> i32 conversion");
        let denominator: u32 = if self.rep_d.is_empty() {
            1
        } else {
            let rep_us = u32::try_from(self.rep_d.len()).expect("rational_value usize -> u32 conversion");
            self.p().pow(rep_us) - 1
        };
        let denominator = i32::try_from(denominator).expect("rational_value u32 -> i32 conversion");
        Rational32::new(
            (finite_val * denominator) - numerator,
            denominator
        )
    }

    /// The big rational representation for the rational number value of the number ([`rational_value`](Self::rational_value))
    ///
    /// # Panics
    /// Panics if usize -> u32 conversion fails
    ///
    /// ```
    /// # use num::{BigInt, BigRational};
    /// # use adic::radic;
    /// assert_eq!(BigRational::new(BigInt::from(-1), BigInt::from(4)), radic!(5, [], [1]).big_rational_value());
    /// ```
    pub fn big_rational_value(&self) -> BigRational {
        let finite_val = BigInt::from(UAdic::new(self.p, self.fix_d.clone()).bigint_value());
        let numerator: BigInt = repeat_n(&0, self.fix_d.len()).chain(self.rep_d.iter())
            .zip(0u32..)
            .map(|(d, k)| BigInt::from(*d) * BigInt::from(self.p()).pow(k))
            .sum();
        let denominator = if self.rep_d.is_empty() {
            BigInt::one()
        } else {
            let rep_us = u32::try_from(self.rep_d.len()).expect("big_rational_value usize -> u32 conversion");
            BigInt::from(self.p()).pow(rep_us) - BigInt::one()
        };
        BigRational::new(finite_val * denominator.clone() - numerator, denominator)
    }

}


impl AdicInteger for RAdic {
    fn p(&self) -> u32 {
        self.p
    }
    fn zero(p: u32) -> Self {
        Self::new(p, vec![], vec![])
    }
    fn one(p: u32) -> Self {
        Self::new(p, vec![1], vec![])
    }
    fn num_digits(&self) -> ZAdicValuation {
        if self.rep_d.is_empty() {
            ZAdicValuation::Finite(self.fix_d.len())
        } else {
            ZAdicValuation::PosInf
        }
    }
    fn digit(&self, n: usize) -> Result<u32, AdicError> {
        if n < self.fix_d.len() {
            Ok(self.fix_d.get(n).copied().unwrap_or(0))
        } else if self.rep_d.is_empty() {
            Ok(0)
        } else {
            let diff = n - self.fix_d.len();
            let n_phase = diff % self.rep_d.len();
            Ok(self.rep_d.get(n_phase).copied().unwrap_or(0))
        }

    }
    fn digits(&self) -> impl Iterator<Item=&u32> {
        self.fix_d.iter().chain(self.rep_d.iter().cycle())
    }
    fn into_digits(self) -> impl Iterator<Item=u32> {
        self.fix_d.into_iter().chain(self.rep_d.into_iter().cycle())
    }
    fn digit_str(&self) -> String {
        if self.rep_d.is_empty() {
            return UAdic::new(self.p, self.fix_d.clone()).digit_str()
        }
        let rep_digits = self.rep_d.iter().join("").chars().rev().collect::<String>();
        let fix_digits = self.fix_d.iter().join("").chars().rev().collect::<String>();
        format!("({rep_digits}){fix_digits}")
    }
    fn into_split(self, n: usize) -> (UAdic, Self) {
        let p = self.p;
        if self.fix_d.len() >= n {
            let (r, q) = UAdic::new(p, self.fix_d).into_split(n);
            (r, Self::new(p, q.into_digits_vec(), self.rep_d))
        } else if self.has_finite_digits() {
            (UAdic::new(p, self.fix_d), Self::zero(p))
        } else {
            let (fix_len, rep_len) = (self.fix_d.len(), self.rep_d.len());
            let rep_clone = self.rep_d.clone();
            let before = UAdic::new(self.p, self.into_digits().take(n).collect());
            let rem_disp = (n - fix_len) % rep_len;
            let after = Self::new(
                p, vec![],
                rep_clone.into_iter().cycle().skip(rem_disp).take(rep_len).collect()
            );
            (before, after)
        }
    }
    fn certainty(&self) -> ZAdicValuation {
        ZAdicValuation::PosInf
    }
}



#[cfg(test)]
mod tests {
    use num::{rational::Ratio, traits::Pow, Rational32};
    use crate::{radic, uadic, zadic_approx, AdicError, SignedAdicInteger, ZAdic, ZAdicValuation, ZAdicVariety};
    use super::{AdicInteger, RAdic};

    use crate::num_adic::test_util::r::*;


    #[test]
    fn r_adic() {
        assert_eq!(uadic!(5, [1, 1, 1]), neg_1_4().into_truncation(3));
        assert_eq!(uadic!(5, [1, 1, 1, 1, 1, 1]), neg_1_4().into_truncation(6));
        assert_eq!(uadic!(5, [1, 1, 1, 1, 1, 1, 1, 1, 1]), neg_1_4().into_truncation(9));
        assert_eq!(radic!(5, [], [1]), radic!(5, [1], [1]));
        assert_eq!(radic!(5, [1], [2]), radic!(5, [1, 2], [2]));
        assert_eq!(radic!(5, [1], []), radic!(5, [1], [0, 0]));
        assert_eq!(radic!(5, [], [1, 0]), radic!(5, [1], [0, 1]));
        assert_eq!(radic!(5, [1, 0, 1], []), RAdic::from_i32(5, 26));
        assert_eq!(radic!(5, [4, 4, 3], [4]), RAdic::from_i32(5, -26));
        assert_eq!(twenty_five().certainty(), ZAdicValuation::PosInf);
        assert_eq!(pos_17_6().certainty(), ZAdicValuation::PosInf);
    }

    #[test]
    fn rational_value() {
        assert_eq!(
            Rational32::from_integer(1),
            radic!(5, [1], []).rational_value()
        );
        assert_eq!(
            Rational32::from_integer(2),
            radic!(5, [2], []).rational_value()
        );
        assert_eq!(
            Rational32::new(-1, 4),
            radic!(5, [], [1]).rational_value()
        );
        assert_eq!(
            Rational32::new(23, 24),
            radic!(5, [2], [0, 1]).rational_value()
        );
        assert_eq!(Rational32::new(-1, 3), neg_1_3_2().rational_value());
        assert_eq!(Rational32::new(1, 9), pos_1_9_2().rational_value());
        assert_eq!(Rational32::new(-8, 3), neg_8_3_2().rational_value());
        assert_eq!(Rational32::new(64, 9), pos_64_9_2().rational_value());
    }

    #[test]
    fn r_adic_norm() {
        assert_eq!(ZAdicValuation::PosInf, zero().valuation());
        assert_eq!(Ratio::ZERO, zero().norm());
        assert_eq!(ZAdicValuation::Finite(0), one().valuation());
        assert_eq!(Ratio::new(1, 1), one().norm());
        assert_eq!(ZAdicValuation::Finite(1), five().valuation());
        assert_eq!(Ratio::new(1, 5), five().norm());
        assert_eq!(ZAdicValuation::Finite(0), neg_1_4().valuation());
        assert_eq!(Ratio::new(1, 1), neg_1_4().norm());
        assert_eq!(ZAdicValuation::Finite(1), neg_5_4().valuation());
        assert_eq!(Ratio::new(1, 5), neg_5_4().norm());
        assert_eq!(ZAdicValuation::Finite(0), neg_1_24().valuation());
        assert_eq!(Ratio::new(1, 1), neg_1_24().norm());
        assert_eq!(ZAdicValuation::Finite(1), neg_5_24().valuation());
        assert_eq!(Ratio::new(1, 5), neg_5_24().norm());
    }

    #[test]
    fn nth_root() {

        let check = |p: u32, a: &RAdic, n: u32, precision: usize, roots: Vec<ZAdic>| {
            // Check each root powers to match a to at least precision digits
            for root in &roots {
                assert_eq!(a.truncation(precision), root.pow(n).into_truncation_to_uadic().unwrap());
            }
            // Check roots match the output of nth_root
            assert_eq!(Ok(ZAdicVariety::new(p, roots)), a.nth_root(n, precision));
        };

        check(5, &radic!(5, [1], []), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);
        check(5, &radic!(5, [1], [0, 0, 0, 0, 0, 1]), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);

        check(5, &radic!(5, [2], []), 2, 6, vec![]);
        check(5, &radic!(5, [2], [0, 0, 0, 0, 0, 1]), 2, 6, vec![]);

        check(7, &radic!(7, [2], []), 2, 6, vec![
            zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2]),
            zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4]),
        ]);
        check(7, &radic!(7, [2], [0, 0, 0, 0, 0, 1]), 2, 6, vec![
            zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2]),
            zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4]),
        ]);

        let zadic_pos_1_4 = ZAdic::new_approx(5, 6, pos_1_4().into_digits().take(6).collect());
        let zadic_neg_1_4 = ZAdic::new_approx(5, 6, neg_1_4().into_digits().take(6).collect());
        check(5, &pos_1_16(), 2, 6, vec![zadic_neg_1_4, zadic_pos_1_4]);

        assert!(matches!(
            zadic_approx!(7, 4, [2]).nth_root(2, 6),
            Err(AdicError::InappropriatePrecision(_))
        ));

    }

}
