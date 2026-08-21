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
    #[serde(rename = "Action.ToggleVisibility")]
    ToggleVisibility(ToggleVisibilityAction),
}

// ToggleVisibility action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleVisibilityAction {
    pub title: Option<String>,
    pub target_elements: Vec<String>,
}

// OpenUrl action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenUrlAction {
    pub title: String,
    pub url: String,
}
