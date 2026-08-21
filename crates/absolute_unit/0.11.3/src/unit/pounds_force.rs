use crate::{Feet, ForceUnit, PoundsMass, Seconds, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct PoundsForce;
impl Unit for PoundsForce {
    const UNIT_NAME: &'static str = "pounds(force)";
    const UNIT_SHORT_NAME: &'static str = "lbf";
    const UNIT_SUFFIX: &'static str = "lbf";
}
impl ForceUnit for PoundsForce {
    const NEWTONS_IN_UNIT: f64 = 1. / 0.224_809;

    type UnitMass = PoundsMass;
    type UnitLength = Feet;
    type UnitTime = Seconds;
}

#[macro_export]
macro_rules! pounds_force {
    ($num:expr) => {
        $crate::Force::<$crate::PoundsForce>::from(&$num)
    };
}

#[macro_export]
macro_rules! pdl {
    ($num:expr) => {
        $crate::Force::<$crate::PoundsForce>::from(&$num)
    };
}
