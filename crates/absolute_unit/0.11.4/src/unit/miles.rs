use crate::{LengthUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct Miles;
impl Unit for Miles {
    const UNIT_NAME: &'static str = "miles";
    const UNIT_SHORT_NAME: &'static str = "miles";
    const UNIT_SUFFIX: &'static str = "miles";
}
impl LengthUnit for Miles {
    const METERS_IN_UNIT: f64 = 1609.34;
}

#[macro_export]
macro_rules! miles {
    ($num:expr) => {
        $crate::Length::<$crate::Miles>::from(&$num)
    };
}

#[macro_export]
macro_rules! miles_per_hour {
    ($num:expr) => {
        $crate::Velocity::<$crate::Miles, $crate::Hours>::from(&$num)
    };
}
