use crate::{PressureUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct PoundsSquareFoot;
impl Unit for PoundsSquareFoot {
    const UNIT_NAME: &'static str = "pounds per square foot";
    const UNIT_SHORT_NAME: &'static str = "lb/ft^2";
    const UNIT_SUFFIX: &'static str = "lb/ft^2";
}
impl PressureUnit for PoundsSquareFoot {
    const PASCALS_IN_UNIT: f64 = 47.880;
}

#[macro_export]
macro_rules! pounds_square_foot {
    ($num:expr) => {
        $crate::Pressure::<$crate::PoundsSquareFoot>::from(&$num)
    };
}

#[macro_export]
macro_rules! psf {
    ($num:expr) => {
        $crate::Pressure::<$crate::PoundsSquareFoot>::from(&$num)
    };
}
