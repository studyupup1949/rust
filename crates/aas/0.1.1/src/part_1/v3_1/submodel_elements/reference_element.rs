use crate::part_1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
pub struct ReferenceElement {
    /// External reference to an external object or entity or a logical reference
    /// to another element within the same or another Asset Administration Shell
    /// (i.e. a model reference to a Referable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Reference>,
}
