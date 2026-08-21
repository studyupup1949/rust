use num::{rational::Ratio, Zero};
use crate::{AdicValuation, AdicValuationRing};


/// Has a non-archimedian norm and valuation
pub trait AdicSized {

    /// Type for the valuation, e.g. the type of v in `a/b p^v`
    type ValuationRing: AdicValuationRing;

    /// Type for the adic unit, the "size 1" type, e.g. `UAdic` if the `AdicSized` is `QAdic<UAdic>`
    type AdicUnit;

    /// The adic valuation for this number: `v(a/b p^v) = v`
    ///
    /// In the digital representation, the number of zeroes to the left (positive)
    ///  or the number of digits to the right (negative) of the decimal point.
    ///
    /// Returns an [`AdicValuation`].
    /// Returns `PosInf` for zero and `Finite(v)` otherwise.
    ///
    /// ```
    /// # use adic::{qadic, radic, uadic, zadic_approx, AdicNumber, AdicSized, AdicValuation, QAdic, UAdic};
    /// assert_eq!(AdicValuation::Finite(1), radic!(5, [0, 3, 1], [2]).valuation());
    /// assert_eq!(AdicValuation::Finite(2), uadic!(5, [0, 0, 3, 1, 2]).valuation());
    /// assert_eq!(AdicValuation::PosInf, UAdic::zero(5).valuation());
    /// assert_eq!(AdicValuation::Finite(4), qadic!(uadic!(5, [1, 2]), 4).valuation());
    /// assert_eq!(AdicValuation::Finite(-4), qadic!(uadic!(5, [1, 2]), -4).valuation());
    /// assert_eq!(AdicValuation::Finite(-2), qadic!(uadic!(5, [0, 0, 1]), -4).valuation());
    /// assert_eq!(AdicValuation::PosInf, QAdic::<UAdic>::zero(5).valuation());
    /// ```
    fn valuation(&self) -> AdicValuation<Self::ValuationRing>;

    /// The adic norm for this number: `|a/b p^v| = p^(-v)`
    ///
    /// ```
    /// # use adic::{qadic, radic, uadic, AdicNumber, AdicSized, UAdic};
    /// # use num::rational::Ratio;
    /// assert_eq!(Ratio::new(1, 5), radic!(5, [0, 3, 1], [2]).norm());
    /// assert_eq!(Ratio::new(1, 25), uadic!(5, [0, 0, 3, 1, 2]).norm());
    /// assert_eq!(Ratio::new(0, 1), UAdic::zero(5).norm());
    /// assert_eq!(Ratio::new(5, 1), qadic!(radic!(5, [0, 3, 1], [2]), -2).norm());
    /// assert_eq!(Ratio::new(1, 625), qadic!(uadic!(5, [0, 0, 3, 1, 2]), 2).norm());
    /// assert_eq!(Ratio::new(0, 1), qadic!(UAdic::zero(5), 1).norm());
    /// ```
    fn norm(&self) -> Ratio<u32>;

    /// Test if it is a unit, valuation zero, if no fractional digits but the zeroth digit is nonzero
    ///
    /// ```
    /// # use adic::{qadic, uadic, AdicSized};
    /// assert!(uadic!(5, [2, 3, 1]).is_unit());
    /// assert!(!uadic!(5, [0, 3, 1]).is_unit());
    /// assert!(qadic!(uadic!(5, [0, 3, 1]), -1).is_unit());
    /// assert!(!qadic!(uadic!(5, [2, 3, 1]), 1).is_unit());
    /// assert!(!qadic!(uadic!(5, [2, 3, 1]), -1).is_unit());
    /// ```
    fn is_unit(&self) -> bool {
        self.valuation() == AdicValuation::Finite(Self::ValuationRing::zero())
    }

    /// The adic unit for this number: `u(a/b p^v) = a/b`
    ///
    /// In the digital representation, the adic integer resulting from moving the first nonzero digit
    ///  directly to the left of the decimal point.
    ///
    /// Returns `Option<Self::AdicUnit>`.
    /// Returns `None` if valuation is infinite.
    ///
    /// ```
    /// # use adic::{qadic, uadic, AdicFraction, AdicNumber, AdicSized, QAdic, AdicValuation, UAdic};
    /// let u = Some(uadic!(5, [1, 2]));
    /// assert_eq!(u, uadic!(5, [0, 0, 1, 2]).unit());
    /// assert_eq!(u, qadic!(uadic!(5, [1, 2]), 4).unit());
    /// assert_eq!(u, qadic!(uadic!(5, [1, 2]), -4).unit());
    /// assert_eq!(None, QAdic::<UAdic>::zero(5).unit());
    /// ```
    fn unit(&self) -> Option<Self::AdicUnit>;

    /// Consume this number to get the adic unit: `u(a/b p^v) = a/b`
    ///
    /// In the digital representation, the adic integer resulting from moving the first nonzero digit
    ///  directly to the left of the decimal point.
    ///
    /// Returns `Option<Self::AdicUnit>`.
    /// Returns `None` if valuation is infinite.
    ///
    /// ```
    /// # use adic::{qadic, uadic, AdicFraction, AdicNumber, AdicSized, QAdic, AdicValuation, UAdic};
    /// let u = Some(uadic!(5, [1, 2]));
    /// assert_eq!(u, uadic!(5, [0, 0, 1, 2]).into_unit());
    /// assert_eq!(u, qadic!(uadic!(5, [1, 2]), 4).into_unit());
    /// assert_eq!(u, qadic!(uadic!(5, [1, 2]), -4).into_unit());
    /// assert_eq!(None, QAdic::<UAdic>::zero(5).into_unit());
    /// ```
    fn into_unit(self) -> Option<Self::AdicUnit>;

    /// Transform into the adic unit and valuation form; transforms zero into `(None, PosInf)`
    ///
    /// ```
    /// # use adic::{qadic, uadic, AdicFraction, AdicNumber, AdicSized, AdicValuation, QAdic, UAdic};
    /// let (unit, valuation) = uadic!(5, [0, 3, 1]).unit_and_valuation();
    /// assert_eq!((Some(uadic!(5, [3, 1])), AdicValuation::Finite(1)), (unit, valuation));
    /// assert_eq!((None, AdicValuation::PosInf), UAdic::zero(5).unit_and_valuation());
    /// let (unit, valuation) = qadic!(uadic!(5, [0, 3, 1]), 4).unit_and_valuation();
    /// assert_eq!((Some(uadic!(5, [3, 1])), AdicValuation::Finite(5)), (unit, valuation));
    /// assert_eq!((None, AdicValuation::PosInf), QAdic::<UAdic>::zero(5).unit_and_valuation());
    /// ```
    fn unit_and_valuation(&self) -> (Option<Self::AdicUnit>, AdicValuation<Self::ValuationRing>) {
        let v = self.valuation();
        let u = self.unit();
        (u, v)
    }

    /// Transform into the adic unit and valuation form; transforms zero into `(None, PosInf)`
    ///
    /// ```
    /// # use adic::{qadic, uadic, AdicFraction, AdicNumber, AdicSized, AdicValuation, QAdic, UAdic};
    /// let (unit, valuation) = uadic!(5, [0, 3, 1]).into_unit_and_valuation();
    /// assert_eq!((Some(uadic!(5, [3, 1])), AdicValuation::Finite(1)), (unit, valuation));
    /// assert_eq!((None, AdicValuation::PosInf), UAdic::zero(5).into_unit_and_valuation());
    /// let (unit, valuation) = qadic!(uadic!(5, [0, 3, 1]), 4).into_unit_and_valuation();
    /// assert_eq!((Some(uadic!(5, [3, 1])), AdicValuation::Finite(5)), (unit, valuation));
    /// assert_eq!((None, AdicValuation::PosInf), QAdic::<UAdic>::zero(5).into_unit_and_valuation());
    /// ```
    fn into_unit_and_valuation(self) -> (Option<Self::AdicUnit>, AdicValuation<Self::ValuationRing>) where Self: Sized {
        let v = self.valuation();
        let u = self.into_unit();
        (u, v)
    }

}
