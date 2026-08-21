use crate::{
    Real, Scalar, Unit, impl_value_type_conversions, supports_absdiffeq, supports_cancellation,
    supports_quantity_ops, supports_scalar_ops, supports_shift_ops, supports_value_type_conversion,
};
use std::{fmt, fmt::Debug, marker::PhantomData};

pub trait PressureUnit: Unit + Copy + Debug + Eq + PartialEq + 'static {
    const PASCALS_IN_UNIT: f64;
}

// Force / Area
// Mass * Length / (Time * Time * Length * Length)
// Mass / (Time * Time * Length)
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct Pressure<UnitPressure: PressureUnit> {
    v: Real,
    #[cfg_attr(feature = "serde", serde(skip))]
    phantom_1: PhantomData<UnitPressure>,
}
supports_quantity_ops!(Pressure<A>, PressureUnit);
supports_shift_ops!(Pressure<A1>, Pressure<A2>, PressureUnit);
supports_scalar_ops!(Pressure<A>, PressureUnit);
supports_cancellation!(Pressure<A1>, Pressure<A2>, PressureUnit);
supports_absdiffeq!(Pressure<A>, PressureUnit);
supports_value_type_conversion!(Pressure<A>, PressureUnit, impl_value_type_conversions);

impl<P> fmt::Display for Pressure<P>
where
    P: PressureUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.v.0, f)?;
        write!(f, "{}", P::UNIT_SHORT_NAME,)
    }
}

impl<'a, PA, PB> From<&'a Pressure<PB>> for Pressure<PA>
where
    PA: PressureUnit,
    PB: PressureUnit,
{
    fn from(v: &'a Pressure<PB>) -> Self {
        let pressure_ratio = Real(PB::PASCALS_IN_UNIT / PA::PASCALS_IN_UNIT);
        Self {
            v: v.v * pressure_ratio,
            phantom_1: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{pascals, psf};
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_pressure() {
        let psf = psf!(2_116.22f32);
        let pas = pascals!(101_325f32);
        println!("{psf}");
        println!("{pas}");
        assert_abs_diff_eq!(psf, psf!(pas), epsilon = 0.01);
    }
}
