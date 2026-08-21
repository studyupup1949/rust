use crate::{TemperatureUnit, Unit};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bevy_reflect", derive(bevy_reflect::Reflect))]
pub struct Rankine;
impl Unit for Rankine {
    const UNIT_NAME: &'static str = "rankine";
    const UNIT_SHORT_NAME: &'static str = "°R";
    const UNIT_SUFFIX: &'static str = "°R";
}
impl TemperatureUnit for Rankine {
    fn convert_to_kelvin(degrees_in: f64) -> f64 {
        degrees_in * 5. / 9.
    }
    fn convert_from_kelvin(degrees_k: f64) -> f64 {
        degrees_k * 9. / 5.
    }
}

#[macro_export]
macro_rules! rankine {
    ($num:expr) => {
        $crate::Temperature::<$crate::Rankine>::from(&$num)
    };
}
