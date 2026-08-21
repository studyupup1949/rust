use crate::part_1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part_1::v3_1::attributes::identifiable::Identifiable;
use crate::part_1::v3_1::attributes::kind::ModellingKind;
use crate::part_1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part_1::v3_1::attributes::semantics::HasSemantics;
use crate::part_1::v3_1::submodel_elements::SubmodelElement;
use serde::{Deserialize, Serialize};

// make it an enum of ModellingKind?
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "modelKind")]
pub struct Submodel {
    #[serde(flatten)]
    pub identifiable: Identifiable,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<ModellingKind>,

    #[serde(flatten)]
    pub semantics: HasSemantics,

    #[serde(flatten)]
    pub qualifier: Qualifiable,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub data_specification: Option<HasDataSpecification>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "submodelElements")]
    pub submodel_elements: Option<Vec<SubmodelElement>>,
}
