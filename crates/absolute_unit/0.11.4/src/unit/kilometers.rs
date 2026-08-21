use crate::{LengthUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct Kilometers;
impl Unit for Kilometers {
    const UNIT_NAME: &'static str = "kilometers";
    const UNIT_SHORT_NAME: &'static str = "km";
    const UNIT_SUFFIX: &'static str = "km";
}
impl LengthUnit for Kilometers {
    const METERS_IN_UNIT: f64 = 1_000.0;
}

#[macro_export]
macro_rules! kilometers {
    ($num:expr) => {
        $crate::Length::<$crate::Kilometers>::from(&$num)
    };
}
