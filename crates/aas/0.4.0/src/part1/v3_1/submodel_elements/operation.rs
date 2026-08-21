use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part1::v3_1::attributes::referable::Referable;
use crate::part1::v3_1::attributes::semantics::HasSemantics;
use crate::part1::v3_1::submodel_elements::SubmodelElement;
use crate::part1::{MetamodelError, ToJsonMetamodel};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(from = "xml::OperationXMLProxy", into = "xml::OperationXMLProxy")
)]
pub struct Operation {
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
    #[serde(rename = "inputVariable")]
    pub input_variable: Option<Box<SubmodelElement>>,

    #[serde(rename = "outputVariable")]
    pub output_variable: Option<Box<SubmodelElement>>,

    #[serde(rename = "inoutputVariable")]
    pub inoutput_variable: Option<Box<SubmodelElement>>,
}

impl ToJsonMetamodel for Operation {
    type Error = MetamodelError;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        // TODO: add modelType tag
        serde_json::to_string(&self).map_err(|e| MetamodelError::FailedSerialisation(e))
    }
}

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
    use crate::part1::v3_1::submodel_elements::SubmodelElement;
    use crate::utilities::deserialize_empty_identifier_as_none;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct OperationXMLProxy {
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
        // ----- end inheritance
        #[serde(rename = "inputVariable")]
        pub input_variable: Option<Box<SubmodelElement>>,

        #[serde(rename = "outputVariable")]
        pub output_variable: Option<Box<SubmodelElement>>,

        #[serde(rename = "inoutputVariable")]
        pub inoutput_variable: Option<Box<SubmodelElement>>,
    }

    impl From<OperationXMLProxy> for super::Operation {
        fn from(value: OperationXMLProxy) -> Self {
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

                input_variable: value.input_variable,
                output_variable: value.output_variable,
                inoutput_variable: value.inoutput_variable,
            }
        }
    }

    impl From<super::Operation> for OperationXMLProxy {
        fn from(value: super::Operation) -> Self {
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

                input_variable: value.input_variable,
                output_variable: value.output_variable,
                inoutput_variable: value.inoutput_variable,
            }
        }
    }
}
