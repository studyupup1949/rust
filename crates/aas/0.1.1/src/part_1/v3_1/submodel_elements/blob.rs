use crate::part_1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part_1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part_1::v3_1::attributes::referable::Referable;
use crate::part_1::v3_1::attributes::semantics::HasSemantics;
use crate::part_1::v3_1::primitives::ContentType;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[serde(tag = "modelType")]
pub struct Blob {
    // Inherited from DataElement
    #[serde(flatten)]
    pub referable: Referable,

    #[serde(flatten)]
    pub semantics: HasSemantics,

    #[serde(flatten)]
    pub qualifiable: Qualifiable,

    #[serde(flatten)]
    pub embedded_data_specifications: HasDataSpecification,
    // ----- end inheritance
    #[serde(skip_serializing_if = "Option::is_none")]
    // TODO "contentEncoding": "base64"
    pub value: Option<String>,

    // TODO typing. Add constraints. New type..
    #[serde(rename = "contentType")]
    pub content_type: ContentType,
}

impl Blob {
    pub fn new(content_type: String) -> Self {
        Self {
            referable: Referable::default(),
            semantics: HasSemantics::default(),
            qualifiable: Qualifiable::default(),
            embedded_data_specifications: HasDataSpecification::default(),
            value: None,
            content_type,
        }
    }
}

// TODO: Test serialization and deserialization
#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn it_deserializes_default() {
        let expect = Blob::default();
        let json = r#"
        {
            "modelType":"Blob",
            "value": null,
            "contentType": ""
        }
        "#;

        let blob: Blob = serde_json::from_str(json).unwrap();

        assert_eq!(expect, blob);
    }

    #[test]
    fn it_serializes() {
        let blob = Blob::new(String::from(""));

        let json = serde_json::to_string(&blob).unwrap();

        println!("{}", json);
    }
}
