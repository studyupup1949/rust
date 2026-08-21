use std::fmt::{self, Display};

use anyhow::{Result, bail};
#[cfg(feature = "seaorm")]
use sea_orm::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use uuid::Uuid;

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, Default, DeriveValueType)]
pub struct HyUuid(pub Uuid);

impl Display for HyUuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.hyphenated().fmt(f)
    }
}

impl HyUuid {
    pub const fn nil() -> Self {
        Self(Uuid::nil())
    }

    pub const fn is_nil(&self) -> bool {
        self.0.is_nil()
    }

    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// # Errors
    /// Will return `Err` when `str` is not uuid v4.
    pub fn parse(str: &str) -> Result<Self> {
        if str.len() != 36 {
            bail!("uuid error: uuid must be 36 bytes");
        }
        Uuid::parse_str(str).map_or_else(|e| Err(e.into()), |v| Ok(Self(v)))
    }
}

pub fn uuids2strings(u: &[HyUuid]) -> Vec<String> {
    u.iter().map(ToString::to_string).collect()
}

impl Serialize for HyUuid {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.0.hyphenated().encode_lower(&mut Uuid::encode_buffer()))
    }
}

impl<'de> Deserialize<'de> for HyUuid {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UuidVisitor;

        impl de::Visitor<'_> for UuidVisitor {
            type Value = HyUuid;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "a UUID v4 string with hyphens")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                HyUuid::parse(value).map_err(|e| de::Error::custom(e))
            }
        }

        deserializer.deserialize_str(UuidVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nil() {
        let n = HyUuid::nil();
        assert!(n.is_nil());
        assert_eq!(n.to_string(), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn new_not_nil() {
        let a = HyUuid::new();
        assert!(!a.is_nil());
    }

    #[test]
    fn parse_ok() {
        let s = "550e8400-e29b-41d4-a716-446655440000";
        let u = HyUuid::parse(s).unwrap();
        assert_eq!(u.to_string(), s);
    }

    #[test]
    fn parse_wrong_len() {
        assert!(HyUuid::parse("550e8400-e29b-41d4-a716-44665544000").is_err());
        assert!(HyUuid::parse("550e8400-e29b-41d4-a716-4466554400000").is_err());
    }

    #[test]
    fn parse_bad_hex() {
        assert!(HyUuid::parse("550e8400-e29b-41d4-a716-44665544000z").is_err());
    }

    #[test]
    fn serde_roundtrip() {
        let u = HyUuid::new();
        let s = serde_json::to_string(&u).unwrap();
        let back: HyUuid = serde_json::from_str(&s).unwrap();
        assert_eq!(u, back);
    }

    #[test]
    fn serde_bad_input() {
        assert!(serde_json::from_str::<HyUuid>("\"not-a-uuid\"").is_err());
    }

    #[test]
    fn uuids2strings_empty() {
        assert!(uuids2strings(&[]).is_empty());
    }

    #[test]
    fn uuids2strings_values() {
        let a = HyUuid::nil();
        let b = HyUuid::new();
        let s = uuids2strings(&[a, b]);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0], a.to_string());
        assert_eq!(s[1], b.to_string());
    }
}
