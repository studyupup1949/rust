use crate::part_1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part_1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part_1::v3_1::attributes::referable::Referable;
use crate::part_1::v3_1::attributes::semantics::HasSemantics;
use crate::part_1::v3_1::submodel_elements::SubmodelElement;
use crate::part_1::{MetamodelError, ToJsonMetamodel};
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Operation {
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
    #[serde(rename = "inputVariable")]
    pub input_variable: Option<Box<SubmodelElement>>,

    #[serde(rename = "outputVariable")]
    pub output_variable: Option<Box<SubmodelElement>>,

    #[serde(rename = "inoutputVariable")]
    pub inoutput_variable: Option<Box<SubmodelElement>>,
}

impl ToJsonMetamodel for Operation {
    type Error = MetamodelError;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        // TODO: add modelType tag
        serde_json::to_string(&self).map_err(|e| MetamodelError::FailedSerialisation(e))
    }
}
