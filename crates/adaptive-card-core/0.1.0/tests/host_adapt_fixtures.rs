//! Host adaptation fixtures for Teams, Outlook, Webex, `WebChat`, Viva.

use adaptive_card_core::{Host, adapt_for_host};
use serde_json::json;

#[test]
fn teams_keeps_v1_6_intact() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [
            { "type": "Table", "columns": [{"width": 1}], "rows": [] },
            { "type": "CodeBlock", "codeSnippet": "x = 1" }
        ]
    });
    let report = adapt_for_host(card, Host::Teams);
    assert_eq!(report.card["body"].as_array().unwrap().len(), 2);
    assert!(report.removed.is_empty());
}

#[test]
fn outlook_removes_media_and_downgrades() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [
            { "type": "TextBlock", "text": "Hi" },
            { "type": "Media", "sources": [] }
        ]
    });
    let report = adapt_for_host(card, Host::Outlook);
    assert_eq!(report.card["version"], "1.4");
    assert_eq!(report.card["body"].as_array().unwrap().len(), 1);
}

#[test]
fn webex_removes_table_and_caps_version() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [
            { "type": "TextBlock", "text": "Hi" },
            { "type": "Table", "columns": [], "rows": [] }
        ]
    });
    let report = adapt_for_host(card, Host::Webex);
    assert_eq!(report.card["version"], "1.3");
    assert_eq!(report.card["body"].as_array().unwrap().len(), 1);
}

#[test]
fn webchat_keeps_all_v1_6_features() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [
            { "type": "Table", "columns": [], "rows": [] },
            { "type": "CodeBlock", "codeSnippet": "console.log('hi')" }
        ]
    });
    let report = adapt_for_host(card, Host::WebChat);
    assert!(report.removed.is_empty());
}

#[test]
fn viva_downgrades_to_1_4() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [{ "type": "TextBlock", "text": "Hi" }]
    });
    let report = adapt_for_host(card, Host::VivaConnections);
    assert_eq!(report.card["version"], "1.4");
}

#[test]
fn execute_becomes_submit_for_outlook() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [],
        "actions": [{ "type": "Action.Execute", "title": "Save", "verb": "save" }]
    });
    let report = adapt_for_host(card, Host::Outlook);
    assert_eq!(report.card["actions"][0]["type"], "Action.Submit");
}

#[test]
fn trim_actions_below_outlook_limit() {
    let actions: Vec<_> = (0..7)
        .map(|i| json!({ "type": "Action.Submit", "title": format!("A{i}") }))
        .collect();
    let card = json!({
        "type": "AdaptiveCard", "version": "1.4",
        "body": [],
        "actions": actions
    });
    let report = adapt_for_host(card, Host::Outlook);
    assert_eq!(report.card["actions"].as_array().unwrap().len(), 4);
}

#[test]
fn nested_table_inside_container_removed_for_webex() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.5",
        "body": [{
            "type": "Container",
            "items": [
                { "type": "TextBlock", "text": "Hi" },
                { "type": "Table", "columns": [], "rows": [] }
            ]
        }]
    });
    let report = adapt_for_host(card, Host::Webex);
    let items = report.card["body"][0]["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
}
