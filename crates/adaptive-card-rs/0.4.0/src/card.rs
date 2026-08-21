use serde::{Deserialize, Serialize};

use crate::actions::{Action, ActionSet};
use crate::common::{Color, FontType, Height, HorizontalAlignment, VerticalContentAlignment};

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
    /// Microsoft Teams-specific properties for the Adaptive Card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msteams: Option<MsTeams>,
    /// The Actions to show in the card's action bar.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<Action>>,
    /// An Action that will be invoked when the card is tapped or selected. Action.ShowCard is not supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select_action: Option<Box<Action>>,
    /// Text shown when the client doesn't support the version specified (may contain markdown).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_text: Option<String>,
    /// Specifies the minimum height of the card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<String>,
    /// Defines how the content should be aligned vertically within the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_content_alignment: Option<VerticalContentAlignment>,
    /// When true, content in this Adaptive Card should be presented right to left.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rtl: Option<bool>,
    /// The 2-letter ISO-639-1 language used in the card. Used to localize any date/time functions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

impl Default for AdaptiveCard {
    fn default() -> Self {
        Self {
            schema: "http://adaptivecards.io/schemas/adaptive-card.json".to_string(),
            version: Version::V1_2,
            body: Vec::new(),
            msteams: None,
            actions: None,
            select_action: None,
            fallback_text: None,
            min_height: None,
            vertical_content_alignment: None,
            rtl: None,
            lang: None,
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
    /// A rich text block element that displays formatted text with inline elements.
    RichTextBlock(RichTextBlock),
    /// An input element that allows text entry.
    #[serde(rename = "Input.Text")]
    InputText(crate::inputs::InputText),
    /// An input element that allows number entry.
    #[serde(rename = "Input.Number")]
    InputNumber(crate::inputs::InputNumber),
    /// An input element that allows date selection.
    #[serde(rename = "Input.Date")]
    InputDate(crate::inputs::InputDate),
    /// An input element that allows time selection.
    #[serde(rename = "Input.Time")]
    InputTime(crate::inputs::InputTime),
    /// An input element that allows toggle/checkbox selection.
    #[serde(rename = "Input.Toggle")]
    InputToggle(crate::inputs::InputToggle),
    /// An input element that allows choice selection.
    #[serde(rename = "Input.ChoiceSet")]
    InputChoiceSet(crate::inputs::InputChoiceSet),
}

/// Represents a text block element in an Adaptive Card.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TextBlock {
    /// The text content to display.
    pub text: String,
    /// The size of the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<TextSize>,
    /// The weight of the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<TextWeight>,
    /// Whether the text should wrap if it exceeds the available space.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap: Option<bool>,
    /// Whether the text should be displayed subtly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_subtle: Option<bool>,
    /// Controls the color of the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    /// Controls the horizontal text alignment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_alignment: Option<HorizontalAlignment>,
    /// Specifies the maximum number of lines to display.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_lines: Option<u32>,
    /// Type of font to use for rendering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_type: Option<FontType>,
    /// A unique identifier associated with the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
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

/// Represents a container element that groups other card elements together.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    /// The card elements contained within this container.
    pub items: Vec<CardElement>,
    /// The style of the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ContainerStyle>,
    /// The spacing around the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<Spacing>,
    /// An Action that will be invoked when the container is tapped or selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select_action: Option<Box<Action>>,
    /// Defines how the content should be aligned vertically within the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_content_alignment: Option<VerticalContentAlignment>,
    /// Determines whether the element should bleed through its parent's padding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bleed: Option<bool>,
    /// Specifies the minimum height of the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<String>,
    /// A unique identifier associated with the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// When true, draw a separating line at the top of the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<bool>,
    /// Specifies the height of the element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<Height>,
    /// If false, this item will be removed from the visual tree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_visible: Option<bool>,
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// The URL of the image.
    pub url: String,
    /// The size of the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<ImageSize>,
    /// Alternative text describing the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    /// Applies a background to a transparent image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    /// Controls the horizontal alignment of the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_alignment: Option<HorizontalAlignment>,
    /// An Action that will be invoked when the image is clicked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub select_action: Option<Box<Action>>,
    /// The desired width of the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    /// A unique identifier associated with the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
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

/// Defines an array of inlines, allowing for inline text formatting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichTextBlock {
    /// The array of inline elements.
    pub inlines: Vec<Inline>,
    /// Controls the horizontal text alignment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_alignment: Option<HorizontalAlignment>,
    /// A unique identifier associated with the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
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

/// Inline element that can be either a string or a TextRun.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Inline {
    /// Plain text string.
    Text(String),
    /// Formatted text run with styling.
    TextRun(TextRun),
}

/// Represents a text run with inline formatting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRun {
    /// Must be "TextRun".
    #[serde(rename = "type")]
    pub type_field: String,
    /// The text to display.
    pub text: String,
    /// Controls the color of the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    /// Type of font to use for rendering.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_type: Option<FontType>,
    /// If true, displays the text highlighted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight: Option<bool>,
    /// If true, displays text slightly toned down to appear less prominent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_subtle: Option<bool>,
    /// If true, displays the text using italic font.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    /// Controls size of text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<TextSize>,
    /// If true, displays the text with strikethrough.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,
    /// Controls the weight of the text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<TextWeight>,
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
    Good,
    Attention,
    Warning,
    Accent,
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
pub enum ColumnWidthKind {
    Auto(String),    // "auto"
    Stretch(String), // "stretch"
    Pixel(String),   // pixel value
    Relative(u32),   // relative width value
}

/// Represents the width of a column, offering predefined constructors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColumnWidth {
    // The `kind` field is private, encapsulating the inner enum.
    kind: ColumnWidthKind,
}

// Helper functions for ColumnWidth
impl ColumnWidth {
    pub fn auto() -> Self {
        Self {
            kind: ColumnWidthKind::Auto("auto".to_string()),
        }
    }

    pub fn stretch() -> Self {
        Self {
            kind: ColumnWidthKind::Auto("stretch".to_string()),
        }
    }

    pub fn pixels(px: u32) -> Self {
        Self {
            kind: ColumnWidthKind::Auto(format!("{}px", px)),
        }
    }

    pub fn weight(w: u32) -> Self {
        Self {
            kind: ColumnWidthKind::Relative(w),
        }
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
                    ..Default::default()
                }),
                CardElement::Image(Image {
                    url: "https://example.com/image.png".to_string(),
                    size: Some(ImageSize::Medium),
                    ..Default::default()
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
                        ..Default::default()
                    })],
                    ..Default::default()
                }),
                CardElement::ActionSet(ActionSet {
                    actions: vec![Action::OpenUrl(OpenUrlAction {
                        title: Some("Open".to_string()),
                        url: "https://www.youtube.com/watch?v=sBW8Vnp8BzU".to_string(),
                        id: None,
                        icon_url: None,
                        style: None,
                        tooltip: None,
                        is_enabled: None,
                        mode: None,
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
                  "text": "Hello, Adaptive Card!",
                  "size": "large",
                  "weight": "bolder",
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
                  "items": [
                    {
                      "type": "TextBlock",
                      "text": "Inside a container",
                      "size": "default",
                      "weight": "default",
                      "wrap": true,
                      "isSubtle": true
                    }
                  ],
                  "style": "emphasis",
                  "spacing": "medium"
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
                            ..Default::default()
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
                            ..Default::default()
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
                            ..Default::default()
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
                            ..Default::default()
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
                          "text": "Auto width column",
                          "size": "default",
                          "weight": "default",
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
                          "text": "Stretch width column",
                          "size": "default",
                          "weight": "default",
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
                          "text": "Pixel width column",
                          "size": "default",
                          "weight": "default",
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
                          "text": "Weight width column",
                          "size": "default",
                          "weight": "default",
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
        use std::io::Read;
        use std::sync::OnceLock;

        static SCHEMA_CONTENT: OnceLock<Value> = OnceLock::new();

        let schema = SCHEMA_CONTENT.get_or_init(|| {
            let mut resp =
                reqwest::blocking::get("http://adaptivecards.io/schemas/adaptive-card.json")
                    .expect("Failed to fetch schema");
            let mut content = String::new();
            resp.read_to_string(&mut content)
                .expect("Failed to read schema response");
            serde_json::from_str(&content).expect("Failed to parse schema JSON")
        });
        let validator = jsonschema::validator_for(schema).unwrap();
        assert!(validator.is_valid(&serde_json::to_value(card).unwrap()));
    }
}
