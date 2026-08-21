use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use langtag::LangTag;

use crate::{Error, impl_default, impl_display};

mod map;

pub use map::LanguageMap;

/// Represents a language tag as defined by [BCP47](https://tools.ietf.org/html/bcp47).
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct LanguageTag(String);

impl LanguageTag {
    /// Creates a new [LanguageTag].
    pub const fn new() -> Self {
        Self(String::new())
    }

    /// Gets the string representation of the [LanguageTag].
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for LanguageTag {
    type Error = Error;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        val.as_str().try_into()
    }
}

impl TryFrom<&str> for LanguageTag {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        LangTag::from_str(val)
            .map(|_| LanguageTag(val.into()))
            .map_err(|err| Error::LanguageTag(format!("{err}")))
    }
}

impl_default!(LanguageTag);
impl_display!(LanguageTag, str);

impl Serialize for LanguageTag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LanguageTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <&str>::deserialize(deserializer).and_then(|s| {
            LanguageTag::try_from(s)
                .map_err(|err| de::Error::custom(format!("invalid LanguageTag: {err}")))
        })
    }
}
