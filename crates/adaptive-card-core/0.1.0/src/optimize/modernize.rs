//! Modernization: upgrade older action patterns to v1.4+ Action.Execute for
//! hosts that support it. Safe no-op for hosts that don't.

use crate::types::Host;
use serde_json::{Value, json};

pub fn fix(card: &mut Value, target_host: Option<Host>) -> Vec<String> {
    let mut fixes: Vec<String> = Vec::new();

    // Only modernize for hosts that support Action.Execute
    let supports_execute = target_host.is_none_or(|h| h.supports_action("Action.Execute"));
    if !supports_execute {
        return fixes;
    }

    if let Some(actions) = card.get_mut("actions").and_then(Value::as_array_mut) {
        for action in actions.iter_mut() {
            if action.get("type").and_then(Value::as_str) == Some("Action.Submit")
                && action.get("data").is_some()
            {
                action["type"] = json!("Action.Execute");
                fixes.push("Action.Submit -> Action.Execute".to_string());
            }
        }
    }
    fixes
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn submit_with_data_becomes_execute_for_teams() {
        let mut card = json!({
            "type": "AdaptiveCard", "version": "1.4",
            "body": [],
            "actions": [{
                "type": "Action.Submit",
                "title": "Save",
                "data": { "verb": "save" }
            }]
        });
        let fixes = fix(&mut card, Some(Host::Teams));
        assert_eq!(card["actions"][0]["type"], "Action.Execute");
        assert_eq!(fixes.len(), 1);
    }

    #[test]
    fn no_modernize_for_outlook() {
        let mut card = json!({
            "type": "AdaptiveCard", "version": "1.4",
            "body": [],
            "actions": [{
                "type": "Action.Submit",
                "data": { "verb": "save" }
            }]
        });
        let fixes = fix(&mut card, Some(Host::Outlook));
        assert_eq!(card["actions"][0]["type"], "Action.Submit");
        assert!(fixes.is_empty());
    }
}
