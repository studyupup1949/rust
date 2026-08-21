use num::Zero;
use crate::AdicValuation;
use super::{AdicSized, HasDigits};


/// An adic number whose certainty can be measured
pub trait AdicApproximate: HasDigits {

    /// The adic valuation of the first unknown digit for this number: `v(...0021.30_5) = 4`
    ///
    /// This is the number of known digits in the digital representation.
    ///
    /// Returns an [`AdicValuation`].
    /// Returns `PosInf` for an exact numbers and `Finite(v)` for an approximate number with digits to the `v-th` valuation.
    ///
    /// ```
    /// # use adic::{qadic, uadic, zadic_approx, AdicApproximate, AdicValuation};
    /// assert_eq!(AdicValuation::Finite(6), zadic_approx!(5, 6, [0, 3, 1, 2]).certainty());
    /// assert_eq!(AdicValuation::PosInf, uadic!(5, [0, 3, 1, 2]).certainty());
    /// assert_eq!(AdicValuation::Finite(4), qadic!(zadic_approx!(5, 6, [0, 3, 1, 2]), -2).certainty());
    /// assert_eq!(AdicValuation::Finite(-2), qadic!(zadic_approx!(5, 2, [0, 3]), -4).certainty());
    /// assert_eq!(AdicValuation::PosInf, qadic!(uadic!(5, [0, 3, 1, 2]), -2).certainty());
    /// ```
    fn certainty(&self) -> AdicValuation<Self::DigitIndex>;

    /// The adic is completely uncertain, has no known digits
    ///
    /// ```
    /// # use adic::{qadic, zadic_approx, AdicApproximate, AdicNumber, AdicValuation, ZAdic};
    /// assert!(!zadic_approx!(5, 6, [0, 3, 1, 2]).has_no_certainty());
    /// assert!(!zadic_approx!(5, 1, [3]).has_no_certainty());
    /// assert!(ZAdic::empty(5).has_no_certainty());
    /// assert!(!qadic!(zadic_approx!(5, 6, [0, 3, 1, 2]), -2).has_no_certainty());
    /// assert!(!qadic!(zadic_approx!(5, 1, [3]), -2).has_no_certainty());
    /// assert!(qadic!(ZAdic::empty(5), -2).has_no_certainty());
    /// assert!(!qadic!(ZAdic::zero(5), 0).has_no_certainty());
    /// ```
    fn has_no_certainty(&self) -> bool {
        if let (AdicValuation::Finite(v), AdicValuation::Finite(c)) = (self.min_index(), self.certainty()) {
            v >= c
        } else {
            false
        }
    }

    /// Test if adic number is completely known
    ///
    /// ```
    /// # use adic::{qadic, uadic, zadic_approx, AdicApproximate};
    /// assert!(uadic!(5, [2, 3, 1, 2, 3, 1]).is_certain());
    /// assert!(!zadic_approx!(5, 6, [2, 3, 1, 2, 3, 1]).is_certain());
    /// assert!(qadic!(uadic!(5, [2, 3, 1, 2, 3, 1]), -2).is_certain());
    /// assert!(!qadic!(zadic_approx!(5, 6, [2, 3, 1, 2, 3, 1]), -2).is_certain());
    /// ```
    fn is_certain(&self) -> bool {
        matches!(self.certainty(), AdicValuation::PosInf)
    }

    /// The digital distance between valuation and certainty.
    ///
    /// Returns an [`AdicValuation`].
    /// Returns `Finite(0)` for zero.
    ///
    /// ```
    /// # use adic::{qadic, uadic, zadic_approx, AdicApproximate, AdicValuation};
    /// assert_eq!(AdicValuation::PosInf, uadic!(5, [1]).significance());
    /// assert_eq!(AdicValuation::PosInf, uadic!(5, [0, 0, 1]).significance());
    /// assert_eq!(AdicValuation::Finite(0), uadic!(5, [0]).significance());
    /// assert_eq!(AdicValuation::Finite(4), zadic_approx!(5, 4, [1, 0, 0, 0]).significance());
    /// assert_eq!(AdicValuation::Finite(2), zadic_approx!(5, 4, [0, 0, 1, 0]).significance());
    /// assert_eq!(AdicValuation::Finite(0), zadic_approx!(5, 4, [0, 0, 0, 0]).significance());
    /// assert_eq!(AdicValuation::PosInf, qadic!(uadic!(5, [0, 0, 1]), -1).significance());
    /// assert_eq!(AdicValuation::Finite(4), qadic!(zadic_approx!(5, 4, [1, 0, 0, 0]), -8).significance());
    /// assert_eq!(AdicValuation::Finite(2), qadic!(zadic_approx!(5, 4, [0, 0, 1, 0]), 4).significance());
    /// assert_eq!(AdicValuation::Finite(0), qadic!(zadic_approx!(5, 4, [0, 0, 0, 0]), 4).significance());
    /// ```
    fn significance(&self) -> AdicValuation<Self::ValuationRing>
    where
    Self: AdicSized<ValuationRing = Self::DigitIndex>,
    Self::ValuationRing: std::ops::Sub<Output=Self::ValuationRing> {
        match (self.certainty(), self.valuation()) {
            (_, AdicValuation::PosInf) => AdicValuation::zero(),
            (AdicValuation::PosInf, AdicValuation::Finite(_)) => AdicValuation::PosInf,
            (AdicValuation::Finite(c), AdicValuation::Finite(v)) => {
                let s = (c - v);
                AdicValuation::Finite(s)
            },
        }
    }

}
