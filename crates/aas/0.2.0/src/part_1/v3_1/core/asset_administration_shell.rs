use crate::part_1::v3_1::attributes::data_specification::HasDataSpecification;
use crate::part_1::v3_1::attributes::identifiable::Identifiable;
use crate::part_1::v3_1::attributes::semantics::HasSemantics;
use crate::part_1::v3_1::primitives::{ContentType, Identifier, Label, Uri};
use crate::part_1::v3_1::reference::Reference;
use crate::part_1::v3_1::reference::deserialize_optional_external_reference;
use crate::part_1::{ToJsonMetamodel, ToJsonValue};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
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

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "AssetAdministrationShellMeta")]
pub struct AssetAdministrationShellMetamodel {
    #[serde(flatten)]
    pub identifiable: Identifiable,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub data_specification: Option<HasDataSpecification>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "derivedFrom")]
    pub derived_from: Option<Reference>,
}

impl From<AssetAdministrationShell> for AssetAdministrationShellMetamodel {
    fn from(value: AssetAdministrationShell) -> Self {
        Self {
            identifiable: value.identifiable,
            data_specification: value.data_specification,
            derived_from: value.derived_from,
        }
    }
}

// Todo: Test
impl ToJsonMetamodel for AssetAdministrationShell {
    type Error = serde_json::Error;

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        let meta = AssetAdministrationShellMetamodel::from(self.clone());

        serde_json::to_string(&meta)
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, EnumString, Display)]
#[serde(tag = "assetKind")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum AssetInformation {
    Instance(AssetInformationInner),
    NotApplicable(AssetInformationInner),
    Role(AssetInformationInner),
    Type(AssetInformationInner),
}

// TODO: Skip option serialization
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct AssetInformationInner {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "globalAssetId")]
    pub global_asset_id: Option<Identifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "specificAssetIds")]
    pub specific_asset_ids: Option<Vec<SpecificAssetId>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "assetType")]
    pub asset_type: Option<Identifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "defaultThumbnail")]
    pub default_thumbnail: Option<Resource>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct SpecificAssetId {
    #[serde(flatten)]
    pub has_semantics: HasSemantics,

    pub name: Label,

    pub value: Identifier,

    /// The unique ID of the (external) subject the specific asset ID value belongs to or
    /// has meaning to
    /// Needs to be an external reference!
    /// TODO: Typesafe with Newtype pattern
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "externalSubjectId")]
    #[serde(deserialize_with = "deserialize_optional_external_reference")]
    pub external_subject_id: Option<Reference>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct Resource {
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    pub path: Uri,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "contentType")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    pub content_type: Option<ContentType>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use iref::UriBuf;
    use std::str::FromStr;

    #[test]
    fn deserialize_resource() {
        let json = r#"
        {
            "path": "file:://anywhere.json",
            "contentType": "application/json"
        }"#;

        let res: Resource = serde_json::from_str(json).unwrap();

        assert_eq!(
            res,
            Resource {
                path: UriBuf::from_str("file:://anywhere.json").unwrap(),
                content_type: Some("application/json".into()),
            }
        )
    }

    #[test]
    fn deserialize_asset_info() {
        let json = include_str!("../../../../tests/asset_information.json");

        let asset_info: AssetInformation = serde_json::from_str(json).unwrap();

        println!("{:?}", asset_info);
    }
}
