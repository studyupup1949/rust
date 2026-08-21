use crate::{FromStrVisitor, Host, HostRef};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Host {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Host {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FromStrVisitor::new("a host string"))
    }
}

impl<'a> Serialize for HostRef<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for HostRef<'a> {
    /// A domain name is borrowed from the input, so it must be lowercase and must not contain
    /// escape sequences. Use `Host` to deserialize mixed-case or escaped names.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let host: &str = Deserialize::deserialize(deserializer)?;
        Self::try_from(host).map_err(Error::custom)
    }
}
