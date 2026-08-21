use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part1::v3_1::attributes::referable::Referable;
use crate::part1::v3_1::attributes::semantics::HasSemantics;
use crate::part1::v3_1::primitives::data_type_def_xs::DataTypeXSDef;
use crate::part1::v3_1::reference::Reference;
use crate::part1::v3_1::submodel_elements::{AasSubmodelElements, SubmodelElement};
use crate::part1::{MetamodelError, ToJsonMetamodel};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;
// TODO: TYPING
// We could make the pair value / type_value_list_element one enum
// Deserialize check for constraints.

/// A submodel element list is an ordered list of submodel elements.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(
        from = "xml::SubmodelElementListXMLProxy",
        into = "xml::SubmodelElementListXMLProxy"
    )
)]
pub struct SubmodelElementList {
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

    /// Defines whether order in list is relevant. If orderRelevant = false, the list represents a set or a bag.
    #[serde(rename = "orderRelevant")]
    #[serde(default = "ordering_default")]
    is_order_relevant: bool,

    /// Semantic ID which the submodel elements contained in the list match
    #[serde(rename = "semanticIdListElement")]
    semantic_id_list_element: Option<Reference>,

    // Question: can value, type_value_list_element be merged into an enum?
    // maybe together with value_type_list_element?
    // newtype or something for type safety.
    /// Submodel elements contained in the list
    value: Option<Vec<SubmodelElement>>,

    /// The submodel element type of the submodel elements contained in the list
    #[serde(rename = "typeValueListElement")]
    type_value_list_element: AasSubmodelElements,

    /// The value type of the submodel element contained in the list
    #[serde(rename = "valueTypeListElement")]
    value_type_list_element: Option<DataTypeXSDef>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SubmodelElementListMeta {
    /// Defines whether order in list is relevant. If orderRelevant = false, the list represents a set or a bag.
    #[serde(rename = "orderRelevant")]
    #[serde(default = "ordering_default")]
    is_order_relevant: bool,

    /// Semantic ID which the submodel elements contained in the list match
    #[serde(rename = "semanticIdListElement")]
    semantic_id_list_element: Option<Reference>,

    /// The submodel element type of the submodel elements contained in the list
    #[serde(rename = "typeValueListElement")]
    type_value_list_element: AasSubmodelElements,

    /// The value type of the submodel element contained in the list
    #[serde(rename = "valueTypeListElement")]
    value_type_list_element: Option<DataTypeXSDef>,
}

fn ordering_default() -> bool {
    true
}

impl From<SubmodelElementList> for SubmodelElementListMeta {
    fn from(element: SubmodelElementList) -> Self {
        Self {
            is_order_relevant: element.is_order_relevant,
            semantic_id_list_element: element.semantic_id_list_element,
            type_value_list_element: element.type_value_list_element,
            value_type_list_element: element.value_type_list_element,
        }
    }
}

impl From<&SubmodelElementList> for SubmodelElementListMeta {
    fn from(element: &SubmodelElementList) -> Self {
        element.clone().into()
    }
}

impl ToJsonMetamodel for SubmodelElementList {
    type Error = MetamodelError;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        serde_json::to_string::<SubmodelElementListMeta>(&self.into())
            .map_err(|e| MetamodelError::FailedSerialisation(e))
    }
}

mod xml {
    use super::SubmodelElementList;
    use crate::part1::v3_1::attributes::data_specification::{
        EmbeddedDataSpecification, HasDataSpecification,
    };
    use crate::part1::v3_1::attributes::extension::{Extension, HasExtensions};
    use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
    use crate::part1::v3_1::attributes::referable::Referable;
    use crate::part1::v3_1::attributes::semantics::HasSemantics;
    use crate::part1::v3_1::primitives::Identifier;
    use crate::part1::v3_1::primitives::data_type_def_xs::DataTypeXSDef;
    use crate::part1::v3_1::primitives::xml::LangStringTextType;
    use crate::part1::v3_1::reference::Reference;
    use crate::part1::v3_1::submodel_elements::submodel_element_list::ordering_default;
    use crate::part1::v3_1::submodel_elements::{AasSubmodelElements, SubmodelElement};
    use crate::utilities::deserialize_empty_identifier_as_none;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct SubmodelElementListXMLProxy {
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

        /// Defines whether order in list is relevant. If orderRelevant = false, the list represents a set or a bag.
        #[serde(rename = "orderRelevant")]
        #[serde(default = "ordering_default")]
        is_order_relevant: bool,

        /// Semantic ID which the submodel elements contained in the list match
        #[serde(rename = "semanticIdListElement")]
        semantic_id_list_element: Option<Reference>,

        // Question: can value, type_value_list_element be merged into an enum?
        // maybe together with value_type_list_element?
        // newtype or something for type safety.
        /// Submodel elements contained in the list
        value: Option<ValueXMLWrapper>,

        /// The submodel element type of the submodel elements contained in the list
        #[serde(rename = "typeValueListElement")]
        type_value_list_element: AasSubmodelElements,

        /// The value type of the submodel element contained in the list
        #[serde(rename = "valueTypeListElement")]
        value_type_list_element: Option<DataTypeXSDef>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct ValueXMLWrapper {
        #[serde(rename = "$value")]
        value: Vec<SubmodelElement>,
    }

    impl From<SubmodelElementList> for SubmodelElementListXMLProxy {
        fn from(value: SubmodelElementList) -> Self {
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

                is_order_relevant: value.is_order_relevant,
                semantic_id_list_element: value.semantic_id_list_element,
                value: value.value.map(|v| ValueXMLWrapper { value: v }),

                // TODO
                type_value_list_element: AasSubmodelElements::RelationshipElement,
                value_type_list_element: None,
            }
        }
    }
    impl From<SubmodelElementListXMLProxy> for SubmodelElementList {
        fn from(value: SubmodelElementListXMLProxy) -> Self {
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
                qualifiable: value.qualifiers.unwrap_or_default(),
                embedded_data_specifications: HasDataSpecification {
                    embedded_data_specifications: value.embedded_data_specifications,
                },

                is_order_relevant: false,
                semantic_id_list_element: None,
                value: value.value.map(|v| v.value),
                type_value_list_element: AasSubmodelElements::RelationshipElement,
                value_type_list_element: None,
            }
        }
    }
}

#[cfg(not(feature = "xml"))]
#[cfg(test)]
mod tests {
    use crate::part1::v3_1::submodel_elements::SubmodelElementList;

    #[test]
    fn deserialize_complex() {
        let json = r#"
            {
              "idShort": "PcfCalculationMethods",
              "displayName": [
                {
                  "language": "en",
                  "text": "impact assessment methods"
                }
              ],
              "description": [
                {
                  "language": "en",
                  "text": "Standards, methods for determining the greenhouse gas emissions of a product."
                }
              ],
              "semanticId": {
                "type": "ExternalReference",
                "keys": [
                  {
                    "type": "GlobalReference",
                    "value": "https://admin-shell.io/idta/CarbonFootprint/PcfCalculationMethods/1/0"
                  }
                ]
              },
              "qualifiers": [
                {
                  "type": "SMT/Cardinality",
                  "valueType": "xs:string",
                  "value": "One"
                }
              ],
              "orderRelevant": true,
              "semanticIdListElement": {
                "type": "ExternalReference",
                "keys": [
                  {
                    "type": "GlobalReference",
                    "value": "0173-1#02-ABG854#003"
                  }
                ]
              },
              "typeValueListElement": "Property",
              "valueTypeListElement": "xs:string",
              "value": [
                {
                  "category": "PARAMETER",
                  "displayName": [
                    {
                      "language": "en",
                      "text": "impact assessment method / calculation method"
                    }
                  ],
                  "description": [
                    {
                      "language": "en",
                      "text": "Standard, method for determining the greenhouse gas emissions of a product."
                    }
                  ],
                  "semanticId": {
                    "type": "ExternalReference",
                    "keys": [
                      {
                        "type": "GlobalReference",
                        "value": "0173-1#02-ABG854#003"
                      }
                    ]
                  },
                  "qualifiers": [
                    {
                      "type": "SMT/Cardinality",
                      "valueType": "xs:string",
                      "value": "One"
                    }
                  ],
                  "valueType": "xs:string",
                  "value": "in-house method - see documentation",
                  "modelType": "Property"
                }
              ],
              "modelType": "SubmodelElementList"
            }
        "#;

        serde_json::from_str::<SubmodelElementList>(json).unwrap();
    }
}
