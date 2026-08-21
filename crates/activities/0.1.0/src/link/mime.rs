#[derive(Debug, Clone)]
pub struct Mime(::mime::Mime);

use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

impl Serialize for Mime {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.essence_str())
    }
}

impl<'de> Deserialize<'de> for Mime {
    fn deserialize<D>(de: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        de.deserialize_str(MimeVisitor)
    }
}

struct MimeVisitor;

impl<'de> Visitor<'de> for MimeVisitor {
    type Value = Mime;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("couldn't parse mime value.")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(mime) = mime::Mime::from_str(s).ok() {
            Ok(Mime(mime))
        } else {
            Err(E::custom("Couldn't parse mime value."))
        }
    }
}
