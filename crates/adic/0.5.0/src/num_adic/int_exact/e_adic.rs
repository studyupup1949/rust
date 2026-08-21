use num::{BigInt, BigRational, BigUint, Rational32};
use crate::{
    divisible::Prime,
    error::AdicResult,
    num_adic::{IAdic, RAdic, UAdic},
    traits::{AdicInteger, AdicPrimitive, HasDigitDisplay},
};
use super::IntegerVariant;


#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Exact adic integer
///
/// An [`AdicInteger`](crate::traits::AdicInteger).
/// We can exactly represent adic integers that are signed ordinary integers or fractions.
/// For adics, fractions repeat to the left, with larger and larger powers of p.
/// In this way, we would say that 5-adically:
///
/// `-1/4 = 1/(1-5) = (geometric series) = 1 + 5 + 5^2 + 5^3 + 5^4 + ... = ...11111._5`
///
/// This seems weird, but 5-adically, the number "5" is small, "10", "15", and "20" are equally small,
///  and "25" is even smaller.
/// This is a *convergent* series in the 5-adics, converging to the rational number `-1/4`.
/// You can see that subtracting `-1/4` from more powers of 5 gets smaller and smaller with the 5-adic norm:
///
/// `1 - (-1/4) = 5/4; 6 - (-1/4) = 25/4; 31 - (-1/4) = 125/4; ...`
///
/// The powers of p in the numerator get larger, and the *size* (norm) of the combined number gets smaller.
///
/// `EAdic` can represent a rational number as an adic digital expansion.
/// Any rational number can be represented this way *except* those with powers of p in the denominator.
/// (Said numbers are not integers and not "small"; see [`QAdic<EAdic>`](crate::QAdic) to represent these.)
/// Even negative numbers can be represented, without a negative sign symbol!
///
/// `-1 = ...44444._5`
///
/// `...44444._5 + 1 -> ...44445._5 -> ...44450._5 -> ...44500._5 -> ...45000._5 -> ... -> 0._5`
///
/// <div class="warning">
///
/// A big caveat to this struct: multiplication can be intensive.
/// When calculating fractions as sets of repeating digits, the fraction repeat gets big **fast**.
/// This means while it is a nice struct for declaring simple adic integers,
///  it is can be inefficient to do **too** much arithmetic with them.
/// Use a [`ZAdic`](crate::ZAdic) if you can afford to approximate.
/// You can also truncate to a [`UAdic`](crate::UAdic).
///
/// </div>
///
/// # Panics
/// Many methods will panic if a provided prime `p` is not prime or digits are outside of `[0, p)`.
pub struct EAdic {
    pub (super) variant: IntegerVariant,
}


impl EAdic {

    // Constructors

    /// Create a positive adic number with the given digits
    pub fn new<P>(p: P, init_digits: Vec<u32>) -> Self
    where P: Into<Prime> {
        Self {
            variant: UAdic::new(p, init_digits).into()
        }
    }

    /// Create a negative adic number with the given digits
    pub fn new_neg<P>(p: P, init_digits: Vec<u32>) -> Self
    where P: Into<Prime> {
        Self {
            variant: IAdic::new_neg(p, init_digits).into()
        }
    }

    /// Create a rational adic number with the given fixed digits and repeating digits.
    ///
    /// E.g. 5-adic `1/4 = 1 + 3 (-1/4) = 1 + 3 (...1111._5) = ...3334._5 = EAdic::new_repeating(5, vec![4], vec![3])`
    pub fn new_repeating<P>(p: P, fix_d: Vec<u32>, rep_d: Vec<u32>) -> Self
    where P: Into<Prime> {
        Self {
            variant: RAdic::new(p, fix_d, rep_d).into()
        }
    }


    /// The natural number value of the number, e.g. 5-adic 123 is `25 + 10 + 3 = 38`
    ///
    /// Warning: This can overflow; use [`bigint_value`](Self::bigint_value) if unsure
    ///
    /// ```
    /// # use adic::{error::AdicError, EAdic};
    /// assert_eq!(Ok(38), EAdic::new(5, vec![3, 2, 1]).u32_value());
    /// assert_eq!(Err(AdicError::BadConversion), EAdic::new_neg(5, vec![2, 2, 3]).u32_value());
    /// assert_eq!(Err(AdicError::BadConversion), EAdic::new_repeating(5, vec![4], vec![3]).u32_value());
    /// ```
    pub fn u32_value(&self) -> AdicResult<u32> {
        let uadic = match &self.variant {
            IntegerVariant::Unsigned(u) => u,
            IntegerVariant::Signed(i) => &UAdic::try_from(i.clone())?,
            IntegerVariant::Rational(r) => &UAdic::try_from(r.clone())?,
        };
        uadic.u32_value()
    }

    /// The bigint representation for the natural number value of the number ([`u32_value`](`Self::u32_value`))
    ///
    /// ```
    /// # use num::BigUint;
    /// # use adic::{error::AdicError, EAdic};
    /// assert_eq!(Ok(BigUint::from(38u32)), EAdic::new(5, vec![3, 2, 1]).bigint_value());
    /// assert_eq!(Err(AdicError::BadConversion), EAdic::new_neg(5, vec![2, 2, 3]).bigint_value());
    /// assert_eq!(Err(AdicError::BadConversion), EAdic::new_repeating(5, vec![4], vec![3]).bigint_value());
    /// ```
    pub fn bigint_value(&self) -> AdicResult<BigUint> {
        let uadic = match &self.variant {
            IntegerVariant::Unsigned(u) => u,
            IntegerVariant::Signed(i) => &UAdic::try_from(i.clone())?,
            IntegerVariant::Rational(r) => &UAdic::try_from(r.clone())?,
        };
        Ok(uadic.bigint_value())
    }

    /// The natural number value of the number, e.g. 5-adic 123 is 25+10+3=38
    ///
    /// Warning: This can overflow; use [`signed_bigint_value`](Self::signed_bigint_value) if unsure
    ///
    /// ```
    /// # use adic::{error::AdicError, EAdic};
    /// assert_eq!(Ok(38), EAdic::new(5, vec![3, 2, 1]).i32_value());
    /// assert_eq!(Ok(-38), EAdic::new_neg(5, vec![2, 2, 3]).i32_value());
    /// assert_eq!(Err(AdicError::BadConversion), EAdic::new_repeating(5, vec![4], vec![3]).i32_value());
    /// ```
    pub fn i32_value(&self) -> AdicResult<i32> {
        let iadic = match &self.variant {
            IntegerVariant::Unsigned(u) => &IAdic::from(u.clone()),
            IntegerVariant::Signed(i) => i,
            IntegerVariant::Rational(r) => &IAdic::try_from(r.clone())?,
        };
        iadic.i32_value()
    }

    /// The bigint representation for the natural number value of the number ([`i32_value`](`Self::i32_value`))
    ///
    /// ```
    /// # use num::BigInt;
    /// # use adic::{error::AdicError, EAdic};
    /// assert_eq!(Ok(BigInt::from(38)), EAdic::new(5, vec![3, 2, 1]).signed_bigint_value());
    /// assert_eq!(Ok(BigInt::from(-38)), EAdic::new_neg(5, vec![2, 2, 3]).signed_bigint_value());
    /// assert_eq!(Err(AdicError::BadConversion), EAdic::new_repeating(5, vec![4], vec![3]).signed_bigint_value());
    /// ```
    pub fn signed_bigint_value(&self) -> AdicResult<BigInt> {
        let iadic = match &self.variant {
            IntegerVariant::Unsigned(u) => &IAdic::from(u.clone()),
            IntegerVariant::Signed(i) => i,
            IntegerVariant::Rational(r) => &IAdic::try_from(r.clone())?,
        };
        Ok(iadic.signed_bigint_value())
    }

    /// The rational number value of the number, e.g. 5-adic ...111 is -1/4
    ///
    /// Warning: This can easily overflow; use [`big_rational_value`](Self::big_rational_value) if unsure
    ///
    /// ```
    /// # use num::Rational32;
    /// # use adic::EAdic;
    /// assert_eq!(Ok(Rational32::from_integer(38)), EAdic::new(5, vec![3, 2, 1]).rational_value());
    /// assert_eq!(Ok(Rational32::from_integer(-38)), EAdic::new_neg(5, vec![2, 2, 3]).rational_value());
    /// assert_eq!(Ok(Rational32::new(1, 4)), EAdic::new_repeating(5, vec![4], vec![3]).rational_value());
    /// ```
    pub fn rational_value(&self) -> AdicResult<Rational32> {
        RAdic::from(self.clone()).rational_value()
    }

    /// The big rational representation for the rational number value of the number ([`rational_value`](Self::rational_value))
    ///
    /// ```
    /// # use num::{BigInt, BigRational};
    /// # use adic::EAdic;
    /// assert_eq!(BigRational::from_integer(38.into()), EAdic::new(5, vec![3, 2, 1]).big_rational_value());
    /// assert_eq!(BigRational::from_integer((-38).into()), EAdic::new_neg(5, vec![2, 2, 3]).big_rational_value());
    /// assert_eq!(BigRational::new(1.into(), 4.into()), EAdic::new_repeating(5, vec![4], vec![3]).big_rational_value());
    /// ```
    pub fn big_rational_value(&self) -> BigRational {
        RAdic::from(self.clone()).big_rational_value()
    }

    #[allow(dead_code)]
    pub (crate) fn into_raw(self) -> IntegerVariant {
        self.variant
    }
    #[allow(dead_code)]
    pub (crate) fn raw(&self) -> &IntegerVariant {
        &self.variant
    }
    #[allow(dead_code)]
    pub (crate) fn mut_raw(&mut self) -> &mut IntegerVariant {
        &mut self.variant
    }

}


impl AdicPrimitive for EAdic {

    fn zero<P>(p: P) -> Self
    where P: Into<Prime> {
        Self::from(UAdic::zero(p))
    }
    fn one<P>(p: P) -> Self
    where P: Into<Prime> {
        Self::from(UAdic::one(p))
    }
    fn p(&self) -> Prime {
        match &self.variant {
            IntegerVariant::Unsigned(u) => u.p(),
            IntegerVariant::Signed(i) => i.p(),
            IntegerVariant::Rational(r) => r.p(),
        }
    }

}


impl AdicInteger for EAdic { }
impl HasDigitDisplay for EAdic {
    type DigitDisplay = String;
    fn digit_display(&self) -> String {
        match &self.variant {
            IntegerVariant::Unsigned(u) => u.digit_display(),
            IntegerVariant::Signed(i) => i.digit_display(),
            IntegerVariant::Rational(r) => r.digit_display(),
        }
    }
}
