use crate::part1::ToJsonMetamodel;
use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part1::v3_1::attributes::referable::Referable;
use crate::part1::v3_1::attributes::semantics::HasSemantics;
use crate::part1::v3_1::submodel_elements::SubmodelElement;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(
        from = "xml::SubmodelElementCollectionXMLProxy",
        into = "xml::SubmodelElementCollectionXMLProxy"
    )
)]
pub struct SubmodelElementCollection {
    #[serde(flatten)]
    pub referable: Referable,

    // HasSemantics
    #[serde(flatten)]
    pub semantics: HasSemantics,

    // Qualifiable
    #[serde(flatten)]
    pub qualifiable: Qualifiable,

    #[serde(flatten)]
    pub embedded_data_specifications: HasDataSpecification,

    value: Option<Vec<SubmodelElement>>,
}

impl ToJsonMetamodel for SubmodelElementCollection {
    type Error = ();

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        Ok(format!(r#"{{"modelType":"SubmodelElementCollection"}}"#))
    }
}

pub(crate) mod xml {
    use crate::part1::v3_1::attributes::data_specification::{
        EmbeddedDataSpecification, HasDataSpecification,
    };
    use crate::part1::v3_1::attributes::extension::{Extension, HasExtensions};
    use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
    use crate::part1::v3_1::attributes::referable::Referable;
    use crate::part1::v3_1::attributes::semantics::HasSemantics;
    use crate::part1::v3_1::primitives::Identifier;
    use crate::part1::v3_1::primitives::xml::LangStringTextType;
    use crate::part1::v3_1::submodel_elements::{SubmodelElement, SubmodelElementCollection};
    use crate::utilities::deserialize_empty_identifier_as_none;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub(crate) struct SubmodelElementCollectionXMLProxy {
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
        pub semantics: Option<HasSemantics>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub qualifiers: Option<Qualifiable>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "embeddedDataSpecifications")]
        embedded_data_specifications: Option<Vec<EmbeddedDataSpecification>>,

        value: Option<ValueXMLWrapper>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct ValueXMLWrapper {
        #[serde(rename = "$value")]
        value: Vec<SubmodelElement>,
    }

    impl From<SubmodelElementCollectionXMLProxy> for SubmodelElementCollection {
        fn from(value: SubmodelElementCollectionXMLProxy) -> Self {
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
                semantics: value.semantics.unwrap_or_default(),
                qualifiable: Qualifiable {
                    qualifiers: value.qualifiers.and_then(|v| v.qualifiers),
                },
                embedded_data_specifications: HasDataSpecification {
                    embedded_data_specifications: value.embedded_data_specifications,
                },
                value: value.value.map(|v| v.value),
            }
        }
    }
    impl From<SubmodelElementCollection> for SubmodelElementCollectionXMLProxy {
        fn from(value: SubmodelElementCollection) -> Self {
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
                semantics: Some(value.semantics),
                qualifiers: Some(value.qualifiable),
                embedded_data_specifications: value
                    .embedded_data_specifications
                    .embedded_data_specifications,
                value: value.value.map(|v| ValueXMLWrapper { value: v }),
            }
        }
    }
}
