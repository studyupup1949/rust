use crate::part_1::v3_1::reference::Reference;
use crate::part_1::v3_1::submodel_elements::SubmodelElementFields;
use crate::part_1::v3_1::submodel_elements::data_element::DataElement;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(tag = "modelType")]
pub struct RelationshipElement {
    #[serde(flatten)]
    pub submodel_element_fields: SubmodelElementFields,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    first: Option<Reference>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    second: Option<Reference>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(tag = "modelType")]
pub struct AnnotatedRelationshipElement {
    // Inherited from RelationshipElement
    #[serde(flatten)]
    pub submodel_element_fields: SubmodelElementFields,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second: Option<Reference>,
    // ----

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<DataElement>>,
}
