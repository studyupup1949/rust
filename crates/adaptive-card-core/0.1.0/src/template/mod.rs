//! Convert a static Adaptive Card into a template with `${expression}` bindings
//! and extract the literal data into a separate sample payload.
//!
//! Heuristic: string fields that look like content ("text", "title", "url",
//! "altText", "placeholder") are replaced with `${camelCase(path_tail)}`.
//! Numeric/boolean literals in "value" fields are also extracted.

use crate::types::TemplateResult;
use serde_json::{Map, Value, json};

/// Convert a card's literal values into `${expression}` bindings.
#[must_use]
pub fn template_card(mut card: Value) -> TemplateResult {
    let mut bindings: Vec<String> = Vec::new();
    let mut sample_data: Map<String, Value> = Map::new();

    walk(&mut card, &mut bindings, &mut sample_data, &[]);

    TemplateResult {
        template: card,
        sample_data: Value::Object(sample_data),
        bindings,
    }
}

const BINDABLE_KEYS: &[&str] = &["text", "title", "url", "altText", "placeholder", "value"];

fn walk(
    value: &mut Value,
    bindings: &mut Vec<String>,
    data: &mut Map<String, Value>,
    path: &[String],
) {
    match value {
        Value::Object(map) => {
            // Snapshot keys to avoid borrow conflicts
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                let mut new_path = path.to_vec();
                new_path.push(key.clone());

                if BINDABLE_KEYS.contains(&key.as_str()) {
                    let v = map.get(&key).cloned().unwrap_or(Value::Null);
                    if let Some(literal) = extract_literal(&v) {
                        let binding_name = camel_case(&new_path);
                        if !data.contains_key(&binding_name) {
                            data.insert(binding_name.clone(), literal);
                            bindings.push(binding_name.clone());
                        }
                        map.insert(key, json!(format!("${{{binding_name}}}")));
                        continue;
                    }
                }

                if let Some(child) = map.get_mut(&key) {
                    walk(child, bindings, data, &new_path);
                }
            }
        }
        Value::Array(arr) => {
            for (i, item) in arr.iter_mut().enumerate() {
                let mut new_path = path.to_vec();
                new_path.push(i.to_string());
                walk(item, bindings, data, &new_path);
            }
        }
        _ => {}
    }
}

fn extract_literal(v: &Value) -> Option<Value> {
    match v {
        Value::String(s) if !s.starts_with("${") => Some(Value::String(s.clone())),
        Value::Number(_) | Value::Bool(_) => Some(v.clone()),
        _ => None,
    }
}

fn camel_case(segments: &[String]) -> String {
    // Use last two path segments, skip numeric indices, join camelCase.
    let filtered: Vec<&str> = segments
        .iter()
        .rev()
        .take(3)
        .rev()
        .filter(|s| s.parse::<usize>().is_err())
        .map(String::as_str)
        .collect();
    if filtered.is_empty() {
        return "value".to_string();
    }
    let mut out = String::new();
    for (i, seg) in filtered.iter().enumerate() {
        if i == 0 {
            out.push_str(seg);
        } else {
            let mut chars = seg.chars();
            if let Some(c) = chars.next() {
                out.push(c.to_ascii_uppercase());
                out.extend(chars);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn basic_text_becomes_binding() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{ "type": "TextBlock", "text": "Hello" }]
        });
        let result = template_card(card);
        let text = result.template["body"][0]["text"].as_str().unwrap();
        assert!(text.starts_with("${"));
        assert!(!result.sample_data.as_object().unwrap().is_empty());
        assert!(
            result
                .bindings
                .contains(&text[2..text.len() - 1].to_string())
        );
    }

    #[test]
    fn title_and_url_both_bound() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [],
            "actions": [{
                "type": "Action.OpenUrl",
                "title": "Go",
                "url": "https://example.com"
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
    fn existing_bindings_preserved() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{ "type": "TextBlock", "text": "${alreadyBound}" }]
        });
        let result = template_card(card);
        assert_eq!(result.template["body"][0]["text"], "${alreadyBound}");
        assert!(result.bindings.is_empty());
    }
}
