use std::{
    fmt,
    iter::{once, repeat},
};
use itertools::Itertools;
use num::{traits::Pow, BigInt, Zero};
use num_prime::nt_funcs::is_prime;
use crate::AdicError;
use super::{AdicInteger, AdicSign, UAdic, ZAdicValuation};


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
/// assert_eq!(2, two.integer_value());
/// let five = IAdic::new_pos(5, vec![0, 1]);
/// assert_eq!(7, (two.clone() + five.clone()).integer_value());
/// assert_eq!(10, (two.clone() * five.clone()).integer_value());
/// ```
///
/// This representation EXACTLY matches the real number base-p digits for a real integer.
/// `2 = 2._5, 123 = 123._5`
/// You can perform the same arithmetic on these numbers.
/// However, as a signed number, we can also represent negative numbers, subtracting but not dividing.
/// Instead, look to rationals, [`RAdic`](crate::RAdic), or if they can be approximate, [`ZAdic`](crate::ZAdic).
///
/// Many calculations truncate `AdicInteger`s to `IAdic`s in order to perform simple calculations.
pub struct IAdic {
    /// Adic prime
    p: u32,
    /// p-1
    pm1: u32,
    /// Adic digits, each 0 to p-1
    d: Vec<u32>,
    /// Positive (trailing zeros) or Negative (trailing p-1)
    sign: AdicSign,
}


impl IAdic {

    /// Create an adic number with the given digits and sign
    ///
    /// # Panics
    /// Panics if `p` is not prime
    pub fn new(p: u32, mut init_digits: Vec<u32>, sign: AdicSign) -> Self {

        assert!(is_prime(&p, None).probably());

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
    /// Panics if `p` is not prime
    pub fn new_pos(p: u32, init_digits: Vec<u32>) -> Self {
        Self::new(p, init_digits, AdicSign::Pos)
    }

    /// Create a negative adic number with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime
    pub fn new_neg(p: u32, init_digits: Vec<u32>) -> Self {
        Self::new(p, init_digits, AdicSign::Neg)
    }

    /// Create adic number associated with (signed) integer n
    pub fn from_integer(p: u32, n: i32) -> Self {
        if n >= 0 {
            Self::new_pos(p, UAdic::from_integer(p, n as u32).into_digits_vec())
        } else {
            -Self::new_pos(p, UAdic::from_integer(p, (-n) as u32).into_digits_vec())
        }
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
    pub fn num_non_trailing(&self) -> u32 {
        self.d.len() as u32
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
    /// Warning: This can overflow; use [`UAdic::big_integer_value`] if unsure
    ///
    /// ```
    /// # use adic::uadic;
    /// assert_eq!(38, uadic!(5, [3, 2, 1]).integer_value());
    /// ```
    pub fn integer_value(&self) -> i32 {
        (self.abs().integer_value() as i32) * i32::from(self.sgn())
    }

    /// The bigint representation for the natural number value of the number
    ///
    /// ```
    /// # use num::BigInt;
    /// # use adic::uadic;
    /// assert_eq!(BigInt::from(38), uadic!(5, [3, 2, 1]).big_integer_value());
    /// ```
    pub fn big_integer_value(&self) -> BigInt {
        self.abs().big_integer_value() * i32::from(self.sgn())
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
            AdicSign::Pos => ZAdicValuation::Finite(self.d.len() as u32),
            AdicSign::Neg => ZAdicValuation::PosInf,
        }
    }
    fn digit(&self, n: u32) -> Result<u32, AdicError> {
        Ok(self.d.get(n as usize).copied().unwrap_or(0))
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
    fn certainty(&self) -> ZAdicValuation {
        ZAdicValuation::PosInf
    }
}


impl From<UAdic> for IAdic {
    fn from(a: UAdic) -> Self {
        IAdic::new_pos(a.p(), a.into_digits_vec())
    }
}


impl TryFrom<IAdic> for UAdic {
    type Error = AdicError;
    fn try_from(a: IAdic) -> Result<Self, Self::Error> {
        match a.sgn() {
            AdicSign::Pos => Ok(UAdic::new(a.p(), a.into_digits().collect())),
            AdicSign::Neg => Err(AdicError::BadConversion),
        }
    }
}


impl std::ops::Add for IAdic {
    type Output = IAdic;
    fn add(self, rhs: Self) -> Self::Output {

        assert!(self.p == rhs.p, "{:?}", AdicError::MixedCharacteristic);
        let p = self.p();

        let mut out_pos = true;
        let mut new_digits = Vec::with_capacity(std::cmp::max(self.d.len(), rhs.d.len()) + 1);

        let (s_pos, s_trail) = if matches!(self.sign, AdicSign::Pos) { (true, 0) } else { (false, p-1) };
        let (r_pos, r_trail) = if matches!(rhs.sign, AdicSign::Pos) { (true, 0) } else { (false, p-1) };
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
            IAdic::new_pos(p, new_digits)
        } else {
            IAdic::new_neg(p, new_digits)
        }

    }
}



impl std::ops::Neg for IAdic {
    type Output = IAdic;
    fn neg(self) -> Self::Output {

        let p = self.p();

        if self.is_zero() {
            self
        } else {

            let mut new_digits = Vec::with_capacity(self.d.len() + 1);
            let mut old_iter = self.d.into_iter().chain(once(self.sign.mod_p(p)));
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

            match self.sign {
                AdicSign::Pos => Self::new_neg(p, new_digits),
                AdicSign::Neg => {
                    Self::new_pos(p, new_digits)
                },
            }

        }

    }
}

impl std::ops::Sub for IAdic {
    type Output = IAdic;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}


impl std::ops::Mul for IAdic {
    type Output = IAdic;
    fn mul(self, rhs: Self) -> Self::Output {

        assert!(self.p == rhs.p, "{:?}", AdicError::MixedCharacteristic);
        let p = self.p();

        if self.is_zero() || rhs.is_zero() {
            IAdic::zero(p)
        } else {

            let new_sign = self.sign * rhs.sign;
            let su = self.into_abs();
            let ru = rhs.into_abs();
            match new_sign {
                AdicSign::Pos => IAdic::new_pos(p, su.mul(ru).into_digits_vec()),
                AdicSign::Neg => -IAdic::new_pos(p, su.mul(ru).into_digits_vec()),
            }

        }

    }
}


impl Pow<u32> for &IAdic {
    type Output = IAdic;
    fn pow(self, power: u32) -> Self::Output {
        repeat(
            self.clone()
        ).take(
            power as usize
        ).reduce(
            |acc, e| acc * e
        ).unwrap_or(
            IAdic::one(self.p)
        )
    }
}


impl fmt::Display for IAdic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p = self.p;
        match self.sign {
            AdicSign::Pos => {
                // Finite digits
                if self.d.is_empty() {
                    write!(f, "0._{p}")
                } else {
                    let digits = self.d.iter().join("").chars().rev().collect::<String>();
                    write!(f, "{digits}._{p}")
                }
            },
            AdicSign::Neg => {
                // "Infinite" digits, show (p-1) and then the finite part
                let pm1_symbol = self.pm1.to_string();
                let digits = self.d.iter().join("").chars().rev().collect::<String>();
                write!(f, "({pm1_symbol}){digits}._{p}")
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use itertools::{Itertools, repeat_n};
    use num::{traits::Pow, Rational32};
    use crate::{iadic_pos, iadic_neg, uadic, zadic_approx, ZAdic, ZAdicValuation, ZAdicVariety};
    use super::{AdicInteger, IAdic};

    fn zero() -> IAdic { iadic_pos!(5, []) }
    fn one() -> IAdic { iadic_pos!(5, [1]) }
    fn two() -> IAdic { iadic_pos!(5, [2]) }
    fn three() -> IAdic { iadic_pos!(5, [3]) }
    fn four() -> IAdic { iadic_pos!(5, [4]) }
    fn five() -> IAdic { iadic_pos!(5, [0, 1]) }
    fn six() -> IAdic { iadic_pos!(5, [1, 1]) }
    fn eight() -> IAdic { iadic_pos!(5, [3, 1]) }
    fn ten() -> IAdic { iadic_pos!(5, [0, 2]) }
    fn twenty_four() -> IAdic { iadic_pos!(5, [4, 4]) }
    fn twenty_five() -> IAdic { iadic_pos!(5, [0, 0, 1]) }
    fn twelve() -> IAdic { iadic_pos!(5, [2, 2]) }
    fn one_twenty_five() -> IAdic { iadic_pos!(5, [0, 0, 0, 1]) }
    fn one_twenty_six() -> IAdic { iadic_pos!(5, [1, 0, 0, 1]) }
    fn one_fifty_six() -> IAdic { iadic_pos!(5, [1, 1, 1, 1]) }
    fn six_twenty_four() -> IAdic { iadic_pos!(5, [4, 4, 4, 4]) }
    fn neg_one() -> IAdic { iadic_neg!(5, []) }
    fn neg_two() -> IAdic { iadic_neg!(5, [3]) }
    fn neg_three() -> IAdic { iadic_neg!(5, [2]) }
    fn _neg_four() -> IAdic { iadic_neg!(5, [1]) }
    fn neg_five() -> IAdic { iadic_neg!(5, [0]) }
    fn neg_six() -> IAdic { iadic_neg!(5, [4, 3]) }
    fn neg_ten() -> IAdic { iadic_neg!(5, [0, 3]) }
    fn neg_twenty_five() -> IAdic { iadic_neg!(5, [0, 0] )}
    fn neg_one_twenty_five() -> IAdic { iadic_neg!(5, [0, 0, 0]) }
    fn neg_one_twenty_six() -> IAdic { iadic_neg!(5, [4, 4, 4, 3]) }


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
    fn test_add_i_adic() {
        assert_eq!(two(), one() + one());
        assert_eq!(three(), two() + one());
        assert_eq!(five(), two() + three());
        let neg_one_plus_neg_one = neg_one() + neg_one();
        assert_eq!(neg_two(), neg_one_plus_neg_one);
        let neg_two_plus_neg_three = neg_two() + neg_three();
        assert_eq!(neg_five(), neg_two_plus_neg_three);
        let neg_five_plus_neg_five = neg_five() + neg_five();
        assert_eq!(neg_ten(), neg_five_plus_neg_five);
        let two_plus_neg_two = two() + neg_two();
        assert_eq!(zero(), two_plus_neg_two);
    }

    #[test]
    fn test_mul_i_adic() {
        assert_eq!(zero(), zero() * one());
        assert_eq!(zero(), zero() * neg_one());
        assert_eq!(one(), one() * one());
        assert_eq!(two(), two() * one());
        assert_eq!(six(), two() * three());
        let neg_one_mul_neg_one = neg_one() * neg_one();
        assert_eq!(one(), neg_one_mul_neg_one);
        let neg_two_mul_neg_three = neg_two() * neg_three();
        assert_eq!(six(), neg_two_mul_neg_three);
        assert_eq!(zero(), zero() * two());
        assert_eq!(zero(), zero() * neg_two());
        assert_eq!(ten(), five() * two());
        assert_eq!(twenty_five(), five() * five());
        assert_eq!(neg_one(), one() * neg_one());
        assert_eq!(neg_one(), neg_one() * one());
        assert_eq!(one(), neg_one() * neg_one());
        assert_eq!(neg_one(), neg_one() * neg_one() * neg_one());
        assert_eq!(neg_two(), neg_one() * two());
        assert_eq!(neg_two(), neg_two() * one());
        assert_eq!(neg_ten(), neg_two() * five());
        assert_eq!(neg_ten(), neg_five() * two());
    }

    #[test]
    fn test_pow_i_adic() {
        assert_eq!(zero(), zero().pow(2));
        assert_eq!(zero(), zero().pow(3));
        assert_eq!(one(), one().pow(2));
        assert_eq!(one(), one().pow(3));
        assert_eq!(four(), two().pow(2));
        assert_eq!(eight(), two().pow(3));
        assert_eq!(twenty_five(), five().pow(2));
        assert_eq!(one(), neg_two().pow(0));
        assert_eq!(neg_one(), neg_one().pow(1));
        assert_eq!(one(), neg_one().pow(2));
        assert_eq!(neg_one(), neg_one().pow(3));
        assert_eq!(four(), neg_two().pow(2));
    }

    #[test]
    fn test_i_adic_ops_many() {
        // Test addition and multiplication over many integers using integer_value
        let p = 5;
        let n1 = 2;
        let n2 = 2;
        let firsts = repeat_n(0..p, n1).multi_cartesian_product().flat_map(
            |digits| [IAdic::new_pos(p, digits[0..n1].to_vec()), IAdic::new_neg(p, digits[0..n1].to_vec())]
        );
        let seconds = repeat_n(0..p, n2).multi_cartesian_product().flat_map(
            |digits| [IAdic::new_pos(p, digits[0..n2].to_vec()), IAdic::new_neg(p, digits[0..n2].to_vec())]
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
        let _ = iadic_pos!(6, [2]);
    }

    #[test]
    #[should_panic]
    fn test_mixed_characteristic() {
        let _ = iadic_pos!(5, [1]) + iadic_neg!(7, [1]);
    }

    #[test]
    fn test_integer_value() {
        assert_eq!(1, iadic_pos!(5, [1]).integer_value());
        assert_eq!(2, iadic_pos!(5, [2]).integer_value());
        assert_eq!(6, iadic_pos!(5, [1, 1]).integer_value());
        assert_eq!(126, iadic_pos!(5, [1, 0, 0, 1]).integer_value());
        assert_eq!(124, iadic_pos!(5, [4, 4, 4]).integer_value());
    }

    #[test]
    fn test_i_adic_norm() {
        assert_eq!(ZAdicValuation::PosInf, zero().valuation());
        assert_eq!(Rational32::ZERO, zero().norm());
        assert_eq!(ZAdicValuation::Finite(0), neg_one().valuation());
        assert_eq!(Rational32::new(1, 1), neg_one().norm());
        assert_eq!(ZAdicValuation::Finite(0), neg_two().valuation());
        assert_eq!(Rational32::new(1, 1), neg_two().norm());
        assert_eq!(ZAdicValuation::Finite(1), neg_five().valuation());
        assert_eq!(Rational32::new(1, 5), neg_five().norm());
        assert_eq!(ZAdicValuation::Finite(0), neg_six().valuation());
        assert_eq!(Rational32::new(1, 1), neg_six().norm());
        assert_eq!(ZAdicValuation::Finite(2), neg_twenty_five().valuation());
        assert_eq!(Rational32::new(1, 25), neg_twenty_five().norm());
        assert_eq!(ZAdicValuation::Finite(3), neg_one_twenty_five().valuation());
        assert_eq!(Rational32::new(1, 125), neg_one_twenty_five().norm());
        assert_eq!(ZAdicValuation::Finite(0), neg_one_twenty_six().valuation());
        assert_eq!(Rational32::new(1, 1), neg_one_twenty_six().norm());
    }

    #[test]
    fn test_nth_root() {

        let check = |p: u32, a: &IAdic, n: u32, precision: u32, roots: Vec<ZAdic>| {
            // Check each root powers to match a to at least precision digits
            for root in &roots {
                assert_eq!(a.truncation(precision as usize), root.pow(n).into_truncation_to_uadic().unwrap());
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
        assert_eq!("(4)._5", neg_one().to_string());
        assert_eq!("(4)3._5", neg_two().to_string());
        assert_eq!("(4)2._5", neg_three().to_string());
        assert_eq!("(4)0._5", neg_five().to_string());
        assert_eq!("(4)34._5", neg_six().to_string());
        assert_eq!("(4)30._5", neg_ten().to_string());
        assert_eq!("(4)00._5", neg_twenty_five().to_string());
        assert_eq!("(4)000._5", neg_one_twenty_five().to_string());
        assert_eq!("(4)3444._5", neg_one_twenty_six().to_string());
    }

}
