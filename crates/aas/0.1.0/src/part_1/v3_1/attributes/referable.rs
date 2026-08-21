use crate::part_1::v3_1::attributes::extension::HasExtensions;
use crate::part_1::v3_1::{IDShort};
use serde::{Deserialize, Serialize};
use crate::part_1::v3_1::primitives::{MultiLanguageNameType};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
pub struct Referable {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "idShort")]
    pub id_short: Option<IDShort>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "displayName")]
    pub display_name: Option<MultiLanguageNameType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<MultiLanguageNameType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated]
    pub category: Option<String>,

    /// HasExtensions
    #[serde(flatten)]
    pub extensions: HasExtensions,
}
