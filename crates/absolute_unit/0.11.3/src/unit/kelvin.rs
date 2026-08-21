use crate::{TemperatureUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct Kelvin;
impl Unit for Kelvin {
    const UNIT_NAME: &'static str = "kelvin";
    const UNIT_SHORT_NAME: &'static str = "°K";
    const UNIT_SUFFIX: &'static str = "°K";
}
impl TemperatureUnit for Kelvin {
    fn convert_to_kelvin(degrees_in: f64) -> f64 {
        degrees_in
    }
    fn convert_from_kelvin(degrees_k: f64) -> f64 {
        degrees_k
    }
}

#[macro_export]
macro_rules! kelvin {
    ($num:expr) => {
        $crate::Temperature::<$crate::Kelvin>::from(&$num)
    };
}
