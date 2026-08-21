use serde::{de, ser};

use crate::{Error, Result, impl_default, impl_display};

use super::validate_f64;

/// Represents the radius from the given latitude and longitude for a [Place](crate::Place).
///
/// The units is expressed by the `units` property.
///
/// If `units` is not specified, the default is assumed to be "m" indicating "meters".
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Radius(f64);

impl Radius {
    /// Represents the minimum [Radius] value.
    pub const MIN: f64 = 0.0;

    /// Creates a new [Radius].
    pub const fn new() -> Self {
        Self(0.0)
    }

    /// Attempts to convert a [`f64`] into an [Radius].
    ///
    /// Returns an error if the float is out-of-range, infinity (+/-), or NaN.
    pub fn from_f64<I: Into<f64>>(val: I) -> Result<Self> {
        validate_f64(val.into()).and_then(Self::validate).map(Self)
    }

    /// Converts the [Radius] to an [`f64`].
    pub const fn to_f64(self) -> f64 {
        self.0
    }

    /// Validates a float used as an [Radius].
    pub fn validate(val: f64) -> Result<f64> {
        validate_f64(val).and_then(|v| {
            if val < Self::MIN {
                Err(Error::place(format!(
                    "invalid radius, out-of-range (>= {}), have: {v}",
                    Self::MIN
                )))
            } else {
                Ok(v)
            }
        })
    }
}

impl Eq for Radius {}

impl TryFrom<f64> for Radius {
    type Error = Error;

    fn try_from(val: f64) -> Result<Self> {
        Self::from_f64(val)
    }
}

impl From<Radius> for f64 {
    fn from(val: Radius) -> Self {
        val.to_f64()
    }
}

impl_default!(Radius);
impl_display!(Radius, json);

impl ser::Serialize for Radius {
    fn serialize<S>(&self, s: S) -> ::core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        Self::validate(self.0)
            .map_err(|err| ser::Error::custom(format!("{err}")))
            .and_then(|f| f.serialize(s))
    }
}

impl<'de> de::Deserialize<'de> for Radius {
    fn deserialize<D>(d: D) -> ::core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        f64::deserialize(d)
            .and_then(|f| Self::validate(f).map_err(|err| de::Error::custom(format!("{err}"))))
            .map(Self)
    }
}
