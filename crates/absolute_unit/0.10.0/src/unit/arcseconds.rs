use crate::{AngleUnit, Unit};
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct ArcSeconds;
impl Unit for ArcSeconds {
    const UNIT_NAME: &'static str = "arcseconds";
    const UNIT_SHORT_NAME: &'static str = "arcsec";
    const UNIT_SUFFIX: &'static str = "\"";
}
impl AngleUnit for ArcSeconds {
    const RADIANS_IN_UNIT: f64 = PI / 180f64 / 60f64 / 60f64;
}

#[macro_export]
macro_rules! arcseconds {
    ($num:expr) => {
        $crate::Angle::<$crate::ArcSeconds>::from(&$num)
    };
}
