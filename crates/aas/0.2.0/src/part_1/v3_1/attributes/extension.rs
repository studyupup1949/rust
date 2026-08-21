use crate::part_1::v3_1::primitives::data_type_def_xs::DataXsd;
use crate::part_1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// HasExtensions
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct HasExtensions {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "extensions")]
    pub extension: Option<Vec<Extension>>,
}

/// `Extension` represents AAS elements that can hold additional, user-defined data through extensions.
///
/// This structure contains an optional list of `Extension` objects, allowing elements to be augmented
/// with supplementary information that extends the standard AAS metamodel.
///
/// Each contained `Extension` must have a unique name within its context to avoid conflicts.
///
/// The `HasExtension` structure enables flexible data modeling and customization,
/// supporting evolving use cases without compromising core interoperability.
///
/// Constraints:
/// - If present, the extensions list must contain at least one element.
/// - Extension names must be unique within this container.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Extension {
    pub name: String,

    /// semantic definition
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "semanticId")]
    pub semantic_id: Option<Reference>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "supplementalSemanticIds")]
    pub supplemental_semantic_ids: Option<Vec<Reference>>,

    #[serde(flatten)]
    pub value: DataXsd,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "refersTo")]
    pub refers_to: Option<Vec<Reference>>,
}

impl Extension {
    pub fn new(name: String) -> Self {
        Self {
            semantic_id: None,
            supplemental_semantic_ids: None,
            name,
            //value_type: None,
            value: DataXsd::default(),
            refers_to: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let extension = Extension {
            name: "".to_string(),
            semantic_id: None,
            supplemental_semantic_ids: None,
            value: DataXsd::Int(Some(123)),
            refers_to: None,
        };

        let expected = r#"{"name":"","valueType":"xs:int","value":123}"#;

        let actual = serde_json::to_string(&extension).expect("Should serialize");
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_deserialize_no_data() {
        let expected = Extension {
            name: "".to_string(),
            semantic_id: None,
            supplemental_semantic_ids: None,
            value: DataXsd::String(None),
            refers_to: None,
        };

        let json = r#"{"name":"","valueType":"xs:string"}"#;

        let actual = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(expected, actual);
    }
}
