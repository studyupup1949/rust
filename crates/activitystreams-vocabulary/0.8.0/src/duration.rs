use std::str::FromStr;

use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use xsd_types::Duration as XsdDuration;

use crate::{Error, impl_default, impl_display};

/// Represents a `duration` as defined by [XSD 1.1 Part 2](https://www.w3.org/TR/xmlschema11-2/#duration).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Duration(String);

impl Duration {
    /// Creates a new [Duration].
    pub const fn new() -> Self {
        Self(String::new())
    }

    /// Gets the string representation of the [Duration].
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl_default!(Duration);
impl_display!(Duration, str);

impl TryFrom<String> for Duration {
    type Error = Error;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        val.as_str().try_into()
    }
}

impl TryFrom<&str> for Duration {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        XsdDuration::from_str(val)
            .map(|_| Duration(val.into()))
            .map_err(|err| Error::Duration(format!("{err}")))
    }
}

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <&str>::deserialize(deserializer).and_then(|s| {
            Duration::try_from(s).map_err(|err| de::Error::custom(format!("parsing error: {err}")))
        })
    }
}
