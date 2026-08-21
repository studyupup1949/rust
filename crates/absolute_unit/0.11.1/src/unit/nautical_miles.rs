use crate::{LengthUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct NauticalMiles;
impl Unit for NauticalMiles {
    const UNIT_NAME: &'static str = "nautical miles";
    const UNIT_SHORT_NAME: &'static str = "nm";
    const UNIT_SUFFIX: &'static str = "nm";
}
impl LengthUnit for NauticalMiles {
    const METERS_IN_UNIT: f64 = 1_852.;
}

#[macro_export]
macro_rules! nautical_miles {
    ($num:expr) => {
        $crate::Length::<$crate::NauticalMiles>::from(&$num)
    };
}

#[macro_export]
macro_rules! nm {
    ($num:expr) => {
        $crate::Length::<$crate::NauticalMiles>::from(&$num)
    };
}

#[macro_export]
macro_rules! nautical_miles_per_hour {
    ($num:expr) => {
        $crate::Velocity::<$crate::NauticalMiles, $crate::Hours>::from(&$num)
    };
}

#[macro_export]
macro_rules! knots {
    ($num:expr) => {
        $crate::Velocity::<$crate::NauticalMiles, $crate::Hours>::from(&$num)
    };
}
