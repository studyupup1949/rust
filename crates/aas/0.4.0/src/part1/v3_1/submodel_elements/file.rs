use crate::part1::MetamodelError;
use crate::part1::ToJsonMetamodel;
use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part1::v3_1::attributes::referable::Referable;
use crate::part1::v3_1::attributes::semantics::HasSemantics;
use crate::part1::v3_1::primitives::{ContentType, Uri};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(all(feature = "openapi"), derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(from = "xml::FileXMLProxy", into = "xml::FileXMLProxy")
)]
pub struct File {
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
    /// Path and name of the file (with file extension)
    /// The path can be absolute or relative.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    pub value: Option<Uri>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "contentType")]
    pub content_type: Option<ContentType>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct FileMeta {
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

impl From<File> for FileMeta {
    fn from(file: File) -> Self {
        Self {
            referable: file.referable,
            semantics: file.semantics,
            qualifiable: file.qualifiable,
            embedded_data_specifications: file.embedded_data_specifications,
        }
    }
}

impl From<&File> for FileMeta {
    fn from(file: &File) -> Self {
        file.clone().into()
    }
}

impl ToJsonMetamodel for File {
    type Error = MetamodelError;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        serde_json::to_string::<FileMeta>(&self.into())
            .map_err(|e| MetamodelError::FailedSerialisation(e))
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
    use crate::part1::v3_1::primitives::xml::LangStringTextType;
    use crate::part1::v3_1::primitives::{ContentType, Identifier, Uri};
    use crate::part1::v3_1::submodel_elements::file::File;
    use crate::utilities::deserialize_empty_identifier_as_none;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    pub struct FileXMLProxy {
        // Inherited from DataElement
        #[serde(skip_serializing_if = "Option::is_none")]
        // use case where "" is needed or can this be ignored?
        #[serde(default)]
        #[serde(deserialize_with = "deserialize_empty_identifier_as_none")]
        #[serde(rename = "idShort")]
        pub id_short: Option<Identifier>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "displayName")]
        pub(crate) display_name: Option<LangStringTextType>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub(crate) description: Option<LangStringTextType>,

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
        // ----- end inheritance
        /// Path and name of the file (with file extension)
        /// The path can be absolute or relative.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub value: Option<Uri>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "contentType")]
        pub content_type: Option<ContentType>,
    }

    impl From<super::File> for FileXMLProxy {
        fn from(value: File) -> Self {
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

                value: value.value,
                content_type: value.content_type,
            }
        }
    }

    impl From<FileXMLProxy> for super::File {
        fn from(value: FileXMLProxy) -> Self {
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

                value: value.value,
                content_type: value.content_type,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn test_deserialize_complex() {
            let xml = r#"
            <file>
                  <idShort>AssemblyInstructions_DigitalFile</idShort>
                  <description>
                    <langStringTextType>
                      <language>en</language>
                      <text>Note: each DigitalFile represents the same content or Document version, but can be provided in different technical formats (PDF, PDFA, html, etc.) or by a link.</text>
                    </langStringTextType>
                  </description>
                  <semanticId>
                    <type>ExternalReference</type>
                    <keys>
                      <key>
                        <type>GlobalReference</type>
                        <value>0173-1#02-ABK126#003</value>
                      </key>
                    </keys>
                  </semanticId>
                  <qualifiers>
                    <qualifier>
                      <semanticId>
                        <type>ExternalReference</type>
                        <keys>
                          <key>
                            <type>GlobalReference</type>
                            <value>https://admin-shell.io/SubmodelTemplates/Cardinality/1/0</value>
                          </key>
                        </keys>
                      </semanticId>
                      <type>Cardinality</type>
                      <valueType>xs:string</valueType>
                      <value>OneToMany</value>
                    </qualifier>
                    <qualifier>
                      <semanticId>
                        <type>ExternalReference</type>
                        <keys>
                          <key>
                            <type>GlobalReference</type>
                            <value>https://admin-shell.io/SubmodelTemplates/ExampleValue/1/0</value>
                          </key>
                        </keys>
                      </semanticId>
                      <type>ExampleValue</type>
                      <valueType>xs:string</valueType>
                      <value>docu_cecc_fullmanual_DE.PDF</value>
                    </qualifier>
                    <qualifier>
                      <semanticId>
                        <type>ExternalReference</type>
                        <keys>
                          <key>
                            <type>GlobalReference</type>
                            <value>https://admin-shell.io/SubmodelTemplates/AllowedIdShort/1/0</value>
                          </key>
                        </keys>
                      </semanticId>
                      <type>AllowedIdShort</type>
                      <valueType>xs:string</valueType>
                      <value>DigitalFile[\d{2,3}]</value>
                    </qualifier>
                  </qualifiers>
                  <value>/aasx/files/safety_information.pdf</value>
                  <contentType>application/pdf</contentType>
                </file>"#;
        }
    }
}
