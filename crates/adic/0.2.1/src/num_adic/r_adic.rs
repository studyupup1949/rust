use std::{
    cmp::max,
    collections::VecDeque,
    fmt,
    iter::{once, repeat}
};
use itertools::Itertools;
use num::{integer::lcm, traits::Pow, BigInt, BigRational, One, Rational32};
use num_prime::nt_funcs::is_prime;
use crate::{util::totient_many, AdicError};
use super::{AdicInteger, IAdic, UAdic, ZAdicValuation};


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
/// # use num::Rational32;
/// # use adic::{AdicInteger, radic};
/// let neg_1_4 = radic!(5, [], [1]);
/// assert_eq!(Rational32::new(1, 1), (-neg_1_4.clone()).norm());
/// assert_eq!(Rational32::new(1, 5), (radic!(5, [1], []) - neg_1_4.clone()).norm());
/// assert_eq!(Rational32::new(1, 25), (radic!(5, [1, 1], []) - neg_1_4.clone()).norm());
/// assert_eq!(Rational32::new(1, 125), (radic!(5, [1, 1, 1], []) - neg_1_4.clone()).norm());
/// assert_eq!(Rational32::new(1, 625), (radic!(5, [1, 1, 1, 1], []) - neg_1_4.clone()).norm());
/// ```
///
/// The powers of p in the numerator get larger, and the SIZE (norm) of the combined number gets smaller.
///
/// `RAdic` represents a rational number as an adic digital expansion.
/// Any rational number can be represented this way EXCEPT those with powers of p in the denominator.
/// (Said numbers are not integers and not "small".)
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
    p: u32,
    /// Adic digits, each 0 to p-1
    fix_d: Vec<u32>,
    /// Repeating digits, each 0 to p-1
    rep_d: Vec<u32>,
}


impl RAdic {

    /// Create an adic number with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime
    pub fn new(p: u32, fix_d: Vec<u32>, rep_d: Vec<u32>) -> Self {
        assert!(is_prime(&p, None).probably());
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

    /// Create adic number associated with (signed) integer n
    pub fn from_integer(p: u32, n: i32) -> Self {
        let negative = n < 0;
        let mut n = n.unsigned_abs();
        let mut digits = vec![];
        while n != 0 {
            digits.push(n % p);
            n = n / p;
        }
        let r = Self::new(p, digits, vec![]);
        if negative {
            -r
        } else {
            r
        }
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
    /// ```
    /// # use num::Rational32;
    /// # use adic::radic;
    /// assert_eq!(Rational32::new(-1, 4), radic!(5, [], [1]).rational_value());
    /// ```
    pub fn rational_value(&self) -> Rational32 {
        let finite_val = UAdic::new(self.p, self.fix_d.clone()).integer_value();
        let repeat_offset = self.p().pow(self.fix_d.len() as u32);
        let numerator: u32 = self.rep_d
            .iter()
            .enumerate()
            .map(|(k, d)| *d * self.p().pow(k as u32))
            .sum();
        let denominator: u32 = if self.rep_d.is_empty() {
            1
        } else {
            self.p().pow(self.rep_d.len() as u32) - 1
        };
        Rational32::new(
            (finite_val * denominator) as i32 - (repeat_offset * numerator) as i32,
            denominator as i32
        )
    }

    /// The big rational representation for the rational number value of the number ([`rational_value`](Self::rational_value))
    ///
    /// ```
    /// # use num::{BigInt, BigRational};
    /// # use adic::radic;
    /// assert_eq!(BigRational::new(BigInt::from(-1), BigInt::from(4)), radic!(5, [], [1]).big_rational_value());
    /// ```
    pub fn big_rational_value(&self) -> BigRational {
        let finite_val = UAdic::new(self.p, self.fix_d.clone()).big_integer_value();
        let repeat_offset = BigInt::from(self.p()).pow(self.fix_d.len() as u32);
        let numerator: BigInt = self.rep_d
            .iter()
            .enumerate()
            .map(|(k, d)| BigInt::from(*d) * BigInt::from(self.p()).pow(k as u32))
            .sum();
        let denominator = if self.rep_d.is_empty() {
            BigInt::one()
        } else {
            BigInt::from(self.p()).pow(self.rep_d.len() as u32) - BigInt::one()
        };
        BigRational::new(finite_val * denominator.clone() - repeat_offset * numerator, denominator)
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
            ZAdicValuation::Finite(self.fix_d.len() as u32)
        } else {
            ZAdicValuation::PosInf
        }
    }
    fn digit(&self, n: u32) -> Result<u32, AdicError> {
        if (n as usize) < self.fix_d.len() {
            Ok(self.fix_d.get(n as usize).copied().unwrap_or(0))
        } else if self.rep_d.is_empty() {
            Ok(0)
        } else {
            let diff = (n as usize) - self.fix_d.len();
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
    fn certainty(&self) -> ZAdicValuation {
        ZAdicValuation::PosInf
    }
}


impl From<UAdic> for RAdic {
    fn from(value: UAdic) -> Self {
        Self::new(value.p(), value.into_digits_vec(), vec![])
    }
}

impl From<IAdic> for RAdic {
    fn from(a: IAdic) -> Self {
        let p = a.p();
        let num_non_trailing = a.num_non_trailing() as usize;
        Self::new(p, a.into_digits().take(num_non_trailing).collect(), vec![p-1])
    }
}

impl TryFrom<RAdic> for UAdic {
    type Error = AdicError;
    fn try_from(a: RAdic) -> Result<Self, Self::Error> {
        let p = a.p();
        if a.rep_d.is_empty() {
            Ok(Self::new(p, a.fix_d))
        } else {
            Err(AdicError::BadConversion)
        }
    }
}

impl TryFrom<RAdic> for IAdic {
    type Error = AdicError;
    fn try_from(a: RAdic) -> Result<Self, Self::Error> {
        let p = a.p();
        let mut repeat_iter = a.repeat_digits();
        match repeat_iter.next() {
            None => Ok(Self::new_pos(p, a.fixed_digits().copied().collect::<Vec<_>>())),
            Some(digit) => {
                if repeat_iter.next().is_none() && *digit == a.p()-1 {
                    Ok(Self::new_neg(p, a.fixed_digits().copied().collect::<Vec<_>>()))
                } else {
                    Err(AdicError::BadConversion)
                }
            }
        }
    }
}


impl std::ops::Add for RAdic {
    type Output = RAdic;
    fn add(self, rhs: Self) -> Self::Output {

        // Two steps:
        // 1 - add finite integers
        // 2 - add repeating digits until it repeats
        //
        // 1
        // Find which adic has finite integer with more digits
        // Fill out the other finite integer with repeats
        // Add those together to get a new finite integer
        // Strip off digits that were carried past and use as a carry for next step
        //
        // 2
        // Add repeating digits one-by-one keeping track of carry
        // Store a vector of length lcm(self.rep_d.len(), rhs.rep_d.len())
        // Look for the digits AND carry to start repeating
        // Once they repeat, add non-repeating digits to finite digits and keep the rest as repeats

        assert!(self.p == rhs.p, "{:?}", AdicError::MixedCharacteristic);
        let p = self.p;

        // Get new finite_int with long_finite_int + short_finite_int + short_repeats
        let (longer, shorter) = if self.fix_d.len() > rhs.fix_d.len() {
            (self, rhs)
        } else {
            (rhs, self)
        };
        let longer_len = longer.fix_d.len();
        let shorter_len = shorter.fix_d.len();
        let longer_digits = longer.fix_d
            .into_iter()
            .chain(longer.rep_d.clone().into_iter().cycle())
            .take(longer_len)
            .collect();
        let shorter_digits = shorter.fix_d
            .into_iter()
            .chain(shorter.rep_d.clone().into_iter().cycle())
            .take(longer_len)
            .collect();
        let mut finite_integer_digits = (
            UAdic::new(p, longer_digits) + UAdic::new(p, shorter_digits)
        ).into_digits_vec();

        // Fill with zeros
        let finite_len = finite_integer_digits.len();
        if finite_len < longer_len {
            finite_integer_digits.extend(repeat(0).take(longer_len - finite_len));
        }

        // Adding may have overshot longer_len, so change the last digits into a carry
        let overshoot = if finite_len > longer_len {
            finite_integer_digits.split_off(longer_len)
        } else {
            vec![]
        };
        let mut carry = overshoot.into_iter().enumerate().map(|(idx, d)| d * p.pow(idx as u32)).sum();

        // Appropriately advance the shorter iterator to match the repeats that have been used
        let longer_replen = max(longer.rep_d.len(), 1);
        let mut longer_repeat_iter = longer.rep_d
            .into_iter()
            .cycle()
            .chain(repeat(0));
        let shorter_replen = max(shorter.rep_d.len(), 1);
        let mut shorter_repeat_iter = shorter.rep_d
            .into_iter()
            .cycle()
            .skip(longer_len - shorter_len)
            .chain(repeat(0));

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
            }
            add_buffer.push((carry, added % p));
        }

        // Add last digits to finite_integer and make max_cycle_len new repeats
        let leftover_finite = add_buffer.len() - max_cycle_len;
        let mut added_iter = add_buffer.into_iter().map(|(_, d)| d);
        for _ in 0..leftover_finite {
            finite_integer_digits.push(added_iter.next().unwrap());
        }
        let new_repeats = added_iter.collect::<Vec<_>>();

        Self::new(p, finite_integer_digits, new_repeats)

    }
}


impl std::ops::Neg for RAdic {
    type Output = RAdic;
    fn neg(self) -> Self::Output {

        let p = self.p();

        if self.fix_d.iter().chain(self.rep_d.iter()).all(|d| *d == 0) {

            // If finite_integer is zero and repeats are zero, return zero
            self

        } else if self.fix_d.iter().all(|d| *d == 0) {

            // If finite_integer is zero, find the first nonzero repeat and turn into finite_integer
            let repeats_len = self.rep_d.len();
            let zeros_len = self.rep_d.iter().take_while(|d| **d == 0).count();
            let new_repeat_order = self.rep_d
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
            let mut new_digits = self.fix_d;
            new_digits.extend(repeat(0).take(zeros_len));
            new_digits.push(p - first_nonzero);
            let new_repeats = new_repeat_order.into_iter().map(|d| p - d - 1).collect::<Vec<_>>();
            Self::new(p, new_digits, new_repeats)

        } else {

            let mut new_digits = Vec::with_capacity(self.fix_d.len());
            let mut finite_iter = self.fix_d.into_iter();
            for d in finite_iter.by_ref() {
                if d == 0 {
                    new_digits.push(0);
                } else {
                    new_digits.push(p - d);
                    break;
                }
            }
            for d in finite_iter.by_ref() {
                new_digits.push(p - d - 1);
            }

            let new_repeats = if self.rep_d.is_empty() {
                vec![p-1]
            } else {
                self.rep_d.into_iter().map(|d| p - d - 1).collect::<Vec<_>>()
            };

            Self::new(p, new_digits, new_repeats)

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

impl std::ops::Mul for RAdic {
    type Output = RAdic;
    fn mul(self, rhs: Self) -> Self::Output {

        // Input
        // a = [d0, d1, ... d(i-1), r0, ... r(m-1), r0, ... r(m-1), r0, ...]
        // b = [e0, e1, ... e(j-1), s0, ... s(n-1), s0, ... s(n-1), s0, ...]
        //
        // Output
        // a * b = o = o_finite + o_ds + o_er + o_rs
        //
        // Pseudocode:
        //
        // Finite part is easy
        //
        // Calculate o_finite = UAdic(d0, ... d(i-1)) * UAdic(e0, ... e(j-1))
        //
        // Finite times repeating is mostly easy
        //
        // Calculate x_ds = UAdic(d0, ... d(i-1)) * UAdic(s0, ... s(n-1))
        // o_ds = p^j * (- x_ds / (p^n-1))
        //      = p^j * (- (x_ds % p^n-1) / (p^n-1) - (x_ds // (p^n-1)))
        //      = p^j * ( (remainder repeating digits) - (integer) )
        //
        // Calculate x_re = UAdic(r0, ... r(m-1)) * UAdic(e0, ... e(j-1))
        // o_re = p^i * (- x_re / (p^m-1))
        //      = p^i * (- (x_re % p^m-1) / (p^m-1) - (x_re // (p^m-1)))
        //      = p^i * ( (remainder repeating digits) - (integer) )
        //
        // All that's left is multiplying the repeating digits, o_rs
        //
        // For this, we need the Euler totient function
        // See https://en.wikipedia.org/wiki/Euler%27s_totient_function
        // Calculate the totient for the repeat size:
        // h = new_repeat_size = phi((p^m - 1) (p^n - 1))
        // Note that m | h and n | h, each input repeat matches with the output
        //
        // We will be multiplying separately the digits and the geometric series:
        // o_rs = p^(i + j) * UAdic(r0, ... r(m-1)) * (-1 / (p^m-1)) * UAdic(s0, ... s(n-1)) * (-1 / (p^n-1))
        // y = UAdic(r0, ... r(m-1)) * UAdic(s0, ... s(n-1))
        // g = (-1 / (p^m - 1)) * (-1 / (p^n - 1)) - 1
        //   = [1, 0, 0, 1, 0, 0, 1, ...] * [1, 0, 1, 0, 1, 0, 1, ...] - 1
        // o_rs = p^(i+j) * (y + y*g) = p^(i+j) * (y + y * (-f / (p^h-1)))
        // The extra "-1" is so that -1 < g < 0, which guarantees repeating digits
        // Truncate a UAdic multiplication to length h to get f
        // f represents the repeating digits, numerator for the power-h geometric series
        // Then calculate y * f and split into quotient and remainder (by (p^h-1) using the method above)
        //
        // o_rs = p^(i+j) * (y - (y*f // (p^h-1)) + (y*f % (p^h-1)) * (-1 / (p^h-1)))
        //
        // Finally
        // c = o_finite + o_ds + o_re + o_rs

        assert!(self.p == rhs.p, "{:?}", AdicError::MixedCharacteristic);
        let p = self.p;

        // Initialize various lengths and digits
        let i = self.fix_d.len();
        let d = UAdic::new(p, self.fix_d);
        let j = rhs.fix_d.len();
        let e = UAdic::new(p, rhs.fix_d);
        let m = self.rep_d.len();
        let r = UAdic::new(p, self.rep_d);
        let n = rhs.rep_d.len();
        let s = UAdic::new(p, rhs.rep_d);

        // Finite digit multiplication
        let o_finite = if i > 0 && j > 0 {
            let int_finite = d.clone() * e.clone();
            RAdic::new(p, int_finite.into_digits_vec(), vec![])
        } else {
            RAdic::zero(p)
        };

        // self.finite * rhs.repeats
        let o_ds = if i > 0 && n > 0 {
            let x_ds = d.clone() * s.clone();
            // Calculate quotient and remainder w.r.t. (p^n-1)
            let (r_ds, q_ds) = x_ds.pseudo_pn_minus_1_rem_quot(n);
            let int_ds = RAdic::new(p, [vec![0; j], q_ds.into_digits_vec()].concat(), vec![]);
            let frac_ds = RAdic::new(p, vec![0; j], r_ds.into_padded_digits(n));
            -int_ds + frac_ds
        } else {
            RAdic::zero(p)
        };

        // self.repeats * rhs.finite
        let o_re = if j > 0 && m > 0 {
            let x_re = r.clone() * e.clone();
            // Calculate quotient and remainder w.r.t. (p^m-1)
            let (r_re, q_re) = x_re.pseudo_pn_minus_1_rem_quot(m);
            let int_re = RAdic::new(p, [vec![0; i], q_re.into_digits_vec()].concat(), vec![]);
            let frac_re = RAdic::new(p, vec![0; i], r_re.into_padded_digits(m));
            -int_re + frac_re
        } else {
            RAdic::zero(p)
        };

        // self.repeats * rhs.repeats
        let o_rs = if m > 0 && n > 0 {

            let h = totient_many(&[p.pow(m as u32) - 1, p.pow(n as u32) - 1]) as usize;

            if h > 1000 {
                println!("WARNING: the computation of {r} x {s} will take {h} digits...");
            }

            let y_r = UAdic::new(p, r.into_digits_vec());
            let y_s = UAdic::new(p, s.into_digits_vec());
            let y_rs = y_r * y_s;
            let g_r = UAdic::new(p, once(1).chain(repeat(0).take(m-1)).cycle().take(h).collect());
            let g_s = UAdic::new(p, once(1).chain(repeat(0).take(n-1)).cycle().take(h).collect());
            let g_rs = (g_r * g_s);
            let f_rs = UAdic::new(p, once(0).chain(
                g_rs.into_digits_vec().into_iter().skip(1).take(h-1)
            ).collect());
            // o_rs = p^(i + j) * (y + (y*f // (p^h-1)) - (y*f % (p^h-1)) * (-1 / (p^h-1)))
            let yf_rs = y_rs.clone() * f_rs;
            // Calculate quotient and remainder w.r.t. (p^h-1)
            let (r_rs, q_rs) = yf_rs.pseudo_pn_minus_1_rem_quot(h);
            let inty_rs = RAdic::new(p, [vec![0; i+j], y_rs.into_digits_vec()].concat(), vec![]);
            let intq_rs = -RAdic::new(p, [vec![0; i+j], q_rs.into_digits_vec()].concat(), vec![]);
            let frac_rs = RAdic::new(p, vec![0; i+j], r_rs.into_padded_digits(h));
            inty_rs + intq_rs + frac_rs

        } else {
            RAdic::zero(p)
        };

        o_finite + o_ds + o_re + o_rs

    }
}

// TODO: Div (much harder?)


impl Pow<u32> for &RAdic {
    type Output = RAdic;
    fn pow(self, power: u32) -> Self::Output {
        repeat(
            self.clone()
        ).take(
            power as usize
        ).reduce(
            |acc, e| acc * e
        ).unwrap_or(
            RAdic::one(self.p)
        )
    }
}


impl fmt::Display for RAdic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p = self.p;

        if self.rep_d.is_empty() {
            return write!(f, "{}", UAdic::new(p, self.fix_d.clone()))
        }

        let rep_digits = self.rep_d.iter().join("").chars().rev().collect::<String>();
        let fix_digits = self.fix_d.iter().join("").chars().rev().collect::<String>();

        write!(f, "({rep_digits}){fix_digits}._{p}")
    }
}


#[cfg(test)]
mod tests {
    use itertools::{Itertools, repeat_n};
    use num::{traits::Pow, Rational32};
    use crate::{radic, uadic, zadic_approx, AdicError, ZAdic, ZAdicValuation, ZAdicVariety};
    use super::{AdicInteger, RAdic};

    fn zero_2() -> RAdic { radic!(2, [], []) }
    fn one_2() -> RAdic { radic!(2, [1], []) }
    fn eight_2() -> RAdic { radic!(2, [0, 0, 0, 1], []) }
    fn neg_one_2() -> RAdic { radic!(2, [], [1]) }
    fn neg_1_3_2() -> RAdic { radic!(2, [], [1, 0]) }
    fn neg_8_3_2() -> RAdic { radic!(2, [0, 0], [0, 1]) }
    fn pos_1_9_2() -> RAdic { radic!(2, [1], [0, 0, 1, 1, 1, 0]) }
    fn pos_64_9_2() -> RAdic { radic!(2, [0, 0, 0, 0, 0, 0, 1], [0, 0, 1, 1, 1, 0]) }

    fn zero() -> RAdic { radic!(5, [], []) }
    fn one() -> RAdic { radic!(5, [1], []) }
    fn two() -> RAdic { radic!(5, [2], []) }
    fn three() -> RAdic { radic!(5, [3], []) }
    fn four() -> RAdic { radic!(5, [4], []) }
    fn five() -> RAdic { radic!(5, [0, 1], []) }
    fn six() -> RAdic { radic!(5, [1, 1], []) }
    fn seven() -> RAdic { radic!(5, [2, 1], []) }
    fn eight() -> RAdic { radic!(5, [3, 1], []) }
    fn nine() -> RAdic { radic!(5, [4, 1], []) }
    fn ten() -> RAdic { radic!(5, [0, 2], []) }
    fn eleven() -> RAdic { radic!(5, [1, 2], []) }
    fn twenty_five() -> RAdic { radic!(5, [0, 0, 1], []) }
    fn neg_one() -> RAdic { radic!(5, [], [4]) }
    fn neg_two() -> RAdic { radic!(5, [3], [4]) }
    fn neg_three() -> RAdic { radic!(5, [2], [4]) }
    fn neg_four() -> RAdic { radic!(5, [1], [4]) }
    fn neg_five() -> RAdic { radic!(5, [0], [4]) }
    fn neg_ten() -> RAdic { radic!(5, [0, 3], [4]) }
    fn neg_1_4() -> RAdic { radic!(5, [], [1]) }
    fn pos_1_4() -> RAdic { radic!(5, [4], [3]) }
    fn neg_5_4() -> RAdic { radic!(5, [0], [1]) }
    fn pos_1_16() -> RAdic { radic!(5, [1], [2, 3, 4, 0]) }
    fn neg_1_64() -> RAdic { radic!(5, [], [1, 3, 1, 1, 2, 4, 2, 2, 3, 0, 4, 3, 4, 1, 0, 0]) }
    fn pos_25_16() -> RAdic { radic!(5, [0, 0, 1], [2, 3, 4, 0]) }
    fn pos_43_4() -> RAdic { radic!(5, [2, 3], [1]) }
    fn neg_1_24() -> RAdic { radic!(5, [], [1, 0]) }
    fn neg_5_24() -> RAdic { radic!(5, [], [0, 1]) }
    fn neg_1_31() -> RAdic { radic!(5, [], [4, 0, 0]) }
    fn pos_30_31() -> RAdic { one() + neg_1_31().clone() }
    fn neg_5_31() -> RAdic { neg_1_31() + neg_1_31() + neg_1_31() + neg_1_31() + neg_1_31() }
    fn neg_30_31() -> RAdic { neg_5_31() + neg_5_31() + neg_5_31() + neg_5_31() + neg_5_31() + neg_5_31() }
    fn neg_one_sixth() -> RAdic { radic!(5, [], [4, 0]) }
    fn seventeen_sixth() -> RAdic { three() + neg_one_sixth() }

    #[test]
    fn test_r_adic() {
        assert_eq!(uadic!(5, [1, 1, 1]), neg_1_4().into_truncation(3));
        assert_eq!(uadic!(5, [1, 1, 1, 1, 1, 1]), neg_1_4().into_truncation(6));
        assert_eq!(uadic!(5, [1, 1, 1, 1, 1, 1, 1, 1, 1]), neg_1_4().into_truncation(9));
        assert_eq!(radic!(5, [], [1]), radic!(5, [1], [1]));
        assert_eq!(radic!(5, [1], [2]), radic!(5, [1, 2], [2]));
        assert_eq!(radic!(5, [1], []), radic!(5, [1], [0, 0]));
        assert_eq!(radic!(5, [], [1, 0]), radic!(5, [1], [0, 1]));
        assert_eq!(radic!(5, [1, 0, 1], []), RAdic::from_integer(5, 26));
        assert_eq!(radic!(5, [4, 4, 3], [4]), RAdic::from_integer(5, -26));
        assert_eq!(twenty_five().certainty(), ZAdicValuation::PosInf);
        assert_eq!(seventeen_sixth().certainty(), ZAdicValuation::PosInf);
    }

    #[test]
    fn test_add_r_integers() {
        assert_eq!(three(), two() + one());
        assert_eq!(two(), one() + one());
        assert_eq!(two() + one(), one() + two());
        assert_eq!(seven(), one() + six());
        assert_eq!(neg_ten(), neg_five() + neg_five());
        assert_eq!(neg_8_3_2(), neg_1_3_2() + neg_1_3_2() + neg_1_3_2() + neg_1_3_2() + neg_1_3_2() + neg_1_3_2() + neg_1_3_2() + neg_1_3_2());
    }

    #[test]
    fn test_neg_r_integers() {
        assert_eq!(neg_one(), -one());
        assert_eq!(zero(), -zero());
        assert_eq!(neg_five(), -five());
        let neg_p_to_third = -radic!(5, [0, 0, 0, 1], []);
        assert_eq!(radic!(5, [0, 0, 0], [4]), neg_p_to_third);
    }

    #[test]
    fn test_sub_r_integers() {
        assert_eq!(one(), two() - one());
        assert_eq!(zero(), one() - one());
        assert_eq!(neg_one(), one() - two());
        assert_eq!(neg_five(), one() - six());
    }

    #[test]
    fn test_mul_r_integers() {
        let check = |c: &RAdic, a: &RAdic, b: &RAdic| {
            assert_eq!(c.clone(), a.clone() * b.clone());
            assert_eq!(c.clone(), b.clone() * a.clone());
        };
        check(&one(), &one(), &one());
        check(&two(), &two(), &one());
        check(&six(), &two(), &three());
        check(&six(), &three(), &two());
        for num in [&zero(), &one(), &two(), &three(), &four(), &five(), &six(), &neg_one()] {
            check(&zero(), &zero(), num);
            check(&zero(), num, &zero());
        }
        check(&neg_one(), &one(), &neg_one());
        check(&neg_two(), &two(), &neg_one());
        check(&neg_four(), &two(), &neg_two());
        check(&one(), &neg_one(), &neg_one());
        check(&six(), &neg_two(), &neg_three());
        check(&ten(), &five(), &two());
        check(&twenty_five(), &five(), &five());
        check(&twenty_five(), &neg_five(), &neg_five());
    }

    #[test]
    fn test_rational_value() {
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
    fn test_r_adic_norm() {
        assert_eq!(ZAdicValuation::PosInf, zero().valuation());
        assert_eq!(Rational32::ZERO, zero().norm());
        assert_eq!(ZAdicValuation::Finite(0), one().valuation());
        assert_eq!(Rational32::new(1, 1), one().norm());
        assert_eq!(ZAdicValuation::Finite(1), five().valuation());
        assert_eq!(Rational32::new(1, 5), five().norm());
        assert_eq!(ZAdicValuation::Finite(0), neg_1_4().valuation());
        assert_eq!(Rational32::new(1, 1), neg_1_4().norm());
        assert_eq!(ZAdicValuation::Finite(1), neg_5_4().valuation());
        assert_eq!(Rational32::new(1, 5), neg_5_4().norm());
        assert_eq!(ZAdicValuation::Finite(0), neg_1_24().valuation());
        assert_eq!(Rational32::new(1, 1), neg_1_24().norm());
        assert_eq!(ZAdicValuation::Finite(1), neg_5_24().valuation());
        assert_eq!(Rational32::new(1, 5), neg_5_24().norm());
    }


    #[test]
    fn test_add_sub_r() {

        assert_eq!(neg_1_4(), -pos_1_4());
        assert_eq!(pos_1_4(), -neg_1_4());
        assert_eq!(pos_43_4(), neg_1_4() + eleven());
        assert_eq!(-pos_43_4(), pos_1_4() - eleven());
        assert_eq!(neg_1_24() + neg_1_24() + neg_1_24() + neg_1_24() + neg_1_24(), neg_5_24());
        assert_eq!(neg_1_24() + neg_1_24() + neg_1_24() + neg_1_24() + neg_1_24() + neg_1_24(), neg_1_4());

        assert_eq!(radic!(5, [0, 1], [0, 4, 0]), pos_30_31());
        assert_eq!(radic!(5, [], [0, 4, 0]), neg_5_31());
        assert_eq!(radic!(5, [], []), pos_30_31() + neg_30_31());

        assert_eq!(radic!(5, [2, 1], [4, 0]), seventeen_sixth());
        assert_eq!(uadic!(5, [2, 1, 4, 0, 4, 0]), seventeen_sixth().into_truncation(6));

    }

    #[test]
    fn test_mul_r() {

        assert_eq!(radic!(5, [1], [2, 3, 4, 0]), neg_1_4() * neg_1_4());
        assert_eq!(radic!(5, [], [4]), neg_1_4() * four());
        assert_eq!(neg_1_4(), neg_1_24() * six());
        assert_eq!(neg_5_24(), neg_1_24() * five());

        // 3-adic
        let neg_9_2 = radic!(3, [0, 0], [1]);
        assert_eq!(Rational32::new(-9, 2), neg_9_2.clone().rational_value());
        let pos_81_4 = radic!(3, [0, 0, 0, 0, 1], [2, 0]);
        assert_eq!(Rational32::new(81, 4), pos_81_4.clone().rational_value());
        let neg_729_8 = radic!(3, [0, 0, 0, 0, 0], [0, 1]);
        assert_eq!(Rational32::new(-729, 8), neg_729_8.clone().rational_value());
        assert_eq!(pos_81_4, neg_9_2.clone() * neg_9_2.clone());
        assert_eq!(neg_729_8, pos_81_4.clone() * neg_9_2.clone());
        assert_eq!(neg_729_8, pos_81_4.clone() * neg_9_2.clone());

        // 7-adic
        let neg_1_6_sq = radic!(7, [], [1]) * radic!(7, [], [1]);
        assert_eq!(radic!(7, [1], [2, 3, 4, 5, 6, 0]), neg_1_6_sq);

        // 2-adic
        assert_eq!(one_2(), neg_one_2() * neg_one_2());
        assert_eq!(pos_1_9_2(), neg_1_3_2() * neg_1_3_2());
        assert_eq!(neg_8_3_2(), eight_2() * neg_1_3_2());
        assert_eq!(pos_64_9_2(), neg_8_3_2() * neg_8_3_2());

    }

    #[test]
    fn test_pow_r_adic() {

        assert_eq!(zero(), zero().pow(2));
        assert_eq!(zero(), zero().pow(3));
        assert_eq!(one(), one().pow(2));
        assert_eq!(one(), one().pow(3));
        assert_eq!(four(), two().pow(2));
        assert_eq!(eight(), two().pow(3));
        assert_eq!(nine(), three().pow(2));
        assert_eq!(twenty_five(), five().pow(2));
        assert_eq!(one(), neg_two().pow(0));
        assert_eq!(neg_one(), neg_one().pow(1));
        assert_eq!(one(), neg_one().pow(2));
        assert_eq!(four(), neg_two().pow(2));
        assert_eq!(twenty_five(), neg_five().pow(2));
        assert_eq!(pos_1_16(), neg_1_4().pow(2));
        assert_eq!(neg_1_64(), neg_1_4().pow(3));
        assert_eq!(pos_25_16(), neg_5_4().pow(2));

        assert_eq!(zero_2(), zero_2().pow(2));
        assert_eq!(one_2(), one_2().pow(2));
        assert_eq!(one_2(), neg_one_2().pow(2));
        assert_eq!(neg_one_2(), neg_one_2().pow(3));
        assert_eq!(pos_1_9_2(), neg_1_3_2().pow(2));
        assert_eq!(pos_64_9_2(), neg_8_3_2().pow(2));

    }

    #[test]
    fn test_nth_root() {

        let check = |p: u32, a: &RAdic, n: u32, precision: u32, roots: Vec<ZAdic>| {
            // Check each root powers to match a to at least precision digits
            for root in &roots {
                assert_eq!(a.truncation(precision as usize), root.pow(n).into_truncation_to_uadic().unwrap());
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

    #[ignore = "Takes five minutes"]
    #[test]
    fn test_r_adic_ops_many() {
        // Test addition and multiplication over many rationals using rational_value
        let p = 5;
        let fix_n = 2;
        let rep_n = 2;
        let firsts = repeat_n(0..p, fix_n).multi_cartesian_product().cartesian_product(
            repeat_n(0..p, rep_n).multi_cartesian_product()
        ).map(
            |(fixed_digits, repeat_digits)| RAdic::new(p, fixed_digits, repeat_digits)
        );
        let seconds = firsts.clone();
        for (first, second) in firsts.cartesian_product(seconds) {
            let first_val = first.big_rational_value();
            let second_val = second.big_rational_value();
            let sum_val = (first.clone() + second.clone()).big_rational_value();
            let prod_val = (first.clone() * second.clone()).big_rational_value();
            assert_eq!(first_val.clone() + second_val.clone(), sum_val);
            assert_eq!(first_val * second_val, prod_val);
        }
    }

    #[test]
    #[should_panic]
    fn test_non_prime() {
        let _ = radic!(6, [2], [1]);
    }

    #[test]
    #[should_panic]
    fn test_mixed_characteristic() {
        let _ = radic!(5, [1], [1]) + radic!(7, [1], [1]);
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
        assert_eq!("12._5", seven().to_string());
        assert_eq!("20._5", ten().to_string());
        assert_eq!("21._5", eleven().to_string());
        assert_eq!("100._5", twenty_five().to_string());
        assert_eq!("(4)._5", neg_one().to_string());
        assert_eq!("(4)3._5", neg_two().to_string());
        assert_eq!("(4)2._5", neg_three().to_string());
        assert_eq!("(4)1._5", neg_four().to_string());
        assert_eq!("(4)0._5", neg_five().to_string());
        assert_eq!("(4)30._5", neg_ten().to_string());
        assert_eq!("(1)._5", neg_1_4().to_string());
        assert_eq!("(3)4._5", pos_1_4().to_string());
        assert_eq!("(1)0._5", neg_5_4().to_string());
        assert_eq!("(1)32._5", pos_43_4().to_string());
        assert_eq!("(01)._5", neg_1_24().to_string());
        assert_eq!("(10)._5", neg_5_24().to_string());
        assert_eq!("(004)._5", neg_1_31().to_string());
        assert_eq!("(04)._5", neg_one_sixth().to_string());
    }

}
