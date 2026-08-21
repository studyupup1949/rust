use serde::de::{self, Deserialize};
use serde::ser::{self, Serialize};
use sqlx::types::uuid::Uuid;

/// Serializer function for UUID.
pub fn ser_uuid<S: ser::Serializer>(val: &Uuid, s: S) -> Result<S::Ok, S::Error> {
    val.to_string().serialize(s)
}

/// Deserializer function for UUID.
pub fn de_uuid<'de, D: de::Deserializer<'de>>(d: D) -> Result<Uuid, D::Error> {
    <&str>::deserialize(d).and_then(|s| {
        Uuid::parse_str(s).map_err(|err| de::Error::custom(format!("invalid UUID: {err}")))
    })
}

/// Serializer function for UUID list.
pub fn ser_uuid_opt<S: ser::Serializer>(val: &Option<Uuid>, s: S) -> Result<S::Ok, S::Error> {
    val.map(|u| u.to_string()).serialize(s)
}

/// Deserializer function for UUID.
pub fn de_uuid_opt<'de, D: de::Deserializer<'de>>(d: D) -> Result<Option<Uuid>, D::Error> {
    <Option<&str>>::deserialize(d).map(|s| s.and_then(|s| Uuid::parse_str(s).ok()))
}

/// Serializer function for UUID list.
pub fn ser_uuid_list<S: ser::Serializer>(val: &[Uuid], s: S) -> Result<S::Ok, S::Error> {
    val.iter()
        .map(|u| u.to_string())
        .collect::<Vec<_>>()
        .serialize(s)
}

/// Deserializer function for UUID.
pub fn de_uuid_list<'de, D: de::Deserializer<'de>>(d: D) -> Result<Vec<Uuid>, D::Error> {
    <Vec<&str>>::deserialize(d).map(|v| {
        v.into_iter()
            .filter_map(|s| Uuid::parse_str(s).ok())
            .collect()
    })
}
