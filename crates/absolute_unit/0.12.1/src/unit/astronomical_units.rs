use crate::{LengthUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct AstronomicalUnits;
impl Unit for AstronomicalUnits {
    const UNIT_NAME: &'static str = "astronomical unit";
    const UNIT_SHORT_NAME: &'static str = "AU";
    const UNIT_SUFFIX: &'static str = "AU";
}
impl LengthUnit for AstronomicalUnits {
    const METERS_IN_UNIT: f64 = 149_597_870_700.0;
}

#[macro_export]
macro_rules! astronomical_units {
    ($num:expr) => {
        $crate::Length::<$crate::AstronomicalUnits>::from(&$num)
    };
}
