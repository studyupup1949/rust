use crate::{AngleUnit, Unit};
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Degrees;
impl Unit for Degrees {
    const UNIT_NAME: &'static str = "degrees";
    const UNIT_SHORT_NAME: &'static str = "deg";
    const UNIT_SUFFIX: &'static str = "°";
}
impl AngleUnit for Degrees {
    const RADIANS_IN_UNIT: f64 = PI / 180f64;
}

#[macro_export]
macro_rules! degrees {
    ($num:expr) => {
        $crate::Angle::<$crate::Degrees>::from(&$num)
    };
}

#[macro_export]
macro_rules! degrees_per_second {
    ($num:expr) => {
        $crate::AngularVelocity::<$crate::Degrees, $crate::Seconds>::from(&$num)
    };
}

#[macro_export]
macro_rules! degrees_per_second2 {
    ($num:expr) => {
        $crate::AngularAcceleration::<$crate::Degrees, $crate::Seconds>::from(&$num)
    };
}
