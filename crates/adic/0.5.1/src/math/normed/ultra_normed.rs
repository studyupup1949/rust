use super::{Normed, Valuation, ValuationRing};


/// Has an [ultrametric](https://en.wikipedia.org/wiki/Ultrametric_space) norm and valuation
pub trait UltraNormed: Normed {

    /// Type for the valuation, e.g. the type of v in `a/b p^v`
    type ValuationRing: ValuationRing;

    /// Create with the given unit and valuation
    fn from_unit_and_valuation(u: Self::Unit, v: Valuation<Self::ValuationRing>) -> Self;

    /// The adic valuation for this number: `v(a/b p^v) = v`
    ///
    /// In the digital representation, the number of zeroes to the left (positive)
    ///  or the number of digits to the right (negative) of the decimal point.
    ///
    /// Returns a [`Valuation`]: `PosInf` for zero and `Finite(v)` otherwise.
    ///
    /// ```
    /// # use adic::{normed::{UltraNormed, Valuation}, traits::AdicPrimitive, EAdic, QAdic, UAdic};
    /// assert_eq!(Valuation::Finite(1), EAdic::new_repeating(5, vec![0, 3, 1], vec![2]).valuation());
    /// assert_eq!(Valuation::Finite(2), UAdic::new(5, vec![0, 0, 3, 1, 2]).valuation());
    /// assert_eq!(Valuation::PosInf, UAdic::zero(5).valuation());
    /// assert_eq!(Valuation::Finite(4), QAdic::new(UAdic::new(5, vec![1, 2]), 4).valuation());
    /// assert_eq!(Valuation::Finite(-4), QAdic::new(UAdic::new(5, vec![1, 2]), -4).valuation());
    /// assert_eq!(Valuation::Finite(-2), QAdic::new(UAdic::new(5, vec![0, 0, 1]), -4).valuation());
    /// assert_eq!(Valuation::PosInf, QAdic::<UAdic>::zero(5).valuation());
    /// ```
    fn valuation(&self) -> Valuation<Self::ValuationRing>;

    /// Transform into the adic unit and valuation form; transforms zero into `(None, PosInf)`
    ///
    /// ```
    /// # use adic::{normed::{UltraNormed, Valuation}, traits::AdicPrimitive, QAdic, UAdic};
    /// let (unit, valuation) = UAdic::new(5, vec![0, 3, 1]).unit_and_valuation();
    /// assert_eq!((Some(UAdic::new(5, vec![3, 1])), Valuation::Finite(1)), (unit, valuation));
    /// assert_eq!((None, Valuation::PosInf), UAdic::zero(5).unit_and_valuation());
    /// let (unit, valuation) = QAdic::new(UAdic::new(5, vec![0, 3, 1]), 4).unit_and_valuation();
    /// assert_eq!((Some(UAdic::new(5, vec![3, 1])), Valuation::Finite(5)), (unit, valuation));
    /// assert_eq!((None, Valuation::PosInf), QAdic::<UAdic>::zero(5).unit_and_valuation());
    /// ```
    fn unit_and_valuation(&self) -> (Option<Self::Unit>, Valuation<Self::ValuationRing>) {
        let v = self.valuation();
        let u = self.unit();
        (u, v)
    }

    /// Transform into the adic unit and valuation form; transforms zero into `(None, PosInf)`
    ///
    /// ```
    /// # use adic::{normed::{UltraNormed, Valuation}, traits::AdicPrimitive, QAdic, UAdic};
    /// let (unit, valuation) = UAdic::new(5, vec![0, 3, 1]).into_unit_and_valuation();
    /// assert_eq!((Some(UAdic::new(5, vec![3, 1])), Valuation::Finite(1)), (unit, valuation));
    /// assert_eq!((None, Valuation::PosInf), UAdic::zero(5).into_unit_and_valuation());
    /// let (unit, valuation) = QAdic::new(UAdic::new(5, vec![0, 3, 1]), 4).into_unit_and_valuation();
    /// assert_eq!((Some(UAdic::new(5, vec![3, 1])), Valuation::Finite(5)), (unit, valuation));
    /// assert_eq!((None, Valuation::PosInf), QAdic::<UAdic>::zero(5).into_unit_and_valuation());
    /// ```
    fn into_unit_and_valuation(self) -> (Option<Self::Unit>, Valuation<Self::ValuationRing>) where Self: Sized {
        let v = self.valuation();
        let u = self.into_unit();
        (u, v)
    }

}



#[cfg(test)]
mod test {

    use num::rational::Ratio;
    use crate::{traits::AdicPrimitive, EAdic, QAdic, UAdic};
    use super::Normed;

    #[test]
    fn ultra_norm() {

        assert_eq!(Ratio::new(1, 5), EAdic::new_repeating(5, vec![0, 3, 1], vec![2]).norm());
        assert_eq!(Ratio::new(1, 25), UAdic::new(5, vec![0, 0, 3, 1, 2]).norm());
        assert_eq!(Ratio::new(0, 1), UAdic::zero(5).norm());
        assert_eq!(Ratio::new(5, 1), QAdic::new(EAdic::new_repeating(5, vec![0, 3, 1], vec![2]), -2).norm());
        assert_eq!(Ratio::new(1, 625), QAdic::new(UAdic::new(5, vec![0, 0, 3, 1, 2]), 2).norm());
        assert_eq!(Ratio::new(0, 1), QAdic::new(UAdic::zero(5), 1).norm());

        assert!(UAdic::new(5, vec![2, 3, 1]).is_unit());
        assert!(!UAdic::new(5, vec![0, 3, 1]).is_unit());
        assert!(QAdic::new(UAdic::new(5, vec![0, 3, 1]), -1).is_unit());
        assert!(!QAdic::new(UAdic::new(5, vec![2, 3, 1]), 1).is_unit());
        assert!(!QAdic::new(UAdic::new(5, vec![2, 3, 1]), -1).is_unit());

        let u = Some(UAdic::new(5, vec![1, 2]));
        assert_eq!(u, UAdic::new(5, vec![0, 0, 1, 2]).unit());
        assert_eq!(u, QAdic::new(UAdic::new(5, vec![1, 2]), 4).unit());
        assert_eq!(u, QAdic::new(UAdic::new(5, vec![1, 2]), -4).into_unit());
        assert_eq!(None, QAdic::<UAdic>::zero(5).unit());

    }

}
