use std::fmt::Debug;
use num::{traits::Euclid, Zero};
use crate::{AdicResult, AdicValuation, AdicValuationRing, Composite};



/// A structure with digits that can be accessed
pub trait HasDigits {

    /// Type for the valuation, e.g. the type of v in `a/b p^v`
    type DigitIndex: Debug + AdicValuationRing + Euclid;

    /// Number of possibilities for digits
    ///
    /// ```
    /// # use adic::{apow, uadic, AdicComposite, Composite, HasDigits};
    /// assert_eq!(Composite::new([(7, 1)]), uadic!(7, [1, 2, 3]).base());
    /// assert_eq!(Composite::new([(5, 2)]), apow!(uadic!(5, [3, 2, 1]), 2).base());
    /// assert_eq!(Composite::new([(2, 1), (5, 1)]), AdicComposite::new([
    ///     apow!(uadic!(2, [0, 1]), 1),
    ///     apow!(uadic!(5, [3, 2]), 1),
    /// ]).base());
    /// ```
    fn base(&self) -> Composite;

    /// Minimum digit index, possibly zero for positive valuation numbers.
    /// This is the index where the first digit of `[digits](Self::digits)` starts.
    ///
    /// ```
    /// # use adic::{qadic, uadic, AdicValuation, HasDigits};
    /// assert_eq!(AdicValuation::Finite(0), uadic!(5, [2]).min_index());
    /// assert_eq!(AdicValuation::Finite(0), uadic!(5, [0, 0, 2]).min_index());
    /// assert_eq!(AdicValuation::Finite(0), qadic!(uadic!(5, [2]), 2).min_index());
    /// assert_eq!(AdicValuation::Finite(-2), qadic!(uadic!(5, [2]), -2).min_index());
    /// ```
    fn min_index(&self) -> AdicValuation<Self::DigitIndex>;

    /// The number of digits this number ultimately has, finite or infinite.
    /// Returns `(num+|valuation|)` if `valuation` is negative and `(num)` if it is positive.
    ///
    /// ```
    /// # use adic::{qadic, radic, uadic, AdicNumber, AdicValuation, HasDigits, UAdic};
    /// assert_eq!(AdicValuation::Finite(3), uadic!(5, [2, 1, 3, 0]).num_digits());
    /// assert_eq!(AdicValuation::Finite(0), UAdic::zero(5).num_digits());
    /// assert_eq!(AdicValuation::PosInf, radic!(5, [2, 1], [3, 0]).num_digits());
    /// assert_eq!(AdicValuation::Finite(3), qadic!(uadic!(5, [2, 1, 3, 0]), -2).num_digits());
    /// assert_eq!(AdicValuation::Finite(5), qadic!(uadic!(5, [2, 1, 3, 0]), 2).num_digits());
    /// assert_eq!(AdicValuation::Finite(0), qadic!(UAdic::zero(5), 2).num_digits());
    /// assert_eq!(AdicValuation::PosInf, qadic!(radic!(5, [2, 1], [3, 0]), -2).num_digits());
    /// ```
    fn num_digits(&self) -> AdicValuation<usize>;

    /// Test if adic number has finite digits
    ///
    /// ```
    /// # use adic::{qadic, radic, uadic, AdicValuation, HasDigits};
    /// assert!(uadic!(5, [2, 3, 1, 2, 3, 1]).has_finite_digits());
    /// assert!(!radic!(5, [2, 3, 1], [2, 1]).has_finite_digits());
    /// assert!(qadic!(uadic!(5, [2, 1, 3, 0]), -2).has_finite_digits());
    /// assert!(!qadic!(radic!(5, [2, 1], [3, 0]), -2).has_finite_digits());
    /// ```
    fn has_finite_digits(&self) -> bool {
        !matches!(self.num_digits(), AdicValuation::PosInf)
    }


    /// Gets the digit at this coefficient of p^n; error if it is beyond known digits (certainty)
    ///
    /// ```
    /// # use adic::{qadic, uadic, zadic_approx, AdicError, HasDigits};
    /// let u = uadic!(5, [2, 1, 3]);
    /// assert_eq!([Ok(2), Ok(3), Ok(0)], [u.digit(0), u.digit(2), u.digit(4)]);
    /// let z = zadic_approx!(5, 4, [2, 1, 3]);
    /// assert_eq!([Ok(2), Ok(3)], [z.digit(0), z.digit(2)]);
    /// assert!(matches!(z.digit(4), Err(AdicError::InappropriatePrecision(_))));
    /// let u = qadic!(uadic!(5, [2, 1, 3]), -1);
    /// assert_eq!([Ok(2), Ok(1), Ok(3), Ok(0)], [u.digit(-1), u.digit(0), u.digit(1), u.digit(2)]);
    /// let z = qadic!(zadic_approx!(5, 3, [2, 1, 3]), -1);
    /// assert_eq!([Ok(2), Ok(1), Ok(3)], [z.digit(-1), z.digit(0), z.digit(1)]);
    /// assert!(matches!(z.digit(2), Err(AdicError::InappropriatePrecision(_))));
    /// ```
    ///
    /// # Errors
    /// Returns error if `n > self.certainty()`
    fn digit(&self, n: Self::DigitIndex) -> AdicResult<u32>;

    /// Returns the digit in the zeroth position or Err if certainty <= 0
    ///
    /// ```
    /// # use adic::{qadic, uadic, zadic_approx, AdicError, HasDigits, ZAdic};
    /// assert_eq!(Ok(2), uadic!(5, [2, 3, 1]).digit0());
    /// assert!(matches!(ZAdic::empty(5).digit0(), Err(AdicError::InappropriatePrecision(_))));
    /// assert_eq!(Ok(3), qadic!(uadic!(5, [2, 3, 1]), -1).digit0());
    /// assert_eq!(Ok(0), qadic!(uadic!(5, [2, 3, 1]), -4).digit0());
    /// assert!(matches!(qadic!(zadic_approx!(5, 2, [1, 2]), -3).digit0(), Err(AdicError::InappropriatePrecision(_))));
    /// ```
    ///
    /// # Errors
    /// Returns error if number is completely uncertain
    fn digit0(&self) -> AdicResult<u32> {
        self.digit(Self::DigitIndex::zero())
    }

    /// Digits for this adic, from the p^v coefficient to p^(v+1), etc.
    ///
    /// ```
    /// # use adic::{qadic, radic, uadic, HasDigits};
    /// let u = uadic!(5, [2, 1, 3]);
    /// assert_eq!(vec![2, 1, 3], u.digits().collect::<Vec<_>>());
    /// let r = radic!(5, [2, 1], [3]);
    /// assert_eq!(vec![2, 1, 3, 3, 3, 3], r.digits().take(6).collect::<Vec<_>>());
    /// let q = qadic!(uadic!(5, [2, 1, 3]), -1);
    /// assert_eq!(vec![2, 1, 3], q.digits().collect::<Vec<_>>());
    /// let q = qadic!(uadic!(5, [2, 1, 3]), 1);
    /// assert_eq!(vec![0, 2, 1, 3], q.digits().collect::<Vec<_>>());
    /// ```
    fn digits(&self) -> impl Iterator<Item=u32>;

    /// Consume `AdicInteger` and get the digits iterator
    ///
    /// ```
    /// # use adic::{qadic, radic, uadic, HasDigits};
    /// let u = uadic!(5, [2, 1, 3]);
    /// assert_eq!(vec![2, 1, 3], u.into_digits().collect::<Vec<_>>());
    /// let r = radic!(5, [2, 1], [3]);
    /// assert_eq!(vec![2, 1, 3, 3, 3, 3], r.into_digits().take(6).collect::<Vec<_>>());
    /// let q = qadic!(uadic!(5, [2, 1, 3]), -1);
    /// assert_eq!(vec![2, 1, 3], q.into_digits().collect::<Vec<_>>());
    /// let q = qadic!(uadic!(5, [2, 1, 3]), 1);
    /// assert_eq!(vec![0, 2, 1, 3], q.into_digits().collect::<Vec<_>>());
    /// ```
    fn into_digits(self) -> impl Iterator<Item=u32>;

}
