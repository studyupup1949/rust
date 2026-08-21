//! Host compatibility checking and adaptation.

pub mod adapt;
pub mod matrix;

use crate::types::{CardVersion, Host, HostCompatReport};
use serde_json::Value;
use std::collections::BTreeSet;

/// Check whether a card is compatible with a target host.
/// Walks the card tree collecting unsupported elements/actions and version issues.
#[must_use]
pub fn check_compatibility(card: &Value, host: Host) -> HostCompatReport {
    let mut unsupported_elements: BTreeSet<String> = BTreeSet::new();
    let mut unsupported_actions: BTreeSet<String> = BTreeSet::new();
    let mut notes: Vec<String> = Vec::new();

    // Version check
    let card_version = card
        .get("version")
        .and_then(Value::as_str)
        .and_then(CardVersion::parse);
    let version_ok = if let Some(v) = card_version {
        v <= host.max_version()
    } else {
        notes.push("Card missing 'version' field".to_string());
        false
    };
    if !version_ok && let Some(v) = card_version {
        notes.push(format!(
            "Card version {} exceeds host max {}",
            v.as_str(),
            host.max_version().as_str()
        ));
    }

    // Walk elements
    walk_collect(
        card,
        host,
        &mut unsupported_elements,
        &mut unsupported_actions,
    );

    // Action count
    let action_count = card
        .get("actions")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let too_many_actions = match host.max_actions() {
        Some(max) if action_count > max => Some((action_count, max)),
        _ => None,
    };

    let compatible = version_ok
        && unsupported_elements.is_empty()
        && unsupported_actions.is_empty()
        && too_many_actions.is_none();

    HostCompatReport {
        host,
        compatible,
        version_ok,
        unsupported_elements: unsupported_elements.into_iter().collect(),
        unsupported_actions: unsupported_actions.into_iter().collect(),
        too_many_actions,
        notes,
    }
}

fn walk_collect(
    value: &Value,
    host: Host,
    unsupported_elements: &mut BTreeSet<String>,
    unsupported_actions: &mut BTreeSet<String>,
) {
    // Body elements
    if let Some(body) = value.get("body").and_then(Value::as_array) {
        for el in body {
            walk_element(el, host, unsupported_elements, unsupported_actions);
        }
    }
    // Actions
    if let Some(actions) = value.get("actions").and_then(Value::as_array) {
        for action in actions {
            if let Some(t) = action.get("type").and_then(Value::as_str)
                && !host.supports_action(t)
            {
                unsupported_actions.insert(t.to_string());
            }
        }
    }
}

#[allow(
    clippy::only_used_in_recursion,
    reason = "unsupported_actions is threaded through for future action-in-element walks"
)]
fn walk_element(
    element: &Value,
    host: Host,
    unsupported_elements: &mut BTreeSet<String>,
    unsupported_actions: &mut BTreeSet<String>,
) {
    if let Some(t) = element.get("type").and_then(Value::as_str)
        && !host.supports_element(t)
    {
        unsupported_elements.insert(t.to_string());
    }
    for key in ["items", "columns", "cards", "actions"] {
        if let Some(children) = element.get(key).and_then(Value::as_array) {
            for child in children {
                walk_element(child, host, unsupported_elements, unsupported_actions);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn teams_compatible_v1_6_card() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{ "type": "TextBlock", "text": "Hi", "wrap": true }]
        });
        let report = check_compatibility(&card, Host::Teams);
        assert!(report.compatible);
        assert!(report.unsupported_elements.is_empty());
    }

    #[test]
    fn outlook_rejects_v1_6_card() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": []
        });
        let report = check_compatibility(&card, Host::Outlook);
        assert!(!report.compatible);
        assert!(!report.version_ok);
    }

    #[test]
    fn webex_rejects_table() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.3",
            "body": [{ "type": "Table", "columns": [] }]
        });
        let report = check_compatibility(&card, Host::Webex);
        assert!(report.unsupported_elements.contains(&"Table".to_string()));
    }

    #[test]
    fn outlook_rejects_execute_action() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.4",
            "body": [],
            "actions": [{ "type": "Action.Execute", "title": "OK" }]
        });
        let report = check_compatibility(&card, Host::Outlook);
        assert!(
            report
                .unsupported_actions
                .contains(&"Action.Execute".to_string())
        );
    }

    #[test]
    fn outlook_rejects_too_many_actions() {
        let actions: Vec<_> = (0..6)
            .map(|i| json!({ "type": "Action.Submit", "title": format!("A{i}") }))
            .collect();
        let card = json!({
            "type": "AdaptiveCard", "version": "1.4",
            "body": [],
            "actions": actions
        });
        let report = check_compatibility(&card, Host::Outlook);
        assert_eq!(report.too_many_actions, Some((6, 4)));
    }
}
