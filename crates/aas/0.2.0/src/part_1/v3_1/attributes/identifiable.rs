use crate::part_1::v3_1::attributes::administrative_information::AdministrativeInformation;
use crate::part_1::v3_1::attributes::referable::Referable;
use crate::part_1::v3_1::primitives::Identifier;
use serde::{Deserialize, Serialize};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

///use crate::v3_1::asset_administration_shell::AdministrativeInformation;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Identifiable {
    pub id: Identifier,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub administrative_information: Option<AdministrativeInformation>,

    #[serde(flatten)]
    pub referable: Referable,
}
