/// com.atproto.repo types (manually entered)

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct Describe {
    pub handle: String,
    pub did: String,
    pub didDoc: serde_json::Value,
    pub collections: Vec<String>,
    pub handleIsCorrect: bool,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct CreateRecord {
    pub did: String,
    pub collection: String,
    pub rkey: Option<String>,
    pub validate: Option<bool>,
    pub record: serde_json::Value,
    pub swapCommit: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct PutRecord {
    pub did: String,
    pub collection: String,
    pub rkey: String,
    pub validate: Option<bool>,
    pub record: serde_json::Value,
    pub swapRecord: Option<String>,
    pub swapCommit: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct DeleteRecord {
    pub did: String,
    pub collection: String,
    pub rkey: String,
    pub swapRecord: Option<String>,
    pub swapCommit: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct BatchWriteBody {
    pub repo: String,
    pub validate: Option<bool>,
    pub writes: Vec<BatchWrite>,
    pub swapCommit: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct BatchWrite {
    #[serde(rename = "$type")]
    pub op_type: String,
    pub collection: String,
    pub rkey: Option<String>,
    pub value: Option<serde_json::Value>,
}
