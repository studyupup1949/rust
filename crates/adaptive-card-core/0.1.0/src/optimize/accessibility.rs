//! Auto-fix accessibility issues: add speak, altText stubs, wrap, labels.

use serde_json::{Value, json};

/// Apply accessibility fixes in place.
/// Returns a list of fix descriptions.
pub fn fix(card: &mut Value) -> Vec<String> {
    let mut fixes: Vec<String> = Vec::new();

    // Add `speak` if missing
    if card
        .get("speak")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        let summary = extract_summary(card);
        card["speak"] = json!(summary);
        fixes.push("added root 'speak' property".to_string());
    }

    // Walk body
    if let Some(body) = card.get_mut("body").and_then(Value::as_array_mut) {
        for el in body.iter_mut() {
            fix_element(el, &mut fixes);
        }
    }

    fixes
}

fn fix_element(element: &mut Value, fixes: &mut Vec<String>) {
    let ty = element
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    match ty.as_str() {
        "Image" => {
            if element.get("altText").is_none() {
                element["altText"] = json!("image");
                fixes.push("Image: added placeholder altText".to_string());
            }
        }
        "TextBlock" => {
            let text_len = element
                .get("text")
                .and_then(Value::as_str)
                .map_or(0, str::len);
            if text_len > 60 && element.get("wrap").and_then(Value::as_bool) != Some(true) {
                element["wrap"] = json!(true);
                fixes.push("TextBlock: enabled wrap for long text".to_string());
            }
        }
        t if t.starts_with("Input.") => {
            if element.get("label").is_none() {
                let id = element.get("id").and_then(Value::as_str).unwrap_or("input");
                element["label"] = json!(format!("{id}:"));
                fixes.push(format!("{t}: added placeholder label"));
            }
        }
        _ => {}
    }

    // Recurse into known child containers
    for key in ["items", "columns"] {
        if let Some(children) = element.get_mut(key).and_then(Value::as_array_mut) {
            for child in children.iter_mut() {
                fix_element(child, fixes);
            }
        }
    }
}

fn extract_summary(card: &Value) -> String {
    // Collect text blocks' text as a summary (up to 200 chars).
    let mut parts: Vec<String> = Vec::new();
    if let Some(body) = card.get("body").and_then(Value::as_array) {
        collect_text(body, &mut parts);
    }
    let joined = parts.join(". ");
    if joined.len() > 200 {
        format!("{}...", &joined[..200])
    } else if joined.is_empty() {
        "Adaptive Card".to_string()
    } else {
        joined
    }
}

fn collect_text(elements: &[Value], out: &mut Vec<String>) {
    for el in elements {
        if el.get("type").and_then(Value::as_str) == Some("TextBlock")
            && let Some(t) = el.get("text").and_then(Value::as_str)
        {
            out.push(t.to_string());
        }
        for key in ["items", "columns"] {
            if let Some(children) = el.get(key).and_then(Value::as_array) {
                collect_text(children, out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_speak_derived_from_text_blocks() {
        let mut card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [
                { "type": "TextBlock", "text": "Hello" },
                { "type": "TextBlock", "text": "World" }
            ]
        });
        let fixes = fix(&mut card);
        assert_eq!(card["speak"], "Hello. World");
        assert!(fixes.iter().any(|f| f.contains("speak")));
    }

    #[test]
    fn adds_image_alt() {
        let mut card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{ "type": "Image", "url": "x.png" }]
        });
        let _ = fix(&mut card);
        assert!(card["body"][0]["altText"].is_string());
    }

    #[test]
    fn enables_wrap_for_long_text() {
        let long = "a".repeat(100);
        let mut card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{ "type": "TextBlock", "text": long }]
        });
        let _ = fix(&mut card);
        assert_eq!(card["body"][0]["wrap"], true);
    }
}
