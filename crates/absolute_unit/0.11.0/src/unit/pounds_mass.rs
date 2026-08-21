use crate::{MassUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct PoundsMass;
impl Unit for PoundsMass {
    const UNIT_NAME: &'static str = "pounds";
    const UNIT_SHORT_NAME: &'static str = "lb";
    const UNIT_SUFFIX: &'static str = "lb";
}
impl MassUnit for PoundsMass {
    const GRAMS_IN_UNIT: f64 = 453.592_37;
}

#[macro_export]
macro_rules! pounds_mass {
    ($num:expr) => {
        $crate::Mass::<$crate::PoundsMass>::from(&$num)
    };
}

#[macro_export]
macro_rules! pounds_mass_per_second {
    ($num:expr) => {
        $crate::MassRate::<$crate::PoundsMass, $crate::Seconds>::from(&$num)
    };
}

#[macro_export]
macro_rules! pounds_per_feet3 {
    ($num:expr) => {
        $crate::Density::<$crate::PoundsMass, $crate::Feet>::from(&$num)
    };
}
