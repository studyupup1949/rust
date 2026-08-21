use crate::part1::v3_1::attributes::semantics::HasSemantics;
use crate::part1::v3_1::primitives::data_type_def_xs::DataXsd;
use crate::part1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "xml", serde(rename = "qualifiers"))]
pub struct Qualifiable {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "xml", serde(rename = "$value"))]
    pub qualifiers: Option<Vec<Qualifier>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(from = "xml::QualifierXMLProxy", into = "xml::QualifierXMLProxy")
)]
pub enum Qualifier {
    ConceptQualifier(QualifierInner),
    TemplateQualifier(QualifierInner),
    ValueQualifier(QualifierInner),

    /// unknown values (kind = null!)
    #[serde(untagged)]
    Unknown(QualifierInner),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct QualifierInner {
    #[serde(flatten)]
    pub semantics: HasSemantics,

    // TODO: Text parsing
    #[serde(rename = "type")]
    pub ty: String,

    #[serde(flatten)]
    pub value: DataXsd,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "valueId")]
    pub value_id: Option<Reference>,
}

#[cfg(feature = "xml")]
pub(crate) mod xml {
    use crate::part1::v3_1::attributes::qualifiable::{Qualifier, QualifierInner};
    use crate::part1::v3_1::attributes::semantics::HasSemantics;
    use crate::part1::v3_1::primitives::data_type_def_xs::DataTypeXSDef;
    use crate::part1::v3_1::reference::Reference;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Default)]
    enum QualifierKind {
        ConceptQualifier,
        TemplateQualifier,
        ValueQualifier,
        #[default]
        Unknown,
    }

    #[derive(Serialize, Deserialize)]
    pub struct QualifierXMLProxy {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "semanticId")]
        pub semantic_id: Option<Reference>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "supplementalSemanticIds")]
        pub supplemental_semantic_ids: Option<Vec<Reference>>,
        // end HasSemantics

        // enum tag
        #[serde(default)]
        kind: QualifierKind,

        // TODO: Text parsing
        #[serde(rename = "type")]
        pub ty: String,

        // needs to be parsed into fitting type+value
        #[serde(rename = "valueType")]
        pub value_type: DataTypeXSDef,
        pub value: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "valueId")]
        pub value_id: Option<Reference>,
    }

    impl From<QualifierXMLProxy> for Qualifier {
        fn from(value: QualifierXMLProxy) -> Self {
            let inner = QualifierInner {
                semantics: HasSemantics {
                    semantic_id: value.semantic_id,
                    supplemental_semantic_ids: value.supplemental_semantic_ids,
                },
                ty: value.ty,
                value: (value.value_type, value.value).try_into().unwrap(),
                value_id: value.value_id,
            };
            match value.kind {
                QualifierKind::ConceptQualifier => Self::ConceptQualifier(inner),
                QualifierKind::TemplateQualifier => Self::TemplateQualifier(inner),
                QualifierKind::ValueQualifier => Self::ValueQualifier(inner),
                QualifierKind::Unknown => Self::Unknown(inner),
            }
        }
    }
    impl From<Qualifier> for QualifierXMLProxy {
        fn from(value: Qualifier) -> Self {
            let (kind, inner) = match value {
                Qualifier::ConceptQualifier(inner) => (QualifierKind::ConceptQualifier, inner),
                Qualifier::TemplateQualifier(inner) => (QualifierKind::TemplateQualifier, inner),
                Qualifier::ValueQualifier(inner) => (QualifierKind::ValueQualifier, inner),
                Qualifier::Unknown(inner) => (QualifierKind::Unknown, inner),
            };

            Self {
                semantic_id: inner.semantics.semantic_id,
                supplemental_semantic_ids: inner.semantics.supplemental_semantic_ids,
                kind,
                ty: inner.ty,
                value_type: inner.value.clone().into(),
                value: inner.value.clone().into(),
                value_id: inner.value_id,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::part1::v3_1::attributes::qualifiable::Qualifiable;
        use crate::part1::v3_1::key::Key;
        use crate::part1::v3_1::primitives::data_type_def_xs::DataXsd;
        use crate::part1::v3_1::reference::ReferenceInner;
        #[test]
        fn test_deserialize_simple() {
            let xml = r#"
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
            </qualifier>"#;

            let expected = Qualifier::TemplateQualifier(QualifierInner {
                semantics: HasSemantics {
                    semantic_id: Some(Reference::ExternalReference(ReferenceInner {
                        referred_semantic_id: None,
                        keys: vec![Key::GlobalReference(
                            "https://admin-shell.io/SubmodelTemplates/Cardinality/1/0".into(),
                        )],
                    })),
                    supplemental_semantic_ids: None,
                },
                ty: "SMT/Cardinality".to_string(),
                value: DataXsd::String(Some("One".into())),
                value_id: None,
            });

            let qualifier: Qualifier = quick_xml::de::from_str(xml).unwrap();

            assert_eq!(expected, qualifier);
        }

        #[test]
        fn test_deserialize_array() {
            let xml = r#"
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
          "#;

            let expected = Qualifiable {
                qualifiers: Some(vec![Qualifier::TemplateQualifier(QualifierInner {
                    semantics: HasSemantics {
                        semantic_id: Some(Reference::ExternalReference(ReferenceInner {
                            referred_semantic_id: None,
                            keys: vec![Key::GlobalReference(
                                "https://admin-shell.io/SubmodelTemplates/Cardinality/1/0".into(),
                            )],
                        })),
                        supplemental_semantic_ids: None,
                    },
                    ty: "SMT/Cardinality".to_string(),
                    value: DataXsd::String(Some("One".into())),
                    value_id: None,
                })]),
            };

            let actual: Qualifiable = quick_xml::de::from_str(xml).unwrap();

            dbg!(actual);
            //assert_eq!(expected, actual);
        }
    }
}

#[cfg(all(feature = "json", test))]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_deserialize() {
        let expected = Qualifier::Unknown(QualifierInner {
            semantics: Default::default(),
            ty: "Test".to_string(),
            value: DataXsd::Boolean(Some(true)),
            value_id: None,
        });

        let json = r#"
        {
            "kind":"Test",
            "type": "Test",
            "valueType":"xs:boolean",
            "value": true
        }"#;

        let actual: Qualifier = serde_json::from_str(json).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_concept_qualifier_deserialize() {
        let expected = Qualifier::ConceptQualifier(QualifierInner {
            semantics: Default::default(),
            ty: "Test".to_string(),
            value: DataXsd::Boolean(Some(true)),
            value_id: None,
        });

        let json = r#"
        {
            "kind":"ConceptQualifier",
            "type": "Test",
            "valueType":"xs:boolean",
            "value": true
        }"#;

        let actual: Qualifier = serde_json::from_str(json).unwrap();

        assert_eq!(expected, actual);
    }
}
