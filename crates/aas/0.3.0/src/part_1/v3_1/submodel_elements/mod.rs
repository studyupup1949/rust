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

use std::collections::HashMap;
pub use submodel_element_collection::*;
mod submodel_element_list;
pub use submodel_element_list::*;

use crate::part_1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part_1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part_1::v3_1::attributes::referable::Referable;
use crate::part_1::v3_1::attributes::semantics::HasSemantics;
use crate::part_1::v3_1::submodel_elements::basic_event::BasicEventElement;
pub use crate::part_1::v3_1::submodel_elements::blob::Blob;
pub use crate::part_1::v3_1::submodel_elements::capability::Capability;
use crate::part_1::v3_1::submodel_elements::data_element::DataElement;
use crate::part_1::v3_1::submodel_elements::entity::Entity;
use crate::part_1::v3_1::submodel_elements::file::File;
use crate::part_1::v3_1::submodel_elements::multi_language_property::MultiLanguageProperty;
use crate::part_1::v3_1::submodel_elements::operation::Operation;
use crate::part_1::v3_1::submodel_elements::property::Property;
use crate::part_1::v3_1::submodel_elements::range::Range;
use crate::part_1::v3_1::submodel_elements::reference_element::ReferenceElement;
use crate::part_1::v3_1::submodel_elements::relationship_element::{
    AnnotatedRelationshipElement, RelationshipElement,
};
use crate::part_1::{MetamodelError, ToJsonMetamodel};
use serde::{Deserialize, Serialize};
use strum::Display;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

// TODO
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Display)]
#[serde(tag = "modelType")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum SubmodelElement {
    RelationshipElement(RelationshipElement),
    AnnotatedRelationshipElement(AnnotatedRelationshipElement),
    BasicEventElement(BasicEventElement),
    Blob(Blob),
    Capability(Capability),
    // TODO: is this needed? Deserializes??
    DataElement(DataElement),
    Entity(Entity),
    File(File),
    MultiLanguageProperty(MultiLanguageProperty),
    Operation(Operation),
    Property(Property),
    Range(Range),
    ReferenceElement(ReferenceElement),
    SubmodelElementCollection(SubmodelElementCollection),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::part_1::v3_1::LangString;
    use crate::part_1::v3_1::primitives::Identifier;

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
