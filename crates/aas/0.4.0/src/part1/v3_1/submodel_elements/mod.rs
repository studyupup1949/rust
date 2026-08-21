mod basic_event;
mod blob;
mod capability;
mod data_element;
mod entity;
mod file;
mod multi_language_property;
mod operation;
mod property;
mod range;
mod reference_element;
mod relationship_element;
mod submodel_element_collection;

use crate::utilities::deserialize_empty_identifier_as_none;
use std::collections::HashMap;
pub use submodel_element_collection::*;
mod submodel_element_list;
pub use submodel_element_list::*;

use crate::part1::v3_1::attributes::data_specification::{
    EmbeddedDataSpecification, HasDataSpecification,
};
use crate::part1::v3_1::attributes::extension::Extension;
use crate::part1::v3_1::attributes::qualifiable::{Qualifiable, Qualifier};
use crate::part1::v3_1::attributes::referable::Referable;
use crate::part1::v3_1::attributes::semantics::HasSemantics;
use crate::part1::v3_1::primitives::Identifier;
use crate::part1::v3_1::reference::Reference;
use crate::part1::v3_1::submodel_elements::basic_event::BasicEventElement;
pub use crate::part1::v3_1::submodel_elements::blob::Blob;
pub use crate::part1::v3_1::submodel_elements::capability::Capability;
use crate::part1::v3_1::submodel_elements::data_element::DataElement;
use crate::part1::v3_1::submodel_elements::entity::Entity;
use crate::part1::v3_1::submodel_elements::file::File;
use crate::part1::v3_1::submodel_elements::multi_language_property::MultiLanguageProperty;
use crate::part1::v3_1::submodel_elements::operation::Operation;
use crate::part1::v3_1::submodel_elements::property::Property;
use crate::part1::v3_1::submodel_elements::range::Range;
use crate::part1::v3_1::submodel_elements::reference_element::ReferenceElement;
use crate::part1::v3_1::submodel_elements::relationship_element::{
    AnnotatedRelationshipElement, RelationshipElement,
};
use crate::part1::{MetamodelError, ToJsonMetamodel};
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::part1::v3_1::primitives::xml::LangStringTextType;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

// alias are made to support for camelCase, PascalCase and lowercase.
#[derive(Debug, Clone, PartialEq, Display, Deserialize)]
#[cfg_attr(not(feature = "xml"), derive(Serialize))]
#[cfg_attr(not(feature = "xml"), serde(tag = "modelType"))]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum SubmodelElement {
    #[serde(
        alias = "RelationshipElement",
        alias = "relationshipElement",
        alias = "relationshipelement"
    )]
    RelationshipElement(RelationshipElement),
    #[serde(
        alias = "AnnotatedRelationshipElement",
        alias = "annotatedRelationshipElement",
        alias = "annotatedrelationshipelement"
    )]
    AnnotatedRelationshipElement(AnnotatedRelationshipElement),
    #[serde(
        alias = "BasicEventElement",
        alias = "basicEventElement",
        alias = "basiceventelement"
    )]
    BasicEventElement(BasicEventElement),
    #[serde(alias = "Blob", alias = "blob", alias = "blob")]
    Blob(Blob),
    #[serde(alias = "Capability", alias = "capability", alias = "capability")]
    Capability(Capability),
    // TODO: is this needed? Deserializes??
    #[serde(alias = "DataElement", alias = "dataElement", alias = "dataelement")]
    DataElement(DataElement),
    #[serde(alias = "Entity", alias = "entity", alias = "entity")]
    Entity(Entity),
    #[serde(alias = "File", alias = "file", alias = "file")]
    File(File),
    #[serde(
        alias = "MultiLanguageProperty",
        alias = "multiLanguageProperty",
        alias = "multilanguageproperty"
    )]
    MultiLanguageProperty(MultiLanguageProperty),
    #[serde(alias = "Operation", alias = "operation", alias = "operation")]
    Operation(Operation),
    #[serde(alias = "Property", alias = "property", alias = "property")]
    Property(Property),
    #[serde(alias = "Range", alias = "range", alias = "range")]
    Range(Range),
    #[serde(
        alias = "ReferenceElement",
        alias = "referenceElement",
        alias = "referenceelement"
    )]
    ReferenceElement(ReferenceElement),
    #[serde(
        alias = "SubmodelElementCollection",
        alias = "submodelElementCollection",
        alias = "submodelelementcollection"
    )]
    SubmodelElementCollection(SubmodelElementCollection),
    #[serde(
        alias = "SubmodelElementList",
        alias = "submodelElementList",
        alias = "submodelelementlist"
    )]
    SubmodelElementList(SubmodelElementList),
}

/// Every SubmodelElement has these
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SubmodelElementFields {
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
}

// fields for submodel elements
#[derive(Serialize, Deserialize)]
pub struct SubmodelElementFieldsXML {
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
}

// maybe without variants?
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Display)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum AasSubmodelElements {
    RelationshipElement,
    AnnotatedRelationshipElement,
    BasicEventElement,
    Blob,
    Capability,
    // TODO: is this needed? Deserializes??
    DataElement,
    Entity,
    File,
    MultiLanguageProperty,
    Operation,
    Property,
    Range,
    ReferenceElement,
    #[serde(alias = "SubmodelElementCollection")]
    SubmodelElementCollection,
    SubmodelElementList,
}

impl ToJsonMetamodel for SubmodelElement {
    type Error = MetamodelError;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        let data = match self {
            SubmodelElement::RelationshipElement(_) => Err(MetamodelError::MetamodelNotSupported),
            SubmodelElement::AnnotatedRelationshipElement(_) => {
                Err(MetamodelError::MetamodelNotSupported)
            }
            SubmodelElement::BasicEventElement(elm) => elm.to_json_metamodel(),
            SubmodelElement::Blob(elm) => elm.to_json_metamodel(),
            SubmodelElement::Capability(elm) => elm.to_json_metamodel(),
            SubmodelElement::DataElement(elm) => elm.to_json_metamodel(),
            SubmodelElement::Entity(elm) => Ok(elm.to_json_metamodel().unwrap()),
            SubmodelElement::File(elm) => elm.to_json_metamodel(),
            SubmodelElement::MultiLanguageProperty(elm) => elm.to_json_metamodel(),
            SubmodelElement::Operation(elm) => elm.to_json_metamodel(),
            SubmodelElement::Property(elm) => elm.to_json_metamodel(),
            SubmodelElement::Range(elm) => Ok(elm.to_json_metamodel().unwrap()),
            SubmodelElement::ReferenceElement(elm) => Ok(elm.to_json_metamodel().unwrap()),
            SubmodelElement::SubmodelElementCollection(elm) => Ok(elm.to_json_metamodel().unwrap()),
            SubmodelElement::SubmodelElementList(elm) => elm.to_json_metamodel(),
        };

        data.and_then(|str| {
            let mut map: HashMap<&str, serde_json::Value> = serde_json::from_str(&str).unwrap();
            map.insert("modelType", serde_json::Value::String(self.to_string()));
            Ok(serde_json::to_string(&map).unwrap())
        })
    }
}

#[cfg(feature = "xml")]
mod xml {
    use super::*;
    use serde::Serializer;
    use serde::ser::SerializeStruct;

    impl Serialize for SubmodelElement {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self {
                SubmodelElement::RelationshipElement(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("RelationshipElement", v)?;
                    st.end()
                }
                SubmodelElement::AnnotatedRelationshipElement(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("AnnotatedRelationshipElement", v)?;
                    st.end()
                }
                SubmodelElement::BasicEventElement(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("BasicEventElement", v)?;
                    st.end()
                }
                SubmodelElement::Blob(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("Blob", v)?;
                    st.end()
                }
                SubmodelElement::Capability(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("Capability", v)?;
                    st.end()
                }
                SubmodelElement::DataElement(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("DataElement", v)?;
                    st.end()
                }
                SubmodelElement::Entity(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("Entity", v)?;
                    st.end()
                }
                SubmodelElement::File(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("File", v)?;
                    st.end()
                }
                SubmodelElement::MultiLanguageProperty(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("MultiLanguageProperty", v)?;
                    st.end()
                }
                SubmodelElement::Operation(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("Operation", v)?;
                    st.end()
                }
                SubmodelElement::Property(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("Property", v)?;
                    st.end()
                }
                SubmodelElement::Range(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("Range", v)?;
                    st.end()
                }
                SubmodelElement::ReferenceElement(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("ReferenceElement", v)?;
                    st.end()
                }
                SubmodelElement::SubmodelElementCollection(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("SubmodelElementCollection", v)?;
                    st.end()
                }
                SubmodelElement::SubmodelElementList(v) => {
                    let mut st = serializer.serialize_struct("submodelElement", 1)?;
                    st.serialize_field("SubmodelElementList", v)?;
                    st.end()
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::part1::v3_1::submodel_elements::{Blob, SubmodelElement};
        use serde::{Deserialize, Serialize};

        #[test]
        fn serialize_xml_simple() {
            #[derive(Serialize, Deserialize)]
            struct SubmodelElements(Vec<SubmodelElement>);

            let expected = SubmodelElements(vec![SubmodelElement::Blob(Blob::new(
                Some("test.png".into()),
                "image/png".into(),
            ))]);

            let xml = quick_xml::se::to_string(&expected).unwrap();

            println!("{}", xml);
        }

        #[test]
        fn deserialize_simple() {
            #[derive(Serialize, Deserialize, Debug, PartialEq)]
            struct SubmodelElements {
                #[serde(rename = "$value")]
                submodel_elements: Vec<SubmodelElement>,
            }

            let xml = r#"
            <SubmodelElements>
                <blob>
                    <value>test.png</value>
                    <contentType>image/png</contentType>
                </blob>
            </SubmodelElements>
            "#;

            let expected = SubmodelElements {
                submodel_elements: vec![SubmodelElement::Blob(Blob::new(
                    Some("test.png".into()),
                    "image/png".into(),
                ))],
            };

            let actual = quick_xml::de::from_str(&xml).unwrap();

            assert_eq!(expected, actual);
        }
    }
}

#[cfg(not(feature = "xml"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::part1::v3_1::LangString;
    use crate::part1::v3_1::primitives::Identifier;

    #[test]
    fn deserialize_blob() {
        let expect = SubmodelElement::Blob(Blob::default());

        let json = r#"
        {
            "modelType":"Blob",
            "value": null,
            "contentType": ""
        }
        "#;

        let blob: Blob = serde_json::from_str(json).unwrap();

        assert_eq!(expect, SubmodelElement::Blob(blob));
    }

    #[test]
    fn serialize_blob() {
        let expected = r#"{
  "modelType": "Blob",
  "idShort": "AnShortId",
  "displayName": [
    {
      "language": "en",
      "text": "Sample text"
    }
  ],
  "description": [
    {
      "language": "en",
      "text": "Sample description"
    }
  ],
  "value": "sample base64 string not in base64",
  "contentType": "application/json"
}"#;
        let actual = SubmodelElement::Blob(Blob {
            referable: Referable {
                id_short: Some(Identifier::try_from("AnShortId").unwrap()),
                display_name: Some(vec![
                    LangString::try_new("en", "Sample text".into()).unwrap(),
                ]),
                description: Some(vec![
                    LangString::try_new("en", "Sample description".into()).unwrap(),
                ]),
                category: None,
                extensions: Default::default(),
            },
            semantics: Default::default(),
            qualifiable: Default::default(),
            embedded_data_specifications: Default::default(),
            value: Some("sample base64 string not in base64".into()),
            content_type: "application/json".to_string(),
        });

        let actual = serde_json::to_string_pretty(&actual).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn serialize_metamodel_blob() {
        let actual = SubmodelElement::Blob(Blob::default());

        println!("{}", &actual.to_json_metamodel().unwrap());
    }
}
