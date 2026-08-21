use crate::part_1::ToJsonMetamodel;
use crate::part_1::v3_1::core::SpecificAssetId;
use crate::part_1::v3_1::primitives::Identifier;
use crate::part_1::v3_1::submodel_elements::SubmodelElement;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// The entity submodel element is designed to be used in submodels defining the relationship between the parts of the composite asset
/// it is composed of (e.g. bill of material).
/// These parts are called entities. Not all entities have a global asset ID.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Display, EnumString)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum Entity {
    /// There is no separate Asset Administration Shell for co-managed entities.
    /// Co-managed entities need to be part of a self-managed entity.
    CoManagedEntity(EntityInner),

    /// Self-managed entities have their own Asset Administration Shell but can be part of another composite self-managed entity.
    /// The asset represented by an Asset Administration Shell is a self-managed entity per definition
    SelfManagedEntity(EntityInner),
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct EntityInner {
    /// Statement applicable to the entity,
    /// each statement described by submodel element - typically with a qualified value
    pub statement: Option<Vec<SubmodelElement>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "globalAssetId")]
    pub global_asset_id: Option<Identifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "specificAssetId")]
    pub specific_asset_id: Option<Vec<SpecificAssetId>>,
}

impl ToJsonMetamodel for Entity {
    type Error = ();

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        // TODO: Add modelType
        match self {
            Entity::CoManagedEntity(_) => Ok(r#"{"entityType":"CoManagedEntity"}"#.into()),
            Entity::SelfManagedEntity(_) => Ok(r#"{"entityType":"SelfManagedEntity"}"#.into()),
        }
    }
}
