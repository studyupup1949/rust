use serde::{Deserialize, Serialize};

// ActionSet element
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionSet {
    pub actions: Vec<Action>,
}

// Action types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    #[serde(rename = "Action.OpenUrl")]
    OpenUrl(OpenUrlAction),
}

// OpenUrl action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenUrlAction {
    pub title: String,
    pub url: String,
}
