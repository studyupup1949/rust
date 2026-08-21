use std::{fmt, iter::{once, repeat}};
use itertools::Itertools;
use num::{traits::Pow, BigInt, Zero};
use num_prime::nt_funcs::is_prime;
use crate::AdicError;
use super::{AdicInteger, UAdic, ZAdicValuation};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Approximate Adic Integer, represented by a partially-known digital expansion
/// ([`zadic_approx`](crate::zadic_approx), [`zadic_exact`](crate::zadic_exact), [`zadic_exact_neg`](crate::zadic_exact_neg))
///
/// An [`AdicInteger`](crate::AdicInteger).
/// Often used to represent irrational adic numbers.
///
/// `ZAdic`s represent approximate adic numbers, known to a "certainty", some number of digits `c`.
/// These are returned from approximate methods like [`nth_root`](AdicInteger::nth_root).
/// They are often held together in a [`ZAdicVariety`](crate::ZAdicVariety).
///
/// ```
/// # use num::Rational32;
/// # use adic::{AdicInteger, ZAdic};
/// assert_eq!("---002341._5", ZAdic::new_approx(5, 6, vec![1, 4, 3, 2]).to_string());
/// assert_eq!("2341._5", ZAdic::new_exact(5, vec![1, 4, 3, 2]).to_string());
/// ```
///
/// Adding and multiplying `ZAdic`s respects the certainty.
/// When adding, the output certainty is the minimum of the input certainties:
///  `---abc._p + ---de._p = ---fg._p`.
/// When multiplying, the output certainty is a little more complicated,
///  since zero digits can make things more certain than just the minumum:
///  `---ab0._p * ---de._p = ---fg0._p`.
///
/// `ZAdic`s can also represent exact integers, both positive and negative.
/// In the case of positive integers, its digits are just the digits of the integer
///  (in base p) and then zeros, going left.
/// In the case of negative integers, it has the digits of the integer
///  (in base p) and then the digit (p-1), going left.
///
/// ```
/// # use adic::{AdicInteger, ZAdic};
/// let one_e = ZAdic::new_exact(5, vec![1]);
/// assert_eq!("1._5", one_e.to_string());
/// let neg_one_e = ZAdic::new_exact_neg(5, vec![]);
/// assert_eq!("...44._5", neg_one_e.to_string());
/// assert!((one_e + neg_one_e).is_zero());
/// ```
///
/// In this way, the exact `ZAdic` is more flexible than [`UAdic`](crate::UAdic),
/// which can only represent non-negative integers,
/// but less than [`RAdic`](crate::RAdic),
/// which can represent rationals without p in the denominator.
pub struct ZAdic {
    /// Adic prime
    p: u32,
    /// One less than prime
    pm1: u32,
    /// Adic digits, each 0 to p-1
    d: Vec<u32>,
    /// Valuation of certainty; number of digits that are known
    c: ZAdicCertainty,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum ZAdicCertainty {
    ExactPos,
    ExactNeg,
    Approx(u32),
}


impl ZAdic {

    /// Create an adic number with the given digits and certainty
    ///
    /// # Panics
    /// Panics if `p` is not prime
    pub fn new_approx(p: u32, certainty: u32, mut init_digits: Vec<u32>) -> Self {

        assert!(is_prime(&p, None).probably());

        // Truncate uncertain digits
        init_digits.truncate(certainty as usize);

        // Truncate zeros so there should never be leading zeros for a ZAdic
        while let Some(0) = init_digits.last() {
            init_digits.pop();
        }

        Self {
            p,
            pm1: p-1,
            d: init_digits,
            c: ZAdicCertainty::Approx(certainty),
        }

    }

    /// Create an exact adic number with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime
    pub fn new_exact(p: u32, mut init_digits: Vec<u32>) -> Self {

        assert!(is_prime(&p, None).probably());

        // Truncate zeros so there should never be leading zeros for a ZAdic
        while Some(&0) == init_digits.last() {
            init_digits.pop();
        }

        Self {
            p,
            pm1: p-1,
            d: init_digits,
            c: ZAdicCertainty::ExactPos,
        }

    }

    /// Create an exact negative adic number (trailing p-1) with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime
    pub fn new_exact_neg(p: u32, mut init_digits: Vec<u32>) -> Self {

        assert!(is_prime(&p, None).probably());

        // Truncate zeros so there should never be leading zeros for a ZAdic
        while Some(&(p-1)) == init_digits.last() {
            init_digits.pop();
        }

        Self {
            p,
            pm1: p-1,
            d: init_digits,
            c: ZAdicCertainty::ExactNeg,
        }

    }

    /// Create an exact adic number that corresponds to the given integer
    pub fn exact_from_integer(p: u32, n: u32) -> Self {
        Self::new_exact(p, UAdic::from_integer(p, n).into_digits_vec())
    }

    /// Push another cerain digit onto the end of the number
    ///
    /// # Errors
    /// Returns error if number already has infinite certainty
    ///
    /// ```
    /// # use adic::{zadic_approx, zadic_exact, zadic_exact_neg, AdicError};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut z = zadic_approx!(5, 4, [1, 2, 3, 4]);
    /// z.push_digit(3)?;
    /// assert_eq!(zadic_approx!(5, 5, [1, 2, 3, 4, 3]), z);
    /// let mut z = zadic_exact!(5, [1, 2, 3, 4]);
    /// assert!(matches!(z.push_digit(3), Err(AdicError::InappropriatePrecision(_))));
    /// let mut z = zadic_exact_neg!(5, [1, 2, 3, 4]);
    /// assert!(matches!(z.push_digit(3), Err(AdicError::InappropriatePrecision(_))));
    /// # Ok(()) }
    /// ```
    pub fn push_digit(&mut self, digit: u32) -> Result<(), AdicError> {
        match self.c {
            ZAdicCertainty::Approx(c) => {
                if (c as usize) < self.d.len() {
                    Err(AdicError::InappropriatePrecision(
                        "Certainty less than digits size; should not happen!".to_string()
                    ))
                } else {
                    let num_leading_zeros = (c as usize) - self.d.len();
                    self.d.extend(repeat(0).take(num_leading_zeros).chain(once(digit)));
                    self.c = ZAdicCertainty::Approx(c+1);
                    Ok(())
                }
            },
            ZAdicCertainty::ExactPos | ZAdicCertainty::ExactNeg => Err(AdicError::InappropriatePrecision(
                "Cannot append to infinite certainty number".to_string()
            )),
        }
    }

    /// Change the certainty of the `ZAdic`, assuming zeros for any new digits
    ///
    /// ```
    /// # use adic::{zadic_approx, zadic_exact, zadic_exact_neg, AdicError, ZAdicValuation};
    /// let mut z = zadic_approx!(5, 4, [1, 2, 3, 4]);
    /// z.set_certainty(ZAdicValuation::Finite(5));
    /// assert_eq!(zadic_approx!(5, 5, [1, 2, 3, 4, 0]), z);
    /// z.set_certainty(ZAdicValuation::Finite(3));
    /// assert_eq!(zadic_approx!(5, 3, [1, 2, 3]), z);
    /// z.set_certainty(ZAdicValuation::PosInf);
    /// assert_eq!(zadic_exact!(5, [1, 2, 3]), z);
    /// ```
    pub fn set_certainty(&mut self, c: ZAdicValuation) {
        if let ZAdicValuation::Finite(fc) = c {
            self.d.truncate(fc as usize);
            while let Some(0) = self.d.last() {
               self.d.pop();
            }
            self.c = ZAdicCertainty::Approx(fc);
        } else {
            // Assume positive number
            self.c = ZAdicCertainty::ExactPos;
        }
    }

    /// The natural number value of the number, e.g. 5-adic 123 is 25+10+3=38
    ///
    /// Warning: This can overflow; use [`UAdic::big_integer_value`] if unsure
    ///
    /// ```
    /// # use adic::zadic_approx;
    /// assert_eq!(38, zadic_approx!(5, 4, [3, 2, 1, 0]).integer_value());
    /// ```
    pub fn integer_value(&self) -> i32 {
        match self.c {
            ZAdicCertainty::Approx(c) => {
                self.digits()
                    .take(c as usize)
                    .enumerate()
                    .map(|(k, d)| *d * self.p.pow(k as u32))
                    .sum::<u32>() as i32
            },
            ZAdicCertainty::ExactPos => {
                self.digits()
                    .enumerate()
                    .map(|(k, d)| *d * self.p.pow(k as u32))
                    .sum::<u32>() as i32
            },
            ZAdicCertainty::ExactNeg => {
                -(
                    (-self.clone()).into_digits()
                        .enumerate()
                        .map(|(k, d)| d * self.p.pow(k as u32))
                        .sum::<u32>() as i32
                )
            }
        }
    }

    /// The bigint representation for the natural number value of the number
    ///
    /// ```
    /// # use num::BigInt;
    /// # use adic::zadic_approx;
    /// assert_eq!(BigInt::from(38), zadic_approx!(5, 4, [3, 2, 1, 0]).big_integer_value());
    /// ```
    pub fn big_integer_value(&self) -> BigInt {
        match self.c {
            ZAdicCertainty::Approx(c) => {
                self.digits()
                    .take(c as usize)
                    .enumerate()
                    .map(|(k, d)| BigInt::from(*d) * BigInt::from(self.p).pow(k as u32))
                    .sum()
            },
            ZAdicCertainty::ExactPos => {
                self.digits()
                    .enumerate()
                    .map(|(k, d)| BigInt::from(*d) * BigInt::from(self.p).pow(k as u32))
                    .sum()
            },
            ZAdicCertainty::ExactNeg => {
                -(
                    (-self.clone()).into_digits()
                        .enumerate()
                        .map(|(k, d)| BigInt::from(d) * BigInt::from(self.p).pow(k as u32))
                        .sum::<BigInt>()
                )
            }
        }
    }

}


impl AdicInteger for ZAdic {
    fn zero(p: u32) -> Self {
        Self::new_exact(p, vec![])
    }
    fn one(p: u32) -> Self {
        Self::new_exact(p, vec![1])
    }
    fn p(&self) -> u32 {
        self.p
    }
    fn num_digits(&self) -> ZAdicValuation {
        match self.c {
            ZAdicCertainty::Approx(c) => ZAdicValuation::Finite(c),
            ZAdicCertainty::ExactPos => ZAdicValuation::Finite(self.d.len() as u32),
            ZAdicCertainty::ExactNeg => ZAdicValuation::PosInf,
        }
    }
    fn digit(&self, n: u32) -> Result<u32, AdicError> {
        match self.c {
            ZAdicCertainty::Approx(c) => {
                if n < c {
                    Ok(self.d.get(n as usize).copied().unwrap_or(0))
                } else {
                    Err(AdicError::InappropriatePrecision(format!("Cannot retrieve digit {n} past certainty {c}")))
                }
            },
            ZAdicCertainty::ExactPos => Ok(self.d.get(n as usize).copied().unwrap_or(0)),
            ZAdicCertainty::ExactNeg => Ok(self.d.get(n as usize).copied().unwrap_or(self.p-1)),
        }
    }
    fn digits(&self) -> impl Iterator<Item=&u32> {
        // Returns infinite iterator if num_digits PosInf and finite else
        fn inner_iter<'a>(pm1: &'a u32, cert: ZAdicCertainty, digits: &'a [u32]) -> Box<dyn Iterator<Item=&'a u32> + 'a> {
            match cert {
                ZAdicCertainty::ExactPos => Box::new(digits.iter()),
                ZAdicCertainty::ExactNeg => Box::new(digits.iter().chain(repeat(pm1))),
                ZAdicCertainty::Approx(nd) => Box::new(digits.iter().chain(repeat(&0)).take(nd as usize)),
            }
        }
        inner_iter(&self.pm1, self.c, &self.d)
    }
    fn into_digits(self) -> impl Iterator<Item=u32> {
        // Returns infinite iterator if num_digits PosInf and finite else
        fn inner_into_iter(pm1: u32, cert: ZAdicCertainty, digits: Vec<u32>) -> Box<dyn Iterator<Item=u32>> {
            match cert {
                ZAdicCertainty::ExactPos => Box::new(digits.into_iter()),
                ZAdicCertainty::ExactNeg => Box::new(digits.into_iter().chain(repeat(pm1))),
                ZAdicCertainty::Approx(nd) => Box::new(digits.into_iter().chain(repeat(0)).take(nd as usize)),
            }
        }
        inner_into_iter(self.pm1, self.c, self.d)
    }
    fn is_zero(&self) -> bool {
        matches!(self.c, ZAdicCertainty::ExactPos) && self.digits().all(Zero::is_zero)
    }
    fn certainty(&self) -> ZAdicValuation {
        match self.c {
            ZAdicCertainty::Approx(c) => ZAdicValuation::Finite(c),
            ZAdicCertainty::ExactPos | ZAdicCertainty::ExactNeg => ZAdicValuation::PosInf,
        }
    }
}


impl From<UAdic> for ZAdic {
    fn from(u: UAdic) -> Self {
        Self::new_exact(u.p(), u.into_digits_vec())
    }
}


impl std::ops::Add for ZAdic {
    type Output = ZAdic;
    fn add(self, rhs: Self) -> Self::Output {

        assert!(self.p == rhs.p, "{:?}", AdicError::MixedCharacteristic);
        let p = self.p();

        let certainty_valuation = match (self.c, rhs.c) {
            (ZAdicCertainty::Approx(sc), ZAdicCertainty::Approx(rc)) => ZAdicValuation::Finite(std::cmp::min(sc, rc)),
            (ZAdicCertainty::Approx(sc), _) => ZAdicValuation::Finite(sc),
            (_, ZAdicCertainty::Approx(rc)) => ZAdicValuation::Finite(rc),
            _ => ZAdicValuation::PosInf,
        };

        match certainty_valuation {

            // If a finite computation, just use UAdic computation
            ZAdicValuation::Finite(c) => {
                let su = self.into_truncation(c as usize);
                let ru = rhs.into_truncation(c as usize);
                ZAdic::new_approx(
                    p, c, su.add(ru).into_digits().take(c as usize).collect()
                )
            },

            // Otherwise, we need to add carefully, watch for carry, and check signs
            ZAdicValuation::PosInf => {

                let mut out_pos = true;
                let mut new_digits = Vec::with_capacity(std::cmp::max(self.d.len(), rhs.d.len()) + 1);

                let (s_pos, s_trail) = if self.c == ZAdicCertainty::ExactPos { (true, 0) } else { (false, p-1) };
                let (r_pos, r_trail) = if rhs.c == ZAdicCertainty::ExactPos { (true, 0) } else { (false, p-1) };
                let s_iter = self.d.into_iter().map(|d| (false, d)).chain(repeat((true, s_trail)));
                let r_iter = rhs.d.into_iter().map(|d| (false, d)).chain(repeat((true, r_trail)));
                let mut carry = false;
                // Add each pair of digits and manage carry
                for ((s_trailing, sd), (r_trailing, rd)) in s_iter.zip(r_iter) {

                    // If finished with both sets digits, check final carry and possibly wrap up with a last digit
                    if s_trailing && r_trailing {
                        if (!s_pos && !r_pos) {
                            out_pos = false;
                            if !carry {
                                new_digits.push(p-2);
                            }
                        } else if (s_pos && r_pos) {
                            out_pos = true;
                            if carry {
                                new_digits.push(1);
                            }
                        } else {
                            out_pos = carry;
                        }
                        break;
                    }

                    // Add digits together with carry and update carry
                    let new_d = sd + rd + if carry { 1 } else { 0 };
                    if new_d >= p {
                        carry = true;
                        new_digits.push(new_d - p);
                    } else {
                        carry = false;
                        new_digits.push(new_d);
                    }

                }

                // Output positive or negative
                if out_pos {
                    ZAdic::new_exact(p, new_digits)
                } else {
                    ZAdic::new_exact_neg(p, new_digits)
                }

            }
        }

    }
}


impl std::ops::Neg for ZAdic {
    type Output = ZAdic;
    fn neg(self) -> Self::Output {

        let p = self.p();

        if self.is_zero() {
            self
        } else {

            let mut new_digits = Vec::with_capacity(self.d.len());
            let mut old_iter = self.d.into_iter();
            for d in old_iter.by_ref() {
                if d == 0 {
                    new_digits.push(0);
                } else {
                    new_digits.push(p - d);
                    break;
                }
            }
            for d in old_iter.by_ref() {
                new_digits.push(p - d - 1);
            }

            match self.c {
                ZAdicCertainty::Approx(c) => {
                    if new_digits.len() < c as usize {
                        new_digits.extend(repeat(p-1).take(c as usize-new_digits.len()));
                    }
                    Self::new_approx(p, c, new_digits)
                },
                ZAdicCertainty::ExactPos => Self::new_exact_neg(p, new_digits),
                ZAdicCertainty::ExactNeg => {
                    if new_digits.is_empty() {
                        Self::one(p)
                    } else {
                        Self::new_exact(p, new_digits)
                    }
                },
            }

        }

    }
}

impl std::ops::Sub for ZAdic {
    type Output = ZAdic;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}


impl std::ops::Mul for ZAdic {
    type Output = ZAdic;
    fn mul(self, rhs: Self) -> Self::Output {

        assert!(self.p == rhs.p, "{:?}", AdicError::MixedCharacteristic);
        let p = self.p();
        let sc = self.certainty();
        let sv = self.valuation();
        let rc = rhs.certainty();
        let rv = rhs.valuation();

        use ZAdicCertainty::{Approx, ExactPos, ExactNeg};
        match (self.c, rhs.c) {
            (Approx(_), _) | (_, Approx(_)) => {
                let certainty = std::cmp::min(sc + rv, rc + sv);
                match certainty {
                    ZAdicValuation::PosInf => {
                        // This should only happen if self or rhs are zero
                        ZAdic::zero(p)
                    },
                    ZAdicValuation::Finite(c) => {
                        let su = self.into_truncation(c as usize);
                        let ru = rhs.into_truncation(c as usize);
                        ZAdic::new_approx(
                            p, c, su.mul(ru).into_digits().take(c as usize).collect()
                        )
                    },
                }
            },
            (ExactPos, ExactPos) => {
                let su = self.into_truncation_to_uadic().unwrap();
                let ru = rhs.into_truncation_to_uadic().unwrap();
                ZAdic::new_exact(p, su.mul(ru).into_digits_vec())
            },
            (ExactPos, ExactNeg) => {
                let su = self.into_truncation_to_uadic().unwrap();
                let ru = (-rhs).into_truncation_to_uadic().unwrap();
                -ZAdic::new_exact(p, su.mul(ru).into_digits_vec())
            },
            (ExactNeg, ExactPos) => {
                let su = (-self).into_truncation_to_uadic().unwrap();
                let ru = rhs.into_truncation_to_uadic().unwrap();
                -ZAdic::new_exact(p, su.mul(ru).into_digits_vec())
            },
            (ExactNeg, ExactNeg) => {
                let su = (-self).into_truncation_to_uadic().unwrap();
                let ru = (-rhs).into_truncation_to_uadic().unwrap();
                ZAdic::new_exact(p, su.mul(ru).into_digits_vec())
            }
        }

    }
}


impl Pow<u32> for &ZAdic {
    type Output = ZAdic;
    fn pow(self, power: u32) -> Self::Output {
        repeat(
            self.clone()
        ).take(
            power as usize
        ).reduce(
            |acc, e| acc * e
        ).unwrap_or(
            ZAdic::one(self.p)
        )
    }
}


impl fmt::Display for ZAdic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p = self.p;
        match self.c {
            ZAdicCertainty::Approx(c) => {
                // Finite digits
                let digits = self.digits().chain(repeat(&0)).take(c as usize).join("").chars().rev().collect::<String>();
                write!(f, "---{digits}._{p}")
            },
            ZAdicCertainty::ExactPos => {
                // "Infinite" digits, but ZAdic's d is still finite
                if self.d.is_empty() {
                    write!(f, "0._{p}")
                } else {
                    let digits = self.d.iter().join("").chars().rev().collect::<String>();
                    write!(f, "{digits}._{p}")
                }
            },
            ZAdicCertainty::ExactNeg => {
                // "Infinite" digits, but ZAdic's d is still finite
                let digits = self.d.iter().join("").chars().chain("44".chars()).rev().collect::<String>();
                write!(f, "...{digits}._{p}")
            },
        }
    }
}


#[cfg(test)]
mod tests {

    use num::traits::Pow;
    use super::{AdicInteger, ZAdic, ZAdicValuation};
    use crate::{zadic_approx, zadic_exact, zadic_exact_neg, AdicError, ZAdicVariety};
    use ZAdicValuation::Finite;

    // Exact numbers
    fn zero_e() -> ZAdic { zadic_exact!(5, []) }
    fn one_e() -> ZAdic { zadic_exact!(5, [1]) }
    fn two_e() -> ZAdic { zadic_exact!(5, [2]) }
    fn three_e() -> ZAdic { zadic_exact!(5, [3]) }
    fn four_e() -> ZAdic { zadic_exact!(5, [4]) }
    fn five_e() -> ZAdic { zadic_exact!(5, [0, 1]) }
    fn six_e() -> ZAdic { zadic_exact!(5, [1, 1]) }
    fn eight_e() -> ZAdic { zadic_exact!(5, [3, 1]) }
    fn ten_e() -> ZAdic { zadic_exact!(5, [0, 2]) }
    fn twenty_five_e() -> ZAdic { zadic_exact!(5, [0, 0, 1]) }
    fn neg_one_e() -> ZAdic { zadic_exact_neg!(5, []) }
    fn neg_two_e() -> ZAdic { zadic_exact_neg!(5, [3]) }
    fn neg_five_e() -> ZAdic { zadic_exact_neg!(5, [0]) }

    // Numbers with 4-digit certainty
    fn zero_4() -> ZAdic { zadic_approx!(5, 4, []) }
    fn one_4() -> ZAdic { zadic_approx!(5, 4, [1]) }
    fn three_4() -> ZAdic { zadic_approx!(5, 4, [3]) }
    fn four_4() -> ZAdic { zadic_approx!(5, 4, [4]) }
    fn five_4() -> ZAdic { zadic_approx!(5, 4, [0, 1]) }
    fn six_4() -> ZAdic { zadic_approx!(5, 4, [1, 1]) }
    fn twenty_five_4() -> ZAdic { zadic_approx!(5, 4, [0, 0, 1]) }
    fn neg_one_4() -> ZAdic { zadic_approx!(5, 4, [4, 4, 4, 4]) }
    fn neg_two_4() -> ZAdic { zadic_approx!(5, 4, [3, 4, 4, 4]) }
    fn neg_three_4() -> ZAdic { zadic_approx!(5, 4, [2, 4, 4, 4]) }
    fn neg_five_4() -> ZAdic { zadic_approx!(5, 4, [0, 4, 4, 4]) }
    fn neg_ten_4() -> ZAdic { zadic_approx!(5, 4, [0, 3, 4, 4]) }
    fn sqrt_2_7_adic() -> ZAdic { zadic_approx!(7, 4, [3, 1, 2, 6]) }
    fn sqrt_2_7_adic2() -> ZAdic { zadic_approx!(7, 4, [4, 5, 4, 0]) }

    #[test]
    fn test_add_z_adic() {
        assert_eq!(two_e(), one_e() + one_e());
        assert_eq!(three_e(), two_e() + one_e());
        assert_eq!(five_e(), two_e() + three_e());
        let neg_one_plus_neg_one = neg_one_4() + neg_one_4();
        assert_eq!(neg_two_4(), neg_one_plus_neg_one);
        let neg_two_plus_neg_three = neg_two_4() + neg_three_4();
        assert_eq!(neg_five_4(), neg_two_plus_neg_three);
        let neg_five_plus_neg_five = neg_five_4() + neg_five_4();
        assert_eq!(neg_ten_4(), neg_five_plus_neg_five);
        let two_plus_neg_two = two_e() + neg_two_4();
        assert_eq!(zero_4(), two_plus_neg_two);
        let four_plus_one_does_not_grow = zadic_approx!(5, 1, [4]) + zadic_exact!(5, [1]);
        assert_eq!(zadic_approx!(5, 1, [0]), four_plus_one_does_not_grow);
        assert_eq!(twenty_five_e().certainty(), ZAdicValuation::PosInf);
        assert_eq!(twenty_five_4().certainty(), ZAdicValuation::Finite(4));
    }

    #[test]
    fn test_neg_z_adic() {
        assert_eq!(neg_one_e(), -one_e());
        assert_eq!(one_e(), -neg_one_e());
        assert_eq!(zero_e(), -zero_e());
        assert_eq!(neg_five_e(), -five_e());
        assert_eq!(neg_three_4(), -three_4());
        assert_eq!(neg_five_4(), -five_4());
        assert_eq!(sqrt_2_7_adic2(), -sqrt_2_7_adic());
    }

    #[test]
    fn test_sub_z_adic() {
        assert_eq!(one_e(), two_e() - one_e());
        assert_eq!(zero_e(), one_e() - one_e());
        assert_eq!(neg_one_e(), one_e() - two_e());
        assert_eq!(neg_five_e(), one_e() - six_e());
        assert_eq!(one_e(), neg_one_e() - neg_two_e());
        assert_eq!(one_4(), neg_one_4() - neg_two_4());
    }

    #[test]
    fn test_mul_z_adic() {
        assert_eq!(one_e(), one_e() * one_e());
        assert_eq!(two_e(), two_e() * one_e());
        assert_eq!(six_e(), two_e() * three_e());
        let neg_one_mul_neg_one = neg_one_4() * neg_one_4();
        assert_eq!(one_4(), neg_one_mul_neg_one);
        let neg_two_mul_neg_three = neg_two_4() * neg_three_4();
        assert_eq!(six_4(), neg_two_mul_neg_three);
        assert_eq!(zero_e(), zero_e() * two_e());
        assert_eq!(zero_e(), zero_e() * neg_two_4());
        assert_eq!(ten_e(), five_e() * two_e());
        assert_eq!(twenty_five_e(), five_e() * five_e());
        assert_eq!(one_e(), neg_one_e() * neg_one_e());
    }

    #[test]
    fn test_pow_z_adic() {
        assert_eq!(zero_e(), zero_e().pow(2));
        assert_eq!(zero_e(), zero_e().pow(3));
        assert_eq!(one_e(), one_e().pow(2));
        assert_eq!(one_e(), one_e().pow(3));
        assert_eq!(four_e(), two_e().pow(2));
        assert_eq!(eight_e(), two_e().pow(3));
        assert_eq!(twenty_five_e(), five_e().pow(2));
        assert_eq!(one_e(), neg_two_4().pow(0));
        assert_eq!(neg_one_4(), neg_one_4().pow(1));
        assert_eq!(one_4(), neg_one_4().pow(2));
        assert_eq!(four_4(), neg_two_4().pow(2));
        let twenty_five_5 = zadic_approx!(5, 5, [0, 0, 1]);
        assert_eq!(twenty_five_5, neg_five_4().pow(2));
    }

    #[test]
    #[should_panic]
    fn test_non_prime() {
        let _ = zadic_approx!(6, 3, [2]);
    }

    #[test]
    #[should_panic]
    fn test_mixed_characteristic() {
        let _ = zadic_approx!(5, 3, [1]) + zadic_approx!(7, 3, [1]);
    }

    #[test]
    fn test_approximate_z_adic() {

        let zero_4 = zadic_approx!(5, 4, [0, 0, 0, 0]);
        assert!(!zero_4.is_zero());
        let one_2 = zadic_approx!(5, 2, [1]);
        let two_4 = zadic_approx!(5, 4, [2]);
        let five_3 = zadic_approx!(5, 3, [0, 1]);

        assert_eq!(Finite(2), (one_2.clone() + one_2.clone()).certainty());
        assert_eq!(Finite(2), (one_2.clone() + two_4.clone()).certainty());
        assert_eq!(Finite(4), (two_4.clone() + two_4.clone()).certainty());
        assert_eq!(Finite(2), (one_2.clone() + five_3.clone()).certainty());
        assert_eq!(Finite(3), (two_4.clone() + five_3.clone()).certainty());

        assert_eq!(Finite(2), (one_2.clone() * one_2.clone()).certainty());
        assert_eq!(Finite(2), (one_2.clone() * two_4.clone()).certainty());
        assert_eq!(Finite(4), (two_4.clone() * two_4.clone()).certainty());
        assert_eq!(Finite(3), (one_2.clone() * five_3.clone()).certainty());
        assert_eq!(Finite(3), (two_4.clone() * five_3.clone()).certainty());
        assert_eq!(Finite(4), (five_3.clone() * five_3.clone()).certainty());

    }

    #[test]
    fn test_nth_root() {

        let check = |p: u32, a: &ZAdic, n: u32, precision: u32, roots: Vec<ZAdic>| {
            // Check each root powers to match a to at least precision digits
            for root in &roots {
                assert_eq!(a.truncation(precision as usize), root.pow(n).into_truncation_to_uadic().unwrap());
            }
            // Check roots match the output of nth_root
            assert_eq!(Ok(ZAdicVariety::new(p, roots)), a.nth_root(n, precision));
        };

        check(5, &zadic_exact!(5, [1]), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);
        check(5, &zadic_approx!(5, 12, [1]), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);

        check(5, &zadic_exact!(5, [2]), 2, 6, vec![]);
        check(5, &zadic_approx!(5, 12, [2]), 2, 6, vec![]);

        check(7, &zadic_exact!(7, [2]), 2, 6, vec![
            zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2]),
            zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4]),
        ]);
        check(7, &zadic_approx!(7, 12, [2]), 2, 6, vec![
            zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2]),
            zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4]),
        ]);

        assert!(matches!(
            zadic_approx!(7, 4, [2]).nth_root(2, 6),
            Err(AdicError::InappropriatePrecision(_))
        ));

    }

    #[test]
    fn test_display() {

        assert_eq!("0._5", zero_e().to_string());
        assert_eq!("1._5", one_e().to_string());
        assert_eq!("11._5", six_e().to_string());

        assert_eq!("23._5", zadic_exact!(5, [3, 2, 0, 0]).to_string());

        assert_eq!("---0000._5", zero_4().to_string());
        assert_eq!("---0001._5", one_4().to_string());
        assert_eq!("---6213._7", sqrt_2_7_adic().to_string());

    }

}
