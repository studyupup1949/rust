//! Structural analysis of Adaptive Cards: counting, nesting depth, duplicate
//! IDs, and other metrics derived without any host or schema context.

pub mod accessibility;

use crate::types::CardAnalysis;
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};

/// Compute structural metrics for a card.
#[must_use]
pub fn analyze_card(card: &Value) -> CardAnalysis {
    let mut state = WalkState::default();
    walk_container(card, 0, &mut state);

    let mut duplicate_ids: Vec<String> = state
        .id_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(id, _)| id)
        .collect();
    duplicate_ids.sort();

    let mut unique: Vec<String> = state.seen_types.into_iter().collect();
    unique.sort();

    CardAnalysis {
        element_count: state.element_count,
        action_count: state.action_count,
        nesting_depth: state.max_depth,
        unique_element_types: unique,
        duplicate_ids,
        total_text_length: state.total_text_length,
        has_images: state.has_images,
        has_inputs: state.has_inputs,
    }
}

/// Return the list of IDs that appear more than once anywhere in the card.
#[must_use]
pub fn find_duplicate_ids(card: &Value) -> Vec<String> {
    analyze_card(card).duplicate_ids
}

/// Count every element (not actions) in the card recursively.
#[must_use]
pub fn count_elements(card: &Value) -> usize {
    analyze_card(card).element_count
}

#[derive(Default)]
struct WalkState {
    element_count: usize,
    action_count: usize,
    max_depth: usize,
    seen_types: BTreeSet<String>,
    id_counts: HashMap<String, usize>,
    total_text_length: usize,
    has_images: bool,
    has_inputs: bool,
}

fn walk_container(value: &Value, depth: usize, state: &mut WalkState) {
    if depth > state.max_depth {
        state.max_depth = depth;
    }
    if let Some(body) = value.get("body").and_then(Value::as_array) {
        for child in body {
            walk_element(child, depth + 1, state);
        }
    }
    if let Some(actions) = value.get("actions").and_then(Value::as_array) {
        for action in actions {
            state.action_count += 1;
            if let Some(t) = action.get("type").and_then(Value::as_str) {
                state.seen_types.insert(t.to_string());
            }
            if let Some(id) = action.get("id").and_then(Value::as_str) {
                *state.id_counts.entry(id.to_string()).or_insert(0) += 1;
            }
        }
    }
}

fn walk_element(element: &Value, depth: usize, state: &mut WalkState) {
    if depth > state.max_depth {
        state.max_depth = depth;
    }
    state.element_count += 1;

    if let Some(t) = element.get("type").and_then(Value::as_str) {
        state.seen_types.insert(t.to_string());
        match t {
            "Image" | "ImageSet" => state.has_images = true,
            t if t.starts_with("Input.") => state.has_inputs = true,
            _ => {}
        }
    }
    if let Some(id) = element.get("id").and_then(Value::as_str) {
        *state.id_counts.entry(id.to_string()).or_insert(0) += 1;
    }
    if let Some(text) = element.get("text").and_then(Value::as_str) {
        state.total_text_length += text.len();
    }

    // Recurse into known child-bearing elements
    for key in [
        "items", "body", "columns", "cards", "actions", "choices", "facts",
    ] {
        if let Some(children) = element.get(key).and_then(Value::as_array) {
            for child in children {
                walk_element(child, depth + 1, state);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn analyze_minimal_card() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{ "type": "TextBlock", "text": "hello" }]
        });
        let a = analyze_card(&card);
        assert_eq!(a.element_count, 1);
        assert_eq!(a.action_count, 0);
        assert_eq!(a.total_text_length, 5);
        assert!(a.unique_element_types.contains(&"TextBlock".to_string()));
    }

    #[test]
    fn analyze_nested_container() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{
                "type": "Container",
                "items": [
                    { "type": "TextBlock", "text": "A" },
                    { "type": "Image", "url": "x.png" }
                ]
            }]
        });
        let a = analyze_card(&card);
        assert_eq!(a.element_count, 3);
        assert!(a.has_images);
        assert!(!a.has_inputs);
        assert!(a.nesting_depth >= 2);
    }

    #[test]
    fn find_duplicate_ids_reports_dupes() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [
                { "type": "TextBlock", "id": "t1", "text": "A" },
                { "type": "TextBlock", "id": "t1", "text": "B" },
                { "type": "TextBlock", "id": "t2", "text": "C" }
            ]
        });
        let dupes = find_duplicate_ids(&card);
        assert_eq!(dupes, vec!["t1".to_string()]);
    }

    #[test]
    fn count_actions() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [],
            "actions": [
                { "type": "Action.Submit", "title": "OK" },
                { "type": "Action.OpenUrl", "title": "Link", "url": "https://example.com" }
            ]
        });
        let a = analyze_card(&card);
        assert_eq!(a.action_count, 2);
    }

    #[test]
    fn input_detected() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{ "type": "Input.Text", "id": "name" }]
        });
        let a = analyze_card(&card);
        assert!(a.has_inputs);
    }
}
