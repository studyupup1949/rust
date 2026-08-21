//! Transform pipeline fixtures: version downgrade and host adaptation.

use adaptive_card_core::{CardVersion, Host, TransformTarget, transform_card};
use serde_json::json;

#[test]
fn downgrade_1_6_to_1_4_removes_codeblock() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [
            { "type": "TextBlock", "text": "Hi" },
            { "type": "CodeBlock", "codeSnippet": "x=1" }
        ]
    });
    let report = transform_card(
        card,
        &TransformTarget {
            version: Some(CardVersion::V1_4),
            host: None,
            strict: false,
        },
    )
    .unwrap();
    assert_eq!(report.card["version"], "1.4");
    assert_eq!(report.card["body"].as_array().unwrap().len(), 1);
}

#[test]
fn downgrade_1_5_to_1_3_removes_table() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.5",
        "body": [{ "type": "Table", "columns": [], "rows": [] }]
    });
    let report = transform_card(
        card,
        &TransformTarget {
            version: Some(CardVersion::V1_3),
            host: None,
            strict: false,
        },
    )
    .unwrap();
    assert_eq!(report.card["body"].as_array().unwrap().len(), 0);
}

#[test]
fn combined_host_plus_version_target() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [{ "type": "CodeBlock", "codeSnippet": "x" }],
        "actions": [{ "type": "Action.Execute", "title": "OK" }]
    });
    let report = transform_card(
        card,
        &TransformTarget {
            version: Some(CardVersion::V1_3),
            host: Some(Host::Webex),
            strict: false,
        },
    )
    .unwrap();
    assert_eq!(report.card["version"], "1.3");
    assert!(report.removed.iter().any(|s| s.contains("CodeBlock")));
}

#[test]
fn strict_errors_on_any_removal() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [{ "type": "CodeBlock", "codeSnippet": "x" }]
    });
    let result = transform_card(
        card,
        &TransformTarget {
            version: Some(CardVersion::V1_4),
            host: None,
            strict: true,
        },
    );
    assert!(result.is_err());
}

#[test]
fn noop_when_target_equals_current() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.4",
        "body": [{ "type": "TextBlock", "text": "Hi" }]
    });
    let report = transform_card(
        card.clone(),
        &TransformTarget {
            version: Some(CardVersion::V1_4),
            host: None,
            strict: false,
        },
    )
    .unwrap();
    assert_eq!(report.card["body"], card["body"]);
}
