//! Adapt a card to match a target host by removing unsupported elements,
//! downgrading actions, and trimming to action limits.

use crate::schema::paths::build_pointer;
use crate::types::{CardVersion, Host, TransformReport};
use serde_json::{Value, json};

/// Adapt a card in-place to fit a target host's capabilities.
/// Returns a [`TransformReport`] listing what was removed, downgraded, or warned.
#[must_use]
pub fn adapt_for_host(mut card: Value, host: Host) -> TransformReport {
    let mut removed: Vec<String> = Vec::new();
    let mut downgraded: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Downgrade version if the card is beyond the host's max
    if let Some(v) = card
        .get("version")
        .and_then(Value::as_str)
        .and_then(CardVersion::parse)
    {
        let max = host.max_version();
        if v > max {
            card["version"] = json!(max.as_str());
            downgraded.push(format!("version: {} -> {}", v.as_str(), max.as_str()));
        }
    }

    // Walk body and remove unsupported elements
    if let Some(body) = card.get_mut("body").and_then(Value::as_array_mut) {
        filter_elements(body, host, &["body"], &mut removed, &mut downgraded);
    }

    // Actions: downgrade Execute → Submit first, then remove unsupported, trim to max
    if let Some(actions) = card.get_mut("actions").and_then(Value::as_array_mut) {
        // Downgrade Action.Execute → Action.Submit when host lacks Execute
        if !host.supports_action("Action.Execute") {
            for action in actions.iter_mut() {
                if action.get("type").and_then(Value::as_str) == Some("Action.Execute") {
                    action["type"] = json!("Action.Submit");
                    downgraded.push("action: Action.Execute -> Action.Submit".to_string());
                }
            }
        }

        actions.retain(|a| {
            let ty = a.get("type").and_then(Value::as_str).unwrap_or("");
            let keep = host.supports_action(ty);
            if !keep {
                removed.push(format!("action: {ty}"));
            }
            keep
        });

        if let Some(max) = host.max_actions()
            && actions.len() > max
        {
            let excess = actions.len() - max;
            actions.truncate(max);
            warnings.push(format!(
                "Trimmed {excess} action(s) to fit host limit of {max}"
            ));
        }
    }

    TransformReport {
        card,
        removed,
        downgraded,
        warnings,
    }
}

#[allow(
    clippy::only_used_in_recursion,
    reason = "downgraded threaded through for future element-level downgrades"
)]
fn filter_elements(
    elements: &mut Vec<Value>,
    host: Host,
    path: &[&str],
    removed: &mut Vec<String>,
    downgraded: &mut Vec<String>,
) {
    let mut i = 0;
    while i < elements.len() {
        let ty = elements[i]
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if !ty.is_empty() && !host.supports_element(&ty) {
            let mut seg_owned: Vec<String> = path.iter().map(|s| (*s).to_string()).collect();
            seg_owned.push(i.to_string());
            let seg_refs: Vec<&str> = seg_owned.iter().map(String::as_str).collect();
            removed.push(format!("{}: {ty}", build_pointer(&seg_refs)));
            elements.remove(i);
            continue;
        }
        // Recurse into child containers
        let seg_owned: Vec<String> = {
            let mut v: Vec<String> = path.iter().map(|s| (*s).to_string()).collect();
            v.push(i.to_string());
            v
        };
        for key in ["items", "columns"] {
            if let Some(children) = elements[i].get_mut(key).and_then(Value::as_array_mut) {
                let mut child_seg: Vec<String> = seg_owned.clone();
                child_seg.push(key.to_string());
                let child_refs: Vec<&str> = child_seg.iter().map(String::as_str).collect();
                filter_elements(children, host, &child_refs, removed, downgraded);
            }
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn adapt_downgrades_version() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{ "type": "TextBlock", "text": "Hi" }]
        });
        let report = adapt_for_host(card, Host::Outlook);
        assert_eq!(report.card["version"], "1.4");
        assert!(report.downgraded.iter().any(|s| s.starts_with("version")));
    }

    #[test]
    fn adapt_removes_unsupported_element() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.3",
            "body": [
                { "type": "TextBlock", "text": "Hi" },
                { "type": "Table", "columns": [] }
            ]
        });
        let report = adapt_for_host(card, Host::Webex);
        assert_eq!(report.card["body"].as_array().unwrap().len(), 1);
        assert!(report.removed.iter().any(|s| s.contains("Table")));
    }

    #[test]
    fn adapt_downgrades_execute_to_submit() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.4",
            "body": [],
            "actions": [{ "type": "Action.Execute", "title": "OK" }]
        });
        let report = adapt_for_host(card, Host::Outlook);
        assert_eq!(report.card["actions"][0]["type"], "Action.Submit");
        assert!(
            report
                .downgraded
                .iter()
                .any(|s| s.contains("Action.Execute"))
        );
    }

    #[test]
    fn adapt_trims_actions_to_host_limit() {
        let actions: Vec<_> = (0..8)
            .map(|i| json!({ "type": "Action.Submit", "title": format!("A{i}") }))
            .collect();
        let card = json!({
            "type": "AdaptiveCard", "version": "1.4",
            "body": [],
            "actions": actions
        });
        let report = adapt_for_host(card, Host::Outlook);
        assert_eq!(report.card["actions"].as_array().unwrap().len(), 4);
        assert!(report.warnings.iter().any(|s| s.contains("Trimmed 4")));
    }
}
