use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part1::v3_1::attributes::referable::Referable;
use crate::part1::v3_1::attributes::semantics::HasSemantics;
use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(from = "xml::CapabilityXMLProxy", into = "xml::CapabilityXMLProxy")
)]
pub struct Capability {
    // Inherited from DataElement
    pub referable: Referable,
    pub semantics: HasSemantics,
    pub qualifiable: Qualifiable,
    pub embedded_data_specifications: HasDataSpecification,
    // ----- end inheritance
}

impl Capability {
    pub fn new() -> Self {
        Self {
            referable: Referable::default(),
            semantics: HasSemantics::default(),
            qualifiable: Qualifiable::default(),
            embedded_data_specifications: HasDataSpecification::default(),
        }
    }
}

#[cfg(feature = "xml")]
mod xml {
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
    use crate::utilities::deserialize_empty_identifier_as_none;

    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct CapabilityXMLProxy {
        // Inherited from DataElement
        // use case where "" is needed or can this be ignored?
        #[serde(default)]
        #[serde(deserialize_with = "deserialize_empty_identifier_as_none")]
        #[serde(rename = "idShort")]
        pub id_short: Option<Identifier>,

        #[serde(rename = "displayName")]
        pub display_name: Option<LangStringTextType>,

        pub description: Option<LangStringTextType>,

        #[deprecated]
        pub category: Option<String>,

        #[serde(rename = "extensions")]
        pub extension: Option<Vec<Extension>>,

        #[serde(rename = "semanticId")]
        pub semantic_id: Option<Reference>,

        #[serde(rename = "supplementalSemanticIds")]
        pub supplemental_semantic_ids: Option<Vec<Reference>>,

        pub qualifiers: Option<Vec<Qualifier>>,

        #[serde(rename = "embeddedDataSpecifications")]
        embedded_data_specifications: Option<Vec<EmbeddedDataSpecification>>,
    }

    impl From<super::Capability> for CapabilityXMLProxy {
        fn from(value: super::Capability) -> Self {
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
            }
        }
    }
    impl From<CapabilityXMLProxy> for super::Capability {
        fn from(value: CapabilityXMLProxy) -> Self {
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
            }
        }
    }
}

mod json {
    use crate::part1::{MetamodelError, ToJsonMetamodel};

    impl ToJsonMetamodel for super::Capability {
        type Error = MetamodelError;

        fn to_json_metamodel(&self) -> Result<String, Self::Error> {
            serde_json::to_string(&self).map_err(|e| MetamodelError::FailedSerialisation(e))
        }
    }
}

// TODO: Test serialization and deserialization
#[cfg(test)]
mod tests {}
