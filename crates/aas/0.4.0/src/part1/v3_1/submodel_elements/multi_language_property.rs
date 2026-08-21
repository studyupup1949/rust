use crate::part1::MetamodelError;
use crate::part1::ToJsonMetamodel;
use crate::part1::v3_1::LangString;
use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part1::v3_1::attributes::referable::Referable;
use crate::part1::v3_1::attributes::semantics::HasSemantics;
use crate::part1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(
        from = "xml::MultiLanguagePropertyXML",
        into = "xml::MultiLanguagePropertyXML"
    )
)]
pub struct MultiLanguageProperty {
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
    pub value: Option<Vec<LangString>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "valueId")]
    pub value_id: Option<Reference>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct MultiLanguagePropertyMeta {
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

impl From<MultiLanguageProperty> for MultiLanguagePropertyMeta {
    fn from(m: MultiLanguageProperty) -> Self {
        Self {
            referable: m.referable,
            semantics: m.semantics,
            qualifiable: m.qualifiable,
            embedded_data_specifications: m.embedded_data_specifications,
        }
    }
}

impl From<&MultiLanguageProperty> for MultiLanguagePropertyMeta {
    fn from(m: &MultiLanguageProperty) -> Self {
        let m = m.clone();
        Self {
            referable: m.referable,
            semantics: m.semantics,
            qualifiable: m.qualifiable,
            embedded_data_specifications: m.embedded_data_specifications,
        }
    }
}

impl ToJsonMetamodel for MultiLanguageProperty {
    type Error = MetamodelError;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        let meta = MultiLanguagePropertyMeta::from(self);
        serde_json::to_string(&meta).map_err(|e| MetamodelError::FailedSerialisation(e))
    }
}

#[cfg(feature = "xml")]
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
    use crate::part1::v3_1::reference::Reference;
    use crate::part1::v3_1::submodel_elements::multi_language_property::MultiLanguageProperty;
    use crate::utilities::deserialize_empty_identifier_as_none;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    pub(crate) struct MultiLanguagePropertyXML {
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

        pub qualifiers: Option<Qualifiable>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "embeddedDataSpecifications")]
        embedded_data_specifications: Option<Vec<EmbeddedDataSpecification>>,
        // ----- end inheritance
        #[serde(skip_serializing_if = "Option::is_none")]
        pub value: Option<LangStringTextType>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "valueId")]
        pub value_id: Option<Reference>,
    }

    impl From<MultiLanguagePropertyXML> for MultiLanguageProperty {
        fn from(value: MultiLanguagePropertyXML) -> Self {
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
                value: value.value.map(LangStringTextType::into),
                value_id: value.value_id,
            }
        }
    }

    impl From<MultiLanguageProperty> for MultiLanguagePropertyXML {
        fn from(value: MultiLanguageProperty) -> Self {
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
                value: value.value.map(|values| LangStringTextType { values }),
                value_id: value.value_id,
            }
        }
    }
}

#[cfg(not(feature = "xml"))]
#[cfg(test)]
// only JSON
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn it_serializes() {
        let expected = r#"{"value":[{"language":"de","text":"Das ist ein deutscher Bezeichner"},{"language":"en","text":"That's an English label"}]}"#;

        let mut ml_property = MultiLanguageProperty::default();
        ml_property.value = Some(vec![
            LangString::from_str(r#""Das ist ein deutscher Bezeichner"@de"#).unwrap(),
            LangString::from_str(r#""That's an English label"@en"#).unwrap(),
        ]);

        let actual =
            serde_json::to_string(&ml_property).expect("Can't serialize MultiLanguageProperty.");

        assert_eq!(actual, expected);
    }

    #[test]
    fn deserialize_complex() {
        let json = r#"
        {
                  "idShort": "CityTown",
                  "displayName": [
                    {
                      "language": "en",
                      "text": "city"
                    }
                  ],
                  "semanticId": {
                    "type": "ExternalReference",
                    "keys": [
                      {
                        "type": "GlobalReference",
                        "value": "0173-1#02-AAO132#002"
                      }
                    ]
                  },
                  "qualifiers": [
                    {
                      "kind": "ConceptQualifier",
                      "type": "Multiplicity",
                      "valueType": "xs:string",
                      "value": "ZeroToOne"
                    }
                  ],
                  "value": [
                    {
                      "language": "de",
                      "text": "Lemgo"
                    }
                  ],
                  "modelType": "MultiLanguageProperty"
                }
        "#;

        serde_json::from_str::<MultiLanguageProperty>(json).unwrap();
    }
}
