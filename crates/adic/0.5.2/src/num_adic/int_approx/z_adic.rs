use std::iter::{once, repeat, repeat_n};
use itertools::{Either, Itertools};
use crate::{
    divisible::{Divisible, Prime},
    error::{validate_digits_mod_p, AdicError, AdicResult},
    local_num::{LocalOne, LocalZero},
    normed::Valuation,
    traits::{AdicInteger, AdicPrimitive, HasDigitDisplay, HasDigits},
    EAdic, Polynomial, UAdic, Variety,
};


#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Approximate adic integer
///
/// An [`AdicInteger`](crate::traits::AdicInteger)
///  represented by a partially-known digital expansion.
/// Often used to represent irrational adic numbers.
/// This is the workhorse for calculations; default to using this.
///
/// `ZAdic`s represent approximate adic numbers, known to a "certainty", some number of digits `c`.
/// These are returned from approximate methods like [`nth_root`](AdicInteger::nth_root)
///  or [`Polynomial::variety`](crate::Polynomial::variety),
///  often held together in a [`Variety`](crate::Variety).
///
/// ```
/// # use num::Rational32;
/// # use adic::{traits::AdicInteger, UAdic, ZAdic};
/// assert_eq!("...002341._5", ZAdic::new_approx(5, 6, vec![1, 4, 3, 2]).to_string());
/// assert_eq!("2341._5", ZAdic::from(UAdic::new(5, vec![1, 4, 3, 2])).to_string());
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
///  anything that [`EAdic`](crate::EAdic) can.
/// It holds an `EAdic` internally, and an `EAdic` can be converted directly into a `ZAdic`.
///
/// ```
/// # use adic::{traits::AdicPrimitive, EAdic, UAdic, ZAdic};
/// let one_e = ZAdic::from(UAdic::new(5, vec![1]));
/// assert_eq!("1._5", one_e.to_string());
/// let neg_one_e = ZAdic::from(EAdic::new_neg(5, vec![]));
/// assert_eq!("(4)._5", neg_one_e.to_string());
/// assert_eq!(ZAdic::zero(5), one_e + neg_one_e);
/// ```
///
/// In this way, the exact `ZAdic` strictly more flexible than [`EAdic`](crate::EAdic),
///  able to represent all ordinary integers and rationals, including approximately.
/// For p-fractional; see [`QAdic`](crate::QAdic).
///
/// # Panics
/// Many methods will panic if a provided prime `p` is not prime or digits are outside of `[0, p)`.
pub struct ZAdic {
    /// Certainty of this adic
    pub (super) c: Valuation<usize>,
    /// Type of `ZAdic`: Approx (certainty + exact adic) or Exact (exact adic)
    pub (super) variant: EAdic,
}


impl ZAdic {

    /// Create an approximate adic number with the given digits and certainty
    pub fn new_approx<P>(p: P, certainty: usize, mut init_digits: Vec<u32>) -> Self
    where P: Into<Prime> {

        let p = p.into();
        validate_digits_mod_p(p, &init_digits);

        // Truncate uncertain digits
        init_digits.truncate(certainty);

        Self {
            c: certainty.into(),
            variant: UAdic::new(p, init_digits).into(),
        }

    }

    /// Create an approximate adic number with no certainty and no digits
    ///
    /// ```
    /// # use adic::{normed::{UltraNormed, Valuation}, traits::{AdicInteger, HasDigits}, ZAdic};
    /// assert_eq!(Valuation::Finite(0), ZAdic::empty(5).valuation());
    /// assert!(ZAdic::empty(5).digits().collect::<Vec<u32>>().is_empty());
    pub fn empty<P>(p: P) -> Self
    where P: Into<Prime> {
        let p = p.into();
        Self {
            c: 0.into(),
            variant: UAdic::zero(p).into(),
        }
    }


    /// Return the certainty if `ZAdic` is approximate or the `EAdic` if it is exact
    pub (crate) fn exact_variant_or_certainty(&self) -> Either<EAdic, usize> {
        match self.c {
            Valuation::PosInf => Either::Left(self.variant.clone()),
            Valuation::Finite(c) => Either::Right(c),
        }
    }

    /// Return the raw `EAdic`
    ///
    /// Warning: Be careful! Only use if (1) the `ZAdic` is certain or (2) you know what you're doing.
    pub (crate) fn exact_variant(&self) -> EAdic {
        self.variant.clone()
    }


    /// Is this adic zero up to its certainty
    ///
    /// ```
    /// # use adic::{traits::AdicPrimitive, EAdic, UAdic, ZAdic};
    /// assert!(ZAdic::zero(5).is_approx_zero());
    /// assert!(ZAdic::empty(5).is_approx_zero());
    /// assert!(ZAdic::from(UAdic::new(5, vec![])).is_approx_zero());
    /// assert!(!ZAdic::from(UAdic::new(5, vec![1])).is_approx_zero());
    /// assert!(!ZAdic::from(EAdic::new_neg(5, vec![])).is_approx_zero());
    /// assert!(ZAdic::new_approx(5, 6, vec![0, 0]).is_approx_zero());
    /// assert!(ZAdic::new_approx(5, 3, vec![0, 0, 0, 1, 2]).is_approx_zero());
    /// assert!(!ZAdic::new_approx(5, 5, vec![0, 0, 0, 1, 2]).is_approx_zero());
    /// assert!(!ZAdic::new_approx(5, 5, vec![0, 0, 0, 1, 2]).is_approx_zero());
    /// ```
    pub fn is_approx_zero(&self) -> bool {
        match self.c {
            Valuation::PosInf => self.is_local_zero(),
            Valuation::Finite(c) => self.digits().take(c).all(|d| d.is_local_zero())
        }
    }

    /// Is this adic one up to its certainty
    ///
    /// ```
    /// # use adic::{traits::AdicPrimitive, EAdic, UAdic, ZAdic};
    /// assert!(ZAdic::one(5).is_approx_one());
    /// assert!(ZAdic::empty(5).is_approx_one());
    /// assert!(ZAdic::from(UAdic::new(5, vec![1])).is_approx_one());
    /// assert!(!ZAdic::from(UAdic::new(5, vec![])).is_approx_one());
    /// assert!(!ZAdic::from(EAdic::new_neg(5, vec![])).is_approx_one());
    /// assert!(ZAdic::new_approx(5, 6, vec![1, 0]).is_approx_one());
    /// assert!(ZAdic::new_approx(5, 3, vec![1, 0, 0, 1, 2]).is_approx_one());
    /// assert!(!ZAdic::new_approx(5, 5, vec![1, 0, 0, 1, 2]).is_approx_one());
    /// assert!(!ZAdic::new_approx(5, 5, vec![1, 0, 0, 1, 2]).is_approx_one());
    /// ```
    pub fn is_approx_one(&self) -> bool {
        match self.c {
            Valuation::PosInf => self.is_local_one(),
            Valuation::Finite(0) => true,
            Valuation::Finite(c) => {
                self.digits().next().is_some_and(|d| d.is_local_one())
                    && self.digits().take(c).skip(1).all(|d| d.is_local_zero())
            },
        }
    }


    /// Push another digit onto the number, increasing its certainty
    ///
    /// # Errors
    /// Returns error if number already has infinite certainty
    ///
    /// ```
    /// # use adic::{error::AdicError, EAdic, UAdic, ZAdic};
    /// let mut z = ZAdic::new_approx(5, 4, vec![1, 2, 3, 4]);
    /// z.push_digit(3)?;
    /// assert_eq!(ZAdic::new_approx(5, 5, vec![1, 2, 3, 4, 3]), z);
    /// let mut z = ZAdic::from(UAdic::new(5, vec![1, 2, 3, 4]));
    /// assert!(matches!(z.push_digit(3), Err(AdicError::InappropriatePrecision(_))));
    /// let mut z = ZAdic::from(EAdic::new_neg(5, vec![1, 2, 3, 4]));
    /// assert!(matches!(z.push_digit(3), Err(AdicError::InappropriatePrecision(_))));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn push_digit(&mut self, digit: u32) -> AdicResult<()> {

        validate_digits_mod_p(self.p(), &[digit]);

        if let Valuation::Finite(c) = &mut self.c {
            if self.variant.digit(*c) != Ok(digit) {
                self.variant.mut_raw().truncate_and_push(*c, digit);
            }
            *c += 1;
            Ok(())
        } else {
            Err(AdicError::InappropriatePrecision(
                "Cannot append to infinite certainty number".to_string()
            ))
        }

    }

    /// Pop a digit off the end of the number, decreasing its certainty
    ///
    /// # Errors
    /// Returns error if number has infinite certainty
    ///
    /// ```
    /// # use adic::{error::AdicError, traits::{AdicInteger, HasApproximateDigits}, EAdic, UAdic, ZAdic};
    /// let mut z = ZAdic::new_approx(5, 2, vec![1, 2]);
    /// assert_eq!(Some(2), z.pop_digit()?);
    /// assert_eq!(ZAdic::new_approx(5, 1, vec![1]), z);
    /// assert_eq!(Some(1), z.pop_digit()?);
    /// assert!(z.has_no_certainty());
    /// assert_eq!(None, z.pop_digit()?);
    /// let mut z = ZAdic::from(UAdic::new(5, vec![1, 2, 3, 4]));
    /// assert!(matches!(z.pop_digit(), Err(AdicError::InappropriatePrecision(_))));
    /// let mut z = ZAdic::from(EAdic::new_neg(5, vec![1, 2, 3, 4]));
    /// assert!(matches!(z.pop_digit(), Err(AdicError::InappropriatePrecision(_))));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn pop_digit(&mut self) -> AdicResult<Option<u32>> {
        match &mut self.c {
            Valuation::Finite(0) => Ok(None),
            Valuation::Finite(c) => {
                let digit = self.variant.digit(*c - 1)?;
                *c -= 1;
                Ok(Some(digit))
            },
            Valuation::PosInf => {
                Err(AdicError::InappropriatePrecision(
                    "Cannot pop from infinite certainty number".to_string()
                ))
            },
        }
    }

    /// Change the certainty of the `ZAdic`, assuming zeros for any new digits
    ///
    /// ```
    /// # use adic::{normed::Valuation, UAdic, ZAdic};
    /// let mut z = ZAdic::new_approx(5, 4, vec![1, 2, 3, 4]);
    /// z.set_certainty(Valuation::Finite(5));
    /// assert_eq!(ZAdic::new_approx(5, 5, vec![1, 2, 3, 4, 0]), z);
    /// z.set_certainty(Valuation::Finite(3));
    /// assert_eq!(ZAdic::new_approx(5, 3, vec![1, 2, 3]), z);
    /// z.set_certainty(Valuation::PosInf);
    /// assert_eq!(ZAdic::from(UAdic::new(5, vec![1, 2, 3])), z);
    /// ```
    pub fn set_certainty(&mut self, c: Valuation<usize>) {
        if self.c < c && let Valuation::Finite(current_c) = self.c {
            self.variant.mut_raw().truncate(current_c);
        }
        self.c = c;
    }



    /// Calculate the roots of unity for given prime `p`.
    /// These are the solutions of `x^2 = 1` if `p = 2` and `x^(p-1) = 1` if `p > 2`.
    ///
    /// # Errors
    /// Returns any errors that [`AdicInteger::nth_root`] returns
    ///
    /// ```
    /// # use num::traits::Pow;
    /// # use adic::{Variety, ZAdic};
    /// assert_eq!(
    ///     Ok(Variety::new(vec![
    ///         ZAdic::new_approx(2, 6, vec![1, 0, 0, 0, 0, 0]),
    ///         ZAdic::new_approx(2, 6, vec![1, 1, 1, 1, 1, 1]),
    ///     ])),
    ///     ZAdic::roots_of_unity(2, 6)
    /// );
    /// assert_eq!(
    ///     Ok(Variety::new(vec![
    ///         ZAdic::new_approx(3, 6, vec![1, 0, 0, 0, 0, 0]),
    ///         ZAdic::new_approx(3, 6, vec![2, 2, 2, 2, 2, 2]),
    ///     ])),
    ///     ZAdic::roots_of_unity(3, 6)
    /// );
    /// assert_eq!(
    ///     Ok(Variety::new(vec![
    ///         ZAdic::new_approx(5, 6, vec![1, 0, 0, 0, 0, 0]),
    ///         ZAdic::new_approx(5, 6, vec![2, 1, 2, 1, 3, 4]),
    ///         ZAdic::new_approx(5, 6, vec![3, 3, 2, 3, 1, 0]),
    ///         ZAdic::new_approx(5, 6, vec![4, 4, 4, 4, 4 ,4]),
    ///     ])),
    ///     ZAdic::roots_of_unity(5, 6)
    /// );
    /// for a in ZAdic::roots_of_unity(5, 6)?.into_roots() {
    ///     assert!(a.pow(4).is_approx_one());
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn roots_of_unity<P>(p: P, precision: usize) -> AdicResult<Variety<Self>>
    where P: Into<Prime> {
        let p = p.into();
        let n = if p.is_two() { 2 } else { u32::from(p)-1 };
        ZAdic::one(p).nth_root(n, precision)
    }

    /// Calculate the Teichmuller characters for given prime `p`.
    /// These are the solutions of `x^p - x = 0`.
    ///
    /// # Errors
    /// Returns any errors that [`AdicInteger::nth_root`] returns
    ///
    /// ```
    /// # use num::traits::Pow;
    /// # use adic::{mapping::Mapping, Polynomial, Variety, ZAdic};
    /// assert_eq!(
    ///     Ok(Variety::new(vec![
    ///         ZAdic::new_approx(2, 6, vec![0, 0, 0, 0, 0, 0]),
    ///         ZAdic::new_approx(2, 6, vec![1, 0, 0, 0, 0, 0]),
    ///     ])),
    ///     ZAdic::teichmuller(2, 6)
    /// );
    /// assert_eq!(
    ///     Ok(Variety::new(vec![
    ///         ZAdic::new_approx(3, 6, vec![0, 0, 0, 0, 0, 0]),
    ///         ZAdic::new_approx(3, 6, vec![1, 0, 0, 0, 0, 0]),
    ///         ZAdic::new_approx(3, 6, vec![2, 2, 2, 2, 2, 2]),
    ///     ])),
    ///     ZAdic::teichmuller(3, 6)
    /// );
    /// assert_eq!(
    ///     Ok(Variety::new(vec![
    ///         ZAdic::new_approx(5, 6, vec![0, 0, 0, 0, 0, 0]),
    ///         ZAdic::new_approx(5, 6, vec![1, 0, 0, 0, 0, 0]),
    ///         ZAdic::new_approx(5, 6, vec![2, 1, 2, 1, 3, 4]),
    ///         ZAdic::new_approx(5, 6, vec![3, 3, 2, 3, 1, 0]),
    ///         ZAdic::new_approx(5, 6, vec![4, 4, 4, 4, 4 ,4]),
    ///     ])),
    ///     ZAdic::teichmuller(5, 6)
    /// );
    /// let teich_poly = Polynomial::new_with_prime(5, vec![0, -1, 0, 0, 0, 1]);
    /// for a in ZAdic::teichmuller(5, 6)?.into_roots() {
    ///     assert!((a.clone().pow(5) - a.clone()).is_approx_zero());
    ///     assert!(teich_poly.eval(a)?.is_approx_zero());
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn teichmuller<P>(p: P, precision: usize) -> AdicResult<Variety<Self>>
    where P: Into<Prime> {
        let p = p.into();
        let zero = ZAdic::zero(p);
        let one = ZAdic::one(p);
        let pm2 = usize::try_from(p.m2()).expect("prime -> usize conversion");
        let coeffs = once(zero.clone())
            .chain(once(-one.clone()))
            .chain(repeat_n(zero.clone(), pm2))
            .chain(once(one.clone()))
            .collect::<Vec<_>>();
        let poly = Polynomial::<ZAdic>::new(coeffs);
        let precision = isize::try_from(precision)?;
        let variety = poly.variety(precision)?;
        let zadic_variety = variety.try_into_integer()?;
        Ok(zadic_variety)
    }

}


impl AdicPrimitive for ZAdic {

    fn zero<P>(p: P) -> Self
    where P: Into<Prime> {
        Self::from(UAdic::zero(p))
    }
    fn one<P>(p: P) -> Self
    where P: Into<Prime> {
        Self::from(UAdic::one(p))
    }
    fn p(&self) -> Prime {
        self.variant.p()
    }

}


impl AdicInteger for ZAdic { }
impl HasDigitDisplay for ZAdic {
    type DigitDisplay = String;
    fn digit_display(&self) -> String {
        match self.c {
            Valuation::PosInf => self.variant.digit_display(),
            Valuation::Finite(c) => {
                let p = self.variant.p();
                let ds = self.variant.digits().chain(repeat(0)).take(c).map(|d| p.display_digit(d)).collect::<Vec<_>>();
                let digits = ds.into_iter().rev().join("");
                format!("...{digits}")
            },
        }
    }
}



#[cfg(test)]
mod tests {
    use num::traits::Pow;
    use crate::{
        error::AdicError,
        local_num::LocalZero,
        normed::{UltraNormed, Valuation},
        traits::{CanApproximate, HasApproximateDigits, HasDigits},
        Variety,
    };
    use super::{AdicInteger, AdicPrimitive, ZAdic};
    use Valuation::Finite;


    #[test]
    fn approximate_z_adic() {

        let zero_4 = zadic_approx!(5, 4, [0, 0, 0, 0]);
        assert!(!zero_4.is_local_zero());
        let one_2 = zadic_approx!(5, 2, [1]);
        let two_4 = zadic_approx!(5, 4, [2]);
        let five_3 = zadic_approx!(5, 3, [0, 1]);

        // Addition handles certainty with minimum certainty
        assert_eq!(Finite(2), (one_2.clone() + one_2.clone()).certainty());
        assert_eq!(Finite(2), (one_2.clone() + two_4.clone()).certainty());
        assert_eq!(Finite(4), (two_4.clone() + two_4.clone()).certainty());
        assert_eq!(Finite(2), (one_2.clone() + five_3.clone()).certainty());
        assert_eq!(Finite(3), (two_4.clone() + five_3.clone()).certainty());

        // Multiplication handles certainty with minimum significance
        assert_eq!(Finite(2), (one_2.clone() * one_2.clone()).certainty());
        assert_eq!(Finite(2), (one_2.clone() * two_4.clone()).certainty());
        assert_eq!(Finite(4), (two_4.clone() * two_4.clone()).certainty());
        assert_eq!(Finite(3), (one_2.clone() * five_3.clone()).certainty());
        assert_eq!(Finite(3), (two_4.clone() * five_3.clone()).certainty());
        assert_eq!(Finite(4), (five_3.clone() * five_3.clone()).certainty());

        // Certainty past valuation truncates
        let small_z = zadic_approx!(5, 3, [4, 3, 2, 1, 2]);
        assert_eq!(Finite(0), small_z.valuation());
        assert_eq!(Finite(3), small_z.certainty());
        assert_eq!(Finite(3), small_z.significance());
        assert_eq!(vec![4, 3, 2], small_z.digits().collect::<Vec<_>>());

    }

    #[test]
    fn empty_z_adic() {

        let empty = ZAdic::empty(5);
        let (unit, val) = empty.unit_and_valuation();
        assert_eq!(None, unit);
        assert_eq!(Valuation::Finite(0), val);
        assert_eq!(empty, empty.clone() + zadic_approx!(5, 4, [1]));
        assert_eq!(empty, empty.clone() * zadic_approx!(5, 4, [1]));
        assert_eq!(ZAdic::zero(5), empty.clone() * ZAdic::zero(5));

        let smaller_empty = ZAdic::from_prime(5).into_approximation(1);
        let (unit, val) = smaller_empty.unit_and_valuation();
        assert_eq!(None, unit);
        assert_eq!(Valuation::Finite(1), val);

    }

    #[test]
    fn nth_root() {

        let check = |a: &ZAdic, n: u32, precision: usize, roots: Vec<ZAdic>| {
            // Check each root powers to match a to at least precision digits
            for root in &roots {
                assert_eq!(a.approximation(precision), root.clone().pow(n));
            }
            // Check roots match the output of nth_root
            assert_eq!(Ok(Variety::new(roots)), a.nth_root(n, precision));
        };

        check(&ZAdic::from(uadic!(5, [1])), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);
        check(&zadic_approx!(5, 12, [1]), 2, 6, vec![
            zadic_approx!(5, 6, [1]),
            zadic_approx!(5, 6, [4, 4, 4, 4, 4, 4]),
        ]);

        check(&ZAdic::from(uadic!(5, [2])), 2, 6, vec![]);
        check(&zadic_approx!(5, 12, [2]), 2, 6, vec![]);

        check(&ZAdic::from(uadic!(7, [2])), 2, 6, vec![
            zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2]),
            zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4]),
        ]);
        check(&zadic_approx!(7, 12, [2]), 2, 6, vec![
            zadic_approx!(7, 6, [3, 1, 2, 6, 1, 2]),
            zadic_approx!(7, 6, [4, 5, 4, 0, 5, 4]),
        ]);

        assert!(matches!(
            zadic_approx!(7, 4, [2]).nth_root(2, 6),
            Err(AdicError::InappropriatePrecision(_))
        ));

    }

}
