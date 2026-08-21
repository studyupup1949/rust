use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeModel {
    pub id: String,
    pub title: String,
    pub desc: String,
    pub uses: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    pub action: serde_json::Value,
}
