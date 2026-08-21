use std::ops::Deref;

use crate::link::LinkType;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone)]
pub struct Url(url::Url);

impl Url {
    pub fn new(href: url::Url) -> Self {
        Self(href)
    }
}

impl LinkType for Url {
    fn href(&self) -> &Url {
        self
    }
}

impl Deref for Url {
    type Target = url::Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for Url {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl<'de> Deserialize<'de> for Url {
    fn deserialize<D>(de: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        de.deserialize_str(UrlVisitor)
    }
}

struct UrlVisitor;

impl<'de> Visitor<'de> for UrlVisitor {
    type Value = Url;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an integer between -2^31 and 2^31")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Url(url::Url::parse(s).unwrap()))
    }
}
