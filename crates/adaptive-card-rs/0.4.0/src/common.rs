use serde::{Deserialize, Serialize};

/// Controls the color of text elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Color {
    Default,
    Dark,
    Light,
    Accent,
    Good,
    Warning,
    Attention,
}

/// Controls horizontal alignment of elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
}

/// Controls vertical alignment of content within a container.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VerticalContentAlignment {
    Top,
    Center,
    Bottom,
}

/// Specifies the height of a block element.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Height {
    /// The height of the element will be determined by its contents.
    Auto,
    /// The element will stretch to fill available space.
    Stretch,
}

/// Type of font to use for rendering text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FontType {
    Default,
    Monospace,
}

/// Controls the style of an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionStyle {
    /// Action is displayed as normal.
    Default,
    /// Action is displayed with a positive style (typically accent color).
    Positive,
    /// Action is displayed with a destructive style (typically red).
    Destructive,
}

/// Determines whether an action should be displayed as a button or in overflow menu.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionMode {
    /// Action is displayed as a button.
    Primary,
    /// Action is placed in an overflow menu.
    Secondary,
}

/// Controls which inputs are associated with a submit action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssociatedInputs {
    /// Inputs on the current card and any parent cards will be validated and submitted.
    Auto,
    /// None of the inputs will be validated or submitted.
    None,
}

/// The style of text input.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextInputStyle {
    Text,
    Tel,
    Url,
    Email,
    Password,
}

/// The style of choice input.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChoiceInputStyle {
    /// Displayed as a dropdown/combo box.
    Compact,
    /// Displayed as radio buttons or checkboxes.
    Expanded,
    /// Allows users to filter choices (added in version 1.5).
    Filtered,
}

/// The style for a TextBlock when used for accessibility purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextBlockStyle {
    /// Default style with no special styling.
    Default,
    /// Marks the TextBlock as a heading for accessibility.
    Heading,
}
