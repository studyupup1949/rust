use crate::common::{ActionMode, ActionStyle, AssociatedInputs};
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
    #[serde(rename = "Action.Submit")]
    Submit(SubmitAction),
    #[serde(rename = "Action.ShowCard")]
    ShowCard(ShowCardAction),
    #[serde(rename = "Action.ToggleVisibility")]
    ToggleVisibility(ToggleVisibilityAction),
}

/// Opens a URL when the action is invoked.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenUrlAction {
    /// Label for button or link that represents this action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The URL to open.
    pub url: String,
    /// A unique identifier associated with this action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Optional icon to be shown on the action in conjunction with the title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Controls the style of an action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ActionStyle>,
    /// Defines text that should be displayed to the end user as they hover the mouse over the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    /// Determines whether the action should be enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
    /// Determines whether the action should be displayed as a button or in the overflow menu.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ActionMode>,
}

/// Gathers input fields, merges with optional data field, and sends an event to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitAction {
    /// Label for button or link that represents this action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Initial data that input fields will be combined with. These are essentially 'hidden' properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Controls which inputs are associated with the submit action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated_inputs: Option<AssociatedInputs>,
    /// A unique identifier associated with this action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Optional icon to be shown on the action in conjunction with the title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Controls the style of an action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ActionStyle>,
    /// Defines text that should be displayed to the end user as they hover the mouse over the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    /// Determines whether the action should be enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
    /// Determines whether the action should be displayed as a button or in the overflow menu.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ActionMode>,
}

/// Shows a card when the action is invoked. Note: AdaptiveCard is forward-declared.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowCardAction {
    /// Label for button or link that represents this action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The Adaptive Card to show when the action is invoked.
    pub card: Box<crate::card::AdaptiveCard>,
    /// A unique identifier associated with this action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Optional icon to be shown on the action in conjunction with the title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Controls the style of an action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ActionStyle>,
    /// Defines text that should be displayed to the end user as they hover the mouse over the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    /// Determines whether the action should be enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
    /// Determines whether the action should be displayed as a button or in the overflow menu.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ActionMode>,
}

/// Toggles the visibility of associated elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToggleVisibilityAction {
    /// Label for button or link that represents this action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The list of element IDs whose visibility should be toggled.
    pub target_elements: Vec<String>,
    /// A unique identifier associated with this action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Optional icon to be shown on the action in conjunction with the title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Controls the style of an action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ActionStyle>,
    /// Defines text that should be displayed to the end user as they hover the mouse over the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
    /// Determines whether the action should be enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
    /// Determines whether the action should be displayed as a button or in the overflow menu.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ActionMode>,
}
