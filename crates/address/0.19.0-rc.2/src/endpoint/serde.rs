use crate::{Endpoint, EndpointRef, FromStrVisitor};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Endpoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Endpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FromStrVisitor::new("an endpoint string"))
    }
}

impl<'a> Serialize for EndpointRef<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for EndpointRef<'a> {
    /// The domain name is borrowed from the input, so it must be lowercase and must not contain
    /// escape sequences. Use `Endpoint` to deserialize mixed-case or escaped names.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let endpoint: &str = Deserialize::deserialize(deserializer)?;
        Self::try_from(endpoint).map_err(Error::custom)
    }
}
