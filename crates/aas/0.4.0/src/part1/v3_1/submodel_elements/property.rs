use crate::part1::MetamodelError;
use crate::part1::ToJsonMetamodel;
use crate::part1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part1::v3_1::attributes::referable::Referable;
use crate::part1::v3_1::attributes::semantics::HasSemantics;
use crate::part1::v3_1::primitives::data_type_def_xs::DataXsd;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(from = "xml::PropertyXMLProxy", into = "xml::PropertyXMLProxy")
)]
pub struct Property {
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
    #[serde(flatten)]
    pub value: DataXsd,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PropertyMeta {
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

impl From<Property> for PropertyMeta {
    fn from(prop: Property) -> Self {
        Self {
            referable: prop.referable,
            semantics: prop.semantics,
            qualifiable: prop.qualifiable,
            embedded_data_specifications: prop.embedded_data_specifications,
        }
    }
}

impl From<&Property> for PropertyMeta {
    fn from(prop: &Property) -> Self {
        let prop = prop.clone();
        Self {
            referable: prop.referable,
            semantics: prop.semantics,
            qualifiable: prop.qualifiable,
            embedded_data_specifications: prop.embedded_data_specifications,
        }
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
    use crate::part1::v3_1::attributes::semantics::xml::SupplementalSemanticIdsWrapper;
    use crate::part1::v3_1::primitives::Identifier;
    use crate::part1::v3_1::primitives::data_type_def_xs::DataTypeXSDef;
    use crate::part1::v3_1::primitives::xml::LangStringTextType;
    use crate::part1::v3_1::reference::Reference;
    use crate::part1::v3_1::submodel_elements::property::Property;
    use crate::utilities::deserialize_empty_identifier_as_none;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct PropertyXMLProxy {
        // Inherited
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
        pub category: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "extensions")]
        pub extension: Option<Vec<Extension>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "semanticId")]
        pub semantic_id: Option<Reference>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "supplementalSemanticIds")]
        pub supplemental_semantic_ids: Option<SupplementalSemanticIdsWrapper>,

        pub qualifiers: Option<Qualifiable>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "embeddedDataSpecifications")]
        embedded_data_specifications: Option<Vec<EmbeddedDataSpecification>>,
        // ----- end inheritance

        // needs to be parsed into fitting type+value
        #[serde(rename = "valueType")]
        pub value_type: DataTypeXSDef,
        pub value: Option<String>,
    }

    impl From<Property> for PropertyXMLProxy {
        fn from(value: Property) -> Self {
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
                supplemental_semantic_ids: Some(SupplementalSemanticIdsWrapper {
                    reference: value.semantics.supplemental_semantic_ids,
                }),
                qualifiers: Some(value.qualifiable),
                embedded_data_specifications: value
                    .embedded_data_specifications
                    .embedded_data_specifications,

                value: value.value.clone().into(),
                value_type: value.value.clone().into(),
            }
        }
    }

    impl From<PropertyXMLProxy> for Property {
        fn from(value: PropertyXMLProxy) -> Self {
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
                    supplemental_semantic_ids: value
                        .supplemental_semantic_ids
                        .and_then(|v| v.reference),
                },
                qualifiable: Qualifiable {
                    qualifiers: value.qualifiers.and_then(|v| v.qualifiers),
                },
                embedded_data_specifications: HasDataSpecification {
                    embedded_data_specifications: value.embedded_data_specifications,
                },
                value: (value.value_type, value.value).try_into().unwrap(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::part1::v3_1::attributes::qualifiable::{Qualifiable, Qualifier, QualifierInner};
        use crate::part1::v3_1::attributes::referable::Referable;
        use crate::part1::v3_1::attributes::semantics::HasSemantics;
        use crate::part1::v3_1::key::Key;
        use crate::part1::v3_1::primitives::Identifier;
        use crate::part1::v3_1::primitives::data_type_def_xs::DataXsd;
        use crate::part1::v3_1::reference::{Reference, ReferenceInner};
        use crate::part1::v3_1::submodel_elements::property::Property;
        use iref::IriRefBuf;

        #[test]
        fn deserialize_simple() {
            let expected = Property {
                referable: Default::default(),
                semantics: Default::default(),
                qualifiable: Default::default(),
                embedded_data_specifications: Default::default(),
                value: DataXsd::AnyURI(Some(
                    IriRefBuf::new("https://smartfactory-owl.de/3dl/__turtle/__00000001".into())
                        .unwrap(),
                )),
            };

            let xml = r#"
            <property>
                <valueType>xs:anyURI</valueType>
                <value>https://smartfactory-owl.de/3dl/__turtle/__00000001</value>
            </property>
        "#;

            let actual: Property = quick_xml::de::from_str(xml).expect("Should deserialize");

            assert_eq!(actual, expected);
        }

        #[test]
        fn deserialize_complex() {
            let expected = Property {
                referable: Referable {
                    id_short: Some(Identifier::try_from("URIOfTheProduct").unwrap()),
                    display_name: None,
                    description: None,
                    category: None,
                    extensions: Default::default(),
                },
                semantics: HasSemantics {
                    semantic_id: Some(Reference::ExternalReference(ReferenceInner {
                        referred_semantic_id: None,
                        keys: vec![Key::GlobalReference("0112/2///61987#ABN590#002".into())],
                    })),
                    supplemental_semantic_ids: Some(vec![Reference::ExternalReference(
                        ReferenceInner {
                            referred_semantic_id: None,
                            keys: vec![Key::GlobalReference("0173-1#02-ABH173#003".into())],
                        },
                    )]),
                },
                qualifiable: Qualifiable {
                    qualifiers: Some(vec![Qualifier::TemplateQualifier(QualifierInner {
                        semantics: HasSemantics {
                            semantic_id: Some(Reference::ExternalReference(ReferenceInner {
                                referred_semantic_id: None,
                                keys: vec![Key::GlobalReference(
                                    "https://admin-shell.io/SubmodelTemplates/Cardinality/1/0"
                                        .into(),
                                )],
                            })),
                            supplemental_semantic_ids: None,
                        },
                        ty: "SMT/Cardinality".to_string(),
                        value: DataXsd::String(Some("One".into())),
                        value_id: None,
                    })]),
                },
                embedded_data_specifications: Default::default(),
                value: DataXsd::AnyURI(Some(
                    IriRefBuf::new("https://smartfactory-owl.de/3dl/__turtle/__00000001".into())
                        .unwrap(),
                )),
            };

            let xml = r#"
            <property>
          <idShort>URIOfTheProduct</idShort>
          <semanticId>
            <type>ExternalReference</type>
            <keys>
              <key>
                <type>GlobalReference</type>
                <value>0112/2///61987#ABN590#002</value>
              </key>
            </keys>
          </semanticId>
          <supplementalSemanticIds>
            <reference>
              <type>ExternalReference</type>
              <keys>
                <key>
                  <type>GlobalReference</type>
                  <value>0173-1#02-ABH173#003</value>
                </key>
              </keys>
            </reference>
          </supplementalSemanticIds>
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
              <kind>TemplateQualifier</kind>
              <type>SMT/Cardinality</type>
              <valueType>xs:string</valueType>
              <value>One</value>
            </qualifier>
          </qualifiers>
          <valueType>xs:anyURI</valueType>
          <value>https://smartfactory-owl.de/3dl/__turtle/__00000001</value>
        </property>
        "#;

            let actual: Property = quick_xml::de::from_str(xml).expect("Should deserialize");

            assert_eq!(actual, expected);
        }
    }
}

impl ToJsonMetamodel for Property {
    type Error = MetamodelError;
    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        serde_json::to_string::<PropertyMeta>(&self.into())
            .map_err(|e| MetamodelError::FailedSerialisation(e))
    }
}
