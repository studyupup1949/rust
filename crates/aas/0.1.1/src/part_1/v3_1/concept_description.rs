use crate::part_1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part_1::v3_1::attributes::identifiable::Identifiable;
use crate::part_1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};

/// The semantics of a property or other elements that may have a semantic description is defined
/// by a concept description.
/// The description of the concept should follow a standardized schema
/// (realized as data specification template).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ConceptDescription {
    #[serde(flatten)]
    pub identifiable: Identifiable,

    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_specification: Option<HasDataSpecification>,

    #[serde(rename = "isCaseOf")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_case_of: Option<Reference>,
}
