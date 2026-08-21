use crate::part1::MetamodelError;
use crate::part1::ToJsonMetamodel;
use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part1::v3_1::attributes::referable::Referable;
use crate::part1::v3_1::attributes::semantics::HasSemantics;
use crate::part1::v3_1::primitives::ContentType;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "xml", serde(from = "xml::BlobXML", into = "xml::BlobXML"))]
pub struct Blob {
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
    // TODO "contentEncoding": "base64"
    pub value: Option<String>,

    // TODO typing. Add constraints. New type..
    #[serde(rename = "contentType")]
    pub content_type: ContentType,
}

impl Blob {
    pub fn new(value: Option<String>, content_type: String) -> Self {
        Self {
            referable: Referable::default(),
            semantics: HasSemantics::default(),
            qualifiable: Qualifiable::default(),
            embedded_data_specifications: HasDataSpecification::default(),
            value,
            content_type,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
pub struct BlobMeta {
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

impl From<Blob> for BlobMeta {
    fn from(blob: Blob) -> Self {
        Self {
            referable: blob.referable,
            semantics: blob.semantics,
            qualifiable: blob.qualifiable,
            embedded_data_specifications: blob.embedded_data_specifications,
        }
    }
}

impl From<&Blob> for BlobMeta {
    fn from(blob: &Blob) -> Self {
        blob.clone().into()
    }
}

impl ToJsonMetamodel for Blob {
    type Error = MetamodelError;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        serde_json::to_string::<BlobMeta>(&self.into())
            .map_err(|e| MetamodelError::FailedSerialisation(e))
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
    use crate::part1::v3_1::primitives::xml::LangStringTextType;
    use crate::part1::v3_1::primitives::{ContentType, Identifier};
    use crate::part1::v3_1::reference::Reference;
    use crate::part1::v3_1::submodel_elements::Blob;
    use crate::utilities::deserialize_empty_identifier_as_none;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
    pub(super) struct BlobXML {
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

        #[serde(skip_serializing_if = "Option::is_none")]
        // TODO "contentEncoding": "base64"
        pub value: Option<String>,

        // TODO typing. Add constraints. New type..
        #[serde(rename = "contentType")]
        pub content_type: ContentType,
    }

    impl From<Blob> for BlobXML {
        fn from(value: Blob) -> Self {
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
                value: value.value,
                content_type: value.content_type,
            }
        }
    }

    impl From<BlobXML> for Blob {
        fn from(value: BlobXML) -> Self {
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
                value: value.value,
                content_type: value.content_type,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        #[ignore]
        fn deserialize_simple() {
            todo!()
        }
    }
}

// TODO: Test serialization and deserialization
#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn it_deserializes_default() {
        let expect = Blob::default();
        let json = r#"
        {
            "modelType":"Blob",
            "value": null,
            "contentType": ""
        }
        "#;

        let blob: Blob = serde_json::from_str(json).unwrap();

        assert_eq!(expect, blob);
    }

    #[test]
    fn it_serializes() {
        let blob = Blob::new(None, String::from(""));

        let json = serde_json::to_string(&blob).unwrap();

        println!("{}", json);
    }
}
