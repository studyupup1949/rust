use crate::{AngleUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Radians;
impl Unit for Radians {
    const UNIT_NAME: &'static str = "radians";
    const UNIT_SHORT_NAME: &'static str = "rad";
    const UNIT_SUFFIX: &'static str = "㎭";
}
impl AngleUnit for Radians {
    const RADIANS_IN_UNIT: f64 = 1.0;
}

#[macro_export]
macro_rules! radians {
    ($num:expr) => {
        $crate::Angle::<$crate::Radians>::from(&$num)
    };
}

#[macro_export]
macro_rules! radians_per_second {
    ($num:expr) => {
        $crate::AngularVelocity::<$crate::Radians, $crate::Seconds>::from(&$num)
    };
}

#[macro_export]
macro_rules! radians_per_second2 {
    ($num:expr) => {
        $crate::AngularAcceleration::<$crate::Radians, $crate::Seconds>::from(&$num)
    };
}
