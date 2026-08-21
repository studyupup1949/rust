use serde::{de, ser};

use crate::{Error, Result, impl_default, impl_display};

use super::validate_f64;

/// Indicates the accuracy of position coordinates on a [Place](crate::Place) objects.
///
/// Expressed in properties of percentage. e.g. "94.0" means "94.0% accurate".
///
/// Valid range is `0.0 ..= 100.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Accuracy(f64);

impl Accuracy {
    /// Represents the minimum [Accuracy] value (percentage).
    pub const MIN: f64 = 0.0;
    /// Represents the maximum [Accuracy] value (percentage).
    pub const MAX: f64 = 100.0;

    /// Creates a new [Accuracy].
    pub const fn new() -> Self {
        Self(0.0)
    }

    /// Attempts to convert a [`f64`] into an [Accuracy].
    ///
    /// Returns an error if the float is out-of-range, infinity (+/-), or NaN.
    pub fn from_f64<I: Into<f64>>(val: I) -> Result<Self> {
        validate_f64(val.into()).and_then(Self::validate).map(Self)
    }

    /// Converts the [Accuracy] to an [`f64`].
    pub const fn to_f64(self) -> f64 {
        self.0
    }

    /// Validates a float used as an [Accuracy].
    pub fn validate(val: f64) -> Result<f64> {
        validate_f64(val).and_then(|v| {
            if !(0.0..=100.0).contains(&val) {
                Err(Error::place(format!(
                    "invalid accuracy, out-of-range ({}..={}), have: {v}",
                    Self::MIN,
                    Self::MAX
                )))
            } else {
                Ok(v)
            }
        })
    }
}

impl Eq for Accuracy {}

impl TryFrom<f64> for Accuracy {
    type Error = Error;

    fn try_from(val: f64) -> Result<Self> {
        Self::from_f64(val)
    }
}

impl From<Accuracy> for f64 {
    fn from(val: Accuracy) -> Self {
        val.to_f64()
    }
}

impl ser::Serialize for Accuracy {
    fn serialize<S>(&self, s: S) -> ::core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        Self::validate(self.0)
            .map_err(|err| ser::Error::custom(format!("{err}")))
            .and_then(|f| f.serialize(s))
    }
}

impl<'de> de::Deserialize<'de> for Accuracy {
    fn deserialize<D>(d: D) -> ::core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        f64::deserialize(d)
            .and_then(|f| Self::validate(f).map_err(|err| de::Error::custom(format!("{err}"))))
            .map(Self)
    }
}

impl_default!(Accuracy);
impl_display!(Accuracy, json);
