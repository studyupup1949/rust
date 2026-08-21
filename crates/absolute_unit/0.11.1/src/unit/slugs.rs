use crate::{MassUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Slugs;
impl Unit for Slugs {
    const UNIT_NAME: &'static str = "slugs";
    const UNIT_SHORT_NAME: &'static str = "slug";
    const UNIT_SUFFIX: &'static str = "slug";
}
impl MassUnit for Slugs {
    const GRAMS_IN_UNIT: f64 = 14_593.90;
}

#[macro_export]
macro_rules! slugs {
    ($num:expr) => {
        $crate::Mass::<$crate::Slugs>::from(&$num)
    };
}

#[macro_export]
macro_rules! slugs_per_foot3 {
    ($num:expr) => {
        $crate::Density::<$crate::Slugs, $crate::Feet>::from(&$num)
    };
}
