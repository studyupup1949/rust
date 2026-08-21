use std::fmt::Display;
use crate::{
    error::AdicResult,
    normed::UltraNormed,
    Polynomial, UAdic, Variety, ZAdic,
};
use super::{AdicPrimitive, CanApproximate, CanTruncate, HasApproximateDigits};


/// An adic number with only integer digits
///
/// # Adic Integer
///
/// Structs implementing this trait represent adic integers, representable as base-p digital expansions,
/// with a possibly-infinite number of digits to the left of a decimal point.
///
/// There is a distinction between adic NUMBERS and adic INTEGERS.
/// Adic integers are adic numbers without digits to the right of the decimal.
/// These are numbers without powers of p in their denominator, if viewed akin to a rational.
/// Using the p-adic norm, these are exactly the numbers where the valuation v >= 0, i.e. `|x| = p^(-v) <= 1`.
/// In the reals, all nonzero integers have a size greater than or equal to 1.
/// In the adics, it is the opposite; all integers have size less than or equal to 1.
///
/// ```
/// # use adic::{normed::Normed, UAdic};
/// # use num::rational::Ratio;
/// assert_eq!(Ratio::new(1, 1), UAdic::new(5, vec![4, 1, 3, 2]).norm());
/// assert_eq!(Ratio::new(1, 25), UAdic::new(5, vec![0, 0, 3, 2]).norm());
/// ```
///
/// Many of the same operations are possible with these integers: addition, multiplication, powers, roots.
/// Division is possible, but it can take you out of the integers just like in the reals.
///
/// Perhaps surprisingly, roots of adic integers are also adic integers (if they exist).
/// While `sqrt(2) = {1.414..., -1.414...}` in the reals, giving fractional digits,
/// in the 7-adics, `sqrt(2) = {...6213._7, ...0454._7}`.
///
/// This is because the rationals like (3/4) are representable in e.g. the 5-adics without nonzero digits to the right of the decimal.
/// Roots tend to fall "between" integers, which in the reals necessarily falls into the fraction space.
/// In the adics, the numbers "between" integers are just more integers!
///
/// ```
/// # use adic::{traits::AdicInteger, UAdic, Variety, ZAdic};
/// assert_eq!(
///     "variety(...6213._7, ...0454._7)",
///     UAdic::new(7, vec![2]).nth_root(2, 4)?.to_string()
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Another interesting observation is that there is no need for a negative sign.
/// In the reals, -1 is distinct and NEEDS a negative sign to be expressed.
/// In the 5-adics, `"-1" = ...44444._5`, since if you add 1 to that integer, you overflow and carry the 1 and overflow and carry...
///
/// ```
/// # use adic::{traits::AdicPrimitive, EAdic};
/// let one = EAdic::one(5);
/// let neg_one = -one.clone();
/// assert_eq!("1._5", one.to_string());
/// assert_eq!("(4)._5", neg_one.to_string());
/// assert_eq!("0._5", (one.clone() + neg_one.clone()).to_string());
/// assert_eq!("2._5", (one.clone() - neg_one.clone()).to_string());
/// assert_eq!("(4)3._5", (-one.clone() + neg_one.clone()).to_string());
/// ```
///
/// <https://en.wikipedia.org/wiki/P-adic_number#p-adic_integers>
pub trait AdicInteger
where Self: AdicPrimitive
    + Display
    + UltraNormed<Unit = Self>
    + HasApproximateDigits<DigitIndex = usize>
    + CanTruncate<Quotient=Self, Truncation=UAdic> + CanApproximate<Approximation = ZAdic> {

    /// Calculate the n-th root, to `precision` digits,
    ///  using [Hensel lifting](https://en.wikipedia.org/wiki/Hensel%27s_lemma#Hensel_lifting).
    ///
    /// This is a specific case of [`Polynomial::variety`](crate::Polynomial::variety),
    ///  for the polynomial `f(x) = x^n - a = 0`.
    ///
    /// If n has a factor of p, then the algorithm is more complicated because you have to take into account more digits.
    ///
    /// 7-adic `sqrt(2)` has two solutions, starting with 3 and with 4
    /// ```
    /// # use adic::{traits::{AdicInteger, PrimedFrom}, UAdic, Variety, ZAdic};
    /// let seven_adic_two = UAdic::primed_from(7, 2);
    /// let variety = seven_adic_two.nth_root(2, 6).unwrap();
    /// let expected = Variety::new(vec![
    ///     ZAdic::new_approx(7, 6, vec![3, 1, 2, 6, 1, 2]),
    ///     ZAdic::new_approx(7, 6, vec![4, 5, 4, 0, 5, 4]),
    /// ]);
    /// assert_eq!(expected, variety);
    /// assert_eq!("variety(...216213._7, ...450454._7)", variety.to_string());
    /// ```
    ///
    /// 5-adic `sqrt(2)` has no solutions, as seen since no element of F_5 has `x^2 = 2 mod 5`
    /// ```
    /// # use adic::{traits::{AdicInteger, PrimedFrom}, UAdic, Variety};
    /// let five_adic_two = UAdic::primed_from(5, 2);
    /// let variety = five_adic_two.nth_root(2, 6);
    /// let expected = Variety::empty();
    /// assert_eq!(Ok(expected), variety);
    /// ```
    ///
    /// Every (p > 2) p-adic has (p-1) roots of unity (related to Teichmuller characters)
    /// ```
    /// # use adic::{traits::{AdicInteger, AdicPrimitive}, UAdic, Variety, ZAdic};
    /// let five_adic_one = UAdic::one(5);
    /// let variety = five_adic_one.nth_root(4, 6);
    /// let expected = Variety::new(vec![
    ///     ZAdic::new_approx(5, 6, vec![1, 0, 0, 0, 0, 0]),
    ///     ZAdic::new_approx(5, 6, vec![2, 1, 2, 1, 3, 4]),
    ///     ZAdic::new_approx(5, 6, vec![3, 3, 2, 3, 1, 0]),
    ///     ZAdic::new_approx(5, 6, vec![4, 4, 4, 4, 4, 4]),
    /// ]);
    /// assert_eq!(Ok(expected), variety);
    /// ```
    ///
    /// 5-adic `fifth_rt(32) = fifth_rt(2^5)` has ONE solution: 2.
    /// ```
    /// # use adic::{traits::AdicInteger, UAdic, Variety, ZAdic};
    /// let thirty_two = UAdic::new(5, vec![2, 1, 1]);
    /// let variety = thirty_two.nth_root(5, 6);
    /// let expected = Variety::new(vec![ZAdic::new_approx(5, 6, vec![2])]);
    /// assert_eq!(Ok(expected), variety);
    /// ```
    ///
    /// # Errors
    /// 1. `AdicInteger`'s `certainty` is not high enough for desired `precision`
    /// 2. n == 0
    ///
    /// # Panics
    /// Panics if certainty does not behave as expected
    fn nth_root(&self, n: u32, precision: usize) -> AdicResult<Variety<ZAdic>>
    where Self: Into<ZAdic> {
        let adic_int: ZAdic = self.clone().into();
        let precision = isize::try_from(precision)?;
        let variety = Polynomial::<ZAdic>::nth_root_polynomial(adic_int, n).variety(precision)?;
        let zadic_variety = variety.try_into_integer()?;
        Ok(zadic_variety)
    }

    /// Return the number of n-th roots of this `AdicInteger`
    ///
    /// ```
    /// # use adic::{traits::{AdicInteger, PrimedFrom}, UAdic};
    /// let two = UAdic::primed_from(7, 2);
    /// assert_eq!(Ok(0), two.num_nth_roots(0));
    /// assert_eq!(Ok(1), two.num_nth_roots(1));
    /// assert_eq!(Ok(2), two.num_nth_roots(2));
    /// assert_eq!(Ok(0), two.num_nth_roots(3));
    /// assert_eq!(Ok(2), two.num_nth_roots(4));
    /// assert_eq!(Ok(1), two.num_nth_roots(5));
    /// assert_eq!(Ok(0), two.num_nth_roots(6));
    /// assert_eq!(Ok(0), two.num_nth_roots(7));
    /// ```
    ///
    /// # Errors
    /// Errors if rootfinding encounters problems, e.g. heavily degenerate roots
    fn num_nth_roots(&self, n: u32) -> AdicResult<usize>
    where Self: Into<ZAdic> {
        let adic_int: ZAdic = self.clone().into();
        Polynomial::<ZAdic>::nth_root_polynomial(adic_int.clone(), n).variety_size()
    }

}
