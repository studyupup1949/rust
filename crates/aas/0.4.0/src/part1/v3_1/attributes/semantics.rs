use crate::part1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

// HasSemantics
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(from = "xml::HasSemanticsXMLProxy", into = "xml::HasSemanticsXMLProxy")
)]
pub struct HasSemantics {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "semanticId")]
    pub semantic_id: Option<Reference>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "supplementalSemanticIds")]
    pub supplemental_semantic_ids: Option<Vec<Reference>>,
}

pub(crate) mod xml {
    use crate::part1::v3_1::attributes::semantics::HasSemantics;
    use crate::part1::v3_1::reference::Reference;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct HasSemanticsXMLProxy {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "semanticId")]
        pub semantic_id: Option<Reference>,

        #[serde(rename = "supplementalSemanticIds")]
        pub supplemental_semantic_ids: SupplementalSemanticIdsWrapper,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct SupplementalSemanticIdsWrapper {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "$value")]
        pub reference: Option<Vec<Reference>>,
    }

    impl From<HasSemantics> for HasSemanticsXMLProxy {
        fn from(value: HasSemantics) -> Self {
            Self {
                semantic_id: value.semantic_id,
                supplemental_semantic_ids: SupplementalSemanticIdsWrapper {
                    reference: value.supplemental_semantic_ids,
                },
            }
        }
    }

    impl From<HasSemanticsXMLProxy> for HasSemantics {
        fn from(value: HasSemanticsXMLProxy) -> Self {
            Self {
                semantic_id: value.semantic_id,
                supplemental_semantic_ids: value.supplemental_semantic_ids.reference,
            }
        }
    }
}
