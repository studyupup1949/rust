use serde::{Deserialize, Serialize};

use super::meta::Meta;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SdmxResponse<T> {
    pub data: T,
    pub meta: Meta,
}
