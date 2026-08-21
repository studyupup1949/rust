use serde::{Deserialize, Serialize};
use serde_with::base64::{Base64, UrlSafe};
use serde_with::formats::Unpadded;
use serde_with::serde_as;
use std::fmt::{Display, Formatter};

/// IDShort can either be a "Raw" String or when used by APIs
/// a base64 encoded String.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum IDShort {
    Base64(Base64String),
    Raw(String),
}

impl Display for IDShort {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match &self {
            IDShort::Base64(base64) => &base64.0,
            IDShort::Raw(s) => s,
        };

        write!(f, "{}", str)
    }
}

/// Struct for signaling that this field should be de-/serialized from/into
/// Base64 UrlSafe Unpadded.
#[serde_as]
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct Base64String(#[serde_as(as = "Base64<UrlSafe, Unpadded>")] pub String);

#[cfg(test)]
mod tests {
    use crate::part_1::v3_1::id_short::{Base64String, IDShort};

    #[test]
    fn serialize_base64() {
        let string = IDShort::Base64(Base64String("Plant1".into()));

        let json = serde_json::to_string_pretty(&string).unwrap();

        assert_eq!(r#""UGxhbnQx""#, json)
    }

    #[test]
    fn serialize_raw() {
        let string = IDShort::Raw("Plant1".into());

        let json = serde_json::to_string_pretty(&string).unwrap();

        assert_eq!(r#""Plant1""#, json)
    }

    #[test]
    fn deserialize_raw() {
        let json = r#""Plant1""#;
        let id: IDShort = serde_json::from_str(&json).unwrap();

        assert_eq!(IDShort::Raw("Plant1".into()), id)
    }

    #[test]
    fn deserialize_base64() {
        let json = r#""UGxhbnQx""#;
        let id: IDShort = serde_json::from_str(&json).unwrap();

        assert_eq!(IDShort::Base64(Base64String("Plant1".into())), id)
    }

    #[test]
    fn id_to_string_raw() {
        let expected = "Plant1";
        let actual = IDShort::Raw("Plant1".into());

        assert_eq!(expected, actual.to_string())
    }

    #[test]
    fn id_to_string_base64() {
        let expected = "Plant1";
        let actual = IDShort::Base64(Base64String("Plant1".into()));

        assert_eq!(expected, actual.to_string())
    }
}
