use num::rational::Ratio;

use crate::{
    divisible::{Prime, PrimePower},
    normed::{Normed, UltraNormed, Valuation, ValuationRing},
    traits::{AdicPrimitive, HasDigits},
    UAdic,
};



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// An adic with the power of a prime as base
///
/// This struct represents a p-power adic number, e.g. 9-adic = (3^2)-adic.
/// It represents exactly the same information as a p-adic number.
/// In other words, the information in an `PowAdic` is actually **independent** of the power.
/// It is scale-free.
///
/// Internally, this holds a raw p-adic number and a power.
/// The struct is generic and can be used
///  e.g. with [`AdicIntegers`](crate::traits::AdicInteger) and [`QAdics`](crate::QAdic).
///
/// This and the [`MAdic`](crate::num_adic::MAdic) struct are composite adic structures,
///  as opposed to prime-based structs like [`UAdic`](crate::UAdic) or [`ZAdic`](crate::ZAdic).
/// `PowAdic` has the same information as the prime structure, so technically the same operations apply.
/// Expect fewer features generally in these composite structures.
///
/// ```
/// # use adic::{divisible::PrimePower, num_adic::PowAdic, UAdic};
/// let three_adic_36 = UAdic::new(3, vec![0, 0, 1, 1]);
/// assert_eq!("1100._3", three_adic_36.to_string());
/// let nine_adic_36 = PowAdic::new(three_adic_36.clone(), 2);
/// assert_eq!("40._9", nine_adic_36.to_string());
/// assert_eq!(PrimePower::from((3, 2)), nine_adic_36.p_pow());
/// assert_eq!(2, nine_adic_36.power());
/// assert_eq!(&three_adic_36, nine_adic_36.adic_ref());
/// ```
pub struct PowAdic<A>
where A: AdicPrimitive {
    pub(super) adic: A,
    pub(super) pp: PrimePower,
}


impl<A> PowAdic<A>
where A: AdicPrimitive {

    /// `PowAdic` constructor
    pub fn new(adic: A, power: u32) -> Self {
        let p = adic.p();
        PowAdic {
            adic,
            pp: PrimePower::from((p, power)),
        }
    }

    /// Power of this p-adic
    pub fn power(&self) -> u32 {
        self.pp.power()
    }

    /// [`power`](Self::power) as a usize
    pub fn power_usize(&self) -> usize {
        self.power().try_into().expect("power u32 -> usize conversion")
    }

    /// [`power`](Self::power) as an isize
    pub fn power_isize(&self) -> isize {
        self.power().try_into().expect("power u32 -> isize conversion")
    }

    /// `p`^`power` as a [`PrimePower`]
    pub fn p_pow(&self) -> PrimePower {
        self.pp
    }

    /// Reference to the base adic number
    pub fn adic_ref(&self) -> &A {
        &self.adic
    }

}

impl<A> PowAdic<A>
where Self: HasDigits, A: AdicPrimitive {
    /// [`power`](Self::power) as a digit index valuation
    pub fn power_valuation(&self) -> <Self as HasDigits>::DigitIndex {
        let power_usize = self.power().try_into().expect("power u32 -> usize conversion");
        <Self as HasDigits>::DigitIndex::try_from_usize(power_usize).expect("convert usize to valuation")
    }
}


impl<A> From<UAdic> for PowAdic<A>
where A: AdicPrimitive {
    fn from(value: UAdic) -> Self {
        Self::new(A::from(value), 1)
    }
}


impl<A> AdicPrimitive for PowAdic<A>
where A: AdicPrimitive {

    fn zero<P>(p: P) -> Self
    where P: Into<Prime> {
        Self::new(A::zero(p), 1)
    }
    fn one<P>(p: P) -> Self
    where P: Into<Prime> {
        Self::new(A::one(p), 1)
    }
    fn p(&self) -> Prime {
        self.pp.p()
    }

}


impl<A> Normed for PowAdic<A>
where A: AdicPrimitive + Normed, A::Unit: AdicPrimitive {
    type Norm = A::Norm;
    type Unit = PowAdic<A::Unit>;
    fn norm(&self) -> A::Norm {
        self.adic.norm()
    }
    fn unit(&self) -> Option<Self::Unit> {
        let power = self.power();
        self.adic.unit().map(|u| PowAdic::new(u, power))
    }
    fn into_unit(self) -> Option<Self::Unit> {
        let power = self.power();
        self.adic.into_unit().map(|u| PowAdic::new(u, power))
    }
    fn from_norm_and_unit(norm: Self::Norm, u: Self::Unit) -> Self {
        let power = u.power();
        Self::new(A::from_norm_and_unit(norm, u.adic), power)
    }
    fn from_unit(u: Self::Unit) -> Self {
        let power = u.power();
        Self::new(A::from_unit(u.adic), power)
    }
    fn is_unit(&self) -> bool {
        self.adic.is_unit()
    }
}

impl<A> UltraNormed for PowAdic<A>
where A: AdicPrimitive + UltraNormed<ValuationRing = usize>, A::Unit: AdicPrimitive {
    type ValuationRing = Ratio<A::ValuationRing>;
    fn from_unit_and_valuation(u: Self::Unit, v: Valuation<Self::ValuationRing>) -> Self {
        match v {
            Valuation::Finite(v) => {
                let power = u.power();
                let v_power = A::ValuationRing::try_from_u32(power).expect("Convert u32 to ValuationRing");
                let unit_valuation = Ratio::new(v.numer() * v_power, *v.denom());
                assert!(unit_valuation.is_integer(), "Cannot create PowAdic with valuation that does not match the power");
                let unit_valuation = unit_valuation.into_raw().0.into();
                Self::new(A::from_unit_and_valuation(u.adic, unit_valuation), power)
            },
            Valuation::PosInf => Self::zero(u.p()),
        }
    }
    fn valuation(&self) -> Valuation<Self::ValuationRing> {
        match self.adic.valuation() {
            Valuation::PosInf => Valuation::PosInf,
            // Need to return a RATIO
            // or maybe CompositeRatio or PrimePowerRatio or another custom fraction struct
            Valuation::Finite(v) => Ratio::new(v, self.power_usize()).into(),
        }
    }
}



#[cfg(test)]
mod tests {
    use assertables::assert_matches;

    use num::rational::Ratio;
    use crate::{
        normed::{Normed, UltraNormed, Valuation},
        traits::{HasApproximateDigits, HasDigits},
    };

    use super::PowAdic;

    #[test]
    fn adic_power() {

        let ap = PowAdic::new(zadic_approx!(5, 7, [0, 1, 2, 3, 4, 0, 1]), 2);

        assert_eq!(vec![5, 17, 4], ap.digits().collect::<Vec<_>>());
        assert_eq!(Ok(5), ap.digit(0));
        assert_eq!(Ok(17), ap.digit(1));
        assert_eq!(Ok(4), ap.digit(2));
        assert_matches!(ap.digit(3), Err(_));

        let qp = PowAdic::new(qadic!(zadic_approx!(5, 7, [1, 2, 3, 4, 0, 1, 2]), -3), 2);

        assert_eq!(Valuation::Finite(-2), qp.min_index());
        assert_eq!(vec![5, 17, 4, 11], qp.digits().collect::<Vec<_>>());
        assert_eq!(Ok(5), qp.digit(-2));
        assert_eq!(Ok(17), qp.digit(-1));
        assert_eq!(Ok(4), qp.digit(0));
        assert_eq!(Ok(11), qp.digit(1));
        assert_matches!(qp.digit(2), Err(_));

    }

    #[test]
    fn normed() {

        let a = PowAdic::new(eadic!(5, [1, 2, 3]), 2);
        assert!(a.is_unit());
        assert_eq!(Ratio::from(1), a.norm());
        assert_eq!(Valuation::Finite(Ratio::from(0)), a.valuation());
        assert_eq!(Some(a.clone()), a.unit());

        let a = PowAdic::new(eadic!(5, [0, 1, 2]), 2);
        assert!(!a.is_unit());
        assert_eq!(Ratio::new(1, 5), a.norm());
        assert_eq!(Valuation::Finite(Ratio::new(1, 2)), a.valuation());
        assert_eq!(Some(PowAdic::new(eadic!(5, [1, 2]), 2)), a.unit());

        let a = PowAdic::new(eadic!(5, [0, 0, 1]), 2);
        assert!(!a.is_unit());
        assert_eq!(Ratio::new(1, 25), a.norm());
        assert_eq!(Valuation::Finite(Ratio::new(1, 1)), a.valuation());
        assert_eq!(Some(PowAdic::new(eadic!(5, [1]), 2)), a.unit());

    }

    #[test]
    fn fractional_size() {

        let ap = PowAdic::new(zadic_approx!(5, 7, [0, 1, 2, 3, 4, 0, 1]), 2);
        assert_eq!(Valuation::Finite(Ratio::new(1, 2)), ap.valuation());
        assert_eq!(Valuation::Finite(3), ap.certainty());

        let up = PowAdic::new(zadic_approx!(5, 6, [1, 2, 3, 4, 0, 1]), 2);
        assert_eq!(Some(up), ap.unit());

    }

}
