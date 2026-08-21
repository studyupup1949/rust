use crate::common::{ChoiceInputStyle, Height, TextInputStyle};
use crate::card::Spacing;
use serde::{Deserialize, Serialize};

/// Lets a user enter text.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputText {
    /// Unique identifier for the value. Used to identify collected input when the Submit action is performed.
    pub id: String,
    /// If true, allow multiple lines of input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_multiline: Option<bool>,
    /// Hint of maximum length characters to collect (may be ignored by some clients).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
    /// Description of the input desired. Displayed when no text has been input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// Regular expression indicating the required format of this text input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regex: Option<String>,
    /// Style hint for text input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<TextInputStyle>,
    /// The initial value for this field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Label for this input (recommended for accessibility).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Whether or not this input is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_required: Option<bool>,
    /// Error message to display when entered input is invalid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// When true, draw a separating line at the top of the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<bool>,
    /// Controls the amount of spacing between this element and the preceding element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<Spacing>,
    /// Specifies the height of the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<Height>,
    /// If false, this item will be removed from the visual tree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_visible: Option<bool>,
}

/// Allows a user to enter a number.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputNumber {
    /// Unique identifier for the value.
    pub id: String,
    /// Hint of minimum value (may be ignored by some clients).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Hint of maximum value (may be ignored by some clients).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Description of the input desired.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// The initial value for this field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    /// Label for this input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Whether or not this input is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_required: Option<bool>,
    /// Error message to display when entered input is invalid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// When true, draw a separating line at the top of the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<bool>,
    /// Controls the amount of spacing between this element and the preceding element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<Spacing>,
    /// Specifies the height of the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<Height>,
    /// If false, this item will be removed from the visual tree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_visible: Option<bool>,
}

/// Lets a user choose a date.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputDate {
    /// Unique identifier for the value.
    pub id: String,
    /// Hint of minimum value expressed in ISO-8601 format (may be ignored by some clients).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<String>,
    /// Hint of maximum value expressed in ISO-8601 format (may be ignored by some clients).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<String>,
    /// Description of the input desired.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// The initial value for this field expressed in ISO-8601 format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Label for this input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Whether or not this input is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_required: Option<bool>,
    /// Error message to display when entered input is invalid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// When true, draw a separating line at the top of the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<bool>,
    /// Controls the amount of spacing between this element and the preceding element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<Spacing>,
    /// Specifies the height of the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<Height>,
    /// If false, this item will be removed from the visual tree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_visible: Option<bool>,
}

/// Lets a user select a time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputTime {
    /// Unique identifier for the value.
    pub id: String,
    /// Hint of minimum value expressed in ISO-8601 time format (may be ignored by some clients).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<String>,
    /// Hint of maximum value expressed in ISO-8601 time format (may be ignored by some clients).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<String>,
    /// Description of the input desired.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// The initial value for this field expressed in ISO-8601 time format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Label for this input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Whether or not this input is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_required: Option<bool>,
    /// Error message to display when entered input is invalid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// When true, draw a separating line at the top of the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<bool>,
    /// Controls the amount of spacing between this element and the preceding element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<Spacing>,
    /// Specifies the height of the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<Height>,
    /// If false, this item will be removed from the visual tree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_visible: Option<bool>,
}

/// Lets a user choose between two options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputToggle {
    /// Unique identifier for the value.
    pub id: String,
    /// Title for the toggle.
    pub title: String,
    /// The current selected value. If the item is selected, the value will be "true", otherwise "false".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// The value when toggle is selected (default is "true").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_on: Option<String>,
    /// The value when toggle is not selected (default is "false").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_off: Option<String>,
    /// If true, allow text to wrap. Otherwise, text is clipped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap: Option<bool>,
    /// Label for this input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Whether or not this input is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_required: Option<bool>,
    /// Error message to display when entered input is invalid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// When true, draw a separating line at the top of the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<bool>,
    /// Controls the amount of spacing between this element and the preceding element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<Spacing>,
    /// Specifies the height of the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<Height>,
    /// If false, this item will be removed from the visual tree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_visible: Option<bool>,
}

/// Allows a user to input a choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputChoiceSet {
    /// Unique identifier for the value.
    pub id: String,
    /// Choice options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<InputChoice>>,
    /// Allow multiple choices to be selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_multi_select: Option<bool>,
    /// Style for the choice input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ChoiceInputStyle>,
    /// The initial choice (or set of choices) that should be selected.
    /// For multi-select, specify a comma-separated string of values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Description of the input desired. Only visible when no selection has been made,
    /// the style is compact and isMultiSelect is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// If true, allow text to wrap. Otherwise, text is clipped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap: Option<bool>,
    /// Label for this input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Whether or not this input is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_required: Option<bool>,
    /// Error message to display when entered input is invalid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// When true, draw a separating line at the top of the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<bool>,
    /// Controls the amount of spacing between this element and the preceding element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<Spacing>,
    /// Specifies the height of the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<Height>,
    /// If false, this item will be removed from the visual tree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_visible: Option<bool>,
}

/// Represents a choice for an Input.ChoiceSet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputChoice {
    /// The text to display for this choice.
    pub title: String,
    /// The value associated with this choice (this is the value that will be submitted).
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{AdaptiveCard, CardElement, Version};

    #[test]
    fn test_input_text_serialization() {
        let card = AdaptiveCard {
            version: Version::V1_3,
            body: vec![CardElement::InputText(InputText {
                id: "nameInput".to_string(),
                is_multiline: Some(false),
                max_length: Some(100),
                placeholder: Some("Enter your name".to_string()),
                label: Some("Name".to_string()),
                is_required: Some(true),
                error_message: Some("Name is required".to_string()),
                regex: None,
                style: None,
                value: None,
                separator: None,
                spacing: None,
                height: None,
                is_visible: None,
            })],
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&card).unwrap();
        assert!(json.contains("Input.Text"));
        assert!(json.contains("nameInput"));
        assert!(json.contains("Name is required"));
    }

    #[test]
    fn test_input_choiceset_serialization() {
        let card = AdaptiveCard {
            version: Version::V1_3,
            body: vec![CardElement::InputChoiceSet(InputChoiceSet {
                id: "colorChoice".to_string(),
                choices: Some(vec![
                    InputChoice {
                        title: "Red".to_string(),
                        value: "1".to_string(),
                    },
                    InputChoice {
                        title: "Green".to_string(),
                        value: "2".to_string(),
                    },
                    InputChoice {
                        title: "Blue".to_string(),
                        value: "3".to_string(),
                    },
                ]),
                is_multi_select: Some(false),
                style: Some(ChoiceInputStyle::Compact),
                value: Some("1".to_string()),
                placeholder: Some("Select a color".to_string()),
                wrap: None,
                label: Some("Choose your favorite color".to_string()),
                is_required: None,
                error_message: None,
                separator: None,
                spacing: None,
                height: None,
                is_visible: None,
            })],
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&card).unwrap();
        assert!(json.contains("Input.ChoiceSet"));
        assert!(json.contains("colorChoice"));
        assert!(json.contains("Red"));
        assert!(json.contains("compact"));
    }

    #[test]
    fn test_input_toggle_serialization() {
        let card = AdaptiveCard {
            version: Version::V1_3,
            body: vec![CardElement::InputToggle(InputToggle {
                id: "acceptTerms".to_string(),
                title: "I accept the terms and conditions".to_string(),
                value: Some("false".to_string()),
                value_on: Some("yes".to_string()),
                value_off: Some("no".to_string()),
                wrap: Some(true),
                label: Some("Terms".to_string()),
                is_required: Some(true),
                error_message: Some("You must accept the terms".to_string()),
                separator: None,
                spacing: None,
                height: None,
                is_visible: None,
            })],
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&card).unwrap();
        assert!(json.contains("Input.Toggle"));
        assert!(json.contains("acceptTerms"));
        assert!(json.contains("I accept the terms"));
    }

    #[test]
    fn test_input_number_serialization() {
        let card = AdaptiveCard {
            version: Version::V1_3,
            body: vec![CardElement::InputNumber(InputNumber {
                id: "ageInput".to_string(),
                min: Some(0.0),
                max: Some(120.0),
                placeholder: Some("Enter your age".to_string()),
                value: None,
                label: Some("Age".to_string()),
                is_required: Some(true),
                error_message: Some("Please enter a valid age".to_string()),
                separator: None,
                spacing: None,
                height: None,
                is_visible: None,
            })],
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&card).unwrap();
        assert!(json.contains("Input.Number"));
        assert!(json.contains("ageInput"));
        assert!(json.contains("120"));
    }

    #[test]
    fn test_input_date_serialization() {
        let card = AdaptiveCard {
            version: Version::V1_3,
            body: vec![CardElement::InputDate(InputDate {
                id: "birthdayInput".to_string(),
                min: Some("1900-01-01".to_string()),
                max: Some("2025-12-31".to_string()),
                placeholder: Some("Select date".to_string()),
                value: None,
                label: Some("Birthday".to_string()),
                is_required: None,
                error_message: None,
                separator: None,
                spacing: None,
                height: None,
                is_visible: None,
            })],
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&card).unwrap();
        assert!(json.contains("Input.Date"));
        assert!(json.contains("birthdayInput"));
        assert!(json.contains("1900-01-01"));
    }

    #[test]
    fn test_input_time_serialization() {
        let card = AdaptiveCard {
            version: Version::V1_3,
            body: vec![CardElement::InputTime(InputTime {
                id: "meetingTime".to_string(),
                min: Some("09:00".to_string()),
                max: Some("17:00".to_string()),
                placeholder: Some("Select time".to_string()),
                value: None,
                label: Some("Meeting Time".to_string()),
                is_required: None,
                error_message: None,
                separator: None,
                spacing: None,
                height: None,
                is_visible: None,
            })],
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&card).unwrap();
        assert!(json.contains("Input.Time"));
        assert!(json.contains("meetingTime"));
        assert!(json.contains("09:00"));
    }
}

