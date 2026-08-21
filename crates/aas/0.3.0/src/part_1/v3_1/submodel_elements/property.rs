use crate::part_1::MetamodelError;
use crate::part_1::ToJsonMetamodel;
use crate::part_1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part_1::v3_1::attributes::qualifiable::Qualifiable;
use crate::part_1::v3_1::attributes::referable::Referable;
use crate::part_1::v3_1::attributes::semantics::HasSemantics;
use crate::part_1::v3_1::primitives::data_type_def_xs::DataXsd;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Property {
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
    #[serde(flatten)]
    pub value: DataXsd,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PropertyMeta {
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

impl From<Property> for PropertyMeta {
    fn from(prop: Property) -> Self {
        Self {
            referable: prop.referable,
            semantics: prop.semantics,
            qualifiable: prop.qualifiable,
            embedded_data_specifications: prop.embedded_data_specifications,
        }
    }
}

impl From<&Property> for PropertyMeta {
    fn from(prop: &Property) -> Self {
        let prop = prop.clone();
        Self {
            referable: prop.referable,
            semantics: prop.semantics,
            qualifiable: prop.qualifiable,
            embedded_data_specifications: prop.embedded_data_specifications,
        }
    }
}

impl ToJsonMetamodel for Property {
    type Error = MetamodelError;
    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        serde_json::to_string::<PropertyMeta>(&self.into())
            .map_err(|e| MetamodelError::FailedSerialisation(e))
    }
}
