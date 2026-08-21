//! Hand-written schema validation fixtures derived from Microsoft spec examples.

use adaptive_card_core::schema;

macro_rules! card {
    ($($t:tt)*) => { serde_json::json!($($t)*) };
}

#[test]
fn valid_minimal_card() {
    let c = card!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [{ "type": "TextBlock", "text": "Hi" }]
    });
    let errors = schema::validate(&c);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn valid_factset_card() {
    let c = card!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [{
            "type": "FactSet",
            "facts": [
                { "title": "Name:", "value": "Alice" },
                { "title": "Age:", "value": "30" }
            ]
        }]
    });
    let errors = schema::validate(&c);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn valid_columnset_card() {
    let c = card!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [{
            "type": "ColumnSet",
            "columns": [
                { "type": "Column", "width": "stretch", "items": [
                    { "type": "TextBlock", "text": "A" }
                ]},
                { "type": "Column", "width": "auto", "items": [
                    { "type": "TextBlock", "text": "B" }
                ]}
            ]
        }]
    });
    let errors = schema::validate(&c);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn valid_input_with_label() {
    let c = card!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [{
            "type": "Input.Text",
            "id": "name",
            "label": "Your name",
            "placeholder": "Enter name"
        }]
    });
    let errors = schema::validate(&c);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn valid_actions_openurl_and_submit() {
    let c = card!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [],
        "actions": [
            { "type": "Action.OpenUrl", "title": "Docs", "url": "https://adaptivecards.io" },
            { "type": "Action.Submit", "title": "OK" }
        ]
    });
    let errors = schema::validate(&c);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn invalid_unknown_root_property_fails() {
    // The AdaptiveCard root has `additionalProperties: false`, so an unknown
    // property is a schema violation. (Note: the v1.6 schema does NOT mark
    // `type` as required on the root — that's a quirk worth documenting.)
    let c = card!({
        "type": "AdaptiveCard",
        "version": "1.6",
        "body": [],
        "unknownRootProperty": "boom"
    });
    assert!(!schema::validate(&c).is_empty());
}

#[test]
fn invalid_wrong_type_value_fails() {
    let c = card!({ "type": "NotACard", "version": "1.6", "body": [] });
    assert!(!schema::validate(&c).is_empty());
}

#[test]
fn invalid_element_type_fails() {
    let c = card!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [{ "type": "UnknownElement", "text": "?" }]
    });
    assert!(!schema::validate(&c).is_empty());
}

#[test]
fn valid_image_with_alt() {
    let c = card!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [{
            "type": "Image",
            "url": "https://example.com/x.png",
            "altText": "Sample image"
        }]
    });
    let errors = schema::validate(&c);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}

#[test]
fn valid_container_nested() {
    let c = card!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [{
            "type": "Container",
            "style": "emphasis",
            "items": [
                { "type": "TextBlock", "text": "Header" },
                { "type": "Container", "items": [
                    { "type": "TextBlock", "text": "Nested" }
                ]}
            ]
        }]
    });
    let errors = schema::validate(&c);
    assert!(errors.is_empty(), "unexpected errors: {errors:?}");
}
