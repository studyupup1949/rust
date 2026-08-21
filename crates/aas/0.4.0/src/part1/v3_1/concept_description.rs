use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part1::v3_1::attributes::identifiable::Identifiable;
use crate::part1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// The semantics of a property or other elements that may have a semantic description is defined
/// by a concept description.
/// The description of the concept should follow a standardized schema
/// (realized as data specification template).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(
        from = "xml::ConceptDescriptionXMLProxy",
        into = "xml::ConceptDescriptionXMLProxy"
    )
)]
pub struct ConceptDescription {
    #[serde(flatten)]
    pub identifiable: Identifiable,

    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_specification: Option<HasDataSpecification>,

    #[serde(rename = "isCaseOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_case_of: Option<Vec<Reference>>,
}

pub(crate) mod xml {
    use crate::part1::v3_1::attributes::administrative_information::AdministrativeInformation;
    use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
    use crate::part1::v3_1::attributes::extension::HasExtensions;
    use crate::part1::v3_1::attributes::identifiable::Identifiable;
    use crate::part1::v3_1::attributes::referable::Referable;
    use crate::part1::v3_1::concept_description::ConceptDescription;
    use crate::part1::v3_1::primitives::Identifier;
    use crate::part1::v3_1::primitives::xml::LangStringTextType;
    use crate::part1::v3_1::reference::Reference;
    use crate::utilities::deserialize_empty_identifier_as_none;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ConceptDescriptionXMLProxy {
        // Identifiable
        pub id: Identifier,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub administration: Option<AdministrativeInformation>,

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
        pub extension: Option<HasExtensions>,
        // -----
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "embeddedDataSpecifications")]
        pub embedded_data_specifications: Option<HasDataSpecification>,

        #[serde(rename = "isCaseOf")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub is_case_of: Option<IsCaseOfWrapper>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct IsCaseOfWrapper {
        #[serde(rename = "$value")]
        #[serde(skip_serializing_if = "Option::is_none")]
        values: Option<Vec<Reference>>,
    }

    impl From<ConceptDescriptionXMLProxy> for ConceptDescription {
        fn from(value: ConceptDescriptionXMLProxy) -> Self {
            Self {
                identifiable: Identifiable {
                    id: value.id,
                    administration: value.administration,
                    referable: Referable {
                        id_short: value.id_short,
                        display_name: value.display_name.map(LangStringTextType::into),
                        description: value.description.map(LangStringTextType::into),
                        #[allow(deprecated)]
                        category: value.category,
                        extensions: value.extension.unwrap_or_default(),
                    },
                },
                data_specification: value.embedded_data_specifications,
                is_case_of: value.is_case_of.and_then(|v| v.values),
            }
        }
    }
    impl From<ConceptDescription> for ConceptDescriptionXMLProxy {
        fn from(value: ConceptDescription) -> Self {
            Self {
                id: value.identifiable.id,
                administration: value.identifiable.administration,
                id_short: value.identifiable.referable.id_short,
                display_name: value
                    .identifiable
                    .referable
                    .display_name
                    .map(|values| LangStringTextType { values }),
                description: value
                    .identifiable
                    .referable
                    .description
                    .map(|values| LangStringTextType { values }),
                #[allow(deprecated)]
                category: value.identifiable.referable.category,
                extension: Some(value.identifiable.referable.extensions),
                embedded_data_specifications: value.data_specification,
                is_case_of: value
                    .is_case_of
                    .map(|v| IsCaseOfWrapper { values: Some(v) }),
            }
        }
    }
}
