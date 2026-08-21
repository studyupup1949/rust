use crate::part_1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};

// HasSemantics
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
pub struct HasSemantics {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "semanticId")]
    pub semantic_id: Option<Reference>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "supplementalSemanticIds")]
    pub supplemental_semantic_ids: Option<Vec<Reference>>,
}
