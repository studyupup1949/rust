use crate::part_1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ValueList {
    #[serde(rename = "valueReferencePairs")]
    pub value_reference_pairs: Vec<ValueReferencePair>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct ValueReferencePair {
    pub value: String,
    #[serde(rename = "valueId")]
    pub value_id: Reference,
}
