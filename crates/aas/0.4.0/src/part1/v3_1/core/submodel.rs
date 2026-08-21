use crate::part1::ToJsonMetamodel;
use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part1::v3_1::attributes::identifiable::Identifiable;
use crate::part1::v3_1::attributes::kind::ModellingKind;
use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part1::v3_1::attributes::semantics::HasSemantics;
use crate::part1::v3_1::submodel_elements::SubmodelElement;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

// make it an enum of ModellingKind?
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(from = "xml::SubmodelXML", into = "xml::SubmodelXML")
)]
pub struct Submodel {
    #[serde(flatten)]
    pub identifiable: Identifiable,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<ModellingKind>,

    #[serde(flatten)]
    pub semantics: HasSemantics,

    #[serde(flatten)]
    pub qualifier: Qualifiable,

    #[serde(flatten)]
    pub data_specification: HasDataSpecification,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "submodelElements")]
    pub submodel_elements: Option<Vec<SubmodelElement>>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SubmodelMeta {
    #[serde(flatten)]
    pub identifiable: Identifiable,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<ModellingKind>,

    #[serde(flatten)]
    pub semantics: HasSemantics,

    #[serde(flatten)]
    pub qualifier: Qualifiable,

    #[serde(flatten)]
    pub data_specification: HasDataSpecification,
}

impl From<Submodel> for SubmodelMeta {
    fn from(value: Submodel) -> Self {
        Self {
            identifiable: value.identifiable,
            kind: value.kind,
            semantics: value.semantics,
            qualifier: value.qualifier,
            data_specification: value.data_specification,
        }
    }
}

// Todo: Test
impl ToJsonMetamodel for Submodel {
    type Error = serde_json::Error;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        let meta = SubmodelMeta::from(self.clone());

        serde_json::to_string(&meta)
    }
}
#[cfg(feature = "xml")]
mod xml {

    use crate::part1::v3_1::attributes::administrative_information::AdministrativeInformation;
    use crate::part1::v3_1::attributes::data_specification::{
        EmbeddedDataSpecification, HasDataSpecification,
    };
    use crate::part1::v3_1::attributes::extension::{Extension, HasExtensions};
    use crate::part1::v3_1::attributes::identifiable::Identifiable;
    use crate::part1::v3_1::attributes::kind::ModellingKind;
    use crate::part1::v3_1::attributes::qualifiable::{Qualifiable, Qualifier};
    use crate::part1::v3_1::attributes::referable::Referable;
    use crate::part1::v3_1::attributes::semantics::HasSemantics;
    use crate::part1::v3_1::core::Submodel;

    use crate::part1::v3_1::primitives::Identifier;
    use crate::part1::v3_1::primitives::xml::LangStringTextType;
    use crate::part1::v3_1::reference::Reference;
    use crate::part1::v3_1::submodel_elements::SubmodelElement;
    use crate::utilities::deserialize_empty_identifier_as_none;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
    pub struct SubmodelElementsXML {
        #[serde(rename = "$value")]
        submodel_elements: Vec<SubmodelElement>,
    }

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    #[serde(rename = "Submodel")]
    pub(crate) struct SubmodelXML {
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
        pub extension: Option<Vec<Extension>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub kind: Option<ModellingKind>,
        // -----
        // HasSemantics
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "semanticId")]
        pub semantic_id: Option<Reference>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "supplementalSemanticIds")]
        pub supplemental_semantic_ids: Option<Vec<Reference>>,
        // ---
        // Qualifiable
        #[serde(skip_serializing_if = "Option::is_none")]
        pub qualifiers: Option<Vec<Qualifier>>,
        // ---
        // HasDataSpecification
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "embeddedDataSpecifications")]
        pub embedded_data_specifications: Option<Vec<EmbeddedDataSpecification>>,
        // --
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "submodelElements")]
        pub submodel_elements: Option<SubmodelElementsXML>,
    }

    impl From<SubmodelXML> for super::Submodel {
        fn from(value: SubmodelXML) -> Self {
            Submodel {
                identifiable: Identifiable {
                    id: value.id,
                    administration: value.administration,
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
                },
                qualifier: Qualifiable {
                    qualifiers: value.qualifiers,
                },
                semantics: HasSemantics {
                    semantic_id: value.semantic_id,
                    supplemental_semantic_ids: value.supplemental_semantic_ids,
                },
                data_specification: HasDataSpecification {
                    embedded_data_specifications: value.embedded_data_specifications,
                },
                kind: value.kind,
                submodel_elements: value
                    .submodel_elements
                    .and_then(|v| Some(v.submodel_elements)),
            }
        }
    }

    impl From<super::Submodel> for SubmodelXML {
        fn from(value: Submodel) -> Self {
            SubmodelXML {
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
                extension: value.identifiable.referable.extensions.extension,
                kind: value.kind,
                semantic_id: value.semantics.semantic_id,
                supplemental_semantic_ids: value.semantics.supplemental_semantic_ids,
                qualifiers: value.qualifier.qualifiers,
                embedded_data_specifications: value.data_specification.embedded_data_specifications,
                submodel_elements: value.submodel_elements.map(|v| SubmodelElementsXML {
                    submodel_elements: v,
                }),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::part1::v3_1::LangString;
        use crate::part1::v3_1::attributes::administrative_information::{
            AdministrativeInformation, Version,
        };
        use crate::part1::v3_1::attributes::identifiable::Identifiable;
        use crate::part1::v3_1::attributes::referable::Referable;
        use crate::part1::v3_1::attributes::semantics::HasSemantics;
        use crate::part1::v3_1::key::Key;
        use crate::part1::v3_1::primitives::Identifier;
        use crate::part1::v3_1::reference::{Reference, ReferenceInner};

        #[test]
        fn serializes_submodel() {
            let submodel = super::Submodel {
                identifiable: Identifiable {
                    id: Identifier::try_from(
                        "https://admin-shell.io/idta/SubmodelTemplate/DigitalNameplate/3/0",
                    )
                    .unwrap(),
                    administration: Some(AdministrativeInformation {
                        version: Version {
                            version: Some("3".into()),
                            revision: Some("0".into()),
                        },
                        creator: None,
                        template_id: Some(
                            Identifier::try_from("https://admin-shell.io/IDTA 02006-3-0").unwrap(),
                        ),
                        data_specification: Default::default(),
                    }),
                    referable: Referable {
                        id_short: Some(Identifier::try_from("Nameplate").unwrap()),
                        display_name: None,
                        description: Some(vec![
                            LangString::try_new(
                                "en",
                                "Contains the nameplate information attached to the product".into(),
                            )
                            .unwrap(),
                        ]),
                        category: None,
                        extensions: Default::default(),
                    },
                },
                kind: Some(super::ModellingKind::Instance),
                semantics: HasSemantics {
                    semantic_id: Some(Reference::ExternalReference(ReferenceInner {
                        referred_semantic_id: None,
                        keys: vec![Key::GlobalReference(
                            "https://admin-shell.io/idta/nameplate/3/0/Nameplate".into(),
                        )],
                    })),
                    supplemental_semantic_ids: None,
                },
                qualifier: Default::default(),
                data_specification: Default::default(),
                submodel_elements: None,
            };
            let xml = quick_xml::se::to_string(&submodel).unwrap();
            println!("{}", xml);
        }
        #[test]
        fn deserialize_submodel() {
            let xml = r#"
             <submodel>
                  <idShort>Nameplate</idShort>
                  <description>
                    <langStringTextType>
                      <language>en</language>
                      <text>Contains the nameplate information attached to the product</text>
                    </langStringTextType>
                  </description>
                  <administration>
                    <version>3</version>
                    <revision>0</revision>
                    <templateId>https://admin-shell.io/IDTA 02006-3-0</templateId>
                  </administration>
                  <id>https://admin-shell.io/idta/SubmodelTemplate/DigitalNameplate/3/0</id>
                  <kind>Instance</kind>
                  <semanticId>
                    <type>ExternalReference</type>
                    <keys>
                      <key>
                        <type>GlobalReference</type>
                        <value>https://admin-shell.io/idta/nameplate/3/0/Nameplate</value>
                      </key>
                    </keys>
                  </semanticId>
                </submodel>
            "#;

            let expected = super::Submodel {
                identifiable: Identifiable {
                    id: Identifier::try_from(
                        "https://admin-shell.io/idta/SubmodelTemplate/DigitalNameplate/3/0",
                    )
                    .unwrap(),
                    administration: Some(AdministrativeInformation {
                        version: Version {
                            version: Some("3".into()),
                            revision: Some("0".into()),
                        },
                        creator: None,
                        template_id: Some(
                            Identifier::try_from("https://admin-shell.io/IDTA 02006-3-0").unwrap(),
                        ),
                        data_specification: Default::default(),
                    }),
                    referable: Referable {
                        id_short: Some(Identifier::try_from("Nameplate").unwrap()),
                        display_name: None,
                        description: Some(vec![
                            LangString::try_new(
                                "en",
                                "Contains the nameplate information attached to the product".into(),
                            )
                            .unwrap(),
                        ]),
                        category: None,
                        extensions: Default::default(),
                    },
                },
                kind: Some(super::ModellingKind::Instance),
                semantics: HasSemantics {
                    semantic_id: Some(Reference::ExternalReference(ReferenceInner {
                        referred_semantic_id: None,
                        keys: vec![Key::GlobalReference(
                            "https://admin-shell.io/idta/nameplate/3/0/Nameplate".into(),
                        )],
                    })),
                    supplemental_semantic_ids: None,
                },
                qualifier: Default::default(),
                data_specification: Default::default(),
                submodel_elements: None,
            };

            let actual = quick_xml::de::from_str(xml).unwrap();

            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn deserialize_submodel_with_blob() {
        let xml = r#"
             <submodel>
                  <idShort>Nameplate</idShort>
                  <description>
                    <langStringTextType>
                      <language>en</language>
                      <text>Contains the nameplate information attached to the product</text>
                    </langStringTextType>
                  </description>
                  <administration>
                    <version>3</version>
                    <revision>0</revision>
                    <templateId>https://admin-shell.io/IDTA 02006-3-0</templateId>
                  </administration>
                  <id>https://admin-shell.io/idta/SubmodelTemplate/DigitalNameplate/3/0</id>
                  <kind>Instance</kind>
                  <semanticId>
                    <type>ExternalReference</type>
                    <keys>
                      <key>
                        <type>GlobalReference</type>
                        <value>https://admin-shell.io/idta/nameplate/3/0/Nameplate</value>
                      </key>
                    </keys>
                  </semanticId>
                  <submodelElements>
                    <blob>
                        <value>test.png</value>
                        <contentType>image/png</contentType>
                    </blob>
                  </submodelElements>
                </submodel>
            "#;

        let expected = super::Submodel {
            identifiable: Identifiable {
                id: Identifier::try_from(
                    "https://admin-shell.io/idta/SubmodelTemplate/DigitalNameplate/3/0",
                )
                .unwrap(),
                administration: Some(AdministrativeInformation {
                    version: Version {
                        version: Some("3".into()),
                        revision: Some("0".into()),
                    },
                    creator: None,
                    template_id: Some(
                        Identifier::try_from("https://admin-shell.io/IDTA 02006-3-0").unwrap(),
                    ),
                    data_specification: Default::default(),
                }),
                referable: Referable {
                    id_short: Some(Identifier::try_from("Nameplate").unwrap()),
                    display_name: None,
                    description: Some(vec![
                        LangString::try_new(
                            "en",
                            "Contains the nameplate information attached to the product".into(),
                        )
                        .unwrap(),
                    ]),
                    category: None,
                    extensions: Default::default(),
                },
            },
            kind: Some(super::ModellingKind::Instance),
            semantics: HasSemantics {
                semantic_id: Some(Reference::ExternalReference(ReferenceInner {
                    referred_semantic_id: None,
                    keys: vec![Key::GlobalReference(
                        "https://admin-shell.io/idta/nameplate/3/0/Nameplate".into(),
                    )],
                })),
                supplemental_semantic_ids: None,
            },
            qualifier: Default::default(),
            data_specification: Default::default(),
            submodel_elements: Some(vec![SubmodelElement::Blob(Blob::new(
                Some("test.png".into()),
                "image/png".into(),
            ))]),
        };

        let actual = quick_xml::de::from_str(xml).unwrap();

        assert_eq!(expected, actual);
    }
}

#[cfg(not(feature = "xml"))]
#[cfg(test)]
mod tests {
    use crate::part1::v3_1::core::Submodel;

    #[test]
    fn deserialize_submodel_mvpdpp() {
        let json = include_str!("../../../../tests/submodel_test_mvpdpp.json");

        serde_json::from_str::<Submodel>(json).unwrap();
    }
}
