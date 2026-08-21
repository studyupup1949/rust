use std::iter::{once, repeat, repeat_n};
use itertools::Itertools;
use crate::{adic_valid, AdicError, ZAdicValuation};
use super::{AdicInteger, IAdic, UAdic};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Approximate Adic Integer, represented by a partially-known digital expansion
/// ([`zadic_approx`](crate::zadic_approx), [`zadic_exact_pos`](crate::zadic_exact_pos), [`zadic_exact_neg`](crate::zadic_exact_neg))
///
/// An [`AdicInteger`](crate::AdicInteger).
/// Often used to represent irrational adic numbers.
///
/// `ZAdic`s represent approximate adic numbers, known to a "certainty", some number of digits `c`.
/// These are returned from approximate methods like [`nth_root`](AdicInteger::nth_root),
///  often held together in a [`ZAdicVariety`](crate::ZAdicVariety).
///
/// ```
/// # use num::Rational32;
/// # use adic::{AdicInteger, ZAdic};
/// assert_eq!("...002341._5", ZAdic::new_approx(5, 6, vec![1, 4, 3, 2]).to_string());
/// assert_eq!("2341._5", ZAdic::new_exact_pos(5, vec![1, 4, 3, 2]).to_string());
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
/// let one_e = ZAdic::new_exact_pos(5, vec![1]);
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
    pub (super) p: u32,
    /// Type of `ZAdic`: Approx (certainty + `UAdic`) or Exact (`IAdic`)
    pub (super) variant: ZAdicVariant,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// `ZAdic` can either be approximate or exact.
/// This distinction is held in this enum.
pub (super) enum ZAdicVariant {
    /// Approx holds a `UAdic` and a finite certainty, with the `UAdic` number of digits <= the certainty.
    Approx((usize, UAdic)),
    /// Exact holds an `IAdic` and generally defers to that struct for calculations.
    Exact(IAdic),
}


impl ZAdic {

    /// Create an adic number with the given digits and certainty
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub fn new_approx(p: u32, certainty: usize, mut init_digits: Vec<u32>) -> Self {

        adic_valid::validate_p(p);
        adic_valid::validate_digits_mod_p(p, &init_digits);

        // Truncate uncertain digits
        init_digits.truncate(certainty);

        Self {
            p,
            variant: ZAdicVariant::Approx((certainty, UAdic::new(p, init_digits)))
        }

    }

    /// Create an exact adic number with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub fn new_exact_pos(p: u32, init_digits: Vec<u32>) -> Self {

        adic_valid::validate_p(p);
        adic_valid::validate_digits_mod_p(p, &init_digits);

        Self {
            p,
            variant: ZAdicVariant::Exact(IAdic::new_pos(p, init_digits))
        }

    }

    /// Create an exact negative adic number (trailing p-1) with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub fn new_exact_neg(p: u32, init_digits: Vec<u32>) -> Self {

        adic_valid::validate_p(p);
        adic_valid::validate_digits_mod_p(p, &init_digits);

        Self {
            p,
            variant: ZAdicVariant::Exact(IAdic::new_neg(p, init_digits))
        }

    }

    /// Create an approximate adic number with no certainty and no digits
    ///
    /// ```
    /// # use adic::{AdicInteger, ZAdic, ZAdicValuation};
    /// assert_eq!(ZAdicValuation::Finite(0), ZAdic::empty(5).valuation());
    /// assert!(ZAdic::empty(5).into_digits().collect::<Vec<u32>>().is_empty());
    pub fn empty(p: u32) -> Self {
        adic_valid::validate_p(p);
        Self {
            p,
            variant: ZAdicVariant::Approx((0, UAdic::zero(p)))
        }
    }


    /// Push another cerain digit onto the end of the number
    ///
    /// # Errors
    /// Returns error if number already has infinite certainty
    ///
    /// # Panics
    /// Panics if digit is outside of [0, p)
    ///
    /// ```
    /// # use adic::{zadic_approx, zadic_exact_pos, zadic_exact_neg, AdicError};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut z = zadic_approx!(5, 4, [1, 2, 3, 4]);
    /// z.push_digit(3)?;
    /// assert_eq!(zadic_approx!(5, 5, [1, 2, 3, 4, 3]), z);
    /// let mut z = zadic_exact_pos!(5, [1, 2, 3, 4]);
    /// assert!(matches!(z.push_digit(3), Err(AdicError::InappropriatePrecision(_))));
    /// let mut z = zadic_exact_neg!(5, [1, 2, 3, 4]);
    /// assert!(matches!(z.push_digit(3), Err(AdicError::InappropriatePrecision(_))));
    /// # Ok(()) }
    /// ```
    pub fn push_digit(&mut self, digit: u32) -> Result<(), AdicError> {

        adic_valid::validate_digit_mod_p(self.p, digit);

        if let ZAdicVariant::Approx(var) = &mut self.variant {
            var.1.extend_digits(&repeat_n(0, var.0 - var.1.finite_num_digits()).chain(once(digit)).collect::<Vec<_>>());
            var.0 += 1;
            Ok(())
        } else {
            Err(AdicError::InappropriatePrecision(
                "Cannot append to infinite certainty number".to_string()
            ))
        }

    }

    /// Pop a certain digit off the end of the number
    ///
    /// # Errors
    /// Returns error if number has infinite certainty
    ///
    /// ```
    /// # use adic::{zadic_approx, zadic_exact_pos, zadic_exact_neg, AdicError, AdicInteger};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut z = zadic_approx!(5, 2, [1, 2]);
    /// assert_eq!(Some(2), z.pop_digit()?);
    /// assert_eq!(zadic_approx!(5, 1, [1]), z);
    /// assert_eq!(Some(1), z.pop_digit()?);
    /// assert!(z.has_no_certainty());
    /// assert_eq!(None, z.pop_digit()?);
    /// let mut z = zadic_exact_pos!(5, [1, 2, 3, 4]);
    /// assert!(matches!(z.pop_digit(), Err(AdicError::InappropriatePrecision(_))));
    /// let mut z = zadic_exact_neg!(5, [1, 2, 3, 4]);
    /// assert!(matches!(z.pop_digit(), Err(AdicError::InappropriatePrecision(_))));
    /// # Ok(()) }
    /// ```
    pub fn pop_digit(&mut self) -> Result<Option<u32>, AdicError> {
        if let ZAdicVariant::Approx(var) = &mut self.variant {
            let d = if var.0 == 0 {
                None
            } else if var.0 > var.1.finite_num_digits() {
                var.0 -= 1;
                Some(0)
            } else {
                var.0 -= 1;
                var.1.pop_digit()
            };
            Ok(d)
        } else {
            Err(AdicError::InappropriatePrecision(
                "Cannot append to infinite certainty number".to_string()
            ))
        }
    }

    /// Change the certainty of the `ZAdic`, assuming zeros for any new digits
    ///
    /// ```
    /// # use adic::{zadic_approx, zadic_exact_pos, zadic_exact_neg, AdicError, ZAdicValuation};
    /// let mut z = zadic_approx!(5, 4, [1, 2, 3, 4]);
    /// z.set_certainty(ZAdicValuation::Finite(5));
    /// assert_eq!(zadic_approx!(5, 5, [1, 2, 3, 4, 0]), z);
    /// z.set_certainty(ZAdicValuation::Finite(3));
    /// assert_eq!(zadic_approx!(5, 3, [1, 2, 3]), z);
    /// z.set_certainty(ZAdicValuation::PosInf);
    /// assert_eq!(zadic_exact_pos!(5, [1, 2, 3]), z);
    /// ```
    pub fn set_certainty(&mut self, c: ZAdicValuation) {
        match (c, &mut self.variant) {
            (ZAdicValuation::Finite(c), ZAdicVariant::Approx(var)) => {
                var.0 = c;
                var.1 = var.1.truncation(c);
            },
            (ZAdicValuation::Finite(c), ZAdicVariant::Exact(var)) => {
                self.variant = ZAdicVariant::Approx((c, var.truncation(c)));
            },
            (ZAdicValuation::PosInf, ZAdicVariant::Approx(var)) => {
                // Assume positive number
                self.variant = ZAdicVariant::Exact(IAdic::new_pos(self.p, var.1.clone().into_digits_vec()));
            },
            (ZAdicValuation::PosInf, ZAdicVariant::Exact(_var)) => { },
        }
    }

}


impl AdicInteger for ZAdic {
    fn zero(p: u32) -> Self {
        Self::new_exact_pos(p, vec![])
    }
    fn one(p: u32) -> Self {
        Self::new_exact_pos(p, vec![1])
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
    fn digit(&self, n: usize) -> Result<u32, AdicError> {
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
                ZAdicVariant::Approx((c, u)) => Box::new(u.digits().chain(repeat(&0)).take(*c)),
                ZAdicVariant::Exact(i) => Box::new(i.digits()),
            }
        }
        inner_iter(&self.variant)
    }
    fn into_digits(self) -> impl Iterator<Item=u32> {
        // Returns infinite iterator if num_digits PosInf and finite else
        fn inner_into_iter(variant: ZAdicVariant) -> Box<dyn Iterator<Item=u32>> {
            match variant {
                ZAdicVariant::Approx((c, u)) => Box::new(u.into_digits().chain(repeat(0)).take(c)),
                ZAdicVariant::Exact(i) => Box::new(i.into_digits()),
            }
        }
        inner_into_iter(self.variant)
    }
    fn digit_str(&self) -> String {
        match &self.variant {
            ZAdicVariant::Approx((c, u)) => {
                // Finite digits
                let digits = u.digits().chain(repeat(&0)).take(*c).join("").chars().rev().collect::<String>();
                format!("...{digits}")
            },
            ZAdicVariant::Exact(i) => i.digit_str(),
        }
    }
    fn is_zero(&self) -> bool {
        if let ZAdicVariant::Exact(i) = &self.variant {
            i.is_zero()
        } else {
            false
        }
    }
    fn into_split(self, n: usize) -> (UAdic, Self) {
        let p = self.p;
        match self.variant {
            ZAdicVariant::Approx((c, ua)) => {
                let (before, after) = ua.into_split(n);
                if n < c {
                    (before, Self::new_approx(p, c - n, after.into_digits_vec()))
                } else {
                    // This may be bad; if certainty is less than n, it still returns a UAdic,
                    //  even though the UAdic is not known to that certainty.
                    // Consider returning an error instead.
                    (before, Self::empty(p))
                }
            },
            ZAdicVariant::Exact(ia) => {
                let (before, after) = ia.into_split(n);
                (before, Self::from(after))
            }
        }
    }
    fn certainty(&self) -> ZAdicValuation {
        match &self.variant {
            ZAdicVariant::Approx(c) => ZAdicValuation::Finite(c.0),
            ZAdicVariant::Exact(_) => ZAdicValuation::PosInf,
        }
    }
}



#[cfg(test)]
mod tests {
    use num::traits::Pow;
    use super::{AdicInteger, ZAdic, ZAdicValuation};
    use crate::{zadic_approx, zadic_exact_pos, AdicError, ZAdicVariety};
    use ZAdicValuation::Finite;


    #[test]
    fn approximate_z_adic() {

        let zero_4 = zadic_approx!(5, 4, [0, 0, 0, 0]);
        assert!(!zero_4.is_zero());
        let one_2 = zadic_approx!(5, 2, [1]);
        let two_4 = zadic_approx!(5, 4, [2]);
        let five_3 = zadic_approx!(5, 3, [0, 1]);

        assert_eq!(Finite(2), (&one_2 + &one_2).certainty());
        assert_eq!(Finite(2), (&one_2 + &two_4).certainty());
        assert_eq!(Finite(4), (&two_4 + &two_4).certainty());
        assert_eq!(Finite(2), (&one_2 + &five_3).certainty());
        assert_eq!(Finite(3), (&two_4 + &five_3).certainty());

        assert_eq!(Finite(2), (&one_2 * &one_2).certainty());
        assert_eq!(Finite(2), (&one_2 * &two_4).certainty());
        assert_eq!(Finite(4), (&two_4 * &two_4).certainty());
        assert_eq!(Finite(3), (&one_2 * &five_3).certainty());
        assert_eq!(Finite(3), (&two_4 * &five_3).certainty());
        assert_eq!(Finite(4), (&five_3 * &five_3).certainty());

    }

    #[test]
    fn nth_root() {

        let check = |p: u32, a: &ZAdic, n: u32, precision: usize, roots: Vec<ZAdic>| {
            // Check each root powers to match a to at least precision digits
            for root in &roots {
                assert_eq!(a.truncation(precision), root.pow(n).into_truncation_to_uadic().unwrap());
            }
            // Check roots match the output of nth_root
            assert_eq!(Ok(ZAdicVariety::new(p, roots)), a.nth_root(n, precision));
        };

        check(5, &zadic_exact_pos!(5, [1]), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);
        check(5, &zadic_approx!(5, 12, [1]), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);

        check(5, &zadic_exact_pos!(5, [2]), 2, 6, vec![]);
        check(5, &zadic_approx!(5, 12, [2]), 2, 6, vec![]);

        check(7, &zadic_exact_pos!(7, [2]), 2, 6, vec![
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

}
