//! Adic number structs of various types
//!
//! - UAdic - Finite set of digits to represent non-negative integers
//! - RAdic - Infinite repeating digits to represent all integers and most rationals
//! - ZAdic (TODO) - Infinite digits to represent all p-adic integers Z_p
//! - QAdic (TODO) - Semi-double infinite digits to represent all p-adic numbers Q_p


use std::{cmp::max, collections::VecDeque};
use num::{integer::lcm, BigInt, BigRational, One, Rational32, Zero};
use crate::adic_error::AdicError;

#[derive(Debug, Clone, PartialEq)]
/// Adic that represents an unsigned integer
pub struct UAdic {
    /// Adic prime
    p: u32,
    /// Adic digits, each 0 to p-1
    digits: Vec<u32>,
}


#[derive(Debug, Clone, PartialEq)]
/// Adic that represents integers and rationals
/// The actual adic is a finite_integer and then repeats digits after
pub struct RAdic {
    /// Unsigned integer for the finite integer part
    finite_integer: UAdic,
    /// Repeating digits, each 0 to p-1
    repeats: Vec<u32>,
}


#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents valuations of adic numbers
pub enum ZAdicValuation {
    /// Positive infinity, e.g. for zero
    PosInf,
    /// Finite integer
    Finite(i32),
}


// type UAdicResult = Result<UAdic, AdicError>;

impl UAdic {

    /// Prime for this adic
    pub fn p(&self) -> u32 {
        self.p
    }

    /// Create the zero adic number
    pub const fn zero(p: u32) -> Self {
        Self {
            p,
            digits: vec![],
        }
    }

    /// Test if it is the zero adic number
    pub fn is_zero(&self) -> bool {
        self.digits.iter().all(|d| d.is_zero())
    }

    /// Create an adic number with the given digits
    pub fn new(p: u32, init_digits: Vec<u32>) -> Self {
        Self {
            p,
            digits: init_digits,
        }
    }

    /// Repeats the repeat_digits for new_size digits to create a UAdic
    pub fn repeat(p: u32, repeat_digits: Vec<u32>, new_size: usize) -> UAdic {
        Self {
            p,
            digits: repeat_digits.into_iter().cycle().take(new_size).collect::<Vec<_>>(),
        }
    }

    /// Truncate an adic number's expansion to new_size
    pub fn truncate(self, new_size: usize) -> Self {
        let mut digits = self.digits;
        digits.truncate(new_size);
        Self {
            p: self.p,
            digits,
        }
    }

    /// The natural number value of the number, e.g. 5-adic 123 is 25+10+3=38
    pub fn integer_value(&self) -> u32 {
        self.digits
            .iter()
            .enumerate()
            .map(|(k, d)| *d * self.p.pow(k as u32))
            .sum()
    }

    /// The bigint representation for the natural number value of the number
    pub fn big_integer_value(&self) -> BigInt {
        self.digits
            .iter()
            .enumerate()
            .map(|(k, d)| BigInt::from(*d) * BigInt::from(self.p).pow(k as u32))
            .sum()
    }

    /// The adic valuation for this number: v(a/b p^k) = k
    pub fn valuation(&self) -> ZAdicValuation {
        if self.is_zero() {
            ZAdicValuation::PosInf
        } else {
            ZAdicValuation::Finite(self.digits.iter().take_while(|d| d.is_zero()).count() as i32)
        }
    }

    /// The adic norm for this number: |a/b p^k| = p^(-k)
    pub fn norm(&self) -> Rational32 {
        match self.valuation() {
            ZAdicValuation::PosInf => Rational32::zero(),
            ZAdicValuation::Finite(valuation) => Rational32::new(1, self.p as i32).pow(valuation),
        }
    }

}


// type RAdicResult = Result<RAdic, AdicError>;

impl RAdic {

    /// Prime for this adic
    pub fn p(&self) -> u32 {
        self.finite_integer.p
    }

    /// Create the zero adic number
    pub const fn zero(p: u32) -> Self {
        Self {
            finite_integer: UAdic::zero(p),
            repeats: vec![],
        }
    }

    /// Test if it is the zero adic number
    pub fn is_zero(&self) -> bool {
        self.finite_integer.is_zero() && self.repeats.iter().all(|d| d.is_zero())
    }

    /// Create an adic number with the given digits
    pub fn new(p: u32, finite_digits: Vec<u32>, repeats: Vec<u32>) -> Self {
        Self {
            finite_integer: UAdic::new(p, finite_digits),
            repeats,
        }
    }

    /// Truncate an adic number's expansion to new_size
    pub fn truncate(&self, new_size: usize) -> UAdic {

        let finite_len = self.finite_integer.digits.len();
        if finite_len >= new_size {
            self.finite_integer.clone().truncate(new_size)
        } else {
            let mut new_adic = self.finite_integer.clone();
            new_adic.digits.extend(self.repeats.iter().cycle().take(new_size - finite_len).cloned());
            new_adic
        }

    }

    /// Check for:
    /// 1. the end of finite_integer matches repeats
    /// 2. repeats has a shorter period
    /// 3. repeats are just zeros
    fn normalize_integer_and_repeats(self) -> RAdic {

        let mut finite_integer = self.finite_integer;
        let repeat_len = self.repeats.len();
        let mut repeat_deque = VecDeque::from(self.repeats);

        while let (Some(int_digit), Some(repeat_digit)) = (
            finite_integer.digits.last(), repeat_deque.back()
        ) {
            if int_digit == repeat_digit {
                finite_integer.digits.pop();
                let digit = repeat_deque.pop_back().unwrap();
                repeat_deque.push_front(digit);
            } else {
                break;
            }
        }

        let mut repeats = Vec::with_capacity(repeat_len);
        let mut repeats_checking_staged = repeats.iter().cycle();
        let mut staged = vec![];
        for repeat in repeat_deque {
            staged.push(repeat);
            if (
                repeats.len() == 0 ||
                repeats_checking_staged.next().is_none_or(|next_rep| repeat != *next_rep)
            ) {
                repeats.extend(staged.drain(..));
                repeats_checking_staged = repeats.iter().cycle();
            }
        }

        // We can discard staged iff its size is a multiple of repeats
        if staged.len() % repeats.len() != 0  {
            repeats.extend(staged.drain(..));
        }
        // If they're all zero, clear repeats
        if repeats.iter().all(|d| *d == 0) {
            repeats = vec![];
        }

        Self {
            finite_integer,
            repeats,
        }

    }

    /// The rational number value of the number, e.g. 5-adic ...111 is -1/4
    pub fn rational_value(&self) -> Rational32 {
        let finite_val = self.finite_integer.integer_value();
        let repeat_offset = self.p().pow(self.finite_integer.digits.len() as u32);
        let numerator: u32 = self.repeats
            .iter()
            .enumerate()
            .map(|(k, d)| *d * self.p().pow(k as u32))
            .sum();
        let denominator: i32 = if self.repeats.is_empty() {
            1
        } else {
            self.p().pow(self.repeats.len() as u32) as i32 - 1
        };
        Rational32::new(finite_val as i32 * denominator - repeat_offset as i32 * numerator as i32, denominator)
    }

    /// The big rational representation for the rational number value of the number
    pub fn big_rational_value(&self) -> BigRational {
        let finite_val = self.finite_integer.big_integer_value();
        let repeat_offset = BigInt::from(self.p()).pow(self.finite_integer.digits.len() as u32);
        let numerator: BigInt = self.repeats
            .iter()
            .enumerate()
            .map(|(k, d)| BigInt::from(*d) * BigInt::from(self.p()).pow(k as u32))
            .sum();
        let denominator = if self.repeats.is_empty() {
            BigInt::one()
        } else {
            BigInt::from(self.p()).pow(self.repeats.len() as u32) - BigInt::one()
        };
        BigRational::new(finite_val * denominator.clone() - repeat_offset * numerator, denominator)
    }

    /// The adic valuation for this number: v(a/b p^k) = k
    pub fn valuation(&self) -> ZAdicValuation {
        if self.is_zero() {
            ZAdicValuation::PosInf
        } else if !self.finite_integer.is_zero() {
            self.finite_integer.valuation()
        } else {
            ZAdicValuation::Finite(
                self.finite_integer.digits.len() as i32 + self.repeats.iter().take_while(|d| d.is_zero()).count() as i32
            )
        }
    }

    /// The adic norm for this number: |a/b p^k| = p^(-k)
    pub fn norm(&self) -> Rational32 {
        match self.valuation() {
            ZAdicValuation::PosInf => Rational32::zero(),
            ZAdicValuation::Finite(valuation) => Rational32::new(1, self.p() as i32).pow(valuation),
        }
    }

}


impl std::ops::Add for UAdic {
    type Output = UAdic;
    fn add(self, rhs: Self) -> Self::Output {

      if self.p != rhs.p {
          panic!("{:?}", AdicError::MixedCharacteristic);
      }
      let p = self.p;

      let (base_digits, adding_digits) = if self.digits.len() >= rhs.digits.len() {
          (self.digits, rhs.digits)
      } else {
          (rhs.digits, self.digits)
      };
      let digits_capacity = base_digits.len() + 1;

      let (
          mut added_digits, mut leftover_carry,
      ) = base_digits.into_iter().zip(
          adding_digits.into_iter().chain(std::iter::repeat(0))
      ).fold(
          (Vec::with_capacity(digits_capacity), 0),
          |(mut new_digits, carry), (base, added)| {
              let new_digit = base + added + carry;
              let reduced_digit = new_digit % self.p;
              // Since new_digit is positive, this matches modular math
              let new_carry = new_digit / self.p;
              new_digits.push(reduced_digit);
              (new_digits, new_carry)
          }
      );
      while leftover_carry > 0 {
          let new_digit = leftover_carry % self.p;
          added_digits.push(new_digit);
          leftover_carry = leftover_carry / self.p;
      }

      Self {
          p,
          digits: added_digits,
      }

    }
}

impl std::ops::Add for RAdic {
    type Output = RAdic;
    fn add(self, rhs: Self) -> Self::Output {

        if self.p() != rhs.p() {
            panic!("{:?}", AdicError::MixedCharacteristic);
        }
        let p = self.p();

        // Get new finite_int with long_finite_int + short_finite_int + short_repeats
        let (longer, shorter) = if self.finite_integer.digits.len() > rhs.finite_integer.digits.len() {
            (self, rhs)
        } else {
            (rhs, self)
        };
        let longer_len = longer.finite_integer.digits.len();
        let shorter_len = shorter.finite_integer.digits.len();
        let longer_digits = longer.finite_integer.digits
            .into_iter()
            .chain(longer.repeats.clone().into_iter().cycle())
            .take(longer_len)
            .collect();
        let shorter_digits = shorter.finite_integer.digits
            .into_iter()
            .chain(shorter.repeats.clone().into_iter().cycle())
            .take(longer_len)
            .collect();
        let mut finite_integer = UAdic {
            p, digits: longer_digits
        } + UAdic {
            p, digits: shorter_digits
        };

        // Adding may have overshot longer_len, so change the last digits into a carry
        let overshoot = finite_integer.digits.split_off(longer_len);
        let longer_replen = max(longer.repeats.len(), 1);
        let mut longer_repeat_iter = longer.repeats
            .into_iter()
            .cycle()
            .chain(std::iter::repeat(0));
        let shorter_replen = max(shorter.repeats.len(), 1);
        let mut shorter_repeat_iter = shorter.repeats
            .into_iter()
            .cycle()
            .skip(longer_len - shorter_len)
            .chain(std::iter::repeat(0));
        let mut carry = 0;
        for overshot in overshoot {
            let longer_rep = longer_repeat_iter.next().unwrap();
            let shorter_rep = shorter_repeat_iter.next().unwrap();
            let added = overshot + longer_rep + shorter_rep + carry;
            finite_integer.digits.push(added % p);
            carry = added / p;
        }

        // Calculate last digits of finite_integer and the new repeats, looking for it to stabilize
        let max_cycle_len = lcm(longer_replen, shorter_replen);
        let mut add_buffer: Vec<(u32, u32)> = vec![];
        loop {
            let longer_rep = longer_repeat_iter.next().unwrap();
            let shorter_rep = shorter_repeat_iter.next().unwrap();
            let added = longer_rep + shorter_rep + carry;
            carry = added / p;
            let new_add = (carry, added % p);
            if (
                add_buffer.len() >= max_cycle_len &&
                *add_buffer.get(add_buffer.len() - max_cycle_len).unwrap() == new_add
            ) {
                break;
            } else {
                add_buffer.push((carry, added % p));
            }
        }

        // Add last digits to finite_integer and make max_cycle_len new repeats
        let leftover_finite = add_buffer.len() - max_cycle_len;
        let mut added_iter = add_buffer.into_iter().map(|(_, d)| d);
        for _ in 0..leftover_finite {
            finite_integer.digits.push(added_iter.next().unwrap());
        }
        let new_repeats = added_iter.collect::<Vec<_>>();

        let new_adic = RAdic {
            finite_integer,
            repeats: new_repeats,
        };

        // Finally, reduce back to normal form
        new_adic.normalize_integer_and_repeats()

    }
}


impl std::ops::Neg for RAdic {
    type Output = RAdic;
    fn neg(self) -> Self::Output {

        let p = self.p();

        if self.finite_integer == UAdic::zero(self.p()) && self.repeats.iter().all(|d| *d == 0) {

            // If finite_integer is zero and repeats are zero, return zero
            self

        } else if self.finite_integer == UAdic::zero(self.p()) {

            // If finite_integer is zero, find the first nonzero repeat and turn into finite_integer
            let repeats_len = self.repeats.len();
            let zeros_len = self.repeats.iter().take_while(|d| **d == 0).count();
            let new_repeat_order = self.repeats
                .into_iter()
                // cycle through repeats
                .cycle()
                // skip until nonzero
                .skip_while(|d| *d == 0)
                // skip one more since that will be pushed to the finite_integer
                .skip(1)
                // take the same number of repeats as before
                .take(repeats_len)
                .collect::<Vec<_>>();
            let first_nonzero = new_repeat_order.last().unwrap();
            let mut new_digits = self.finite_integer.digits;
            new_digits.extend(std::iter::repeat(0).take(zeros_len));
            new_digits.push(p - first_nonzero);
            let new_finite = UAdic {
                p,
                digits: new_digits,
            };
            let new_repeats = new_repeat_order.into_iter().map(|d| p - d - 1).collect::<Vec<_>>();
            RAdic {
                finite_integer: new_finite,
                repeats: new_repeats,
            }

        } else {

            let mut new_digits = Vec::with_capacity(self.finite_integer.digits.len());
            let mut finite_iter = self.finite_integer.digits.into_iter();
            while let Some(d) = finite_iter.next() {
                if d == 0 {
                    new_digits.push(0);
                } else {
                    new_digits.push(p - d);
                    break;
                }
            }
            while let Some(d) = finite_iter.next() {
                new_digits.push(p - d - 1);
            }
            let new_finite = UAdic {
                p,
                digits: new_digits,
            };

            let new_repeats = if self.repeats.is_empty() {
                vec![4]
            } else {
                self.repeats.into_iter().map(|d| p - d - 1).collect::<Vec<_>>()
            };

            let new_adic = RAdic {
                finite_integer: new_finite,
                repeats: new_repeats,
            };

            new_adic.normalize_integer_and_repeats()

        }

    }
}

impl std::ops::Sub for RAdic {
    type Output = RAdic;
    fn sub(self, rhs: Self) -> Self::Output {
        // Could save a bit of performance by implementing directly
        self + (-rhs)
    }
}

// TODO: Mul

// TODO: Div (much harder?)

// TODO: Into<IIntAdic> etc.


#[cfg(test)]
mod tests {
    use num::Rational32;

    use super::{UAdic, RAdic, ZAdicValuation};

    #[test]
    fn test_add_u_adic() {
        let one_plus_one = UAdic::new(5, vec![1]) + UAdic::new(5, vec![1]);
        assert_eq!(UAdic::new(5, vec![2]), one_plus_one);
        let two_plus_one = UAdic::new(5, vec![2]) + UAdic::new(5, vec![1]);
        assert_eq!(UAdic::new(5, vec![3]), two_plus_one);
        let two_plus_three = UAdic::new(5, vec![2, 0]) + UAdic::new(5, vec![3, 0]);
        assert_eq!(UAdic::new(5, vec![0, 1]), two_plus_three);
        let neg_one_plus_neg_one = UAdic::new(5, vec![4, 4, 4, 4]) + UAdic::new(5, vec![4, 4, 4, 4]);
        assert_eq!(UAdic::new(5, vec![3, 4, 4, 4, 1]), neg_one_plus_neg_one);
        let neg_two_plus_neg_three = UAdic::new(5, vec![3, 4, 4, 4]) + UAdic::new(5, vec![2, 4, 4, 4]);
        assert_eq!(UAdic::new(5, vec![0, 4, 4, 4, 1]), neg_two_plus_neg_three);
        let two_plus_neg_two = UAdic::new(5, vec![2, 0]) + UAdic::new(5, vec![3, 4]);
        assert_eq!(UAdic::new(5, vec![0, 0, 1]), two_plus_neg_two);
        let four_plus_one_grows = UAdic::new(5, vec![4]) + UAdic::new(5, vec![1]);
        assert_eq!(UAdic::new(5, vec![0, 1]), four_plus_one_grows);
    }

    #[test]
    #[should_panic]
    fn test_mixed_characteristic() {
        let _ = UAdic::new(5, vec![1]) + UAdic::new(7, vec![1]);
    }

    #[test]
    fn test_integer_value() {
        assert_eq!(1, UAdic::new(5, vec![1]).integer_value());
        assert_eq!(2, UAdic::new(5, vec![2]).integer_value());
        assert_eq!(6, UAdic::new(5, vec![1, 1]).integer_value());
        assert_eq!(126, UAdic::new(5, vec![1, 0, 0, 1]).integer_value());
        assert_eq!(124, UAdic::new(5, vec![4, 4, 4]).integer_value());
    }

    #[test]
    fn test_u_adic_norm() {
        let zero = UAdic::zero(5);
        let one = UAdic::new(5, vec![1]);
        let two = UAdic::new(5, vec![2]);
        let five = UAdic::new(5, vec![0, 1]);
        let six = UAdic::new(5, vec![1, 1]);
        let twenty_five = UAdic::new(5, vec![0, 0, 1]);
        let one_twenty_five = UAdic::new(5, vec![0, 0, 0, 1]);
        let one_twenty_six = UAdic::new(5, vec![1, 0, 0, 1]);
        assert_eq!(ZAdicValuation::PosInf, zero.valuation());
        assert_eq!(Rational32::ZERO, zero.norm());
        assert_eq!(ZAdicValuation::Finite(0), one.valuation());
        assert_eq!(Rational32::new(1, 1), one.norm());
        assert_eq!(ZAdicValuation::Finite(0), two.valuation());
        assert_eq!(Rational32::new(1, 1), two.norm());
        assert_eq!(ZAdicValuation::Finite(1), five.valuation());
        assert_eq!(Rational32::new(1, 5), five.norm());
        assert_eq!(ZAdicValuation::Finite(0), six.valuation());
        assert_eq!(Rational32::new(1, 1), six.norm());
        assert_eq!(ZAdicValuation::Finite(2), twenty_five.valuation());
        assert_eq!(Rational32::new(1, 25), twenty_five.norm());
        assert_eq!(ZAdicValuation::Finite(3), one_twenty_five.valuation());
        assert_eq!(Rational32::new(1, 125), one_twenty_five.norm());
        assert_eq!(ZAdicValuation::Finite(0), one_twenty_six.valuation());
        assert_eq!(Rational32::new(1, 1), one_twenty_six.norm());
    }

    #[test]
    fn test_r_adic() {
        let neg_1_4 = RAdic::new(5, vec![], vec![1]);
        assert_eq!(UAdic::new(5, vec![1, 1, 1]), neg_1_4.clone().truncate(3));
        assert_eq!(UAdic::new(5, vec![1, 1, 1, 1, 1, 1]), neg_1_4.clone().truncate(6));
        assert_eq!(UAdic::new(5, vec![1, 1, 1, 1, 1, 1, 1, 1, 1]), neg_1_4.truncate(9));
    }

    #[test]
    fn test_add_r_integers() {
        let two_add_one = RAdic::new(5, vec![2], vec![]) + RAdic::new(5, vec![1], vec![]);
        assert_eq!(RAdic::new(5, vec![3], vec![]), two_add_one);
        let one_add_one = RAdic::new(5, vec![1], vec![]) + RAdic::new(5, vec![1], vec![]);
        assert_eq!(RAdic::new(5, vec![2], vec![]), one_add_one);
        let one_add_two = RAdic::new(5, vec![1], vec![]) + RAdic::new(5, vec![2], vec![]);
        assert_eq!(two_add_one, one_add_two);
        let one_add_six = RAdic::new(5, vec![1], vec![]) + RAdic::new(5, vec![1, 1], vec![]);
        assert_eq!(RAdic::new(5, vec![2, 1], vec![]), one_add_six);
    }

    #[test]
    fn test_neg_r_integers() {
        let neg_one = -RAdic::new(5, vec![1], vec![]);
        assert_eq!(RAdic::new(5, vec![], vec![4]), neg_one);
        let neg_zero = -RAdic::new(5, vec![], vec![]);
        assert_eq!(RAdic::new(5, vec![], vec![]), neg_zero);
        let neg_five = -RAdic::new(5, vec![0, 1], vec![]);
        assert_eq!(RAdic::new(5, vec![0], vec![4]), neg_five);
        let neg_p_to_third = -RAdic::new(5, vec![0, 0, 0, 1], vec![]);
        assert_eq!(RAdic::new(5, vec![0, 0, 0], vec![4]), neg_p_to_third);
    }

    #[test]
    fn test_sub_r_integers() {
        let two_sub_one = RAdic::new(5, vec![2], vec![]) - RAdic::new(5, vec![1], vec![]);
        assert_eq!(RAdic::new(5, vec![1], vec![]), two_sub_one);
        let one_sub_one = RAdic::new(5, vec![1], vec![]) - RAdic::new(5, vec![1], vec![]);
        assert_eq!(RAdic::new(5, vec![], vec![]), one_sub_one);
        let one_sub_two = RAdic::new(5, vec![1], vec![]) - RAdic::new(5, vec![2], vec![]);
        assert_eq!(RAdic::new(5, vec![], vec![4]), one_sub_two);
        let one_sub_six = RAdic::new(5, vec![1], vec![]) - RAdic::new(5, vec![1, 1], vec![]);
        assert_eq!(RAdic::new(5, vec![0], vec![4]), one_sub_six);
    }

    #[test]
    fn test_rational_value() {
        assert_eq!(
            Rational32::from_integer(1),
            RAdic::new(5, vec![1], vec![]).rational_value()
        );
        assert_eq!(
            Rational32::from_integer(2),
            RAdic::new(5, vec![2], vec![]).rational_value()
        );
        assert_eq!(
            Rational32::new(-1, 4),
            RAdic::new(5, vec![], vec![1]).rational_value()
        );
        assert_eq!(
            Rational32::new(23, 24),
            RAdic::new(5, vec![2], vec![0, 1]).rational_value()
        );
    }

    #[test]
    fn test_r_adic_norm() {
        let zero = RAdic::zero(5);
        let one = RAdic::new(5, vec![1], vec![]);
        let five = RAdic::new(5, vec![0, 1], vec![]);
        let neg_one_fourth = RAdic::new(5, vec![], vec![1]);
        let neg_five_fourth = RAdic::new(5, vec![0], vec![1]);
        let neg_one_twenty_fourth = RAdic::new(5, vec![], vec![1, 0]);
        let neg_five_twenty_fourth = RAdic::new(5, vec![], vec![0, 1]);
        assert_eq!(ZAdicValuation::PosInf, zero.valuation());
        assert_eq!(Rational32::ZERO, zero.norm());
        assert_eq!(ZAdicValuation::Finite(0), one.valuation());
        assert_eq!(Rational32::new(1, 1), one.norm());
        assert_eq!(ZAdicValuation::Finite(1), five.valuation());
        assert_eq!(Rational32::new(1, 5), five.norm());
        assert_eq!(ZAdicValuation::Finite(0), neg_one_fourth.valuation());
        assert_eq!(Rational32::new(1, 1), neg_one_fourth.norm());
        assert_eq!(ZAdicValuation::Finite(1), neg_five_fourth.valuation());
        assert_eq!(Rational32::new(1, 5), neg_five_fourth.norm());
        assert_eq!(ZAdicValuation::Finite(0), neg_one_twenty_fourth.valuation());
        assert_eq!(Rational32::new(1, 1), neg_one_twenty_fourth.norm());
        assert_eq!(ZAdicValuation::Finite(1), neg_five_twenty_fourth.valuation());
        assert_eq!(Rational32::new(1, 5), neg_five_twenty_fourth.norm());
    }


    #[test]
    fn test_add_sub_r() {

        let neg_1_4 = RAdic::new(5, vec![], vec![1]);
        let pos_1_4 = RAdic::new(5, vec![4], vec![3]);
        let eleven = RAdic::new(5, vec![1, 2], vec![]);
        let pos_43_4 = RAdic::new(5, vec![2, 3], vec![1]);
        let neg_1_24 = RAdic::new(5, vec![], vec![1, 0]);
        let neg_5_24 = RAdic::new(5, vec![], vec![0, 1]);

        assert_eq!(neg_1_4, -pos_1_4.clone());
        assert_eq!(pos_1_4, -neg_1_4.clone());
        assert_eq!(pos_43_4, neg_1_4.clone() + eleven.clone());
        assert_eq!(-pos_43_4, pos_1_4 - eleven.clone());
        assert_eq!(neg_1_24.clone() + neg_1_24.clone() + neg_1_24.clone() + neg_1_24.clone() + neg_1_24.clone(), neg_5_24);
        assert_eq!(neg_1_24.clone() + neg_1_24.clone() + neg_1_24.clone() + neg_1_24.clone() + neg_1_24.clone() + neg_1_24.clone(), neg_1_4);

        let one = RAdic::new(5, vec![1], vec![]);
        let neg_1_31 = RAdic::new(5, vec![], vec![4, 0, 0]);
        let pos_30_31 = one + neg_1_31.clone();
        let neg_5_31 = neg_1_31.clone() + neg_1_31.clone() + neg_1_31.clone() + neg_1_31.clone() + neg_1_31.clone();
        let neg_30_31 = neg_5_31.clone() + neg_5_31.clone() + neg_5_31.clone() + neg_5_31.clone() + neg_5_31.clone() + neg_5_31.clone();

        assert_eq!(RAdic::new(5, vec![0, 1], vec![0, 4, 0]), pos_30_31);
        assert_eq!(RAdic::new(5, vec![], vec![0, 4, 0]), neg_5_31);
        assert_eq!(RAdic::zero(5), pos_30_31 + neg_30_31);

        let three = RAdic::new(5, vec![3], vec![]);
        let neg_one_sixth = RAdic::new(5, vec![], vec![4, 0]);
        let seventeen_sixth = three + neg_one_sixth;
        assert_eq!(RAdic::new(5, vec![2, 1], vec![4, 0]), seventeen_sixth);
        assert_eq!(UAdic::new(5, vec![2, 1, 4, 0, 4, 0]), seventeen_sixth.truncate(6));

    }

}
