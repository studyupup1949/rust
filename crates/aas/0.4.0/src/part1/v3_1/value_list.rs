use crate::part1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ValueList {
    #[cfg_attr(feature = "json", serde(rename = "valueReferencePairs"))]
    #[cfg_attr(feature = "xml", serde(rename = "$value"))]
    pub value_reference_pairs: Vec<ValueReferencePair>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ValueReferencePair {
    pub value: String,
    #[serde(rename = "valueId")]
    pub value_id: Reference,
}
