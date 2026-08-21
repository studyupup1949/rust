use crate::{ForceUnit, Kilograms, Meters, Seconds, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct Newtons;
impl Unit for Newtons {
    const UNIT_NAME: &'static str = "newtons";
    const UNIT_SHORT_NAME: &'static str = "N";
    const UNIT_SUFFIX: &'static str = "N";
}
impl ForceUnit for Newtons {
    const NEWTONS_IN_UNIT: f64 = 1.0;

    type UnitMass = Kilograms;
    type UnitLength = Meters;
    type UnitTime = Seconds;
}

#[macro_export]
macro_rules! newtons {
    ($num:expr) => {
        $crate::Force::<$crate::Newtons>::from(&$num)
    };
}

#[macro_export]
macro_rules! newton_meters {
    ($num:expr) => {
        $crate::Torque::<$crate::Newtons, $crate::Meters>::from(&$num)
    };
}
