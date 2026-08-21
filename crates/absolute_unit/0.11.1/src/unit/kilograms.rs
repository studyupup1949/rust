use crate::{MassUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Kilograms;
impl Unit for Kilograms {
    const UNIT_NAME: &'static str = "kilograms";
    const UNIT_SHORT_NAME: &'static str = "kg";
    const UNIT_SUFFIX: &'static str = "kg";
}
impl MassUnit for Kilograms {
    const GRAMS_IN_UNIT: f64 = 1_000.0;
}

#[macro_export]
macro_rules! kilograms {
    ($num:expr) => {
        $crate::Mass::<$crate::Kilograms>::from(&$num)
    };
}

#[macro_export]
macro_rules! kilograms_per_meter3 {
    ($num:expr) => {
        $crate::Density::<$crate::Kilograms, $crate::Meters>::from(&$num)
    };
}

#[macro_export]
macro_rules! kilograms_meter2 {
    ($num:expr) => {
        $crate::RotationalInertia::<$crate::Kilograms, $crate::Meters>::from(&$num)
    };
}

#[macro_export]
macro_rules! kilograms_per_second {
    ($num:expr) => {
        $crate::MassRate::<$crate::Kilograms, $crate::Seconds>::from(&$num)
    };
}
