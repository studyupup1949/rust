use std::iter::repeat;
use itertools::{Either, Itertools};
use crate::{
    adic_valid,
    AdicError, AdicInteger, AdicNumber, AdicResult, AdicValuation,
    Divisible, ExactIntegerVariant, HasDigits, IAdic, Prime, UAdic, ZAdicValuation,
};


#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Approximate adic integer, represented by a partially-known digital expansion
/// ([`zadic_approx`](crate::zadic_approx), [`zadic_exact`](crate::zadic_exact))
///
/// An [`AdicInteger`](crate::AdicInteger).
/// Often used to represent irrational adic numbers.
/// This is the workhorse for calculations; default to using this.
///
/// `ZAdic`s represent approximate adic numbers, known to a "certainty", some number of digits `c`.
/// These are returned from approximate methods like [`nth_root`](AdicInteger::nth_root),
///  often held together in a [`AdicVariety`](crate::AdicVariety).
///
/// ```
/// # use num::Rational32;
/// # use adic::{uadic, AdicInteger, ZAdic};
/// assert_eq!("...002341._5", ZAdic::new_approx(5, 6, vec![1, 4, 3, 2]).to_string());
/// assert_eq!("2341._5", ZAdic::from(uadic!(5, [1, 4, 3, 2])).to_string());
/// ```
///
/// Adding and multiplying `ZAdic`s respects the certainty.
/// When adding, the output certainty is the minimum of the input certainties:
///  `...abc._p + ...de._p = ...fg._p`.
/// When multiplying, the output certainty is a little more complicated,
///  since zero digits can make things more certain than just the minumum:
///  `...ab0._p * ...de._p = ...fg0._p`.
///
/// `ZAdic`s can also represent exact integers,
///  anything that [`UAdic`](crate::UAdic), [`IAdic`](crate::IAdic), or [`RAdic`](crate::RAdic) can.
/// It holds a variant of those three and so any of them can be converted directly into a `ZAdic`.
///
/// ```
/// # use adic::{iadic_neg, uadic, AdicNumber, ZAdic};
/// let one_e = ZAdic::from(uadic!(5, [1]));
/// assert_eq!("1._5", one_e.to_string());
/// let neg_one_e = ZAdic::from(iadic_neg!(5, []));
/// assert_eq!("(4)._5", neg_one_e.to_string());
/// assert!((one_e + neg_one_e).is_zero());
/// ```
///
/// In this way, the exact `ZAdic` strictly more flexible than [`RAdic`](crate::RAdic),
///  able to represent all ordinary integers and rationals
///  (other than p-fractional; see [`QAdic`](crate::QAdic)).
pub struct ZAdic {
    /// Certainty of this adic
    pub (super) c: AdicValuation<usize>,
    /// Type of `ZAdic`: Approx (certainty + `UAdic`) or Exact (`IAdic`)
    pub (super) variant: ExactIntegerVariant,
}


impl ZAdic {

    /// Create an adic number with the given digits and certainty
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub fn new_approx<P>(p: P, certainty: usize, mut init_digits: Vec<u32>) -> Self
    where P: Into<Prime> {

        let p = p.into();
        adic_valid::validate_digits_mod_p(p, &init_digits);

        // Truncate uncertain digits
        init_digits.truncate(certainty);

        Self {
            c: certainty.into(),
            variant: ExactIntegerVariant::Unsigned(UAdic::new(p, init_digits)),
        }

    }

    #[deprecated]
    /// Create an exact adic number with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub fn new_exact_pos<P>(p: P, init_digits: Vec<u32>) -> Self
    where P: Into<Prime> {

        let p = p.into();
        adic_valid::validate_digits_mod_p(p, &init_digits);

        Self {
            c: AdicValuation::PosInf,
            variant: ExactIntegerVariant::Signed(IAdic::new_pos(p, init_digits))
        }

    }

    #[deprecated]
    /// Create an exact negative adic number (trailing p-1) with the given digits
    ///
    /// # Panics
    /// Panics if `p` is not prime or digits are outside of [0, p)
    pub fn new_exact_neg<P>(p: P, init_digits: Vec<u32>) -> Self
    where P: Into<Prime> {

        let p = p.into();
        adic_valid::validate_digits_mod_p(p, &init_digits);

        Self {
            c: AdicValuation::PosInf,
            variant: ExactIntegerVariant::Signed(IAdic::new_neg(p, init_digits))
        }

    }

    /// Create an approximate adic number with no certainty and no digits
    ///
    /// ```
    /// # use adic::{AdicInteger, AdicSized, HasDigits, ZAdic, AdicValuation};
    /// assert_eq!(AdicValuation::Finite(0), ZAdic::empty(5).valuation());
    /// assert!(ZAdic::empty(5).into_digits().collect::<Vec<u32>>().is_empty());
    pub fn empty<P>(p: P) -> Self
    where P: Into<Prime> {
        let p = p.into();
        Self {
            c: 0.into(),
            variant: ExactIntegerVariant::Unsigned(UAdic::zero(p)),
        }
    }


    /// Return the certainty if `ZAdic` is approximate or the `ExactIntegerVariant` if it is exact
    pub (crate) fn exact_variant_or_certainty(&self) -> Either<&ExactIntegerVariant, usize> {
        match self.c {
            AdicValuation::PosInf => Either::Left(&self.variant),
            AdicValuation::Finite(c) => Either::Right(c),
        }
    }


    /// Is this adic zero up to its certainty
    ///
    /// ```
    /// # use adic::{iadic_neg, uadic, zadic_approx, AdicNumber, ZAdic};
    /// assert_eq!(true, ZAdic::zero(5).is_approx_zero());
    /// assert_eq!(true, ZAdic::empty(5).is_approx_zero());
    /// assert_eq!(true, ZAdic::from(uadic!(5, [])).is_approx_zero());
    /// assert_eq!(false, ZAdic::from(uadic!(5, [1])).is_approx_zero());
    /// assert_eq!(false, ZAdic::from(iadic_neg!(5, [])).is_approx_zero());
    /// assert_eq!(true, zadic_approx!(5, 6, [0, 0]).is_approx_zero());
    /// assert_eq!(true, zadic_approx!(5, 3, [0, 0, 0, 1, 2]).is_approx_zero());
    /// assert_eq!(false, zadic_approx!(5, 5, [0, 0, 0, 1, 2]).is_approx_zero());
    /// assert_eq!(false, zadic_approx!(5, 5, [0, 0, 0, 1, 2]).is_approx_zero());
    /// ```
    pub fn is_approx_zero(&self) -> bool {
        // TODO: Actually need to check the digits below certainty instead
        match &self.variant {
            ExactIntegerVariant::Unsigned(u) => u.is_zero(),
            ExactIntegerVariant::Signed(i) => i.is_zero(),
            ExactIntegerVariant::Rational(r) => r.is_zero(),
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
    /// # use adic::{iadic_neg, uadic, zadic_approx, AdicError, ZAdic};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut z = zadic_approx!(5, 4, [1, 2, 3, 4]);
    /// z.push_digit(3)?;
    /// assert_eq!(zadic_approx!(5, 5, [1, 2, 3, 4, 3]), z);
    /// let mut z = ZAdic::from(uadic!(5, [1, 2, 3, 4]));
    /// assert!(matches!(z.push_digit(3), Err(AdicError::InappropriatePrecision(_))));
    /// let mut z = ZAdic::from(iadic_neg!(5, [1, 2, 3, 4]));
    /// assert!(matches!(z.push_digit(3), Err(AdicError::InappropriatePrecision(_))));
    /// # Ok(()) }
    /// ```
    pub fn push_digit(&mut self, digit: u32) -> AdicResult<()> {

        adic_valid::validate_digit_mod_p(self.p(), digit);

        if let AdicValuation::Finite(c) = &mut self.c {
            self.variant.truncate_and_push(*c, digit);
            *c += 1;
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
    /// # use adic::{iadic_neg, uadic, zadic_approx, AdicApproximate, AdicError, AdicInteger, ZAdic};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut z = zadic_approx!(5, 2, [1, 2]);
    /// assert_eq!(Some(2), z.pop_digit()?);
    /// assert_eq!(zadic_approx!(5, 1, [1]), z);
    /// assert_eq!(Some(1), z.pop_digit()?);
    /// assert!(z.has_no_certainty());
    /// assert_eq!(None, z.pop_digit()?);
    /// let mut z = ZAdic::from(uadic!(5, [1, 2, 3, 4]));
    /// assert!(matches!(z.pop_digit(), Err(AdicError::InappropriatePrecision(_))));
    /// let mut z = ZAdic::from(iadic_neg!(5, [1, 2, 3, 4]));
    /// assert!(matches!(z.pop_digit(), Err(AdicError::InappropriatePrecision(_))));
    /// # Ok(()) }
    /// ```
    pub fn pop_digit(&mut self) -> AdicResult<Option<u32>> {
        match &mut self.c {
            ZAdicValuation::Finite(0) => Ok(None),
            ZAdicValuation::Finite(c) => {
                let digit = self.variant.truncate_and_pop(*c);
                *c -= 1;
                Ok(digit)
            },
            ZAdicValuation::PosInf => {
                Err(AdicError::InappropriatePrecision(
                    "Cannot pop from infinite certainty number".to_string()
                ))
            },
        }
    }

    /// Change the certainty of the `ZAdic`, assuming zeros for any new digits
    ///
    /// ```
    /// # use adic::{uadic, zadic_approx, AdicError, AdicValuation, ZAdic};
    /// let mut z = zadic_approx!(5, 4, [1, 2, 3, 4]);
    /// z.set_certainty(AdicValuation::Finite(5));
    /// assert_eq!(zadic_approx!(5, 5, [1, 2, 3, 4, 0]), z);
    /// z.set_certainty(AdicValuation::Finite(3));
    /// assert_eq!(zadic_approx!(5, 3, [1, 2, 3]), z);
    /// z.set_certainty(AdicValuation::PosInf);
    /// assert_eq!(ZAdic::from(uadic!(5, [1, 2, 3])), z);
    /// ```
    pub fn set_certainty(&mut self, c: AdicValuation<usize>) {
        if self.c < c {
            if let AdicValuation::Finite(current_c) = self.c {
                self.variant.truncate(current_c);
            }
        }
        self.c = c;
    }

}


impl AdicNumber for ZAdic {

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
            ExactIntegerVariant::Unsigned(u) => u.p(),
            ExactIntegerVariant::Signed(i) => i.p(),
            ExactIntegerVariant::Rational(r) => r.p(),
        }
    }

}


impl AdicInteger for ZAdic {

    fn digit_str(&self) -> String {
        match self.c {
            AdicValuation::PosInf => match &self.variant {
                ExactIntegerVariant::Unsigned(u) => u.digit_str(),
                ExactIntegerVariant::Signed(i) => i.digit_str(),
                ExactIntegerVariant::Rational(r) => r.digit_str(),
            },
            AdicValuation::Finite(c) => {
                let p = self.p();
                match &self.variant {
                    ExactIntegerVariant::Unsigned(u) => {
                        let ds = u.digits().chain(repeat(0)).take(c).map(|d| p.display_digit(d)).collect::<Vec<_>>();
                        let digits = ds.into_iter().rev().join("");
                        format!("...{digits}")
                    },
                    ExactIntegerVariant::Signed(i) => {
                        let ds = i.digits().chain(repeat(0)).take(c).map(|d| p.display_digit(d)).collect::<Vec<_>>();
                        let digits = ds.into_iter().rev().join("");
                        format!("...{digits}")
                    },
                    ExactIntegerVariant::Rational(r) => {
                        let ds = r.digits().chain(repeat(0)).take(c).map(|d| p.display_digit(d)).collect::<Vec<_>>();
                        let digits = ds.into_iter().rev().join("");
                        format!("...{digits}")
                    },
                }
            },
        }
    }
    fn into_split(self, n: usize) -> (UAdic, Self) {
        let p = self.p();
        match self.c {
            AdicValuation::PosInf => match self.variant {
                ExactIntegerVariant::Unsigned(u) => {
                    let (before, after) = u.into_split(n);
                    (before, Self::from(after))
                },
                ExactIntegerVariant::Signed(i) => {
                    let (before, after) = i.into_split(n);
                    (before, Self::from(after))
                },
                ExactIntegerVariant::Rational(r) => {
                    let (before, after) = r.into_split(n);
                    (before, Self::from(after))
                },
            },
            AdicValuation::Finite(c) => match self.variant {
                ExactIntegerVariant::Unsigned(u) => {
                    let (before, after) = u.into_split(n);
                    if n < c {
                        (before, ZAdic { c: (c - n).into(), variant: ExactIntegerVariant::Unsigned(after) })
                    } else {
                        // This may be bad; if certainty is less than n, it still returns a UAdic,
                        //  even though the UAdic is not known to that certainty.
                        // Consider returning an error instead.
                        (before, Self::empty(p))
                    }
                },
                ExactIntegerVariant::Signed(i) => {
                    let (before, after) = i.into_split(n);
                    if n < c {
                        (before, ZAdic { c: (c - n).into(), variant: ExactIntegerVariant::Signed(after) })
                    } else {
                        // This may be bad; if certainty is less than n, it still returns a UAdic,
                        //  even though the UAdic is not known to that certainty.
                        // Consider returning an error instead.
                        (before, Self::empty(p))
                    }
                },
                ExactIntegerVariant::Rational(r) => {
                    let (before, after) = r.into_split(n);
                    if n < c {
                        (before, ZAdic { c: (c - n).into(), variant: ExactIntegerVariant::Rational(after) })
                    } else {
                        // This may be bad; if certainty is less than n, it still returns a UAdic,
                        //  even though the UAdic is not known to that certainty.
                        // Consider returning an error instead.
                        (before, Self::empty(p))
                    }
                },
            },
        }
    }

}



#[cfg(test)]
mod tests {
    use num::traits::Pow;
    use crate::{
        uadic, zadic_approx,
        AdicApproximate, AdicError, AdicSized, AdicValuation, HasDigits, AdicVariety,
    };
    use super::{AdicInteger, AdicNumber, ZAdic};
    use AdicValuation::Finite;


    #[test]
    fn approximate_z_adic() {

        let zero_4 = zadic_approx!(5, 4, [0, 0, 0, 0]);
        assert!(!zero_4.is_zero());
        let one_2 = zadic_approx!(5, 2, [1]);
        let two_4 = zadic_approx!(5, 4, [2]);
        let five_3 = zadic_approx!(5, 3, [0, 1]);

        // Addition handles certainty with minimum certainty
        assert_eq!(Finite(2), (&one_2 + &one_2).certainty());
        assert_eq!(Finite(2), (&one_2 + &two_4).certainty());
        assert_eq!(Finite(4), (&two_4 + &two_4).certainty());
        assert_eq!(Finite(2), (&one_2 + &five_3).certainty());
        assert_eq!(Finite(3), (&two_4 + &five_3).certainty());

        // Multiplication handles certainty with minimum significance
        assert_eq!(Finite(2), (&one_2 * &one_2).certainty());
        assert_eq!(Finite(2), (&one_2 * &two_4).certainty());
        assert_eq!(Finite(4), (&two_4 * &two_4).certainty());
        assert_eq!(Finite(3), (&one_2 * &five_3).certainty());
        assert_eq!(Finite(3), (&two_4 * &five_3).certainty());
        assert_eq!(Finite(4), (&five_3 * &five_3).certainty());

        // Certainty past valuation truncates
        let small_z = zadic_approx!(5, 3, [4, 3, 2, 1, 2]);
        assert_eq!(Finite(0), small_z.valuation());
        assert_eq!(Finite(3), small_z.certainty());
        assert_eq!(Finite(3), small_z.significance());
        assert_eq!(vec![4, 3, 2], small_z.into_digits().collect::<Vec<_>>());

    }

    #[test]
    fn empty_z_adic() {

        let empty = ZAdic::empty(5);
        let (unit, val) = empty.unit_and_valuation();
        assert_eq!(Some(ZAdic::empty(5)), unit);
        assert_eq!(AdicValuation::Finite(0), val);
        assert_eq!(empty, &empty + zadic_approx!(5, 4, [1]));
        assert_eq!(empty, &empty * zadic_approx!(5, 4, [1]));
        assert_eq!(ZAdic::zero(5), &empty * ZAdic::zero(5));

    }

    #[test]
    fn nth_root() {

        let check = |p: u32, a: &ZAdic, n: u32, precision: usize, roots: Vec<ZAdic>| {
            // Check each root powers to match a to at least precision digits
            for root in &roots {
                assert_eq!(a.approximation(precision), root.pow(n));
            }
            // Check roots match the output of nth_root
            assert_eq!(Ok(AdicVariety::new(p, roots)), a.nth_root(n, precision));
        };

        check(5, &ZAdic::from(uadic!(5, [1])), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);
        check(5, &zadic_approx!(5, 12, [1]), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);

        check(5, &ZAdic::from(uadic!(5, [2])), 2, 6, vec![]);
        check(5, &zadic_approx!(5, 12, [2]), 2, 6, vec![]);

        check(7, &ZAdic::from(uadic!(7, [2])), 2, 6, vec![
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
