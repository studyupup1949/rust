use std::{
    fmt::{Debug, Display},
    hash::Hash,
    iter::{once, repeat},
};
use num::{rational::Ratio, Zero};
use crate::{nth_root, AdicError, LazyIntDiv, ZAdicValuation, ZAdicVariety};
use super::{UAdic, IAdic, ZAdic};


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
/// # use adic::{uadic, AdicInteger};
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
    + std::ops::Add<Self, Output=Self>
    + std::ops::Mul<Self, Output=Self> + std::ops::Mul<u32, Output=Self>
    + std::ops::Div<Self, Output=LazyIntDiv<Self>> {

    // Constructors

    /// Create the `zero` adic number
    ///
    /// ```
    /// # use adic::{AdicInteger, UAdic};
    /// assert_eq!("0._5", UAdic::zero(5).to_string());
    /// ```
    fn zero(p: u32) -> Self;

    /// Create the `one` adic number
    ///
    /// ```
    /// # use adic::{AdicInteger, UAdic};
    /// assert_eq!("1._5", UAdic::one(5).to_string());
    /// ```
    fn one(p: u32) -> Self;

    /// Create `AdicInteger` from u32
    ///
    /// ```
    /// # use adic::{AdicInteger, IAdic, RAdic, UAdic, ZAdic};
    /// assert_eq!("123._5", UAdic::from_u32(5, 38).to_string());
    /// assert_eq!("123._5", IAdic::from_u32(5, 38).to_string());
    /// assert_eq!("123._5", RAdic::from_u32(5, 38).to_string());
    /// assert_eq!("123._5", ZAdic::from_u32(5, 38).to_string());
    /// ```
    fn from_u32(p: u32, mut n: u32) -> Self {
        let mut digits = vec![];
        while n != 0 {
            digits.push(n % p);
            n = n / p;
        }
        Self::from(UAdic::new(p, digits))
    }

    /// Create an `AdicInteger` representing a power of p
    ///
    /// ```
    /// # use adic::{uadic, AdicInteger, UAdic};
    /// assert_eq!(uadic!(5, [0, 0, 0, 1]), UAdic::p_power(5, 3));
    /// ```
    fn p_power(p: u32, n: usize) -> Self {
        Self::from(UAdic::new(p, repeat(0).take(n).chain(once(1)).collect::<Vec<_>>()))
    }


    // Data fetch

    /// Prime for this adic
    fn p(&self) -> u32;

    /// The number of digits this integer ultimately has, finite or infinite
    ///
    /// ```
    /// # use adic::{uadic, radic, AdicInteger, ZAdicValuation};
    /// assert_eq!(ZAdicValuation::Finite(3), uadic!(5, [2, 1, 3, 0]).num_digits());
    /// assert_eq!(ZAdicValuation::PosInf, radic!(5, [2, 1], [3, 0]).num_digits());
    /// ```
    fn num_digits(&self) -> ZAdicValuation;

    /// Gets the digit at this coefficient of p^n; error if it is beyond known digits (certainty)
    ///
    /// ```
    /// # use adic::{uadic, zadic_approx, AdicError, AdicInteger};
    /// let u = uadic!(5, [2, 1, 3]);
    /// assert_eq!([Ok(2), Ok(3), Ok(0)], [u.digit(0), u.digit(2), u.digit(4)]);
    /// let z = zadic_approx!(5, 4, [2, 1, 3]);
    /// assert_eq!([Ok(2), Ok(3)], [z.digit(0), z.digit(2)]);
    /// assert!(matches!(z.digit(4), Err(AdicError::InappropriatePrecision(_))));
    /// ```
    ///
    /// # Errors
    /// Returns error if `n > self.certainty()`
    fn digit(&self, n: usize) -> Result<u32, AdicError>;

    /// Digits for this adic, from one's place to p to p^2, etc.
    ///
    /// ```
    /// # use adic::{radic, uadic, AdicInteger};
    /// let u = uadic!(5, [2, 1, 3]);
    /// assert_eq!(vec![2, 1, 3], u.digits().cloned().collect::<Vec<_>>());
    /// let r = radic!(5, [2, 1], [3]);
    /// assert_eq!(vec![2, 1, 3, 3, 3, 3], r.digits().take(6).cloned().collect::<Vec<_>>());
    /// ```
    fn digits(&self) -> impl Iterator<Item=&u32>;

    /// Consume `AdicInteger` and get the digits iterator
    ///
    /// ```
    /// # use adic::{radic, uadic, AdicInteger};
    /// let u = uadic!(5, [2, 1, 3]);
    /// assert_eq!(vec![2, 1, 3], u.into_digits().collect::<Vec<_>>());
    /// let r = radic!(5, [2, 1], [3]);
    /// assert_eq!(vec![2, 1, 3, 3, 3, 3], r.into_digits().take(6).collect::<Vec<_>>());
    /// ```
    fn into_digits(self) -> impl Iterator<Item=u32>;

    /// Get a vector of the digits, padding with zeros, to exactly n.
    /// See also: [`into_padded_digits`](Self::into_padded_digits)
    ///
    /// ```
    /// # use adic::{uadic, AdicInteger};
    /// assert_eq!(vec![2, 3, 1, 0, 0, 0], uadic!(5, [2, 3, 1]).padded_digits(6));
    /// ```
    fn padded_digits(&self, n: usize) -> Vec<u32> {
        self.digits().copied().chain(repeat(0)).take(n).collect()
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

    /// Returns the digit in the zeroth position or None if certainty is 0
    ///
    /// ```
    /// # use adic::{uadic, AdicInteger};
    /// assert_eq!(Some(2), uadic!(5, [2, 3, 1]).zeroth_digit());
    /// ```
    fn zeroth_digit(&self) -> Option<u32> {
        match self.certainty() {
            ZAdicValuation::Finite(0) => None,
            _ => Some(self.digits().next().copied().unwrap_or(0)),
        }
    }

    /// A format a string representing the digits of this adic number; used to Display the integer.
    ///
    /// ```
    /// # use adic::{iadic_neg, radic, uadic, zadic_approx, AdicInteger};
    /// assert_eq!("321", uadic!(5, [1, 2, 3]).digit_str());
    /// assert_eq!("(4)31", iadic_neg!(5, [1, 3]).digit_str());
    /// assert_eq!("(24)31", radic!(5, [1, 3], [4, 2]).digit_str());
    /// assert_eq!("...3412", zadic_approx!(5, 4, [2, 1, 4, 3]).digit_str());
    /// ```
    fn digit_str(&self) -> String;


    // Logic

    /// Test if it is the zero adic number
    ///
    /// ```
    /// # use adic::{uadic, AdicInteger};
    /// assert!(uadic!(5, []).is_zero());
    /// assert!(!uadic!(5, [2, 3, 1, 2, 3, 1]).is_zero());
    /// ```
    fn is_zero(&self) -> bool {
        self.digits().all(Zero::is_zero)
    }

    /// Test if adic number has finite digits
    ///
    /// ```
    /// # use adic::{radic, uadic, AdicInteger};
    /// assert!(uadic!(5, [2, 3, 1, 2, 3, 1]).has_finite_digits());
    /// assert!(!radic!(5, [2, 3, 1], [2, 1]).has_finite_digits());
    /// ```
    fn has_finite_digits(&self) -> bool {
        !matches!(self.num_digits(), ZAdicValuation::PosInf)
    }

    /// Test if adic number is completely known
    ///
    /// ```
    /// # use adic::{zadic_approx, zadic_exact_pos, AdicInteger};
    /// assert!(zadic_exact_pos!(5, [2, 3, 1, 2, 3, 1]).is_certain());
    /// assert!(!zadic_approx!(5, 6, [2, 3, 1, 2, 3, 1]).is_certain());
    /// ```
    fn is_certain(&self) -> bool {
        matches!(self.certainty(), ZAdicValuation::PosInf)
    }

    /// Test if it is a unit, i.e. if the first digit is nonzero
    ///
    /// ```
    /// # use adic::{uadic, AdicInteger};
    /// assert!(uadic!(5, [2, 3, 1]).is_unit());
    /// assert!(!uadic!(5, [0, 3, 1]).is_unit());
    /// ```
    fn is_unit(&self) -> bool {
        self.digits().next().copied().unwrap_or(0) != 0
    }


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
        UAdic::new(self.p(), self.digits().copied().take(n).collect())
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

    /// Create adic number associated with (unsigned) integer n.
    /// See also: [`into_truncation_to_uadic`](Self::into_truncation_to_uadic)
    ///
    /// ```
    /// # use adic::{zadic_approx, uadic, AdicInteger};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let u = uadic!(5, [0, 3, 2, 2, 1]);
    /// assert_eq!(u, u.truncation_to_uadic()?);
    /// let z = zadic_approx!(5, 6, [0, 3, 2, 2, 1, 0]);
    /// assert_eq!(u, z.truncation_to_uadic()?);
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    /// Returns error if `AdicInteger` has an infinite number of digits
    fn truncation_to_uadic(&self) -> Result<UAdic, AdicError> {
        match self.num_digits() {
            ZAdicValuation::PosInf => Err(AdicError::InappropriatePrecision("Cannot truncate an infinite number to uadic length".to_string())),
            ZAdicValuation::Finite(nd) => Ok(self.truncation(nd)),
        }
    }

    /// Consume `AdicInteger` and get the `truncation_to_uadic`.
    /// See also: [`truncation_to_uadic`](Self::truncation_to_uadic)
    ///
    /// ```
    /// # use adic::{zadic_approx, uadic, AdicInteger};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let u = uadic!(5, [0, 3, 2, 2, 1]);
    /// assert_eq!(u, u.clone().into_truncation_to_uadic()?);
    /// let z = zadic_approx!(5, 6, [0, 3, 2, 2, 1, 0]);
    /// assert_eq!(u, z.into_truncation_to_uadic()?);
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    /// Returns error if `AdicInteger` has an infinite number of digits
    fn into_truncation_to_uadic(self) -> Result<UAdic, AdicError> {
        match self.num_digits() {
            ZAdicValuation::PosInf => Err(AdicError::InappropriatePrecision("Cannot truncate an infinite number to uadic length".to_string())),
            ZAdicValuation::Finite(nd) => Ok(self.into_truncation(nd)),
        }
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
    /// let r = radic!(5, [1, 3], [2, 1]);
    /// assert_eq!(zs, r.approximation(5));
    /// ```
    fn approximation(&self, n: usize) -> ZAdic {
        ZAdic::new_approx(self.p(), n, self.digits().copied().take(n).collect())
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
    /// let r = radic!(5, [1, 3], [2, 1]);
    /// assert_eq!(zs, r.into_approximation(5));
    /// ```
    fn into_approximation(self, n: usize) -> ZAdic {
        ZAdic::new_approx(self.p(), n, self.into_digits().take(n).collect())
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

    /// Transform into the adic unit and valuation form; transforms zero into `(0, PosInf)`
    ///
    /// ```
    /// # use adic::{uadic, AdicInteger, UAdic, ZAdicValuation};
    /// let (unit, valuation) = uadic!(5, [0, 3, 1]).unit_and_valuation();
    /// assert_eq!((uadic!(5, [3, 1]), ZAdicValuation::Finite(1)), (unit, valuation));
    /// assert_eq!((UAdic::zero(5), ZAdicValuation::PosInf), UAdic::zero(5).unit_and_valuation());
    /// ```
    fn unit_and_valuation(&self) -> (Self, ZAdicValuation) {
        match self.valuation() {
            ZAdicValuation::PosInf => (Self::zero(self.p()), ZAdicValuation::PosInf),
            ZAdicValuation::Finite(valuation) => {
                (self.split(valuation).1, ZAdicValuation::Finite(valuation))
            }
        }
    }

    /// Transform into the adic unit and valuation form; transforms zero into `(0, PosInf)`
    ///
    /// ```
    /// # use adic::{uadic, AdicInteger, UAdic, ZAdicValuation};
    /// let (unit, valuation) = uadic!(5, [0, 3, 1]).unit_and_valuation();
    /// assert_eq!((uadic!(5, [3, 1]), ZAdicValuation::Finite(1)), (unit, valuation));
    /// assert_eq!((UAdic::zero(5), ZAdicValuation::PosInf), UAdic::zero(5).unit_and_valuation());
    /// ```
    fn into_unit_and_valuation(self) -> (Self, ZAdicValuation) {
        match self.valuation() {
            ZAdicValuation::PosInf => (Self::zero(self.p()), ZAdicValuation::PosInf),
            ZAdicValuation::Finite(valuation) => {
                (self.split(valuation).1, ZAdicValuation::Finite(valuation))
            }
        }
    }


    // Size

    /// The adic valuation of the first unknown digit for this number: v(...002130._5) = 6
    ///
    /// This is the number of known digits in the digital representation.
    ///
    /// Returns a [`ZAdicValuation`].
    /// Returns `PosInf` for an exact numbers and `Finite(v)` for an approximate number with v digits.
    ///
    /// ```
    /// # use adic::{zadic_approx, zadic_exact_pos, AdicInteger, ZAdicValuation};
    /// let z = zadic_approx!(5, 6, [0, 3, 1, 2]);
    /// assert_eq!(ZAdicValuation::Finite(6), z.certainty());
    /// let z = zadic_exact_pos!(5, [0, 3, 1, 2]);
    /// assert_eq!(ZAdicValuation::PosInf, z.certainty());
    /// ```
    fn certainty(&self) -> ZAdicValuation;

    /// The adic valuation for this number: `v(a/b p^v) = v`
    ///
    /// In the digital representation, the number of zeros to the left of the decimal point.
    ///
    /// Returns a [`ZAdicValuation`].
    /// Returns `PosInf` for zero and `Finite(v)` otherwise.
    ///
    /// ```
    /// # use adic::{radic, uadic, AdicInteger, UAdic, ZAdicValuation};
    /// let r = radic!(5, [0, 3, 1], [2]);
    /// assert_eq!(ZAdicValuation::Finite(1), r.valuation());
    /// let u = uadic!(5, [0, 0, 3, 1, 2]);
    /// assert_eq!(ZAdicValuation::Finite(2), u.valuation());
    /// let u = UAdic::zero(5);
    /// assert_eq!(ZAdicValuation::PosInf, u.valuation());
    /// ```
    fn valuation(&self) -> ZAdicValuation {
        if self.is_zero() {
            ZAdicValuation::PosInf
        } else {
            ZAdicValuation::Finite(
                self.digits().take_while(|d| d.is_zero()).count()
            )
        }
    }

    /// The digital distance between valuation and certainty.
    ///
    /// Returns a [`ZAdicValuation`].
    /// Returns `Finite(0)` for zero.
    ///
    /// ```
    /// # use adic::{uadic, zadic_approx, AdicInteger, ZAdicValuation};
    /// assert_eq!(ZAdicValuation::PosInf, uadic!(5, [1]).significance());
    /// assert_eq!(ZAdicValuation::PosInf, uadic!(5, [0, 0, 1]).significance());
    /// assert_eq!(ZAdicValuation::Finite(0), uadic!(5, [0]).significance());
    /// assert_eq!(ZAdicValuation::Finite(4), zadic_approx!(5, 4, [1, 0, 0, 0]).significance());
    /// assert_eq!(ZAdicValuation::Finite(2), zadic_approx!(5, 4, [0, 0, 1, 0]).significance());
    /// assert_eq!(ZAdicValuation::Finite(0), zadic_approx!(5, 4, [0, 0, 0, 0]).significance());
    /// ```
    fn significance(&self) -> ZAdicValuation {
        match (self.certainty(), self.valuation()) {
            (_, ZAdicValuation::PosInf) => ZAdicValuation::Finite(0),
            (ZAdicValuation::PosInf, ZAdicValuation::Finite(_)) => ZAdicValuation::PosInf,
            (ZAdicValuation::Finite(c), ZAdicValuation::Finite(v)) => ZAdicValuation::Finite(c-v),
        }
    }

    /// The adic norm for this number: `|a/b p^v| = p^(-v)`
    ///
    /// ```
    /// # use adic::{radic, uadic, AdicInteger, UAdic};
    /// # use num::rational::Ratio;
    /// let r = radic!(5, [0, 3, 1], [2]);
    /// assert_eq!(Ratio::new(1, 5), r.norm());
    /// let u = uadic!(5, [0, 0, 3, 1, 2]);
    /// assert_eq!(Ratio::new(1, 25), u.norm());
    /// let u = UAdic::zero(5);
    /// assert_eq!(Ratio::new(0, 1), u.norm());
    /// ```
    ///
    /// # Panics
    /// Panics if usize -> u32 conversion fails
    fn norm(&self) -> Ratio<u32> {
        match self.valuation() {
            ZAdicValuation::PosInf => Ratio::zero(),
            ZAdicValuation::Finite(valuation) => {
                let v = u32::try_from(valuation).expect("norm usize -> u32 conversion");
                let inv_norm = self.p().pow(v);
                Ratio::new(1, inv_norm)
            },
        }
    }

    /// Calculate the n-th root, to {precision} digits, using Hensel lifting ([`nth_root`](crate::nth_root))
    ///
    /// The algorithm is roughly:
    /// - Find all solutions mod p^(2*v+1), where v is the largest derivative valuation of the found solutions
    /// - Use the Newton approximation/Taylor series to calculate the p^{n+1} digit from the previous ones
    ///
    /// In the simplest case, you find solutions in F_p (solutions mod p) and "lift" those to F_p^2, F_p^3, etc.
    /// You are looking for solutions to `f(x) = x^n - a = 0`, to find the n-th roots of a.
    /// The first step is done basically manually, trying each x in [0, 1, ... p-1] to see where it becomes zero.
    /// Then using those solutions, look for more solutions mod p^(n+1).
    /// With the Newton approximation of `f(y) = f(x) + f'(y-x) * (y-x)`, you plug in each new digit:
    /// - `$f(r_{k+1}) = f(r_k + d * p^{k+1}) = f(r) + d * p^{k+1} * f'(r) mod p^k$`
    /// - `f(r_{k+1}) = 0 mod p^{k+1}`
    /// - `f(r_{k+1}) = f(r) + d * p^k f'(r) = (r^n - a) + d * p^k * n * r^{n-1}`
    /// - ` = ((r^n-a)/p^k) * p^k + d * p^k * n * r^{n-1}`
    /// - `(r^n-a)/p^k + d * n * r^{n-1} = 0 mod p*`
    ///
    /// This gives the next digit d of the root r from the last guess, `r=r_k`.
    /// If n has a factor of p, then the algorithm is more complicated because you have to take into account more digits.
    /// (Currently our algorithm is not the best in this more general setting.)
    ///
    /// 7-adic `sqrt(2)` has two solutions, starting with 3 and with 4
    /// ```
    /// # use adic::{nth_root, uadic, zadic_variety};
    /// let seven_adic_two = uadic!(7, [2]);
    /// let variety = nth_root(&seven_adic_two, 2, 6).unwrap();
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
    /// Every (p > 2) p-adic has (p-1) roots of unity
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
    /// 5-adic `sqrt(32) = sqrt(2^5)` has ONE solution, 2.
    /// ```
    /// # use adic::{uadic, zadic_variety, AdicInteger};
    /// let thirty_two = uadic!(5, [2, 1, 1]);
    /// let variety = thirty_two.nth_root(5, 6);
    /// let expected = zadic_variety!(5, 6, [[2]]);
    /// assert_eq!(Ok(expected), variety);
    /// ```
    ///
    /// <div class="warning">
    ///
    /// 2-adic numbers are exceptional in many ways, so handling them requires some care.
    /// There may be some bugs for 2-adic numbers; please report them if you find them.
    ///
    /// </div>
    ///
    /// # Errors
    /// Errors if:
    /// 1. n == 0
    /// 2. `precision` is not high enough (roughly, `self.certainty() >= n * precision`)
    fn nth_root(&self, n: u32, precision: usize) -> Result<ZAdicVariety, AdicError> {
        nth_root(self, n, precision)
    }

}


/// # Signed Adic Integer
///
/// Structs implementing this trait are adic integers that can represent signed adics, [`IAdic`](crate::IAdic).
/// For the most part, these are the same as [`AdicInteger`](crate::AdicInteger).
/// The main difference is that `SignedAdicInteger` can be transformed between e.g. `IAdic` and `i32`.
pub trait SignedAdicInteger: AdicInteger
    + std::ops::Neg<Output=Self> + std::ops::Sub<Self, Output=Self> {

    /// Create `SignedAdicInteger` from i32
    ///
    /// ```
    /// # use adic::{AdicInteger, IAdic, RAdic, SignedAdicInteger, ZAdic};
    /// assert_eq!("123._5", IAdic::from_u32(5, 38).to_string());
    /// assert_eq!("(4)23._5", IAdic::from_i32(5, -12).to_string());
    /// assert_eq!("(4)23._5", RAdic::from_i32(5, -12).to_string());
    /// assert_eq!("(4)23._5", ZAdic::from_i32(5, -12).to_string());
    /// ```
    fn from_i32(p: u32, n: i32) -> Self;

}

impl<T> SignedAdicInteger for T
where T: AdicInteger + From<IAdic>
    + std::ops::Neg<Output=Self> + std::ops::Sub<Self, Output=Self> {

    fn from_i32(p: u32, n: i32) -> Self {
        let ia = if n.is_negative() {
            -IAdic::new_pos(p, UAdic::from_u32(p, n.unsigned_abs()).into_digits_vec())
        } else {
            IAdic::new_pos(p, UAdic::from_u32(p, n.unsigned_abs()).into_digits_vec())
        };
        Self::from(ia)
    }

}
