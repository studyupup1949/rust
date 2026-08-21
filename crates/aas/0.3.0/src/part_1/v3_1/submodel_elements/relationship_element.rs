use crate::part_1::v3_1::reference::Reference;
use crate::part_1::v3_1::submodel_elements::SubmodelElementFields;
use crate::part_1::v3_1::submodel_elements::data_element::DataElement;
use crate::part_1::{MetamodelError, ToJsonMetamodel};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct RelationshipElement {
    #[serde(flatten)]
    pub submodel_element_fields: SubmodelElementFields,

    #[serde(skip_serializing_if = "Option::is_none")]
    first: Option<Reference>,

    #[serde(skip_serializing_if = "Option::is_none")]
    second: Option<Reference>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AnnotatedRelationshipElement {
    // Inherited from RelationshipElement
    #[serde(flatten)]
    pub submodel_element_fields: SubmodelElementFields,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second: Option<Reference>,
    // ----
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<DataElement>>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct RelationshipElementMeta {
    #[serde(flatten)]
    pub submodel_element_fields: SubmodelElementFields,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AnnotatedRelationshipElementMeta {
    #[serde(flatten)]
    pub submodel_element_fields: SubmodelElementFields,
}

impl From<RelationshipElement> for RelationshipElementMeta {
    fn from(element: RelationshipElement) -> Self {
        Self {
            submodel_element_fields: element.submodel_element_fields,
        }
    }
}

impl From<&RelationshipElement> for RelationshipElementMeta {
    fn from(element: &RelationshipElement) -> Self {
        Self {
            submodel_element_fields: element.submodel_element_fields.clone(),
        }
    }
}

impl From<AnnotatedRelationshipElement> for AnnotatedRelationshipElementMeta {
    fn from(element: AnnotatedRelationshipElement) -> Self {
        Self {
            submodel_element_fields: element.submodel_element_fields,
        }
    }
}

impl From<&AnnotatedRelationshipElement> for AnnotatedRelationshipElementMeta {
    fn from(element: &AnnotatedRelationshipElement) -> Self {
        Self {
            submodel_element_fields: element.submodel_element_fields.clone(),
        }
    }
}

impl ToJsonMetamodel for RelationshipElement {
    type Error = MetamodelError;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        serde_json::to_string::<RelationshipElementMeta>(&self.into())
            .map_err(|e| MetamodelError::FailedSerialisation(e))
    }
}

impl ToJsonMetamodel for AnnotatedRelationshipElement {
    type Error = MetamodelError;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        serde_json::to_string::<AnnotatedRelationshipElementMeta>(&self.into())
            .map_err(|e| MetamodelError::FailedSerialisation(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::part_1::v3_1::attributes::qualifiable::Qualifiable;
    use crate::part_1::v3_1::attributes::referable::Referable;
    use crate::part_1::v3_1::attributes::semantics::HasSemantics;
    use crate::part_1::v3_1::key::Key;
    use crate::part_1::v3_1::primitives::Identifier;
    use crate::part_1::v3_1::reference::ReferenceInner;
    #[test]
    fn test_relationship_element_to_metamodel() {
        // expect to remove "first" & "second" fields.
        let expected = r#"{"idShort":"relationship_test"}"#;
        let actual = RelationshipElement {
            submodel_element_fields: SubmodelElementFields {
                referable: Referable {
                    id_short: Some(Identifier::try_from("relationship_test").unwrap()),
                    display_name: None,
                    description: None,
                    category: None,
                    extensions: Default::default(),
                },
                semantics: HasSemantics {
                    semantic_id: None,
                    supplemental_semantic_ids: None,
                },
                qualifiable: Qualifiable { qualifiers: None },
                embedded_data_specifications: Default::default(),
            },
            first: Some(Reference::ExternalReference(ReferenceInner {
                referred_semantic_id: None,
                keys: vec![Key::RelationshipElement("https://example.com/1".into())],
            })),
            second: Some(Reference::ExternalReference(ReferenceInner {
                referred_semantic_id: None,
                keys: vec![Key::RelationshipElement("https://example.com/2".into())],
            })),
        };

        let actual = actual.to_json_metamodel().expect("Serialize to metamodel");

        assert_eq!(expected, actual);
    }
}
