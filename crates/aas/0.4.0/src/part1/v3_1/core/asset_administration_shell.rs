use crate::part1::ToJsonMetamodel;
use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part1::v3_1::attributes::identifiable::Identifiable;
use crate::part1::v3_1::attributes::semantics::HasSemantics;
use crate::part1::v3_1::primitives::{ContentType, Identifier, Label, Uri};
use crate::part1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use strum::{Display, EnumString};

#[cfg(feature = "json")]
use crate::part1::v3_1::reference::deserialize_optional_external_reference;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename = "assetAdministrationShell")]
#[cfg_attr(
    feature = "xml",
    serde(
        from = "xml::AssetAdministrationShellXML",
        into = "xml::AssetAdministrationShellXML"
    )
)]
pub struct AssetAdministrationShell {
    #[serde(rename = "assetInformation")]
    pub asset_information: AssetInformation,

    #[serde(flatten)]
    pub identifiable: Identifiable,

    #[serde(flatten)]
    pub data_specification: HasDataSpecification,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "derivedFrom")]
    pub derived_from: Option<Reference>,

    // TODO: What kind of submodel keys are supported?
    // 1. Only one Key
    // Only key type "Submodel" allowed?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submodels: Option<Vec<Reference>>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "AssetAdministrationShellMeta")]
pub struct AssetAdministrationShellMetamodel {
    #[serde(flatten)]
    pub identifiable: Identifiable,

    #[serde(flatten)]
    pub data_specification: HasDataSpecification,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "derivedFrom")]
    pub derived_from: Option<Reference>,
}

impl From<AssetAdministrationShell> for AssetAdministrationShellMetamodel {
    fn from(value: AssetAdministrationShell) -> Self {
        Self {
            identifiable: value.identifiable,
            data_specification: value.data_specification,
            derived_from: value.derived_from,
        }
    }
}

// Todo: Test
impl ToJsonMetamodel for AssetAdministrationShell {
    type Error = serde_json::Error;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        let meta = AssetAdministrationShellMetamodel::from(self.clone());

        serde_json::to_string(&meta)
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, EnumString, Display)]
#[serde(tag = "assetKind")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(from = "xml::AssetInformationXML", into = "xml::AssetInformationXML")
)]
pub enum AssetInformation {
    Instance(AssetInformationInner),
    NotApplicable(AssetInformationInner),
    Role(AssetInformationInner),
    Type(AssetInformationInner),
}

impl Deref for AssetInformation {
    type Target = AssetInformationInner;

    fn deref(&self) -> &Self::Target {
        match self {
            AssetInformation::Instance(i)
            | AssetInformation::NotApplicable(i)
            | AssetInformation::Role(i)
            | AssetInformation::Type(i) => i,
        }
    }
}

// TODO: Skip option serialization
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AssetInformationInner {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "globalAssetId")]
    pub global_asset_id: Option<Identifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "specificAssetIds")]
    pub specific_asset_ids: Option<Vec<SpecificAssetId>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "assetType")]
    pub asset_type: Option<Identifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "defaultThumbnail")]
    pub default_thumbnail: Option<Resource>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(from = "xml::SpecificAssetIdXML", into = "xml::SpecificAssetIdXML")
)]
pub struct SpecificAssetId {
    #[serde(flatten)]
    pub has_semantics: HasSemantics,

    pub name: Label,

    pub value: Identifier,

    /// The unique ID of the (external) subject the specific asset ID value belongs to or
    /// has meaning to
    /// Needs to be an external reference!
    /// TODO: Typesafe with Newtype pattern
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "externalSubjectId")]
    #[serde(deserialize_with = "deserialize_optional_external_reference")]
    pub external_subject_id: Option<Reference>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Resource {
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub path: Uri,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "contentType")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    pub content_type: Option<ContentType>,
}

#[cfg(feature = "xml")]
mod xml {
    use crate::part1::v3_1::attributes::administrative_information::AdministrativeInformation;
    use crate::part1::v3_1::attributes::data_specification::{
        EmbeddedDataSpecification, HasDataSpecification,
    };
    use crate::part1::v3_1::attributes::extension::{Extension, HasExtensions};
    use crate::part1::v3_1::attributes::identifiable::Identifiable;
    use crate::part1::v3_1::attributes::referable::Referable;
    use crate::part1::v3_1::attributes::semantics::HasSemantics;
    use crate::part1::v3_1::core::{
        AssetAdministrationShell, AssetInformation, AssetInformationInner, Resource,
        SpecificAssetId,
    };
    use crate::part1::v3_1::primitives::xml::LangStringTextType;
    use crate::part1::v3_1::primitives::{Identifier, Label};
    use crate::part1::v3_1::reference::Reference;
    use crate::part1::v3_1::reference::deserialize_optional_external_reference;
    use crate::utilities::deserialize_empty_identifier_as_none;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AssetKind {
        Instance,
        NotApplicable,
        Role,
        Type,
    }
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub(super) struct AssetInformationXML {
        #[serde(rename = "assetKind")]
        pub asset_kind: AssetKind,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "globalAssetId")]
        pub global_asset_id: Option<Identifier>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "specificAssetIds")]
        pub specific_asset_ids: Option<Vec<SpecificAssetId>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "assetType")]
        pub asset_type: Option<Identifier>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "defaultThumbnail")]
        pub default_thumbnail: Option<Resource>,
    }

    impl From<AssetInformationXML> for AssetInformation {
        fn from(value: AssetInformationXML) -> Self {
            let inner = AssetInformationInner {
                global_asset_id: value.global_asset_id,
                specific_asset_ids: value.specific_asset_ids,
                asset_type: value.asset_type,
                default_thumbnail: value.default_thumbnail,
            };

            match value.asset_kind {
                AssetKind::Instance => AssetInformation::Instance(inner),
                AssetKind::NotApplicable => AssetInformation::NotApplicable(inner),
                AssetKind::Role => AssetInformation::Role(inner),
                AssetKind::Type => AssetInformation::Type(inner),
            }
        }
    }

    impl From<AssetInformation> for AssetInformationXML {
        fn from(value: AssetInformation) -> Self {
            let (kind, value) = match value {
                AssetInformation::Instance(inner) => (AssetKind::Instance, inner),
                AssetInformation::NotApplicable(inner) => (AssetKind::NotApplicable, inner),
                AssetInformation::Role(inner) => (AssetKind::Role, inner),
                AssetInformation::Type(inner) => (AssetKind::Type, inner),
            };

            Self {
                asset_kind: kind,
                global_asset_id: value.global_asset_id,
                specific_asset_ids: value.specific_asset_ids,
                asset_type: value.asset_type,
                default_thumbnail: value.default_thumbnail,
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SpecificAssetIdXML {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "semanticId")]
        pub semantic_id: Option<Reference>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "supplementalSemanticIds")]
        pub supplemental_semantic_ids: Option<Vec<Reference>>,

        pub name: Label,

        pub value: Identifier,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "externalSubjectId")]
        #[serde(deserialize_with = "deserialize_optional_external_reference")]
        pub external_subject_id: Option<Reference>,
    }

    impl From<SpecificAssetIdXML> for SpecificAssetId {
        fn from(value: SpecificAssetIdXML) -> Self {
            Self {
                has_semantics: HasSemantics {
                    semantic_id: value.semantic_id,
                    supplemental_semantic_ids: value.supplemental_semantic_ids,
                },
                name: value.name,
                value: value.value,
                external_subject_id: value.external_subject_id,
            }
        }
    }
    impl From<SpecificAssetId> for SpecificAssetIdXML {
        fn from(value: SpecificAssetId) -> Self {
            Self {
                semantic_id: value.has_semantics.semantic_id,
                supplemental_semantic_ids: value.has_semantics.supplemental_semantic_ids,
                name: value.name,
                value: value.value,
                external_subject_id: value.external_subject_id,
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Submodels {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub reference: Option<Vec<Reference>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename = "assetAdministrationShell")]
    pub(super) struct AssetAdministrationShellXML {
        #[serde(rename = "assetInformation")]
        pub asset_information: AssetInformation,

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
        pub extension: Option<Vec<Extension>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "embeddedDataSpecifications")]
        pub embedded_data_specifications: Option<Vec<EmbeddedDataSpecification>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "derivedFrom")]
        pub derived_from: Option<Reference>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub submodels: Option<Submodels>,
    }

    impl From<AssetAdministrationShellXML> for AssetAdministrationShell {
        fn from(value: AssetAdministrationShellXML) -> Self {
            Self {
                asset_information: value.asset_information,
                identifiable: Identifiable {
                    id: value.id,
                    administration: value.administration,
                    referable: Referable {
                        id_short: value.id_short,
                        display_name: value.display_name.map(|v| v.values),
                        description: value.description.map(|v| v.values),
                        #[allow(deprecated)]
                        category: value.category,
                        extensions: HasExtensions {
                            extension: value.extension,
                        },
                    },
                },
                data_specification: HasDataSpecification {
                    embedded_data_specifications: value.embedded_data_specifications,
                },
                derived_from: value.derived_from,
                submodels: value.submodels.and_then(|v| v.reference),
            }
        }
    }

    impl From<AssetAdministrationShell> for AssetAdministrationShellXML {
        fn from(value: AssetAdministrationShell) -> Self {
            Self {
                asset_information: value.asset_information,
                id: value.identifiable.id,
                administration: value.identifiable.administration,
                id_short: value.identifiable.referable.id_short,
                display_name: value
                    .identifiable
                    .referable
                    .display_name
                    .map(|v| LangStringTextType { values: v }),
                description: value
                    .identifiable
                    .referable
                    .description
                    .map(|v| LangStringTextType { values: v }),
                #[allow(deprecated)]
                category: value.identifiable.referable.category,
                extension: value.identifiable.referable.extensions.extension,
                embedded_data_specifications: value.data_specification.embedded_data_specifications,
                derived_from: value.derived_from,
                submodels: value.submodels.map(|v| Submodels { reference: Some(v) }),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::part1::v3_1::attributes::administrative_information::{
            AdministrativeInformation, Version,
        };
        use crate::part1::v3_1::attributes::identifiable::Identifiable;
        use crate::part1::v3_1::attributes::referable::Referable;
        use crate::part1::v3_1::core::{
            AssetAdministrationShell, AssetInformation, AssetInformationInner, Resource,
        };
        use crate::part1::v3_1::key::Key;
        use crate::part1::v3_1::primitives::{Identifier, Uri};
        use crate::part1::v3_1::reference::{Reference, ReferenceInner};

        #[test]
        fn deserialize_asset_administration_shell_xml_simple() {
            let json = r#"
            <assetAdministrationShell>
                <assetInformation><assetKind>Instance</assetKind></assetInformation>
                <id>https://smartfactory-owl.de/ids/aas/2001_1172_9042_4560</id>
            </assetAdministrationShell>
            "#;

            let asset_administration_shell: AssetAdministrationShell =
                quick_xml::de::from_str(&json).unwrap();

            println!("{:?}", asset_administration_shell);
        }

        #[test]
        fn serialize() {
            let expected = AssetAdministrationShell {
                asset_information: AssetInformation::NotApplicable(AssetInformationInner {
                    global_asset_id: None,
                    specific_asset_ids: None,
                    asset_type: None,
                    default_thumbnail: Some(Resource {
                        path: Uri::new("/aasx/files/turtle_dpp_thumbnail.jpg".into()).unwrap(),
                        content_type: Some("image/jpeg".into()),
                    }),
                }),
                identifiable: Identifiable {
                    id: Identifier::try_from("https://smartfactory-owl.de/ids/aas/2001_1172_9042_4560").unwrap(),
                    administration: None,
                    referable: Referable {
                        id_short: Some(Identifier::try_from("AAS_Tortoise_DigitalProductPassport").unwrap()),
                        display_name: None,
                        description: None,
                        category: None,
                        extensions: Default::default(),
                    },
                },
                data_specification: Default::default(),
                derived_from: None,
                submodels: Some(vec![
                    Reference::ModelReference(ReferenceInner {
                        referred_semantic_id: None,
                        keys: vec![
                            Key::Submodel("https://admin-shell.io/idta/SubmodelTemplate/DigitalNameplate/3/0".into())
                        ],
                    }),
                    Reference::ModelReference(ReferenceInner {
                        referred_semantic_id: None,
                        keys: vec![
                            Key::Submodel("https://admin-shell.io/idta/SubmodelTemplate/CarbonFootprint/1/0".into())
                        ],
                    }),
                    Reference::ModelReference(ReferenceInner {
                        referred_semantic_id: None,
                        keys: vec![
                            Key::Submodel("https://admin-shell.io/idta/SubmodelTemplate/HandoverDocumentation/1/0".into())
                        ],
                    }),
                ]),
            };

            let xml = quick_xml::se::to_string(&expected).unwrap();

            println!("{}", xml);
        }

        #[test]
        fn deserialize_complex() {
            let xml = r#"
            <assetAdministrationShell>
              <idShort>AAS_Tortoise_DigitalProductPassport</idShort>
              <id>https://smartfactory-owl.de/ids/aas/2001_1172_9042_4560</id>
              <assetInformation>
                <assetKind>NotApplicable</assetKind>
                <defaultThumbnail>
                    <path>/aasx/files/turtle_dpp_thumbnail.jpg</path>
                    <contentType>image/jpeg</contentType>
                </defaultThumbnail>
              </assetInformation>
              <administration>
                  <version>1.0</version>
                  <revision>0</revision>
              </administration>
              <submodels>
                <reference>
                  <type>ModelReference</type>
                  <keys>
                    <key>
                      <type>Submodel</type>
                      <value>https://admin-shell.io/idta/SubmodelTemplate/DigitalNameplate/3/0</value>
                    </key>
                  </keys>
                </reference>
                <reference>
                  <type>ModelReference</type>
                  <keys>
                    <key>
                      <type>Submodel</type>
                      <value>https://admin-shell.io/idta/SubmodelTemplate/CarbonFootprint/1/0</value>
                    </key>
                  </keys>
                </reference>
                <reference>
                  <type>ModelReference</type>
                  <keys>
                    <key>
                      <type>Submodel</type>
                      <value>https://admin-shell.io/idta/SubmodelTemplate/HandoverDocumentation/1/0</value>
                    </key>
                  </keys>
                </reference>
              </submodels>
            </assetAdministrationShell>
            "#;

            let expected = AssetAdministrationShell {
                asset_information: AssetInformation::NotApplicable(AssetInformationInner {
                    global_asset_id: None,
                    specific_asset_ids: None,
                    asset_type: None,
                    default_thumbnail: Some(Resource {
                        path: Uri::new("/aasx/files/turtle_dpp_thumbnail.jpg".into()).unwrap(),
                        content_type: Some("image/jpeg".into()),
                    }),
                }),
                identifiable: Identifiable {
                    id: Identifier::try_from("https://smartfactory-owl.de/ids/aas/2001_1172_9042_4560").unwrap(),
                    administration: Some(AdministrativeInformation {
                        version: Version { version: Some("1.0".into()), revision: Some("0".into()) },
                        creator: None,
                        template_id: None,
                        data_specification: Default::default(),
                    }),
                    referable: Referable {
                        id_short: Some(Identifier::try_from("AAS_Tortoise_DigitalProductPassport").unwrap()),
                        display_name: None,
                        description: None,
                        category: None,
                        extensions: Default::default(),
                    },
                },
                data_specification: Default::default(),
                derived_from: None,
                submodels: Some(vec![
                    Reference::ModelReference(ReferenceInner {
                        referred_semantic_id: None,
                        keys: vec![
                            Key::Submodel("https://admin-shell.io/idta/SubmodelTemplate/DigitalNameplate/3/0".into())
                        ],
                    }),
                    Reference::ModelReference(ReferenceInner {
                        referred_semantic_id: None,
                        keys: vec![
                            Key::Submodel("https://admin-shell.io/idta/SubmodelTemplate/CarbonFootprint/1/0".into())
                        ],
                    }),
                    Reference::ModelReference(ReferenceInner {
                        referred_semantic_id: None,
                        keys: vec![
                            Key::Submodel("https://admin-shell.io/idta/SubmodelTemplate/HandoverDocumentation/1/0".into())
                        ],
                    }),
                ]),
            };

            let actual = quick_xml::de::from_str(xml).unwrap();

            assert_eq!(expected, actual);
        }
    }
}

#[cfg(not(feature = "xml"))]
#[cfg(test)]
// only JSON
mod tests {
    use super::*;
    use iref::UriRefBuf;
    use std::str::FromStr;

    #[test]
    fn deserialize_resource() {
        let json = r#"
        {
            "path": "file://anywhere.json",
            "contentType": "application/json"
        }"#;

        let res: Resource = serde_json::from_str(json).unwrap();

        assert_eq!(
            res,
            Resource {
                path: UriRefBuf::from_str("file://anywhere.json").unwrap(),
                content_type: Some("application/json".into()),
            }
        )
    }

    #[test]
    fn deserialize_asset_info() {
        let json = include_str!("../../../../tests/asset_information.json");

        let asset_info: AssetInformation = serde_json::from_str(json).unwrap();

        println!("{:?}", asset_info);
    }
}
