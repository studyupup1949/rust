//! Version downgrade rules for Adaptive Cards.
//!
//! Rules derived from Microsoft schema changelog:
//! <https://adaptivecards.microsoft.com/?topic=SchemaVersions>
//!
//! Downgrade strategy: remove or replace features that did not exist in the
//! target version, preferring preservation via substitution over deletion.

use crate::types::CardVersion;
use serde_json::{Value, json};

/// Downgrade a card in place from its current version to `target`.
/// Returns `(removed, downgraded)` lists describing changes made.
pub fn downgrade_to(card: &mut Value, target: CardVersion) -> (Vec<String>, Vec<String>) {
    let mut removed: Vec<String> = Vec::new();
    let mut downgraded: Vec<String> = Vec::new();

    card["version"] = json!(target.as_str());
    downgraded.push(format!("version -> {}", target.as_str()));

    if target < CardVersion::V1_6 {
        // CodeBlock removed in pre-1.6
        if let Some(body) = card.get_mut("body").and_then(Value::as_array_mut) {
            remove_type(body, "CodeBlock", &mut removed, "body");
        }
    }
    if target < CardVersion::V1_5 {
        // Table removed in pre-1.5
        if let Some(body) = card.get_mut("body").and_then(Value::as_array_mut) {
            remove_type(body, "Table", &mut removed, "body");
        }
    }
    if target < CardVersion::V1_4 {
        // Action.Execute → Action.Submit
        if let Some(actions) = card.get_mut("actions").and_then(Value::as_array_mut) {
            for action in actions.iter_mut() {
                if action.get("type").and_then(Value::as_str) == Some("Action.Execute") {
                    action["type"] = json!("Action.Submit");
                    downgraded.push("Execute -> Submit".to_string());
                }
            }
        }
    }
    if target < CardVersion::V1_2 {
        // RichTextBlock removed pre-1.2
        if let Some(body) = card.get_mut("body").and_then(Value::as_array_mut) {
            remove_type(body, "RichTextBlock", &mut removed, "body");
        }
    }
    if target < CardVersion::V1_1 {
        // Media removed pre-1.1
        if let Some(body) = card.get_mut("body").and_then(Value::as_array_mut) {
            remove_type(body, "Media", &mut removed, "body");
        }
    }

    (removed, downgraded)
}

fn remove_type(elements: &mut Vec<Value>, type_name: &str, removed: &mut Vec<String>, base: &str) {
    let mut i = 0;
    while i < elements.len() {
        let ty = elements[i]
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("");
        if ty == type_name {
            removed.push(format!("{base}/{i}: {type_name}"));
            elements.remove(i);
            continue;
        }
        for key in ["items", "columns"] {
            if let Some(children) = elements[i].get_mut(key).and_then(Value::as_array_mut) {
                remove_type(children, type_name, removed, &format!("{base}/{i}/{key}"));
            }
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downgrades_version_string() {
        let mut card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": []
        });
        let (_, d) = downgrade_to(&mut card, CardVersion::V1_4);
        assert_eq!(card["version"], "1.4");
        assert!(d.iter().any(|s| s.contains("1.4")));
    }

    #[test]
    fn removes_codeblock_on_downgrade_to_v1_5() {
        let mut card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [
                { "type": "TextBlock", "text": "Hi" },
                { "type": "CodeBlock", "codeSnippet": "x = 1" }
            ]
        });
        let (removed, _) = downgrade_to(&mut card, CardVersion::V1_5);
        assert_eq!(card["body"].as_array().unwrap().len(), 1);
        assert!(removed.iter().any(|s| s.contains("CodeBlock")));
    }

    #[test]
    fn execute_becomes_submit_below_v1_4() {
        let mut card = json!({
            "type": "AdaptiveCard", "version": "1.4",
            "body": [],
            "actions": [{ "type": "Action.Execute", "title": "OK" }]
        });
        let (_, d) = downgrade_to(&mut card, CardVersion::V1_3);
        assert_eq!(card["actions"][0]["type"], "Action.Submit");
        assert!(d.iter().any(|s| s.contains("Execute -> Submit")));
    }
}
