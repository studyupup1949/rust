use crate::{LengthUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Feet;
impl Unit for Feet {
    const UNIT_NAME: &'static str = "feet";
    const UNIT_SHORT_NAME: &'static str = "ft";
    const UNIT_SUFFIX: &'static str = "'";
}
impl LengthUnit for Feet {
    const METERS_IN_UNIT: f64 = 0.304_800_000;
}

#[macro_export]
macro_rules! feet {
    ($num:expr) => {
        $crate::Length::<$crate::Feet>::from(&$num)
    };
}

#[macro_export]
macro_rules! feet2 {
    ($num:expr) => {
        $crate::Area::<$crate::Feet>::from(&$num)
    };
}

#[macro_export]
macro_rules! feet_per_second {
    ($num:expr) => {
        $crate::Velocity::<$crate::Feet, $crate::Seconds>::from(&$num)
    };
}

#[macro_export]
macro_rules! feet_per_second2 {
    ($num:expr) => {
        $crate::Acceleration::<$crate::Feet, $crate::Seconds>::from(&$num)
    };
}
