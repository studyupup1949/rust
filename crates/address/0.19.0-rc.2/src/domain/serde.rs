use crate::{Domain, DomainRef, FromStrVisitor};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Domain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Domain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FromStrVisitor::new("a domain string"))
    }
}

impl<'a> Serialize for DomainRef<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for DomainRef<'a> {
    /// The name is borrowed from the input, so it must be lowercase and must not contain escape
    /// sequences. Use `Domain` to deserialize mixed-case or escaped names.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name: &str = Deserialize::deserialize(deserializer)?;
        Self::try_from(name).map_err(Error::custom)
    }
}
