use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part1::v3_1::attributes::referable::Referable;
use crate::part1::v3_1::attributes::semantics::HasSemantics;
use crate::part1::v3_1::reference::Reference;
use crate::part1::v3_1::submodel_elements::data_element::DataElement;
use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(
        from = "xml::RelationshipElementXMLProxy",
        into = "xml::RelationshipElementXMLProxy"
    )
)]
pub struct RelationshipElement {
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
    first: Option<Reference>,

    #[serde(skip_serializing_if = "Option::is_none")]
    second: Option<Reference>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(
        from = "xml::AnnotatedRelationshipElementXMLProxy",
        into = "xml::AnnotatedRelationshipElementXMLProxy"
    )
)]
pub struct AnnotatedRelationshipElement {
    // Inherited from RelationshipElement
    #[serde(flatten)]
    pub referable: Referable,

    #[serde(flatten)]
    pub semantics: HasSemantics,

    #[serde(flatten)]
    pub qualifiable: Qualifiable,

    #[serde(flatten)]
    pub embedded_data_specifications: HasDataSpecification,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second: Option<Reference>,
    // --- end inheritance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<DataElement>>,
}

#[cfg(feature = "json")]
pub mod json {
    use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
    use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
    use crate::part1::v3_1::attributes::referable::Referable;
    use crate::part1::v3_1::attributes::semantics::HasSemantics;
    use crate::part1::v3_1::submodel_elements::relationship_element::{
        AnnotatedRelationshipElement, RelationshipElement,
    };
    use crate::part1::{MetamodelError, ToJsonMetamodel};
    use serde::{Deserialize, Serialize};

    #[cfg(feature = "openapi")]
    use utoipa::ToSchema;

    #[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
    #[cfg_attr(feature = "openapi", derive(ToSchema))]
    pub struct RelationshipElementMeta {
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
    }

    #[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
    #[cfg_attr(feature = "openapi", derive(ToSchema))]
    #[allow(unused)]
    pub struct AnnotatedRelationshipElementMeta {
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
    }

    impl From<RelationshipElement> for RelationshipElementMeta {
        fn from(element: RelationshipElement) -> Self {
            Self {
                referable: element.referable,
                semantics: element.semantics,
                qualifiable: element.qualifiable,
                embedded_data_specifications: element.embedded_data_specifications,
            }
        }
    }

    impl From<AnnotatedRelationshipElement> for AnnotatedRelationshipElementMeta {
        fn from(element: AnnotatedRelationshipElement) -> Self {
            Self {
                referable: element.referable,
                semantics: element.semantics,
                qualifiable: element.qualifiable,
                embedded_data_specifications: element.embedded_data_specifications,
            }
        }
    }

    impl From<&RelationshipElement> for RelationshipElementMeta {
        fn from(element: &RelationshipElement) -> Self {
            let element = element.clone();
            Self {
                referable: element.referable,
                semantics: element.semantics,
                qualifiable: element.qualifiable,
                embedded_data_specifications: element.embedded_data_specifications,
            }
        }
    }
    impl From<&AnnotatedRelationshipElement> for AnnotatedRelationshipElementMeta {
        fn from(element: &AnnotatedRelationshipElement) -> Self {
            let element = element.clone();
            Self {
                referable: element.referable,
                semantics: element.semantics,
                qualifiable: element.qualifiable,
                embedded_data_specifications: element.embedded_data_specifications,
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
        use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
        use crate::part1::v3_1::attributes::referable::Referable;
        use crate::part1::v3_1::attributes::semantics::HasSemantics;
        use crate::part1::v3_1::key::Key;
        use crate::part1::v3_1::primitives::Identifier;
        use crate::part1::v3_1::reference::{Reference, ReferenceInner};

        #[test]
        fn test_relationship_element_to_metamodel() {
            // expect to remove "first" & "second" fields.
            let expected = r#"{"idShort":"relationship_test"}"#;
            let actual = RelationshipElement {
                referable: Referable {
                    id_short: Some(Identifier::try_from("relationship_test").unwrap()),
                    display_name: None,
                    description: None,
                    #[allow(deprecated)]
                    category: None,
                    extensions: Default::default(),
                },
                semantics: HasSemantics {
                    semantic_id: None,
                    supplemental_semantic_ids: None,
                },
                qualifiable: Qualifiable { qualifiers: None },
                embedded_data_specifications: Default::default(),

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
}

#[cfg(feature = "xml")]
pub(crate) mod xml {
    use crate::part1::v3_1::attributes::data_specification::{
        EmbeddedDataSpecification, HasDataSpecification,
    };
    use crate::part1::v3_1::attributes::extension::{Extension, HasExtensions};
    use crate::part1::v3_1::attributes::qualifiable::{Qualifiable, Qualifier};
    use crate::part1::v3_1::attributes::referable::Referable;
    use crate::part1::v3_1::attributes::semantics::HasSemantics;
    use crate::part1::v3_1::primitives::Identifier;
    use crate::part1::v3_1::primitives::xml::LangStringTextType;
    use crate::part1::v3_1::reference::Reference;
    use crate::part1::v3_1::submodel_elements::data_element::DataElement;
    use crate::part1::v3_1::submodel_elements::relationship_element::{
        AnnotatedRelationshipElement, RelationshipElement,
    };
    use crate::utilities::deserialize_empty_identifier_as_none;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct RelationshipElementXMLProxy {
        // Inherited from DataElement
        #[serde(skip_serializing_if = "Option::is_none")]
        // use case where "" is needed or can this be ignored?
        #[serde(default)]
        #[serde(deserialize_with = "deserialize_empty_identifier_as_none")]
        #[serde(rename = "idShort")]
        pub id_short: Option<Identifier>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "displayName")]
        pub display_name: Option<LangStringTextType>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<LangStringTextType>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[deprecated]
        pub category: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "extensions")]
        pub extension: Option<Vec<Extension>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "semanticId")]
        pub semantic_id: Option<Reference>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "supplementalSemanticIds")]
        pub supplemental_semantic_ids: Option<Vec<Reference>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub qualifiers: Option<Vec<Qualifier>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "embeddedDataSpecifications")]
        embedded_data_specifications: Option<Vec<EmbeddedDataSpecification>>,
        // end submodelfields
        #[serde(skip_serializing_if = "Option::is_none")]
        first: Option<Reference>,

        #[serde(skip_serializing_if = "Option::is_none")]
        second: Option<Reference>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct AnnotatedRelationshipElementXMLProxy {
        // Inherited from DataElement
        #[serde(skip_serializing_if = "Option::is_none")]
        // use case where "" is needed or can this be ignored?
        #[serde(default)]
        #[serde(deserialize_with = "deserialize_empty_identifier_as_none")]
        #[serde(rename = "idShort")]
        pub id_short: Option<Identifier>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "displayName")]
        pub display_name: Option<LangStringTextType>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub description: Option<LangStringTextType>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[deprecated]
        pub category: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "extensions")]
        pub extension: Option<Vec<Extension>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "semanticId")]
        pub semantic_id: Option<Reference>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "supplementalSemanticIds")]
        pub supplemental_semantic_ids: Option<Vec<Reference>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub qualifiers: Option<Vec<Qualifier>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "embeddedDataSpecifications")]
        embedded_data_specifications: Option<Vec<EmbeddedDataSpecification>>,
        //-- end SubmodelElementFields
        #[serde(skip_serializing_if = "Option::is_none")]
        pub first: Option<Reference>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub second: Option<Reference>,
        // ---- end RelationshipElement
        #[serde(skip_serializing_if = "Option::is_none")]
        pub annotations: Option<Vec<DataElement>>,
    }

    impl From<RelationshipElementXMLProxy> for RelationshipElement {
        fn from(value: RelationshipElementXMLProxy) -> Self {
            Self {
                referable: Referable {
                    id_short: value.id_short,
                    display_name: value.display_name.map(LangStringTextType::into),
                    description: value.description.map(LangStringTextType::into),
                    #[allow(deprecated)]
                    category: value.category,
                    extensions: HasExtensions {
                        extension: value.extension,
                    },
                },
                semantics: HasSemantics {
                    semantic_id: value.semantic_id,
                    supplemental_semantic_ids: value.supplemental_semantic_ids,
                },
                qualifiable: Qualifiable {
                    qualifiers: value.qualifiers,
                },
                embedded_data_specifications: HasDataSpecification {
                    embedded_data_specifications: value.embedded_data_specifications,
                },

                first: value.first,
                second: value.second,
            }
        }
    }

    impl From<RelationshipElement> for RelationshipElementXMLProxy {
        fn from(value: RelationshipElement) -> Self {
            Self {
                id_short: value.referable.id_short,
                display_name: value
                    .referable
                    .display_name
                    .map(|values| LangStringTextType { values }),
                description: value
                    .referable
                    .description
                    .map(|values| LangStringTextType { values }),
                #[allow(deprecated)]
                category: value.referable.category,
                extension: value.referable.extensions.extension,
                semantic_id: value.semantics.semantic_id,
                supplemental_semantic_ids: value.semantics.supplemental_semantic_ids,
                qualifiers: value.qualifiable.qualifiers,
                embedded_data_specifications: value
                    .embedded_data_specifications
                    .embedded_data_specifications,

                first: value.first,
                second: value.second,
            }
        }
    }

    impl From<AnnotatedRelationshipElementXMLProxy> for AnnotatedRelationshipElement {
        fn from(value: AnnotatedRelationshipElementXMLProxy) -> Self {
            Self {
                referable: Referable {
                    id_short: value.id_short,
                    display_name: value.display_name.map(LangStringTextType::into),
                    description: value.description.map(LangStringTextType::into),
                    #[allow(deprecated)]
                    category: value.category,
                    extensions: HasExtensions {
                        extension: value.extension,
                    },
                },
                semantics: HasSemantics {
                    semantic_id: value.semantic_id,
                    supplemental_semantic_ids: value.supplemental_semantic_ids,
                },
                qualifiable: Qualifiable {
                    qualifiers: value.qualifiers,
                },
                embedded_data_specifications: HasDataSpecification {
                    embedded_data_specifications: value.embedded_data_specifications,
                },

                first: value.first,
                second: value.second,
                annotations: value.annotations,
            }
        }
    }

    impl From<AnnotatedRelationshipElement> for AnnotatedRelationshipElementXMLProxy {
        fn from(value: AnnotatedRelationshipElement) -> Self {
            Self {
                id_short: value.referable.id_short,
                display_name: value
                    .referable
                    .display_name
                    .map(|values| LangStringTextType { values }),
                description: value
                    .referable
                    .description
                    .map(|values| LangStringTextType { values }),
                #[allow(deprecated)]
                category: value.referable.category,
                extension: value.referable.extensions.extension,
                semantic_id: value.semantics.semantic_id,
                supplemental_semantic_ids: value.semantics.supplemental_semantic_ids,
                qualifiers: value.qualifiable.qualifiers,
                embedded_data_specifications: value
                    .embedded_data_specifications
                    .embedded_data_specifications,

                first: value.first,
                second: value.second,
                annotations: value.annotations,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        // TODO
    }
}
