//! Optimize pipeline fixtures: accessibility, performance, modernize.

use adaptive_card_core::{Host, OptimizeOpts, optimize_card};
use serde_json::json;

#[test]
fn a11y_fixes_image_alt_and_speak() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [
            { "type": "Image", "url": "x.png" },
            { "type": "TextBlock", "text": "A long piece of text that is definitely more than sixty characters long here" }
        ]
    });
    let report = optimize_card(
        card,
        &OptimizeOpts {
            accessibility: true,
            ..Default::default()
        },
    );
    assert!(report.card["speak"].is_string());
    assert!(report.card["body"][0]["altText"].is_string());
    assert_eq!(report.card["body"][1]["wrap"], true);
}

#[test]
fn performance_flattens_single_child() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [{
            "type": "Container",
            "items": [{ "type": "TextBlock", "text": "A" }]
        }]
    });
    let report = optimize_card(
        card,
        &OptimizeOpts {
            performance: true,
            ..Default::default()
        },
    );
    assert_eq!(report.card["body"][0]["type"], "TextBlock");
}

#[test]
fn modernize_execute_for_teams() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.4",
        "body": [],
        "actions": [{ "type": "Action.Submit", "title": "OK", "data": { "verb": "ok" } }]
    });
    let report = optimize_card(
        card,
        &OptimizeOpts {
            modernize: true,
            target_host: Some(Host::Teams),
            ..Default::default()
        },
    );
    assert_eq!(report.card["actions"][0]["type"], "Action.Execute");
}

#[test]
fn all_optimizations_combined() {
    let card = json!({
        "type": "AdaptiveCard", "version": "1.6",
        "body": [
            { "type": "Container", "items": [
                { "type": "Image", "url": "x.png" }
            ]}
        ]
    });
    let report = optimize_card(
        card,
        &OptimizeOpts {
            accessibility: true,
            performance: true,
            modernize: true,
            target_host: Some(Host::Teams),
        },
    );
    assert!(report.card["speak"].is_string());
}
