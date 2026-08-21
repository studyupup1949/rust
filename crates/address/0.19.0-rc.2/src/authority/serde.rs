use crate::{Authority, AuthorityRef, FromStrVisitor};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Authority {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Authority {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FromStrVisitor::new("an authority string"))
    }
}

impl<'a> Serialize for AuthorityRef<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for AuthorityRef<'a> {
    /// A domain name is borrowed from the input, so it must be lowercase and must not contain
    /// escape sequences. Use `Authority` to deserialize mixed-case or escaped names.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let authority: &str = Deserialize::deserialize(deserializer)?;
        Self::try_from(authority).map_err(Error::custom)
    }
}
