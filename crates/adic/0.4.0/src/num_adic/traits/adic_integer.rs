use std::{
    fmt::{Debug, Display},
    hash::Hash,
    iter::{once, repeat, repeat_n},
};
use crate::{
    nth_root, num_nth_roots,
    AdicResult, AdicValuation, LazyDiv, Prime, UAdic, ZAdic, ZAdicVariety,
};
use super::{AdicApproximate, AdicNumber, AdicSized, HasDigits};


/// An adic number without fractional digits
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
/// # use adic::{uadic, AdicSized};
/// # use num::rational::Ratio;
/// assert_eq!(Ratio::new(1, 1), uadic!(5, [4, 1, 3, 2]).norm());
/// assert_eq!(Ratio::new(1, 25), uadic!(5, [0, 0, 3, 2]).norm());
/// ```
///
/// Many of the same operations are possible with these integers: addition, multiplication, powers, roots.
/// Division is possible, but it can take you out of the integers just like in the reals.
///
/// Perhaps surprisingly, roots of adic integers are also adic integers (if they exist).
/// While `sqrt(2) = {1.414..., -1.414...}` in the reals, giving digits to the right of the decimal,
/// in the 7-adics, `sqrt(2) = {...6213._7, ...0454._7}`.
///
/// This is because the rationals like (3/4) are representable in e.g. the 5-adics without nonzero digits to the right of the decimal.
/// Roots tend to fall "between" integers, which in the reals necessarily falls into the fraction space.
/// In the adics, the numbers "between" integers are just more integers!
///
/// ```
/// # use adic::{uadic, zadic_variety, AdicInteger};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// assert_eq!(
///     zadic_variety!(7, 4, [[3, 1, 2, 6], [4, 5, 4, 0]]),
///     uadic!(7, [2]).nth_root(2, 4)?
/// );
/// # Ok(()) }
/// ```
///
/// Another interesting observation is that there is no need for a negative sign.
/// In the reals, -1 is distinct and NEEDS a negative sign to be expressed.
/// In the 5-adics, `"-1" = ...44444._5`, since if you add 1 to that integer, you overflow and carry the 1 and overflow and carry...
///
/// ```
/// # use adic::{uadic, radic, AdicInteger};
/// let pos_one = radic!(5, [1], []);
/// let neg_one = radic!(5, [], [4]);
/// assert_eq!("1._5", pos_one.to_string());
/// assert_eq!("(4)._5", neg_one.to_string());
/// let zero = pos_one + neg_one;
/// assert_eq!("0._5", zero.to_string());
/// ```
///
/// <https://en.wikipedia.org/wiki/P-adic_number#p-adic_integers>
pub trait AdicInteger
where Self: Debug + Clone + PartialEq + Eq + Hash + Display + From<UAdic>
    + AdicNumber
    + AdicSized<ValuationRing = usize, AdicUnit = Self>
    + HasDigits<DigitIndex = usize>
    + std::ops::Add<Self, Output=Self>
    + std::ops::Mul<Self, Output=Self> + std::ops::Mul<u32, Output=Self>
    + std::ops::Div<Self, Output=LazyDiv<Self>> {

    // Constructors

    /// Create an `AdicInteger` representing a power of p
    ///
    /// ```
    /// # use adic::{uadic, AdicInteger, UAdic};
    /// assert_eq!(uadic!(5, [0, 0, 0, 1]), UAdic::p_power(5, 3));
    /// ```
    fn p_power<P, I>(p: P, n: I) -> Self
    where P: Into<Prime>, I: Into<AdicValuation<usize>> {
        match n.into() {
            AdicValuation::PosInf => Self::zero(p.into()),
            AdicValuation::Finite(n) => {
                Self::from(UAdic::new(p.into(), repeat_n(0, n).chain(once(1)).collect::<Vec<_>>()))
            }
        }
    }


    // Data fetch

    /// Get a vector of the digits, padding with zeros, to exactly n.
    /// See also: [`into_padded_digits`](Self::into_padded_digits)
    ///
    /// ```
    /// # use adic::{uadic, AdicInteger};
    /// assert_eq!(vec![2, 3, 1, 0, 0, 0], uadic!(5, [2, 3, 1]).padded_digits(6));
    /// ```
    fn padded_digits(&self, n: usize) -> Vec<u32> {
        self.digits().chain(repeat(0)).take(n).collect()
    }

    /// Get a vector of the digits, padding with zeros, to exactly n.
    /// See also: [`padded_digits`](Self::padded_digits)
    ///
    /// ```
    /// # use adic::{uadic, AdicInteger};
    /// assert_eq!(vec![2, 3, 1, 0, 0, 0], uadic!(5, [2, 3, 1]).into_padded_digits(6));
    /// ```
    fn into_padded_digits(self, n: usize) -> Vec<u32>
    where Self: Sized {
        self.into_digits().chain(repeat(0)).take(n).collect()
    }

    /// A string representing the digits of this adic number; used to Display the integer.
    ///
    /// ```
    /// # use adic::{iadic_neg, radic, uadic, zadic_approx, AdicInteger};
    /// assert_eq!("321", uadic!(5, [1, 2, 3]).digit_str());
    /// assert_eq!("(4)31", iadic_neg!(5, [1, 3]).digit_str());
    /// assert_eq!("(24)31", radic!(5, [1, 3], [4, 2]).digit_str());
    /// assert_eq!("...3412", zadic_approx!(5, 4, [2, 1, 4, 3]).digit_str());
    /// ```
    fn digit_str(&self) -> String;


    // Transformation

    /// Truncate an adic number's expansion to n.
    /// This can be thought of as the remainder `a % p^n`.
    /// See also: [`into_truncation`](Self::into_truncation)
    ///
    /// ```
    /// # use adic::{radic, uadic, AdicInteger};
    /// let u = uadic!(5, [1, 3, 2, 1, 2, 1, 2]);
    /// assert_eq!(u, u.truncation(9));
    /// assert_eq!(u, u.truncation(7));
    /// let r = radic!(5, [1, 3], [2, 1]);
    /// assert_eq!(u, r.truncation(7));
    /// ```
    fn truncation(&self, n: usize) -> UAdic {
        UAdic::new(self.p(), self.digits().take(n).collect())
    }

    /// Consume `AdicInteger` and get the truncation.
    /// See also: [`truncation`](Self::truncation)
    ///
    /// ```
    /// # use adic::{radic, uadic, AdicInteger};
    /// let u = uadic!(5, [1, 3, 2, 1, 2, 1, 2]);
    /// assert_eq!(u, u.clone().into_truncation(9));
    /// assert_eq!(u, u.clone().into_truncation(7));
    /// let r = radic!(5, [1, 3], [2, 1]);
    /// assert_eq!(u, r.into_truncation(7));
    /// ```
    fn into_truncation(self, n: usize) -> UAdic {
        UAdic::new(self.p(), self.into_digits().take(n).collect())
    }

    /// Approximate an adic number's expansion to n digits.
    /// See also: [`into_approximation`](Self::into_approximation)
    ///
    /// ```
    /// # use adic::{radic, uadic, zadic_approx, AdicInteger};
    /// let u = uadic!(5, [1, 3, 2, 1, 2, 1, 2]);
    /// let z = zadic_approx!(5, 9, [1, 3, 2, 1, 2, 1, 2, 0, 0]);
    /// let zs = zadic_approx!(5, 5, [1, 3, 2, 1, 2]);
    /// assert_eq!(z, u.approximation(9));
    /// assert_eq!(zs, u.approximation(5));
    /// assert_eq!(zs, z.approximation(5));
    /// assert_eq!(zs, zs.approximation(9));
    /// let r = radic!(5, [1, 3], [2, 1]);
    /// assert_eq!(zs, r.approximation(5));
    /// ```
    fn approximation(&self, n: usize) -> ZAdic
    where Self: AdicApproximate {
        let c = match self.certainty() {
            AdicValuation::PosInf => n,
            AdicValuation::Finite(v) => std::cmp::min(v, n),
        };
        ZAdic::new_approx(self.p(), c, self.digits().take(c).collect())
    }

    /// Consume `AdicInteger` and get the approximation.
    /// See also: [`approximation`](Self::approximation)
    ///
    /// ```
    /// # use adic::{radic, uadic, zadic_approx, AdicInteger};
    /// let u = uadic!(5, [1, 3, 2, 1, 2, 1, 2]);
    /// let z = zadic_approx!(5, 9, [1, 3, 2, 1, 2, 1, 2, 0, 0]);
    /// let zs = zadic_approx!(5, 5, [1, 3, 2, 1, 2]);
    /// assert_eq!(z, u.clone().into_approximation(9));
    /// assert_eq!(zs, u.clone().into_approximation(5));
    /// assert_eq!(zs, z.clone().into_approximation(5));
    /// assert_eq!(zs, zs.clone().into_approximation(9));
    /// let r = radic!(5, [1, 3], [2, 1]);
    /// assert_eq!(zs, r.into_approximation(5));
    /// ```
    fn into_approximation(self, n: usize) -> ZAdic
    where Self: AdicApproximate {
        let c = match self.certainty() {
            AdicValuation::PosInf => n,
            AdicValuation::Finite(v) => std::cmp::min(v, n),
        };
        ZAdic::new_approx(self.p(), c, self.into_digits().take(c).collect())
    }

    /// Split adic into digits [0, n) and [n, ...).
    /// This splits the number into remainder and quotient.
    /// See also: [`into_split`](Self::into_split)
    ///
    /// `a = (a % p^n) + p^n * (a // p^n)`
    ///
    /// ```
    /// # use adic::{iadic_neg, radic, uadic, zadic_approx, AdicInteger};
    /// assert_eq!((uadic!(5, [1, 2]), uadic!(5, [3, 4])), uadic!(5, [1, 2, 3, 4]).split(2));
    /// assert_eq!((uadic!(5, [1, 2, 4, 4]), iadic_neg!(5, [])), iadic_neg!(5, [1, 2]).split(4));
    /// assert_eq!((uadic!(7, [1, 2, 3, 4, 5, 3]), radic!(7, [], [4, 5, 3])), radic!(7, [1, 2], [3, 4, 5]).split(6));
    /// assert_eq!((uadic!(5, [1, 2]), zadic_approx!(5, 2, [3, 4])), zadic_approx!(5, 4, [1, 2, 3, 4]).split(2));
    /// ```
    ///
    /// Note especially the approximate behavior: splitting past certainty gives a `UAdic` and an empty `ZAdic`!
    ///
    /// ```
    /// # use adic::{iadic_neg, radic, uadic, zadic_approx, AdicInteger};
    /// assert_eq!((uadic!(5, [1, 2]), zadic_approx!(5, 0, [])), zadic_approx!(5, 2, [1, 2]).split(4));
    /// ```
    fn split(&self, n: usize) -> (UAdic, Self) {
        self.clone().into_split(n)
    }

    /// Split adic into digits [0, n) and [n, ...).
    /// This splits the number into remainder and quotient.
    /// See also: [`split`](Self::split)
    ///
    /// `a = (a % p^n) + p^n * (a // p^n)`
    ///
    /// ```
    /// # use adic::{iadic_neg, radic, uadic, zadic_approx, AdicInteger};
    /// assert_eq!((uadic!(5, [1, 2]), uadic!(5, [3, 4])), uadic!(5, [1, 2, 3, 4]).into_split(2));
    /// assert_eq!((uadic!(5, [1, 2, 4, 4]), iadic_neg!(5, [])), iadic_neg!(5, [1, 2]).into_split(4));
    /// assert_eq!((uadic!(7, [1, 2, 3, 4, 5, 3]), radic!(7, [], [4, 5, 3])), radic!(7, [1, 2], [3, 4, 5]).into_split(6));
    /// assert_eq!((uadic!(5, [1, 2]), zadic_approx!(5, 2, [3, 4])), zadic_approx!(5, 4, [1, 2, 3, 4]).into_split(2));
    /// ```
    ///
    /// Note especially the approximate behavior: splitting past certainty gives a `UAdic` and an empty `ZAdic`!
    ///
    /// ```
    /// # use adic::{iadic_neg, radic, uadic, zadic_approx, AdicInteger};
    /// assert_eq!((uadic!(5, [1, 2]), zadic_approx!(5, 0, [])), zadic_approx!(5, 2, [1, 2]).into_split(4));
    /// ```
    fn into_split(self, n: usize) -> (UAdic, Self);

    #[must_use]
    /// Divide an adic number by p^n.
    /// This can be thought of as the quotient a // p^n
    ///
    /// ```
    /// # use adic::{uadic, AdicInteger};
    /// let u = uadic!(5, [1, 2, 3, 4]);
    /// let q = u.quotient(2);
    /// assert_eq!(uadic!(5, [3, 4]), q);
    /// ```
    fn quotient(&self, n: usize) -> Self {
        self.split(n).1
    }

    #[must_use]
    /// Divide an adic number by p^n.
    /// This can be thought of as the quotient a // p^n
    ///
    /// ```
    /// # use adic::{uadic, AdicInteger};
    /// let u = uadic!(5, [1, 2, 3, 4]);
    /// let q = u.into_quotient(2);
    /// assert_eq!(uadic!(5, [3, 4]), q);
    /// ```
    fn into_quotient(self, n: usize) -> Self {
        self.into_split(n).1
    }


    /// Calculate the n-th root, to {precision} digits,
    ///  using [Hensel lifting](https://en.wikipedia.org/wiki/Hensel%27s_lemma#Hensel_lifting).
    ///
    /// This is a specific case of [`AdicPolynomial::variety`](crate::AdicPolynomial::variety),
    ///  for the polynomial `f(x) = x^n - a = 0`.
    ///
    /// If n has a factor of p, then the algorithm is more complicated because you have to take into account more digits.
    ///
    /// 7-adic `sqrt(2)` has two solutions, starting with 3 and with 4
    /// ```
    /// # use adic::{uadic, zadic_variety, AdicInteger};
    /// let seven_adic_two = uadic!(7, [2]);
    /// let variety = seven_adic_two.nth_root(2, 6).unwrap();
    /// let expected = zadic_variety!(7, 6, [
    ///     [3, 1, 2, 6, 1, 2],
    ///     [4, 5, 4, 0, 5, 4],
    /// ]);
    /// assert_eq!(expected, variety);
    /// assert_eq!("variety(...216213._7, ...450454._7)", variety.to_string());
    /// ```
    ///
    /// 5-adic `sqrt(2)` has no solutions, as seen since no element of F_5 has `x^2 = 2 mod 5`
    /// ```
    /// # use adic::{uadic, AdicInteger, ZAdicVariety};
    /// let five_adic_two = uadic!(5, [2]);
    /// let variety = five_adic_two.nth_root(2, 6);
    /// let expected = ZAdicVariety::empty(5);
    /// assert_eq!(Ok(expected), variety);
    /// ```
    ///
    /// Every (p > 2) p-adic has (p-1) roots of unity, called "Teichmuller characters"
    /// ```
    /// # use adic::{uadic, zadic_variety, AdicInteger};
    /// let five_adic_one = uadic!(5, [1]);
    /// let variety = five_adic_one.nth_root(4, 6);
    /// let expected = zadic_variety!(5, 6, [
    ///     [1, 0, 0, 0, 0, 0],
    ///     [2, 1, 2, 1, 3, 4],
    ///     [3, 3, 2, 3, 1, 0],
    ///     [4, 4, 4, 4, 4, 4],
    /// ]);
    /// assert_eq!(Ok(expected), variety);
    /// ```
    ///
    /// 5-adic `sqrt(32) = sqrt(2^5)` has ONE solution: 2.
    /// ```
    /// # use adic::{uadic, zadic_variety, AdicInteger};
    /// let thirty_two = uadic!(5, [2, 1, 1]);
    /// let variety = thirty_two.nth_root(5, 6);
    /// let expected = zadic_variety!(5, 6, [[2]]);
    /// assert_eq!(Ok(expected), variety);
    /// ```
    ///
    /// # Errors
    /// 1. `AdicInteger`'s `certainty` is not high enough for desired `precision`
    /// 2. n == 0
    ///
    /// # Panics
    /// Panics if certainty does not behave as expected
    fn nth_root(&self, n: u32, precision: usize) -> AdicResult<ZAdicVariety>
    where Self: AdicApproximate {
        nth_root(self, n, precision)
    }

    /// Return the number of n-th roots of this `AdicInteger`
    ///
    /// ```
    /// use adic::{uadic, AdicInteger};
    /// assert_eq!(Ok(0), uadic!(7, [2]).num_nth_roots(0));
    /// assert_eq!(Ok(1), uadic!(7, [2]).num_nth_roots(1));
    /// assert_eq!(Ok(2), uadic!(7, [2]).num_nth_roots(2));
    /// assert_eq!(Ok(0), uadic!(7, [2]).num_nth_roots(3));
    /// assert_eq!(Ok(2), uadic!(7, [2]).num_nth_roots(4));
    /// assert_eq!(Ok(1), uadic!(7, [2]).num_nth_roots(5));
    /// assert_eq!(Ok(0), uadic!(7, [2]).num_nth_roots(6));
    /// assert_eq!(Ok(0), uadic!(7, [2]).num_nth_roots(7));
    /// ```
    ///
    /// # Errors
    /// Errors if rootfinding encounters problems, e.g. heavily degenerate roots
    fn num_nth_roots(&self, n: u32) -> AdicResult<usize>
    where Self: Into<ZAdic> {
        num_nth_roots(self, n)
    }

}
