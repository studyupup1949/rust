use crate::part_1::v3_1::attributes::semantics::HasSemantics;
use crate::part_1::v3_1::primitives::data_type_def_xs::DataXsd;
use crate::part_1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Qualifiable {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qualifiers: Option<Vec<Qualifier>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct QualifierInner {
    #[serde(flatten)]
    pub semantics: HasSemantics,

    // TODO: Text parsing
    #[serde(rename = "type")]
    pub ty: String,

    #[serde(flatten)]
    pub value: DataXsd,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "valueId")]
    pub value_id: Option<Reference>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum Qualifier {
    ConceptQualifier(QualifierInner),
    TemplateQualifier(QualifierInner),
    ValueQualifier(QualifierInner),

    /// unknown values (kind = null!)
    #[serde(untagged)]
    Unknown(QualifierInner),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_deserialize() {
        let expected = Qualifier::Unknown(QualifierInner {
            semantics: Default::default(),
            ty: "Test".to_string(),
            value: DataXsd::Boolean(Some(true)),
            value_id: None,
        });

        let json = r#"
        {
            "kind":"Test",
            "type": "Test",
            "valueType":"xs:boolean",
            "value": true
        }"#;

        let actual: Qualifier = serde_json::from_str(json).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_concept_qualifier_deserialize() {
        let expected = Qualifier::ConceptQualifier(QualifierInner {
            semantics: Default::default(),
            ty: "Test".to_string(),
            value: DataXsd::Boolean(Some(true)),
            value_id: None,
        });

        let json = r#"
        {
            "kind":"ConceptQualifier",
            "type": "Test",
            "valueType":"xs:boolean",
            "value": true
        }"#;

        let actual: Qualifier = serde_json::from_str(json).unwrap();

        assert_eq!(expected, actual);
    }
}
