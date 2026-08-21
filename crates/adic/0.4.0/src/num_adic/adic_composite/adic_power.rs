use num::{rational::Ratio, traits::Euclid};

use crate::{
    AdicApproximate, AdicNumber, AdicSized, AdicValuation, AdicValuationRing,
    HasDigits, Prime, PrimePower, UAdic,
};



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// An adic with the power of a prime as base ([`apow`](crate::apow))
///
/// This struct represents a p-power adic number, e.g. 9-adic = (3^2)-adic.
/// It represents exactly the same information as a p-adic number.
/// In other words, the information in an `AdicPower` is actually INDEPENDENT of the power.
/// It is scale-free.
///
/// Internally, this holds a raw p-adic number and a power.
/// The [`AdicNumber`] is generic and can be used
///  e.g. with [`AdicIntegers`](crate::AdicInteger) and [`AdicFractions`](crate::AdicFraction).
///
/// This and the [`AdicComposite`](crate::AdicComposite) struct are composite adic structures,
///  as opposed to prime-based structs like [`UAdic`](crate::UAdic) or [`ZAdic`](crate::ZAdic).
/// `AdicPower` is the same informaiton as the prime structure, so technically the same operations apply.
/// But expect fewer features in these composite structures.
///
/// ```
/// # use adic::{PrimePower, apow, qadic, uadic, zadic_approx};
/// let three_adic_36 = uadic!(3, [0, 0, 1, 1]);
/// assert_eq!("1100._3", three_adic_36.to_string());
/// let nine_adic_36 = apow!(three_adic_36.clone(), 2);
/// assert_eq!("40._9", nine_adic_36.to_string());
/// assert_eq!(PrimePower::from((3, 2)), nine_adic_36.p_pow());
/// assert_eq!(2, nine_adic_36.power());
/// assert_eq!(&three_adic_36, nine_adic_36.adic_ref());
/// ```
pub struct AdicPower<A>
where A: AdicNumber {
    pub(super) adic: A,
    pub(super) pp: PrimePower,
}


impl<A> AdicPower<A>
where A: AdicNumber {

    /// `AdicPower` constructor
    pub fn new(adic: A, power: u32) -> Self {
        let p = adic.p();
        AdicPower {
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

impl<A> AdicPower<A>
where A: AdicApproximate + AdicNumber + HasDigits {
    /// [`power`](Self::power) as a digit index valuation
    pub fn power_valuation(&self) -> <Self as HasDigits>::DigitIndex {
        let power_usize = self.power().try_into().expect("power u32 -> usize conversion");
        <Self as HasDigits>::DigitIndex::try_from_usize(power_usize).expect("convert usize to valuation")
    }
}


impl<A> From<UAdic> for AdicPower<A>
where A: AdicNumber {
    fn from(value: UAdic) -> Self {
        Self::new(A::from(value), 1)
    }
}


impl<A> AdicNumber for AdicPower<A>
where A: AdicNumber {

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


impl<A> AdicSized for AdicPower<A>
where A: AdicNumber + AdicSized<ValuationRing = usize>, A::AdicUnit: AdicNumber {

    type ValuationRing = Ratio<A::ValuationRing>;
    type AdicUnit = AdicPower<A::AdicUnit>;

    fn valuation(&self) -> AdicValuation<Self::ValuationRing> {
        match self.adic.valuation() {
            AdicValuation::PosInf => AdicValuation::PosInf,
            // Need to return a RATIO
            // or maybe CompositeRatio or PrimePowerRatio or another custom fraction struct
            AdicValuation::Finite(v) => Ratio::new(v, self.power_usize()).into(),
        }
    }

    fn norm(&self) -> Ratio<u32> {
        self.adic.norm()
    }

    fn unit(&self) -> Option<Self::AdicUnit> {
        let power = self.power();
        self.adic.unit().map(|u| AdicPower::new(u, power))
    }

    fn into_unit(self) -> Option<Self::AdicUnit> {
        let power = self.power();
        self.adic.into_unit().map(|u| AdicPower::new(u, power))
    }

}

impl<A> AdicApproximate for AdicPower<A>
where A: AdicNumber + AdicApproximate {

    fn certainty(&self) -> AdicValuation<A::DigitIndex> {
        match (self.adic.certainty(), self.min_index()) {
            (AdicValuation::Finite(c), AdicValuation::Finite(mi)) => {
                AdicValuation::Finite(c.div_euclid(&self.power_valuation()) + mi)
            },
            _ => AdicValuation::PosInf
        }
    }

}



#[cfg(test)]
mod tests {

    use num::rational::Ratio;
    use crate::{qadic, zadic_approx, AdicApproximate, AdicSized, AdicValuation, HasDigits};

    use super::AdicPower;

    #[test]
    fn adic_power() {

        let ap = AdicPower::new(zadic_approx!(5, 7, [0, 1, 2, 3, 4, 0, 1]), 2);

        assert_eq!(vec![5, 17, 4], ap.digits().collect::<Vec<_>>());
        assert_eq!(Ok(5), ap.digit(0));
        assert_eq!(Ok(17), ap.digit(1));
        assert_eq!(Ok(4), ap.digit(2));
        assert!(matches!(ap.digit(3), Err(_)));

        let qp = AdicPower::new(qadic!(zadic_approx!(5, 7, [1, 2, 3, 4, 0, 1, 2]), -3), 2);

        assert_eq!(AdicValuation::Finite(-2), qp.min_index());
        assert_eq!(vec![5, 17, 4, 11], qp.digits().collect::<Vec<_>>());
        assert_eq!(Ok(5), qp.digit(-2));
        assert_eq!(Ok(17), qp.digit(-1));
        assert_eq!(Ok(4), qp.digit(0));
        assert_eq!(Ok(11), qp.digit(1));
        assert!(matches!(qp.digit(2), Err(_)));

    }

    #[test]
    fn fractional_size() {

        let ap = AdicPower::new(zadic_approx!(5, 7, [0, 1, 2, 3, 4, 0, 1]), 2);
        assert_eq!(AdicValuation::Finite(Ratio::new(1, 2)), ap.valuation());
        assert_eq!(AdicValuation::Finite(3), ap.certainty());

        let up = AdicPower::new(zadic_approx!(5, 6, [1, 2, 3, 4, 0, 1]), 2);
        assert_eq!(Some(up), ap.unit());

    }

}
