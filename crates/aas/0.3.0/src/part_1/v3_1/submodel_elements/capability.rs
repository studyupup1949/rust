use crate::part_1::MetamodelError;
use crate::part_1::ToJsonMetamodel;
use crate::part_1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part_1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part_1::v3_1::attributes::referable::Referable;
use crate::part_1::v3_1::attributes::semantics::HasSemantics;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Capability {
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
}

impl Capability {
    pub fn new() -> Self {
        Self {
            referable: Referable::default(),
            semantics: HasSemantics::default(),
            qualifiable: Qualifiable::default(),
            embedded_data_specifications: HasDataSpecification::default(),
        }
    }
}

impl ToJsonMetamodel for Capability {
    type Error = MetamodelError;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        // TODO: Add modelType!
        serde_json::to_string(&self).map_err(|e| MetamodelError::FailedSerialisation(e))
    }
}

// TODO: Test serialization and deserialization
#[cfg(test)]
mod tests {}
