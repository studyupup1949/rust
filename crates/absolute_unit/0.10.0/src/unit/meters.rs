use crate::{LengthUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Meters;
impl Unit for Meters {
    const UNIT_NAME: &'static str = "meters";
    const UNIT_SHORT_NAME: &'static str = "m";
    const UNIT_SUFFIX: &'static str = "m";
}
impl LengthUnit for Meters {
    const METERS_IN_UNIT: f64 = 1.0;
}

#[macro_export]
macro_rules! meters {
    ($num:expr) => {
        $crate::Length::<$crate::Meters>::from(&$num)
    };
}

#[macro_export]
macro_rules! meters2 {
    ($num:expr) => {
        $crate::Area::<$crate::Meters>::from(&$num)
    };
}

#[macro_export]
macro_rules! meters_per_second {
    ($num:expr) => {
        $crate::Velocity::<$crate::Meters, $crate::Seconds>::from(&$num)
    };
}

#[macro_export]
macro_rules! meters_per_second2 {
    ($num:expr) => {
        $crate::Acceleration::<$crate::Meters, $crate::Seconds>::from(&$num)
    };
}
