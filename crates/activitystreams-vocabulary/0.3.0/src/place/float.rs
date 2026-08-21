use serde::{de, ser};

use crate::{Error, Result, impl_default, impl_display};

use super::validate_f64;

/// Represents a floating-point value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Float(f64);

impl Float {
    /// Creates a new [Float].
    pub const fn new() -> Self {
        Self(0.0)
    }

    /// Attempts to convert a [`f64`] into a [Float].
    pub fn from_f64<I: Into<f64>>(val: I) -> Result<Self> {
        validate_f64(val.into()).map(Self)
    }

    /// Converts a [Float] into a [`f64`].
    pub const fn to_f64(self) -> f64 {
        self.0
    }
}

impl Eq for Float {}

impl TryFrom<f64> for Float {
    type Error = Error;

    fn try_from(val: f64) -> Result<Self> {
        Self::from_f64(val)
    }
}

impl From<Float> for f64 {
    fn from(val: Float) -> Self {
        val.to_f64()
    }
}

impl_default!(Float);
impl_display!(Float, json);

impl ser::Serialize for Float {
    fn serialize<S>(&self, s: S) -> ::core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        validate_f64(self.0)
            .map_err(|err| ser::Error::custom(format!("{err}")))
            .and_then(|f| f.serialize(s))
    }
}

impl<'de> de::Deserialize<'de> for Float {
    fn deserialize<D>(d: D) -> ::core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        f64::deserialize(d)
            .and_then(|f| validate_f64(f).map_err(|err| de::Error::custom(format!("{err}"))))
            .map(Self)
    }
}
