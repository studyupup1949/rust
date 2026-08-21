use crate::{AngleUnit, Unit};
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct ArcMinutes;
impl Unit for ArcMinutes {
    const UNIT_NAME: &'static str = "arcminutes";
    const UNIT_SHORT_NAME: &'static str = "arcmin";
    const UNIT_SUFFIX: &'static str = "'";
}
impl AngleUnit for ArcMinutes {
    const RADIANS_IN_UNIT: f64 = PI / 180f64 / 60f64;
}

#[macro_export]
macro_rules! arcminutes {
    ($num:expr) => {
        $crate::Angle::<$crate::ArcMinutes>::from(&$num)
    };
}
