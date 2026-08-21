use crate::{PressureUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Pascals;
impl Unit for Pascals {
    const UNIT_NAME: &'static str = "pascals";
    const UNIT_SHORT_NAME: &'static str = "Pa";
    const UNIT_SUFFIX: &'static str = "Pa";
}
impl PressureUnit for Pascals {
    const PASCALS_IN_UNIT: f64 = 1.0;
}

#[macro_export]
macro_rules! pascals {
    ($num:expr) => {
        $crate::Pressure::<$crate::Pascals>::from(&$num)
    };
}
