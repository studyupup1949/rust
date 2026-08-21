use crate::{TemperatureUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Fahrenheit;
impl Unit for Fahrenheit {
    const UNIT_NAME: &'static str = "fahrenheit";
    const UNIT_SHORT_NAME: &'static str = "°F";
    const UNIT_SUFFIX: &'static str = "°F";
}
impl TemperatureUnit for Fahrenheit {
    fn convert_to_kelvin(degrees_in: f64) -> f64 {
        (degrees_in - 32.) * 5. / 9. + 273.15
    }
    fn convert_from_kelvin(degrees_k: f64) -> f64 {
        (degrees_k - 273.15) * 9. / 5. + 32.
    }
}

#[macro_export]
macro_rules! fahrenheit {
    ($num:expr) => {
        $crate::Temperature::<$crate::Fahrenheit>::from(&$num)
    };
}
