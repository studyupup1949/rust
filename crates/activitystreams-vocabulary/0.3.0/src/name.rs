use serde::{de, ser};

use crate::{Error, impl_default, impl_display};

mod map;

pub use map::NameMap;

/// A simple, human-readable, plain-text name for an object.
///
/// HTML markup MUST NOT be included.
///
/// **MUST NOT** contain HTML.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Name(String);

impl Name {
    /// Creates a new [Name].
    #[inline]
    pub const fn new() -> Self {
        Self(String::new())
    }

    /// Gets a reference to the inner string.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Attempts to convert a string value into a [Name].
    #[inline]
    pub fn from_string<I: Into<String>>(val: I) -> Result<Self, Error> {
        let name = val.into();
        Self::validate(name.as_str()).map(|_| Self(name))
    }

    /// Validates that the name does not contain HTML.
    #[inline]
    pub fn validate(name: &str) -> Result<(), Error> {
        if Self::contains_html(name) {
            Err(Error::name(format!("name contains HTML: {name}")))
        } else {
            Ok(())
        }
    }

    /// Gets whether the name contains any values with HTML.
    #[inline]
    pub fn contains_html(name: &str) -> bool {
        ammonia::is_html(name)
    }
}

impl From<Name> for String {
    fn from(val: Name) -> Self {
        val.0
    }
}

impl<'n> From<&'n Name> for &'n str {
    fn from(val: &'n Name) -> Self {
        val.as_str()
    }
}

impl TryFrom<String> for Name {
    type Error = Error;

    fn try_from(val: String) -> Result<Self, Self::Error> {
        Self::from_string(val)
    }
}

impl TryFrom<&str> for Name {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        Self::from_string(val)
    }
}

impl_default!(Name);
impl_display!(Name, str);

impl ser::Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        Self::validate(self.as_str())
            .map_err(|err| ser::Error::custom(err.to_string()))
            .and_then(|_| self.0.serialize(serializer))
    }
}

impl<'de> de::Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        <&str>::deserialize(deserializer).and_then(|name| {
            Name::validate(name)
                .map_err(|err| de::Error::custom(format!("{err}")))
                .map(|_| Name(name.into()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let name_str = "A simple note";
        let name_json = format!(r#""{name_str}""#);
        let name: Name = name_str.try_into().unwrap();

        assert_eq!(name.as_str(), name_str);
        assert_eq!(Name::from_string(name_str).as_ref(), Ok(&name));
        assert_eq!(Name::try_from(name_str).as_ref(), Ok(&name));

        assert_eq!(serde_json::to_string(&name).unwrap(), name_json);
        assert_eq!(serde_json::from_str::<Name>(&name_json).unwrap(), name);
    }

    #[test]
    fn test_invalid_name() {
        let name_str = "A <em>simple</em> note";
        let name_json = format!(r#""{name_str}""#);

        assert!(Name::from_string(name_str).is_err());
        assert!(Name::try_from(name_str).is_err());
        assert!(serde_json::from_str::<Name>(&name_json).is_err());
    }
}
