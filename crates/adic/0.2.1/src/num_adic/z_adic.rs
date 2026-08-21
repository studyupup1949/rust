use std::{fmt, iter::{once, repeat}};
use itertools::Itertools;
use num::{traits::Pow, BigInt};
use num_prime::nt_funcs::is_prime;
use crate::AdicError;
use super::{AdicInteger, AdicSign, IAdic, UAdic, ZAdicValuation};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Approximate Adic Integer, represented by a partially-known digital expansion
/// ([`zadic_approx`](crate::zadic_approx), [`zadic_exact_pos`](crate::zadic_exact_pos), [`zadic_exact_neg`](crate::zadic_exact_neg))
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
/// assert_eq!("...002341._5", ZAdic::new_approx(5, 6, vec![1, 4, 3, 2]).to_string());
/// assert_eq!("2341._5", ZAdic::new_exact(5, vec![1, 4, 3, 2]).to_string());
/// ```
///
/// Adding and multiplying `ZAdic`s respects the certainty.
/// When adding, the output certainty is the minimum of the input certainties:
///  `...abc._p + ...de._p = ...fg._p`.
/// When multiplying, the output certainty is a little more complicated,
///  since zero digits can make things more certain than just the minumum:
///  `...ab0._p * ...de._p = ...fg0._p`.
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
/// assert_eq!("(4)._5", neg_one_e.to_string());
/// assert!((one_e + neg_one_e).is_zero());
/// ```
///
/// In this way, the exact `ZAdic` is as flexible as [`IAdic`](crate::IAdic),
/// able to represent all real integers.
pub struct ZAdic {
    /// Adic prime
    p: u32,
    /// Valuation of certainty; number of digits that are known
    variant: ZAdicVariant,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `ZAdic` can either be approximate or exact.
/// This distinction is held in this enum.
/// Approx holds a `UAdic` and a finite certainty, with the `UAdic` number of digits <= the certainty.
/// Exact holds an `IAdic` and generally defers to that struct for calculations.
enum ZAdicVariant {
    Approx((u32, UAdic)),
    Exact(IAdic),
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

        Self {
            p,
            variant: ZAdicVariant::Approx((certainty, UAdic::new(p, init_digits)))
        }

    }

    /// Create an exact adic number with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime
    pub fn new_exact(p: u32, init_digits: Vec<u32>) -> Self {

        assert!(is_prime(&p, None).probably());

        Self {
            p,
            variant: ZAdicVariant::Exact(IAdic::new_pos(p, init_digits))
        }

    }

    /// Create an exact negative adic number (trailing p-1) with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime
    pub fn new_exact_neg(p: u32, init_digits: Vec<u32>) -> Self {

        assert!(is_prime(&p, None).probably());

        Self {
            p,
            variant: ZAdicVariant::Exact(IAdic::new_neg(p, init_digits))
        }

    }

    /// Create an exact adic number that corresponds to the given integer
    pub fn exact_from_integer(p: u32, n: i32) -> Self {
        let int = IAdic::from_integer(p, n);
        match int.sgn() {
            AdicSign::Pos => Self::new_exact(p, int.into_abs().into_digits_vec()),
            AdicSign::Neg => -Self::new_exact(p, int.into_abs().into_digits_vec()),
        }
    }

    /// Push another cerain digit onto the end of the number
    ///
    /// # Errors
    /// Returns error if inappropriate precision or if number already has infinite certainty
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
        match &mut self.variant {
            ZAdicVariant::Approx(var) => {
                if ZAdicValuation::Finite(var.0) < var.1.num_digits() {
                    Err(AdicError::InappropriatePrecision(
                        "Certainty less than digits size; should not happen!".to_string()
                    ))
                } else {
                    match var.1.num_digits() {
                        ZAdicValuation::PosInf => Err(
                            AdicError::InappropriatePrecision("Infinite digits; should not happen!".to_string())
                        ),
                        ZAdicValuation::Finite(digits_len) => {
                            let num_leading_zeros = var.0 - digits_len;
                            var.0 += 1;
                            var.1.extend_digits(repeat(0).take(num_leading_zeros as usize).chain(once(digit)));
                            Ok(())
                        },
                    }
                }
            },
            ZAdicVariant::Exact(_) => Err(AdicError::InappropriatePrecision(
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
        match (c, &mut self.variant) {
            (ZAdicValuation::Finite(c), ZAdicVariant::Approx(var)) => {
                var.0 = c;
                var.1 = var.1.truncation(c as usize);
            },
            (ZAdicValuation::Finite(c), ZAdicVariant::Exact(var)) => {
                self.variant = ZAdicVariant::Approx((c, var.truncation(c as usize)));
            },
            (ZAdicValuation::PosInf, ZAdicVariant::Approx(var)) => {
                // Assume positive number
                self.variant = ZAdicVariant::Exact(IAdic::new_pos(self.p, var.1.clone().into_digits_vec()));
            },
            (ZAdicValuation::PosInf, ZAdicVariant::Exact(_var)) => { },
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
        match &self.variant {
            ZAdicVariant::Approx((_, u)) => u.integer_value() as i32,
            ZAdicVariant::Exact(i) => i.integer_value(),
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
        match &self.variant {
            ZAdicVariant::Approx((_, u)) => u.big_integer_value(),
            ZAdicVariant::Exact(i) => i.big_integer_value(),
        }
    }

}


impl From<UAdic> for ZAdic {
    fn from(a: UAdic) -> Self {
        Self::new_exact(a.p(), a.into_digits_vec())
    }
}

impl From<IAdic> for ZAdic {
    fn from(a: IAdic) -> Self {
        let p = a.p();
        let sgn = a.sgn();
        let num_non_trailing = a.num_non_trailing() as usize;
        let digits = a.into_digits().take(num_non_trailing).collect::<Vec<_>>();
        match sgn {
            AdicSign::Pos => Self::new_exact(p, digits),
            AdicSign::Neg => Self::new_exact_neg(p, digits),
        }
    }
}

impl TryFrom<ZAdic> for UAdic {
    type Error = AdicError;
    fn try_from(a: ZAdic) -> Result<Self, Self::Error> {
        if a.has_finite_digits() {
            Ok(Self::new(a.p(), a.into_digits().collect()))
        } else {
            Err(AdicError::BadConversion)
        }
    }
}

impl TryFrom<ZAdic> for IAdic {
    type Error = AdicError;
    fn try_from(a: ZAdic) -> Result<Self, Self::Error> {
        match a.variant {
            ZAdicVariant::Approx(_) => Err(AdicError::BadConversion),
            ZAdicVariant::Exact(var) => Ok(var),
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
        match &self.variant {
            ZAdicVariant::Approx((c, _)) => ZAdicValuation::Finite(*c),
            ZAdicVariant::Exact(i) => i.num_digits(),
        }
    }
    fn digit(&self, n: u32) -> Result<u32, AdicError> {
        match &self.variant {
            ZAdicVariant::Approx((c, u)) => {
                if n < *c {
                    u.digit(n).or(Ok(0))
                } else {
                    Err(AdicError::InappropriatePrecision(format!("Cannot retrieve digit {n} past certainty {c}")))
                }
            },
            ZAdicVariant::Exact(i) => {
                i.digit(n)
            },
        }
    }
    fn digits(&self) -> impl Iterator<Item=&u32> {
        // Returns infinite iterator if num_digits PosInf and finite else
        fn inner_iter<'a>(variant: &'a ZAdicVariant) -> Box<dyn Iterator<Item=&'a u32> + 'a> {
            match variant {
                ZAdicVariant::Approx((c, u)) => Box::new(u.digits().chain(repeat(&0)).take(*c as usize)),
                ZAdicVariant::Exact(i) => Box::new(i.digits()),
            }
        }
        inner_iter(&self.variant)
    }
    fn into_digits(self) -> impl Iterator<Item=u32> {
        // Returns infinite iterator if num_digits PosInf and finite else
        fn inner_into_iter(variant: ZAdicVariant) -> Box<dyn Iterator<Item=u32>> {
            match variant {
                ZAdicVariant::Approx((c, u)) => Box::new(u.into_digits().chain(repeat(0)).take(c as usize)),
                ZAdicVariant::Exact(i) => Box::new(i.into_digits()),
            }
        }
        inner_into_iter(self.variant)
    }
    fn is_zero(&self) -> bool {
        if let ZAdicVariant::Exact(i) = &self.variant {
            i.is_zero()
        } else {
            false
        }
    }
    fn certainty(&self) -> ZAdicValuation {
        match &self.variant {
            ZAdicVariant::Approx(c) => ZAdicValuation::Finite(c.0),
            ZAdicVariant::Exact(_) => ZAdicValuation::PosInf,
        }
    }
}


impl std::ops::Add for ZAdic {
    type Output = ZAdic;
    fn add(self, rhs: Self) -> Self::Output {

        assert!(self.p == rhs.p, "{:?}", AdicError::MixedCharacteristic);
        let p = self.p();

        match (self.variant, rhs.variant) {
            (ZAdicVariant::Approx((sc, su)), ZAdicVariant::Approx((rc, ru))) => {
                let c = std::cmp::min(sc, rc);
                let su = su.into_truncation(c as usize);
                let ru = ru.into_truncation(c as usize);
                ZAdic::new_approx(
                    p, c, su.add(ru).into_digits().take(c as usize).collect()
                )
            },
            (ZAdicVariant::Exact(si), ZAdicVariant::Approx((rc, ru))) => {
                let su = si.into_truncation(rc as usize);
                let ru = ru.into_truncation(rc as usize);
                ZAdic::new_approx(
                    p, rc, su.add(ru).into_digits().take(rc as usize).collect()
                )
            },
            (ZAdicVariant::Approx((sc, su)), ZAdicVariant::Exact(ri)) => {
                let su = su.into_truncation(sc as usize);
                let ru = ri.into_truncation(sc as usize);
                ZAdic::new_approx(
                    p, sc, su.add(ru).into_digits().take(sc as usize).collect()
                )
            },
            (ZAdicVariant::Exact(si), ZAdicVariant::Exact(ri)) => {
                let i = si + ri;
                match i.sgn() {
                    AdicSign::Pos => ZAdic::new_exact(p, i.into_digits().collect()),
                    AdicSign::Neg => {
                        let non_trailing = i.num_non_trailing() as usize;
                        ZAdic::new_exact_neg(p, i.into_digits().take(non_trailing).collect())
                    }
                }
            },
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

            match self.variant {
                ZAdicVariant::Approx((c, u)) => {
                    ZAdic::new_approx(p, c, (-IAdic::from(u)).truncation(c as usize).into_digits_vec())
                },
                ZAdicVariant::Exact(i) => {
                    match i.sgn() {
                        AdicSign::Pos => {
                            let num = -i;
                            let non_trailing = num.num_non_trailing() as usize;
                            ZAdic::new_exact_neg(p, num.into_digits().take(non_trailing).collect())
                        },
                        AdicSign::Neg => ZAdic::new_exact(p, (-i).into_digits().collect()),
                    }
                }
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

        fn approx_mult<AI1, AI2>(p: u32, c: ZAdicValuation, s: AI1, r: AI2) -> ZAdic
        where AI1: AdicInteger, AI2: AdicInteger {
            if let ZAdicValuation::Finite(cert) = c {
                let su = s.into_truncation(cert as usize);
                let ru = r.into_truncation(cert as usize);
                ZAdic::new_approx(
                    p, cert, su.mul(ru).into_digits().take(cert as usize).collect()
                )
            } else {
                // This should only happen if self or rhs are zero
                ZAdic::zero(p)
            }
        }

        assert!(self.p == rhs.p, "{:?}", AdicError::MixedCharacteristic);
        let p = self.p();
        let sc = self.certainty();
        let sv = self.valuation();
        let rc = rhs.certainty();
        let rv = rhs.valuation();

        let c = std::cmp::min(sc + rv, rc + sv);
        match (self.variant, rhs.variant) {
            (ZAdicVariant::Approx((_, su)), ZAdicVariant::Approx((_, ru))) => {
                approx_mult(p, c, su, ru)
            },
            (ZAdicVariant::Exact(si), ZAdicVariant::Approx((_, ru))) => {
                approx_mult(p, c, si, ru)
            },
            (ZAdicVariant::Approx((_, su)), ZAdicVariant::Exact(ri)) => {
                approx_mult(p, c, su, ri)
            },
            (ZAdicVariant::Exact(si), ZAdicVariant::Exact(ri)) => {
                let i = si * ri;
                match i.sgn() {
                    AdicSign::Pos => ZAdic::new_exact(p, i.into_digits().collect()),
                    AdicSign::Neg => {
                        let non_trailing = i.num_non_trailing() as usize;
                        ZAdic::new_exact_neg(p, i.into_digits().take(non_trailing).collect())
                    }
                }
            },
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
        match &self.variant {
            ZAdicVariant::Approx((c, u)) => {
                // Finite digits
                let digits = u.digits().chain(repeat(&0)).take(*c as usize).join("").chars().rev().collect::<String>();
                write!(f, "...{digits}._{p}")
            },
            ZAdicVariant::Exact(i) => i.fmt(f),
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

        assert_eq!("...0000._5", zero_4().to_string());
        assert_eq!("...0001._5", one_4().to_string());
        assert_eq!("...6213._7", sqrt_2_7_adic().to_string());

    }

}
