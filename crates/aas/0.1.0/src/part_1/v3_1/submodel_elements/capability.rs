use serde::{Deserialize, Serialize};
use crate::part_1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part_1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part_1::v3_1::attributes::referable::Referable;
use crate::part_1::v3_1::attributes::semantics::HasSemantics;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[serde(tag = "modelType")]
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

// TODO: Test serialization and deserialization
#[cfg(test)]
mod tests {}
