use crate::{
    divisible::Prime,
    error::{AdicError, AdicResult},
    normed::{Valuation, ValuationRing},
    traits::{AdicInteger, AdicPrimitive, CanTruncate},
    Polynomial, UAdic, Variety, ZAdic,
};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Fractional adic number
///
/// The struct holds an adic integer and a valuation.
/// Digitally, there are `-valuation` digits to the right of the decimal.
/// With this, you can represent any adic number.
///
/// The adic integer is generic and so can be e.g.
/// - natural number [`UAdic`](crate::UAdic)
/// - exact number [`EAdic`](crate::EAdic)
/// - approximate number [`ZAdic`](crate::ZAdic)
///
/// ```
/// # use num::rational::Ratio;
/// # use adic::{normed::{Normed, Valuation}, EAdic, QAdic, UAdic};
/// let twenty_three_and_11_25 = QAdic::new(UAdic::new(5, vec![1, 2, 3, 4]), Valuation::Finite(-2));
/// assert_eq!("43.21_5", twenty_three_and_11_25.to_string());
/// let fifty = QAdic::new(UAdic::new(5, vec![0, 2]), 1);
/// assert_eq!("200._5", fifty.to_string());
/// let neg_one_tenth = QAdic::new(EAdic::new_repeating(5, vec![], vec![2]), -1);
/// assert_eq!("(2).2_5", neg_one_tenth.to_string());
///
/// assert_eq!(
///     QAdic::new(UAdic::new(5, vec![1, 2, 4, 1, 1]), -2),
///     QAdic::new(UAdic::new(5, vec![1, 2, 3, 4]), -2) + QAdic::new(UAdic::new(5, vec![1, 2]), 0)
/// );
/// assert_eq!(
///     QAdic::new(UAdic::new(5, vec![2, 2]), -3),
///     QAdic::new(UAdic::new(5, vec![3]), -2) * QAdic::new(UAdic::new(5, vec![4]), -1)
/// );
///
/// assert_eq!(Ratio::new(1, 1), QAdic::new(UAdic::new(5, vec![4, 1, 3, 2]), 0).norm());
/// assert_eq!(Ratio::new(1, 25), QAdic::new(UAdic::new(5, vec![4, 1, 3, 2]), 2).norm());
/// assert_eq!(Ratio::new(25, 1), QAdic::new(UAdic::new(5, vec![4, 1, 3, 2]), -2).norm());
/// ```
///
/// This struct represents adic numbers as base-p digital expansions,
/// with a possibly-infinite number of digits to the left of a decimal point
/// and a finite number of digits to the right.
///
/// [`AdicIntegers`](crate::traits::AdicInteger) are similar, but without digits to the right of the decimal.
/// `QAdics` can represent all rational numbers as well as many irrational (distinct from the real number irrationals).
/// Using the p-adic norm, these numbers have valuation -inf < v <= inf, i.e. `|x| = p^(-v)`.
///
/// ```
/// # use adic::{traits::{AdicInteger, AdicPrimitive}, EAdic, QAdic, Variety, ZAdic};
/// let neg_one = -EAdic::one(5);
/// let neg_one_fifth = QAdic::new(neg_one.clone(), -1);
/// let neg_one_twenty_fifth = QAdic::new(neg_one.clone(), -2);
/// let sqrt_neg_one = ZAdic::new_approx(5, 6, vec![2, 1, 2, 1, 3, 4]);
/// assert_eq!("...431212._5", sqrt_neg_one.to_string());
/// let sqrt_neg_one_twenty_fifth = QAdic::new(sqrt_neg_one.clone(), -1);
/// assert_eq!("...43121.2_5", sqrt_neg_one_twenty_fifth.to_string());
/// assert_eq!(
///     Ok(Variety::new(vec![sqrt_neg_one.clone(), -sqrt_neg_one.clone()])),
///     neg_one.nth_root(2, 6)
/// );
/// assert!(neg_one_fifth.nth_root(2, 5).is_ok_and(|variety| variety.is_empty()));
/// assert_eq!(
///     Ok(Variety::new(vec![sqrt_neg_one_twenty_fifth.clone(), -sqrt_neg_one_twenty_fifth])),
///     neg_one_twenty_fifth.nth_root(2, 5)
/// );
/// ```
///
/// <https://en.wikipedia.org/wiki/P-adic_number>
///
/// # Panics
/// Many methods will panic if a provided prime `p` is not prime or digits are outside of `[0, p)`.
pub struct QAdic<A>
where A: AdicInteger {
    // This member should be (1) a unit, (2) zero, or (3) empty (no certainty)
    pub (super) internal_int: A,
    pub (super) valuation: Valuation<isize>,
}


impl<A> QAdic<A>
where A: AdicInteger {

    /// Create an adic number with the given digits and valuation
    pub fn new<V>(adic_int: A, valuation: V) -> Self
    where V: Into<Valuation<isize>> {

        let p = adic_int.p();
        let (adic_unit, int_valuation) = adic_int.unit_and_valuation();
        match (valuation.into(), int_valuation) {
            (Valuation::Finite(v), Valuation::Finite(iv)) => {
                let iv = iv.try_into_isize().expect("valuation conversion");
                let adjusted_valuation = v + iv;
                let internal_int = if let Some(unit) = adic_unit {
                    unit
                } else if iv > 0 {
                    let uv = iv.try_into_usize().expect("valuation conversion");
                    adic_int.quotient(uv)
                } else {
                    let val_adjust = A::from_prime_power((p, (-iv).try_into_u32().expect("valuation conversion")));
                    adic_int * val_adjust
                };
                Self {
                    internal_int,
                    valuation: adjusted_valuation.into(),
                }
            },
            _ => Self::zero(p),
        }

    }


    // Transformation

    /// Split `QAdic` into fraction (as `QAdic<UAdic>`) and integer (as `A`)
    ///
    /// ```
    /// # use adic::{EAdic, QAdic};
    /// let r = EAdic::new_repeating(7, vec![1, 2], vec![3, 4, 5]);
    /// assert_eq!("(543)21._7", r.to_string());
    /// let q = QAdic::new(r, -6);
    /// assert_eq!("(354).354321_7", q.to_string());
    /// let (q_frac, q_int) = q.frac_and_int();
    /// assert_eq!("0.354321_7", q_frac.to_string());
    /// assert_eq!("(354)._7", q_int.to_string());
    /// ```
    pub fn frac_and_int(&self) -> (QAdic<UAdic>, A) {
        self.split(0)
    }

    /// Create an adic number with the given digits and zero valuation
    pub fn from_integer(adic_int: A) -> Self {
        Self::new(adic_int, 0)
    }

    /// Try to convert into an [`AdicInteger`], returning error if there are fractional digits
    ///
    /// ```
    /// # use adic::{error::AdicError, EAdic, QAdic};
    /// let r = EAdic::new_repeating(7, vec![1, 2], vec![3, 4, 5]);
    /// assert_eq!("(543)21._7", r.to_string());
    /// let q = QAdic::new(r.clone(), 3);
    /// assert_eq!("(543)21000._7", q.to_string());
    /// let q_int = q.try_into_integer();
    /// assert_eq!(Ok("(543)21000._7".to_string()), q_int.map(|a| a.to_string()));
    /// let q = QAdic::new(r, -3);
    /// assert_eq!("(354).321_7", q.to_string());
    /// let q_int = q.try_into_integer();
    /// assert_eq!(Err(AdicError::AdicIntegerExpected), q_int);
    /// ```
    pub fn try_into_integer(self) -> AdicResult<A> {
        let (frac, int) = self.frac_and_int();
        if frac == QAdic::zero(self.p()) {
            Ok(int)
        } else {
            Err(AdicError::AdicIntegerExpected)
        }

    }


    /// Calculate the n-th root, to `precision` digits,
    ///  using [Hensel lifting](https://en.wikipedia.org/wiki/Hensel%27s_lemma#Hensel_lifting).
    ///
    /// This is a specific case of [`Polynomial::variety`](crate::Polynomial::variety),
    ///  for the polynomial `f(x) = x^n - a = 0`.
    ///
    /// If n has a factor of p, then the algorithm is more complicated because you have to take into account more digits.
    ///
    /// 7-adic `sqrt(1/98)` has two solutions, starting with 3 and with 4
    /// ```
    /// # use adic::{traits::AdicInteger, QAdic, UAdic, Variety, ZAdic};
    /// let seven_adic_2_49 = QAdic::new(UAdic::new(7, vec![2]), -2);
    /// let variety = seven_adic_2_49.nth_root(2, 6).unwrap();
    /// let expected = Variety::new(vec![
    ///     QAdic::new(ZAdic::new_approx(7, 7, vec![3, 1, 2, 6, 1, 2, 1]), -1),
    ///     QAdic::new(ZAdic::new_approx(7, 7, vec![4, 5, 4, 0, 5, 4, 5]), -1),
    /// ]);
    /// assert_eq!(expected, variety);
    /// assert_eq!("variety(...121621.3_7, ...545045.4_7)", variety.to_string());
    /// ```
    ///
    /// # Errors
    /// 1. `QAdic`'s `certainty` is not high enough for desired `precision`
    /// 2. n == 0
    ///
    /// # Panics
    /// Panics if certainty does not behave as expected
    pub fn nth_root(&self, n: u32, precision: isize) -> AdicResult<Variety<QAdic<ZAdic>>>
    where Self: Into<QAdic<ZAdic>> {
        let adic_frac: QAdic<ZAdic> = self.clone().into();
        let variety = Polynomial::<QAdic<ZAdic>>::nth_root_polynomial(adic_frac, n).variety(precision)?;
        Ok(variety)
    }

    /// Return the number of n-th roots of this `QAdic`
    ///
    /// ```
    /// # use num::Rational32;
    /// # use adic::{traits::PrimedFrom, EAdic, QAdic};
    /// let two_49ths = QAdic::<EAdic>::primed_from(7, Rational32::new(2, 49));
    /// assert_eq!(Ok(0), two_49ths.num_nth_roots(0));
    /// assert_eq!(Ok(1), two_49ths.num_nth_roots(1));
    /// assert_eq!(Ok(2), two_49ths.num_nth_roots(2));
    /// assert_eq!(Ok(0), two_49ths.num_nth_roots(3));
    /// assert_eq!(Ok(0), two_49ths.num_nth_roots(4));
    /// assert_eq!(Ok(0), two_49ths.num_nth_roots(5));
    /// assert_eq!(Ok(0), two_49ths.num_nth_roots(6));
    /// assert_eq!(Ok(0), two_49ths.num_nth_roots(7));
    /// ```
    ///
    /// # Errors
    /// Errors if rootfinding encounters problems, e.g. heavily degenerate roots
    pub fn num_nth_roots(&self, n: u32) -> AdicResult<usize>
    where Self: Into<QAdic<ZAdic>> {
        let adic_frac: QAdic<ZAdic> = self.clone().into();
        Polynomial::<QAdic<ZAdic>>::nth_root_polynomial(adic_frac.clone(), n).variety_size()
    }

}


impl QAdic<ZAdic> {

    /// Create the empty `QAdic<ZAdic>` with valuation `v`
    pub fn empty<P, V>(p: P, v: V) -> Self
    where P: Into<Prime>, V: Into<Valuation<isize>> {
        QAdic::new(ZAdic::empty(p), v)
    }

}


impl<A> AdicPrimitive for QAdic<A>
where A: AdicInteger {

    fn zero<P>(p: P) -> Self
    where P: Into<Prime> {
        Self {
            internal_int: A::zero(p),
            valuation: Valuation::PosInf,
        }
    }
    fn one<P>(p: P) -> Self
    where P: Into<Prime> {
        Self {
            internal_int: A::one(p),
            valuation: Valuation::Finite(0),
        }
    }
    fn p(&self) -> Prime {
        self.internal_int.p()
    }

}
