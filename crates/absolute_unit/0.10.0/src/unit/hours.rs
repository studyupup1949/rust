use crate::{TimeUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Hours;
impl Unit for Hours {
    const UNIT_NAME: &'static str = "hours";
    const UNIT_SHORT_NAME: &'static str = "h";
    const UNIT_SUFFIX: &'static str = "h";
}
impl TimeUnit for Hours {
    const SECONDS_IN_UNIT: f64 = 3_600.;
}

#[macro_export]
macro_rules! hours {
    ($num:expr) => {
        $crate::Time::<$crate::Hours>::from(&$num)
    };
}
