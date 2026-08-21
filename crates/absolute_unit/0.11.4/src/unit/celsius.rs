use crate::{TemperatureUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct Celsius;
impl Unit for Celsius {
    const UNIT_NAME: &'static str = "celsius";
    const UNIT_SHORT_NAME: &'static str = "°C";
    const UNIT_SUFFIX: &'static str = "°C";
}
impl TemperatureUnit for Celsius {
    fn convert_to_kelvin(degrees_in: f64) -> f64 {
        degrees_in + 273.15
    }
    fn convert_from_kelvin(degrees_k: f64) -> f64 {
        degrees_k - 273.15
    }
}

#[macro_export]
macro_rules! celsius {
    ($num:expr) => {
        $crate::Temperature::<$crate::Celsius>::from(&$num)
    };
}
