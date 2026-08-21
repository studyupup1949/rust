//! Performance optimizations: flatten unnecessary containers, dedupe IDs.

use serde_json::Value;
use std::collections::HashMap;

/// Apply performance fixes in place. Returns a list of fix descriptions.
pub fn fix(card: &mut Value) -> Vec<String> {
    let mut fixes: Vec<String> = Vec::new();

    if let Some(body) = card.get_mut("body").and_then(Value::as_array_mut) {
        flatten(body, &mut fixes);
    }
    dedupe_ids(card, &mut fixes);
    fixes
}

fn flatten(elements: &mut [Value], fixes: &mut Vec<String>) {
    for el in elements.iter_mut() {
        // Recurse first
        if let Some(items) = el.get_mut("items").and_then(Value::as_array_mut) {
            flatten(items, fixes);
        }
        if let Some(cols) = el.get_mut("columns").and_then(Value::as_array_mut) {
            for col in cols.iter_mut() {
                if let Some(col_items) = col.get_mut("items").and_then(Value::as_array_mut) {
                    flatten(col_items, fixes);
                }
            }
        }

        // Flatten Container with single child and no special styling
        let is_container = el.get("type").and_then(Value::as_str) == Some("Container");
        let has_style = el.get("style").is_some()
            || el.get("bleed").is_some()
            || el.get("backgroundImage").is_some();
        if is_container && !has_style {
            let items_len = el
                .get("items")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            if items_len == 1 {
                let child = el["items"][0].clone();
                *el = child;
                fixes.push("flattened Container with single child".to_string());
            }
        }
    }
}

fn dedupe_ids(card: &mut Value, fixes: &mut Vec<String>) {
    let mut counts: HashMap<String, usize> = HashMap::new();
    count_ids(card, &mut counts);
    let duplicates: Vec<String> = counts
        .into_iter()
        .filter(|(_, c)| *c > 1)
        .map(|(id, _)| id)
        .collect();

    if duplicates.is_empty() {
        return;
    }

    let mut seen: HashMap<String, usize> = HashMap::new();
    rename_dupes(card, &duplicates, &mut seen, fixes);
}

fn count_ids(value: &Value, counts: &mut HashMap<String, usize>) {
    if let Some(id) = value.get("id").and_then(Value::as_str) {
        *counts.entry(id.to_string()).or_insert(0) += 1;
    }
    for key in ["body", "items", "columns", "actions", "cards", "choices"] {
        if let Some(children) = value.get(key).and_then(Value::as_array) {
            for c in children {
                count_ids(c, counts);
            }
        }
    }
}

fn rename_dupes(
    value: &mut Value,
    dupes: &[String],
    seen: &mut HashMap<String, usize>,
    fixes: &mut Vec<String>,
) {
    if let Some(id) = value
        .get("id")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        && dupes.contains(&id)
    {
        let n = seen.entry(id.clone()).or_insert(0);
        *n += 1;
        if *n > 1 {
            let new_id = format!("{id}-{n}");
            value["id"] = Value::String(new_id.clone());
            fixes.push(format!("renamed duplicate id: {id} -> {new_id}"));
        }
    }
    for key in ["body", "items", "columns", "actions", "cards", "choices"] {
        if let Some(children) = value.get_mut(key).and_then(Value::as_array_mut) {
            for c in children.iter_mut() {
                rename_dupes(c, dupes, seen, fixes);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn flattens_single_child_container() {
        let mut card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{
                "type": "Container",
                "items": [{ "type": "TextBlock", "text": "Hi" }]
            }]
        });
        let fixes = fix(&mut card);
        assert_eq!(card["body"][0]["type"], "TextBlock");
        assert!(!fixes.is_empty());
    }

    #[test]
    fn renames_duplicate_ids() {
        let mut card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [
                { "type": "TextBlock", "id": "t1", "text": "A" },
                { "type": "TextBlock", "id": "t1", "text": "B" }
            ]
        });
        let fixes = fix(&mut card);
        assert_ne!(card["body"][0]["id"], card["body"][1]["id"]);
        assert!(fixes.iter().any(|f| f.contains("t1")));
    }

    #[test]
    fn does_not_flatten_container_with_style() {
        let mut card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{
                "type": "Container",
                "style": "emphasis",
                "items": [{ "type": "TextBlock", "text": "Hi" }]
            }]
        });
        let _ = fix(&mut card);
        assert_eq!(card["body"][0]["type"], "Container");
    }
}
