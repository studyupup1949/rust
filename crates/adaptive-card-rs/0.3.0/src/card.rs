use serde::{Deserialize, Serialize};

use crate::actions::ActionSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Version {
    #[serde(rename = "1.0")]
    V1_0,
    #[serde(rename = "1.1")]
    V1_1,
    #[serde(rename = "1.2")]
    V1_2,
    #[serde(rename = "1.3")]
    V1_3,
    #[serde(rename = "1.4")]
    V1_4,
    #[serde(rename = "1.5")]
    V1_5,
    #[serde(rename = "1.6")]
    V1_6,
}

/// Represents an Adaptive Card, which is a container for card elements and actions.
/// Adaptive Cards are designed to be rendered in the Microsoft Adaptive Card ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct AdaptiveCard {
    /// The schema URL for Adaptive Cards, always "http://adaptivecards.io/schemas/adaptive-card.json".
    #[serde(rename = "$schema")]
    pub schema: String,
    /// The version of the Adaptive Card.
    pub version: Version,
    /// The body of the Adaptive Card, containing a collection of card elements.
    pub body: Vec<CardElement>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub msteams: Option<MsTeams>,
}

impl Default for AdaptiveCard {
    fn default() -> Self {
        Self {
            schema: "http://adaptivecards.io/schemas/adaptive-card.json".to_string(),
            version: Version::V1_2,
            body: Vec::new(),
            msteams: None,
        }
    }
}

/// Represents a card element within an Adaptive Card.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CardElement {
    /// A text block element that displays text.
    TextBlock(TextBlock),
    /// A container element that groups other elements.
    Container(Container),
    /// A column set element that arranges elements in columns.
    ColumnSet(ColumnSet),
    /// An image element that displays an image.
    Image(Image),
    /// An action set element that groups actions together.
    ActionSet(ActionSet),
    /// A fact set element that groups a collection of facts together.
    FactSet(FactSet),
}

/// Represents a text block element in an Adaptive Card.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextBlock {
    /// The size of the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<TextSize>,
    /// The weight of the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<TextWeight>,
    /// The text content to display.
    pub text: String,
    /// Whether the text should wrap if it exceeds the available space.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap: Option<bool>,
    /// Whether the text should be displayed subtly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_subtle: Option<bool>,
}

/// Represents a container element that groups other card elements together.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    /// The style of the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ContainerStyle>,
    /// The spacing around the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<Spacing>,
    /// The card elements contained within this container.
    pub items: Vec<CardElement>,
}

/// Represents the available width values for Microsoft Teams Adaptive Cards.
/// https://docs.microsoft.com/en-us/microsoftteams/platform/task-modules-and-cards/cards/cards-format#full-width-adaptive-card
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MsTeamsWidth {
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MsTeams {
    /// The width of the card in Microsoft Teams (currently only supports "full").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<MsTeamsWidth>,
}

// ColumnSet element
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct ColumnSet {
    pub columns: Vec<Column>,
}

// Column element
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub struct Column {
    pub width: ColumnWidth,
    pub items: Vec<CardElement>,
}

/// Represents an image element in an Adaptive Card.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// The URL of the image.
    pub url: String,
    /// The size of the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageSize>,
}

/// Represents a fact set element in an Adaptive Card.
/// A FactSet contains a collection of facts, which are key-value pairs that provide additional information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactSet {
    /// The collection of facts in the fact set.
    pub facts: Vec<Fact>,
}

/// Represents an individual fact in a FactSet element.
/// Each fact has a title and a value, which are displayed as a key-value pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fact {
    /// The title of the fact, typically displayed as the key.
    pub title: String,
    /// The value of the fact, typically displayed as the value associated with the key.
    pub value: String,
}

/// Represents the size of the text in a TextBlock element.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextSize {
    Small,
    Default,
    Medium,
    Large,
    ExtraLarge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextWeight {
    Lighter,
    Default,
    Bolder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContainerStyle {
    Default,
    Emphasis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Spacing {
    None,
    Small,
    Default,
    Medium,
    Large,
    ExtraLarge,
    Padding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColumnWidth {
    Auto(String),    // "auto"
    Stretch(String), // "stretch"
    Pixel(String),   // pixel value
    Relative(u32),   // relative width value
}

// Helper functions for ColumnWidth
impl ColumnWidth {
    pub fn auto() -> Self {
        Self::Auto("auto".to_string())
    }

    pub fn stretch() -> Self {
        Self::Stretch("stretch".to_string())
    }

    pub fn pixels(px: u32) -> Self {
        Self::Pixel(format!("{}px", px))
    }

    pub fn weight(w: u32) -> Self {
        Self::Relative(w)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImageSize {
    Auto,
    Stretch,
    Small,
    Medium,
    Large,
}

#[cfg(test)]
mod tests {
    use crate::actions::{Action, OpenUrlAction};

    use super::*;
    use expect_test::expect;
    use serde_json::{self, Value};

    #[test]
    fn test_adaptive_card_serialization() {
        let card = AdaptiveCard {
            version: Version::V1_3,
            body: vec![
                CardElement::TextBlock(TextBlock {
                    size: Some(TextSize::Large),
                    weight: Some(TextWeight::Bolder),
                    text: "Hello, Adaptive Card!".to_string(),
                    wrap: Some(true),
                    is_subtle: Some(false),
                }),
                CardElement::Image(Image {
                    url: "https://example.com/image.png".to_string(),
                    size: Some(ImageSize::Medium),
                }),
                CardElement::Container(Container {
                    style: Some(ContainerStyle::Emphasis),
                    spacing: Some(Spacing::Medium),
                    items: vec![CardElement::TextBlock(TextBlock {
                        size: Some(TextSize::Default),
                        weight: Some(TextWeight::Default),
                        text: "Inside a container".to_string(),
                        wrap: Some(true),
                        is_subtle: Some(true),
                    })],
                }),
                CardElement::ActionSet(ActionSet {
                    actions: vec![Action::OpenUrl(OpenUrlAction {
                        title: "Open".to_string(),
                        url: "https://www.youtube.com/watch?v=sBW8Vnp8BzU".to_string(),
                    })],
                }),
            ],
            ..Default::default()
        };

        let expected = expect![[r#"
            {
              "type": "AdaptiveCard",
              "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
              "version": "1.3",
              "body": [
                {
                  "type": "TextBlock",
                  "size": "large",
                  "weight": "bolder",
                  "text": "Hello, Adaptive Card!",
                  "wrap": true,
                  "isSubtle": false
                },
                {
                  "type": "Image",
                  "url": "https://example.com/image.png",
                  "size": "medium"
                },
                {
                  "type": "Container",
                  "style": "emphasis",
                  "spacing": "medium",
                  "items": [
                    {
                      "type": "TextBlock",
                      "size": "default",
                      "weight": "default",
                      "text": "Inside a container",
                      "wrap": true,
                      "isSubtle": true
                    }
                  ]
                },
                {
                  "type": "ActionSet",
                  "actions": [
                    {
                      "type": "Action.OpenUrl",
                      "title": "Open",
                      "url": "https://www.youtube.com/watch?v=sBW8Vnp8BzU"
                    }
                  ]
                }
              ]
            }"#]];

        expected.assert_eq(&serde_json::to_string_pretty(&card).unwrap());

        validate_card_against_schema(&card);
    }

    #[test]
    fn test_adaptive_card_microsift_teams_serialization() {
        let card = AdaptiveCard {
            version: Version::V1_6,
            msteams: Some(MsTeams {
                width: Some(MsTeamsWidth::Full),
            }),
            ..Default::default()
        };

        let expected = expect![[r#"
            {
              "type": "AdaptiveCard",
              "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
              "version": "1.6",
              "body": [],
              "msteams": {
                "width": "full"
              }
            }"#]];

        expected.assert_eq(&serde_json::to_string_pretty(&card).unwrap());

        // NOTE: https://github.com/microsoft/AdaptiveCards/issues/8603
        // validate_card_against_schema(&card);
    }

    #[test]
    fn test_column_width_serialization() {
        let card = AdaptiveCard {
            body: vec![CardElement::ColumnSet(ColumnSet {
                columns: vec![
                    Column {
                        width: ColumnWidth::auto(),
                        items: vec![CardElement::TextBlock(TextBlock {
                            size: Some(TextSize::Default),
                            weight: Some(TextWeight::Default),
                            text: "Auto width column".to_string(),
                            wrap: Some(true),
                            is_subtle: Some(false),
                        })],
                    },
                    Column {
                        width: ColumnWidth::stretch(),
                        items: vec![CardElement::TextBlock(TextBlock {
                            size: Some(TextSize::Default),
                            weight: Some(TextWeight::Default),
                            text: "Stretch width column".to_string(),
                            wrap: Some(true),
                            is_subtle: Some(false),
                        })],
                    },
                    Column {
                        width: ColumnWidth::pixels(200),
                        items: vec![CardElement::TextBlock(TextBlock {
                            size: Some(TextSize::Default),
                            weight: Some(TextWeight::Default),
                            text: "Pixel width column".to_string(),
                            wrap: Some(true),
                            is_subtle: Some(false),
                        })],
                    },
                    Column {
                        width: ColumnWidth::weight(2),
                        items: vec![CardElement::TextBlock(TextBlock {
                            size: Some(TextSize::Default),
                            weight: Some(TextWeight::Default),
                            text: "Weight width column".to_string(),
                            wrap: Some(true),
                            is_subtle: Some(false),
                        })],
                    },
                ],
            })],
            ..Default::default()
        };

        let expected = expect![[r#"
            {
              "type": "AdaptiveCard",
              "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
              "version": "1.2",
              "body": [
                {
                  "type": "ColumnSet",
                  "type": "ColumnSet",
                  "columns": [
                    {
                      "type": "Column",
                      "width": "auto",
                      "items": [
                        {
                          "type": "TextBlock",
                          "size": "default",
                          "weight": "default",
                          "text": "Auto width column",
                          "wrap": true,
                          "isSubtle": false
                        }
                      ]
                    },
                    {
                      "type": "Column",
                      "width": "stretch",
                      "items": [
                        {
                          "type": "TextBlock",
                          "size": "default",
                          "weight": "default",
                          "text": "Stretch width column",
                          "wrap": true,
                          "isSubtle": false
                        }
                      ]
                    },
                    {
                      "type": "Column",
                      "width": "200px",
                      "items": [
                        {
                          "type": "TextBlock",
                          "size": "default",
                          "weight": "default",
                          "text": "Pixel width column",
                          "wrap": true,
                          "isSubtle": false
                        }
                      ]
                    },
                    {
                      "type": "Column",
                      "width": 2,
                      "items": [
                        {
                          "type": "TextBlock",
                          "size": "default",
                          "weight": "default",
                          "text": "Weight width column",
                          "wrap": true,
                          "isSubtle": false
                        }
                      ]
                    }
                  ]
                }
              ]
            }"#]];

        expected.assert_eq(&serde_json::to_string_pretty(&card).unwrap());

        validate_card_against_schema(&card);
    }

    fn validate_card_against_schema(card: &AdaptiveCard) {
        lazy_static::lazy_static! {
            static ref SCHEMA_CONTENT: Value = serde_json::from_str(include_str!("../schema.json")).unwrap();
        }
        let validator = jsonschema::validator_for(&SCHEMA_CONTENT).unwrap();

        assert!(validator.is_valid(&serde_json::to_value(card).unwrap()));
    }
}
