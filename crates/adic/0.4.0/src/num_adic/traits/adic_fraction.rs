use std::{
    fmt::{Debug, Display},
    hash::Hash,
};
use crate::{
    AdicResult, AdicValuation,
    LazyDiv, Prime, QAdic, UAdic, ZAdic,
};
use super::{AdicApproximate, AdicInteger, AdicNumber, AdicSized, HasDigits};

/// An adic number with fractional digits
///
/// # Adic Fraction
///
/// Structs implementing this trait represent adic numbers, representable as base-p digital expansions,
/// with a possibly-infinite number of digits to the left of a decimal point
/// and a finite number of digits to the right.
///
/// [`AdicInteger`](crate::AdicInteger)s are `AdicFraction`s without digits to the right of the decimal.
/// `AdicFraction`s include all rational numbers as well as many irrational (distinct from the real number irrationals).
/// Using the p-adic norm, these numbers have valuation -inf < v <= inf, i.e. `|x| = p^(-v)`.
///
/// ```
/// # use adic::{qadic, uadic, AdicFraction, AdicSized};
/// # use num::rational::Ratio;
/// assert_eq!(Ratio::new(1, 1), qadic!(uadic!(5, [4, 1, 3, 2]), 0).norm());
/// assert_eq!(Ratio::new(1, 25), qadic!(uadic!(5, [4, 1, 3, 2]), 2).norm());
/// assert_eq!(Ratio::new(25, 1), qadic!(uadic!(5, [4, 1, 3, 2]), -2).norm());
/// ```
///
/// `AdicFractions` have the expected arithmetic for numbers: addition, subtraction, multiplication, division, powers, roots.
/// Only some roots exist, e.g. the 7-adics have `sqrt(1)`, `sqrt(2)`, and `sqrt(4)`, but not `sqrt(3)`, `sqrt(5)`, or `sqrt(6)`.
/// This is similar to how the real numbers don't contain `sqrt(-1)` and similar.
///
/// Missing roots can be added to the adic numbers, creating "finite extensions".
/// If all possible roots are added, it creates the "algebraic closure", which can be completed into the "complex adics" and the "spherically complete adics".
/// These are planned in the future of this crate.
///
/// <div class="warning">
/// This trait may be removed, as [`QAdic`](crate::QAdic) is generic and could work similar to how [`AdicPower`](crate::AdicPower) does, without a trait.
/// Likely best to depend directly on [`QAdic`] instead.
/// </div>
///
/// TODO: Show root-finding for a qadic, once that is implemented
///
/// <https://en.wikipedia.org/wiki/P-adic_number>
pub trait AdicFraction
where Self: Debug + Clone + PartialEq + Eq + Hash + Display + From<UAdic> + From<(UAdic, AdicValuation<isize>)>
    + AdicNumber
    + AdicSized<ValuationRing = isize, AdicUnit = Self::AI>
    + HasDigits<DigitIndex = isize>
    + std::ops::Add<Self, Output=Self>
    + std::ops::Mul<Self, Output=Self> + std::ops::Mul<u32, Output=Self>
    + std::ops::Div<Self, Output=LazyDiv<Self>> {

    /// Associated [`AdicInteger`] type used e.g. for the fraction's adic unit
    ///
    /// The adic integer is generic and so can be e.g.
    /// - natural number [`UAdic`](crate::UAdic)
    /// - signed integer [`IAdic`](crate::IAdic)
    /// - unit fraction [`RAdic`](crate::RAdic)
    /// - approximate number [`ZAdic`](crate::ZAdic)
    type AI: AdicInteger;


    /// The adic unit for this number: `u(a/b p^v) = a/b`
    ///
    /// In the digital representation, the adic integer resulting from moving the first nonzero digit
    ///  directly to the left of the decimal point.
    ///
    /// Returns an [`AdicInteger`]. Returns `A::zero` if `AdicFraction` is zero.
    fn unit_ref(&self) -> &Self::AI;


    // Constructors

    /// Create an `AdicFraction` representing a power of p
    ///
    /// ```
    /// # use adic::{qadic, uadic, AdicFraction, AdicNumber, QAdic, AdicValuation, UAdic};
    /// assert_eq!(qadic!(uadic!(5, [0, 0, 0, 1]), 0), QAdic::p_power(5, AdicValuation::Finite(3)));
    /// assert_eq!(qadic!(uadic!(5, [1]), -3), QAdic::p_power(5, AdicValuation::Finite(-3)));
    /// assert_eq!(QAdic::<UAdic>::zero(5), QAdic::p_power(5, AdicValuation::PosInf));
    /// ```
    fn p_power<P, I>(p: P, n: I) -> Self
    where P: Into<Prime>, I: Into<AdicValuation<isize>> {
        Self::from((UAdic::one(p.into()), n.into()))
    }


    // Data fetch

    /// Gets the digit at this coefficient of p^n; error if it is beyond known digits (certainty)
    ///
    /// ```
    /// # use adic::{qadic, uadic, zadic_approx, AdicError, AdicFraction};
    /// let u = qadic!(uadic!(5, [2, 1, 3]), -1);
    /// assert_eq!([Ok(2), Ok(1), Ok(3), Ok(0)], [u.digit(-1), u.digit(0), u.digit(1), u.digit(2)]);
    /// let z = qadic!(zadic_approx!(5, 3, [2, 1, 3]), -1);
    /// assert_eq!([Ok(2), Ok(1), Ok(3)], [z.digit(-1), z.digit(0), z.digit(1)]);
    /// assert!(matches!(z.digit(2), Err(AdicError::InappropriatePrecision(_))));
    /// ```
    ///
    /// # Errors
    /// Returns error if `n > self.certainty()`
    fn digit(&self, n: isize) -> AdicResult<u32> {
        match self.valuation() {
            AdicValuation::PosInf => Ok(0),
            AdicValuation::Finite(v) => {
                if n < v {
                    Ok(0)
                } else {
                    self.unit_ref().digit((n-v).unsigned_abs())
                }
            }
        }
    }


    // Logic

    /// Test if adic number has finite digits
    ///
    /// ```
    /// # use adic::{qadic, radic, uadic, AdicFraction, AdicValuation};
    /// assert!(qadic!(uadic!(5, [2, 1, 3, 0]), -2).has_finite_digits());
    /// assert!(!qadic!(radic!(5, [2, 1], [3, 0]), -2).has_finite_digits());
    /// ```
    fn has_finite_digits(&self) -> bool {
        !matches!(self.num_digits(), AdicValuation::PosInf)
    }

    /// A string representing the digits of this adic number; used to Display the fraction.
    ///
    /// ```
    /// # use adic::{iadic_neg, qadic, radic, uadic, zadic_approx, AdicFraction};
    /// assert_eq!(("".to_string(), "32100".to_string()), qadic!(uadic!(5, [1, 2, 3]), 2).frac_int_digit_strs());
    /// assert_eq!(("1".to_string(), "(4)3".to_string()), qadic!(iadic_neg!(5, [1, 3]), -1).frac_int_digit_strs());
    /// assert_eq!(("42431".to_string(), "(42)".to_string()), qadic!(radic!(5, [1, 3], [4, 2]), -5).frac_int_digit_strs());
    /// assert_eq!(("".to_string(), "...341200".to_string()), qadic!(zadic_approx!(5, 4, [2, 1, 4, 3]), 2).frac_int_digit_strs());
    /// ```
    fn frac_int_digit_strs(&self) -> (String, String) {

        let (frac, int) = self.frac_and_int();

        let int_str = int.digit_str();

        let frac_str = match (frac.valuation(), frac.unit(), frac.unit().map(|u| u.num_digits())) {
            (AdicValuation::Finite(v), Some(frac_unit), Some(AdicValuation::Finite(n))) if v < 0 => {
                let num_zeros = v.unsigned_abs() - n;
                [str::repeat("0", num_zeros), frac_unit.digit_str()].concat()
            },
            _ => String::new()
        };

        (frac_str, int_str)

    }


    // Transformation

    /// Truncate an adic number's expansion to n.
    /// This can be thought of as the remainder `a % p^n`.
    /// See also: [`into_truncation`](Self::into_truncation)
    ///
    /// ```
    /// # use adic::{qadic, radic, uadic, AdicFraction};
    /// let u = qadic!(uadic!(5, [1, 3, 2, 1, 2, 1, 2]), -2);
    /// assert_eq!(u, u.truncation(7));
    /// assert_eq!(u, u.truncation(5));
    /// let r = qadic!(radic!(5, [1, 3], [2, 1]), -2);
    /// assert_eq!(u, r.truncation(5));
    /// ```
    fn truncation(&self, n: isize) -> QAdic<UAdic> {
        let p = self.p();
        match self.unit_and_valuation() {
            (Some(unit), AdicValuation::Finite(v)) if n > v => {
                QAdic::new(unit.into_truncation((n-v).unsigned_abs()), v)
            },
            _ => QAdic::zero(p),
        }
    }

    /// Consume `AdicFraction` and get the truncation.
    /// See also: [`truncation`](Self::truncation)
    ///
    /// ```
    /// # use adic::{qadic, radic, uadic, AdicFraction};
    /// let u = qadic!(uadic!(5, [1, 3, 2, 1, 2, 1, 2]), -2);
    /// assert_eq!(u, u.clone().into_truncation(7));
    /// assert_eq!(u, u.clone().into_truncation(5));
    /// let r = qadic!(radic!(5, [1, 3], [2, 1]), -2);
    /// assert_eq!(u, r.into_truncation(5));
    /// ```
    fn into_truncation(self, n: isize) -> QAdic<UAdic> {
        let p = self.p();
        match self.into_unit_and_valuation() {
            (Some(unit), AdicValuation::Finite(v)) if n > v => {
                QAdic::new(unit.into_truncation((n-v).unsigned_abs()), v)
            },
            _ => QAdic::zero(p),
        }
    }

    /// Approximate an adic number's expansion to n digits.
    /// See also: [`into_approximation`](Self::into_approximation)
    ///
    /// ```
    /// # use adic::{qadic, radic, uadic, zadic_approx, AdicFraction};
    /// let q = qadic!(uadic!(5, [1, 3, 2, 1, 2, 1, 2]), -5);
    /// let z = qadic!(zadic_approx!(5, 9, [1, 3, 2, 1, 2, 1, 2, 0, 0]), -5);
    /// let zs = qadic!(zadic_approx!(5, 6, [1, 3, 2, 1, 2, 1]), -5);
    /// assert_eq!(z, q.approximation(4));
    /// assert_eq!(zs, q.approximation(1));
    /// let qr = qadic!(radic!(5, [1, 3], [2, 1]), -5);
    /// assert_eq!(zs, qr.approximation(1));
    /// ```
    fn approximation(&self, n: isize) -> QAdic<ZAdic>
    where Self: AdicApproximate, Self::AI: AdicApproximate {
        let p = self.p();
        let c = match self.certainty() {
            AdicValuation::PosInf => n,
            AdicValuation::Finite(v) => std::cmp::min(v, n),
        };
        match self.unit_and_valuation() {
            (Some(unit), AdicValuation::Finite(v)) if n > v => {
                QAdic::new(unit.into_approximation((n-v).unsigned_abs()), v)
            },
            _ => QAdic::new(ZAdic::empty(p), c),
        }
    }

    /// Consume `AdicFraction` and get the approximation.
    /// See also: [`approximation`](Self::approximation)
    ///
    /// ```
    /// # use adic::{qadic, radic, uadic, zadic_approx, AdicFraction};
    /// let q = qadic!(uadic!(5, [1, 3, 2, 1, 2, 1, 2]), -5);
    /// let z = qadic!(zadic_approx!(5, 9, [1, 3, 2, 1, 2, 1, 2, 0, 0]), -5);
    /// let zs = qadic!(zadic_approx!(5, 6, [1, 3, 2, 1, 2, 1]), -5);
    /// assert_eq!(z, q.clone().into_approximation(4));
    /// assert_eq!(zs, q.clone().into_approximation(1));
    /// let qr = qadic!(radic!(5, [1, 3], [2, 1]), -5);
    /// assert_eq!(zs, qr.into_approximation(1));
    /// ```
    fn into_approximation(self, n: isize) -> QAdic<ZAdic>
    where Self: AdicApproximate, Self::AI: AdicApproximate {
        let p = self.p();
        let c = match self.certainty() {
            AdicValuation::PosInf => n,
            AdicValuation::Finite(v) => std::cmp::min(v, n),
        };
        match self.into_unit_and_valuation() {
            (Some(unit), AdicValuation::Finite(v)) if n > v => {
                QAdic::new(unit.into_approximation((n-v).unsigned_abs()), v)
            },
            _ => QAdic::new(ZAdic::empty(p), c),
        }
    }

    /// Split `AdicFraction` at p^n into remainder (as `QAdic<UAdic>`) and quotient (as `AI`)
    ///
    /// ```
    /// # use adic::{qadic, radic, AdicFraction};
    /// let r = radic!(7, [1, 2], [3, 4, 5]);
    /// assert_eq!("(543)21._7", r.to_string());
    /// let q = qadic!(r, -6);
    /// assert_eq!("(354).354321_7", q.to_string());
    /// let (q_rem, q_int) = q.split(1);
    /// assert_eq!("4.354321_7", q_rem.to_string());
    /// assert_eq!("(435)._7", q_int.to_string());
    /// let (q_rem, q_int) = q.split(-5);
    /// assert_eq!("0.000001_7", q_rem.to_string());
    /// assert_eq!("(543)2._7", q_int.to_string());
    /// let (q_rem, q_int) = q.split(-7);
    /// assert_eq!("0._7", q_rem.to_string());
    /// assert_eq!("(543)210._7", q_int.to_string());
    /// ```
    fn split(&self, n: isize) -> (QAdic<UAdic>, Self::AI) {
        let p = self.p();
        match self.unit_and_valuation() {
            (Some(unit), AdicValuation::Finite(v)) if n > v => {
                let (rem, quot) = unit.into_split((n-v).unsigned_abs());
                (QAdic::new(rem, v), quot)
            },
            (Some(unit), AdicValuation::Finite(v)) if n <= v => {
                (QAdic::zero(p), Self::AI::p_power(p, (v-n).unsigned_abs()) * unit)
            },
            _ => (QAdic::zero(p), Self::AI::zero(p))
        }
    }

    /// Consume `AdicFraction` to split at p^n into remainder (as `QAdic<UAdic>`) and quotient (as `AI`)
    ///
    /// ```
    /// # use adic::{qadic, radic, AdicFraction};
    /// let r = radic!(7, [1, 2], [3, 4, 5]);
    /// assert_eq!("(543)21._7", r.to_string());
    /// let q = qadic!(r, -6);
    /// assert_eq!("(354).354321_7", q.to_string());
    /// let (q_rem, q_int) = q.clone().into_split(1);
    /// assert_eq!("4.354321_7", q_rem.to_string());
    /// assert_eq!("(435)._7", q_int.to_string());
    /// let (q_rem, q_int) = q.clone().into_split(-5);
    /// assert_eq!("0.000001_7", q_rem.to_string());
    /// assert_eq!("(543)2._7", q_int.to_string());
    /// let (q_rem, q_int) = q.clone().into_split(-7);
    /// assert_eq!("0._7", q_rem.to_string());
    /// assert_eq!("(543)210._7", q_int.to_string());
    /// ```
    fn into_split(self, n: isize) -> (QAdic<UAdic>, Self::AI) {
        let p = self.p();
        match self.into_unit_and_valuation() {
            (Some(unit), AdicValuation::Finite(v)) if n > v => {
                let (rem, quot) = unit.into_split((n-v).unsigned_abs());
                (QAdic::new(rem, v), quot)
            },
            (Some(unit), AdicValuation::Finite(v)) if n <= v => {
                (QAdic::zero(p), Self::AI::p_power(p, (v-n).unsigned_abs()) * unit)
            },
            _ => (QAdic::zero(p), Self::AI::zero(p))
        }
    }

    /// Split `AdicFraction` into fraction and integer
    ///
    /// ```
    /// # use adic::{qadic, radic, AdicFraction};
    /// let r = radic!(7, [1, 2], [3, 4, 5]);
    /// assert_eq!("(543)21._7", r.to_string());
    /// let q = qadic!(r, -6);
    /// assert_eq!("(354).354321_7", q.to_string());
    /// let (q_frac, q_int) = q.frac_and_int();
    /// assert_eq!("0.354321_7", q_frac.to_string());
    /// assert_eq!("(354)._7", q_int.to_string());
    /// ```
    fn frac_and_int(&self) -> (QAdic<UAdic>, Self::AI) {
        self.split(0)
    }

    #[must_use]
    /// Divide an adic number by p^n.
    /// This can be thought of as the quotient a // p^n
    ///
    /// ```
    /// # use adic::{qadic, uadic, AdicFraction};
    /// let q = qadic!(uadic!(5, [1, 2, 3, 4]), -1);
    /// assert_eq!("432.1_5", q.to_string());
    /// let quot = q.quotient(2);
    /// assert_eq!(uadic!(5, [4]), quot);
    /// ```
    fn quotient(&self, n: isize) -> Self::AI {
        self.split(n).1
    }


    // TODO: Implement by splitting into unit and valuation and then calling to the AdicInteger's nth_root
    //
    // /// Calculate the n-th root, to {precision} digits, using Hensel lifting ([`nth_root`](crate::nth_root))
    // ///
    // /// The algorithm is roughly:
    // /// - Find all solutions mod p^(2*v+1), where v is the largest derivative valuation of the found solutions
    // /// - Use the Newton approximation/Taylor series to calculate the p^{n+1} digit from the previous ones
    // ///
    // /// In the simplest case, you find solutions in F_p (solutions mod p) and "lift" those to F_p^2, F_p^3, etc.
    // /// You are looking for solutions to `f(x) = x^n - a = 0`, to find the n-th roots of a.
    // /// The first step is done basically manually, trying each x in [0, 1, ... p-1] to see where it becomes zero.
    // /// Then using those solutions, look for more solutions mod p^(n+1).
    // /// With the Newton approximation of `f(y) = f(x) + f'(y-x) * (y-x)`, you plug in each new digit:
    // /// - `$f(r_{k+1}) = f(r_k + d * p^{k+1}) = f(r) + d * p^{k+1} * f'(r) mod p^k$`
    // /// - `f(r_{k+1}) = 0 mod p^{k+1}`
    // /// - `f(r_{k+1}) = f(r) + d * p^k f'(r) = (r^n - a) + d * p^k * n * r^{n-1}`
    // /// - ` = ((r^n-a)/p^k) * p^k + d * p^k * n * r^{n-1}`
    // /// - `(r^n-a)/p^k + d * n * r^{n-1} = 0 mod p*`
    // ///
    // /// This gives the next digit d of the root r from the last guess, `r=r_k`.
    // /// If n has a factor of p, then the algorithm is more complicated because you have to take into account more digits.
    // /// (Currently our algorithm is not the best in this more general setting.)
    // ///
    // /// 7-adic `sqrt(2)` has two solutions, starting with 3 and with 4
    // /// ```
    // /// # use adic::{nth_root, uadic, zadic_variety};
    // /// let seven_adic_two = uadic!(7, [2]);
    // /// let variety = nth_root(&seven_adic_two, 2, 6).unwrap();
    // /// let expected = zadic_variety!(7, 6, [
    // ///     [3, 1, 2, 6, 1, 2],
    // ///     [4, 5, 4, 0, 5, 4],
    // /// ]);
    // /// assert_eq!(expected, variety);
    // /// assert_eq!("variety(...216213._7, ...450454._7)", variety.to_string());
    // /// ```
    // ///
    // /// 5-adic `sqrt(2)` has no solutions, as seen since no element of F_5 has `x^2 = 2 mod 5`
    // /// ```
    // /// # use adic::{uadic, AdicInteger, ZAdicVariety};
    // /// let five_adic_two = uadic!(5, [2]);
    // /// let variety = five_adic_two.nth_root(2, 6);
    // /// let expected = ZAdicVariety::empty(5);
    // /// assert_eq!(Ok(expected), variety);
    // /// ```
    // ///
    // /// Every (p > 2) p-adic has (p-1) roots of unity
    // /// ```
    // /// # use adic::{uadic, zadic_variety, AdicInteger};
    // /// let five_adic_one = uadic!(5, [1]);
    // /// let variety = five_adic_one.nth_root(4, 6);
    // /// let expected = zadic_variety!(5, 6, [
    // ///     [1, 0, 0, 0, 0, 0],
    // ///     [2, 1, 2, 1, 3, 4],
    // ///     [3, 3, 2, 3, 1, 0],
    // ///     [4, 4, 4, 4, 4, 4],
    // /// ]);
    // /// assert_eq!(Ok(expected), variety);
    // /// ```
    // ///
    // /// 5-adic `sqrt(32) = sqrt(2^5)` has ONE solution, 2.
    // /// ```
    // /// # use adic::{uadic, zadic_variety, AdicInteger};
    // /// let thirty_two = uadic!(5, [2, 1, 1]);
    // /// let variety = thirty_two.nth_root(5, 6);
    // /// let expected = zadic_variety!(5, 6, [[2]]);
    // /// assert_eq!(Ok(expected), variety);
    // /// ```
    // ///
    // /// <div class="warning">
    // ///
    // /// 2-adic numbers are exceptional in many ways, so handling them requires some care.
    // /// There may be some bugs for 2-adic numbers; please report them if you find them.
    // ///
    // /// </div>
    // ///
    // /// # Errors
    // /// Errors if:
    // /// 1. n == 0
    // /// 2. `precision` is not high enough (roughly, `self.certainty() >= n * precision`)
    // fn nth_root(&self, n: u32, precision: usize) -> Result<ZAdicVariety, AdicError> {
    //     nth_root(self, n, precision)
    // }

}
