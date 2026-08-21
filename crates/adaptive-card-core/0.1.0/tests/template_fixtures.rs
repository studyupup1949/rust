//! Template conversion fixtures: literal values become `${binding}` expressions.

use adaptive_card_core::template_card;
use serde_json::json;

#[test]
fn literal_text_becomes_binding() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [{ "type": "TextBlock", "text": "Welcome Alice" }]
    });
    let result = template_card(card);
    assert!(
        result.template["body"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("${")
    );
    assert!(!result.bindings.is_empty());
}

#[test]
fn openurl_action_binds_url_and_title() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [],
        "actions": [{
            "type": "Action.OpenUrl",
            "title": "Docs",
            "url": "https://adaptivecards.io"
        }]
    });
    let result = template_card(card);
    assert!(
        result.template["actions"][0]["title"]
            .as_str()
            .unwrap()
            .starts_with("${")
    );
    assert!(
        result.template["actions"][0]["url"]
            .as_str()
            .unwrap()
            .starts_with("${")
    );
}

#[test]
fn nested_container_text_is_bound() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [{
            "type": "Container",
            "items": [{ "type": "TextBlock", "text": "Inside" }]
        }]
    });
    let result = template_card(card);
    let text = result.template["body"][0]["items"][0]["text"]
        .as_str()
        .unwrap();
    assert!(text.starts_with("${"));
}

#[test]
fn image_alt_and_url_bound() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [{ "type": "Image", "url": "a.png", "altText": "A" }]
    });
    let result = template_card(card);
    assert!(
        result.template["body"][0]["url"]
            .as_str()
            .unwrap()
            .starts_with("${")
    );
    assert!(
        result.template["body"][0]["altText"]
            .as_str()
            .unwrap()
            .starts_with("${")
    );
}
