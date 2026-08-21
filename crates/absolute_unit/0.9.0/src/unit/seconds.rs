use crate::{TimeUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Seconds;
impl Unit for Seconds {
    const UNIT_NAME: &'static str = "seconds";
    const UNIT_SHORT_NAME: &'static str = "s";
    const UNIT_SUFFIX: &'static str = "s";
}
impl TimeUnit for Seconds {
    const SECONDS_IN_UNIT: f64 = 1.;
}

#[macro_export]
macro_rules! seconds {
    ($num:expr) => {
        $crate::Time::<$crate::Seconds>::from(&$num)
    };
}
