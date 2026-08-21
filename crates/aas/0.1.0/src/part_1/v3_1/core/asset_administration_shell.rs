use crate::part_1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part_1::v3_1::attributes::identifiable::Identifiable;
use crate::part_1::v3_1::primitives::{ContentType, Identifier, LabelType, Uri};
use crate::part_1::v3_1::reference::Reference;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "AssetAdministrationShell")]
pub struct AssetAdministrationShell {
    #[serde(rename = "assetInformation")]
    pub asset_information: AssetInformation,

    #[serde(flatten)]
    pub identifiable: Identifiable,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub data_specification: Option<HasDataSpecification>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "derivedFrom")]
    pub derived_from: Option<Reference>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub submodels: Option<Vec<Reference>>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, EnumString, Display)]
#[serde(tag = "assetKind")]
pub enum AssetInformation {
    Instance(AssetInformationInner),
    NotApplicable(AssetInformationInner),
    Role(AssetInformationInner),
    Type(AssetInformationInner),
}

// TODO: Skip option serialization
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct AssetInformationInner {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "globalAssetId")]
    pub global_asset_id: Option<Identifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "specificAssetIds")]
    pub specific_asset_ids: Option<SpecificAssetId>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "assetType")]
    pub asset_type: Option<Identifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "defaultThumbnail")]
    pub default_thumbnail: Option<Resource>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SpecificAssetId {
    pub name: LabelType,

    pub value: Identifier,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "externalSubjectId")]
    pub external_subject_id: Option<Resource>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Resource {
    path: Uri,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "contentType")]
    content_type: Option<ContentType>,
}
