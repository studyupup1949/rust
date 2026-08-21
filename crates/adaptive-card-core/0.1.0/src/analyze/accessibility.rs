//! Accessibility checker: scores a card 0-100 and emits a list of issues with
//! fix hints. Derived from WCAG 2.1 guidance + Microsoft Adaptive Cards
//! accessibility documentation.

use crate::schema::paths::build_pointer;
use crate::types::{A11yIssue, A11ySeverity, AccessibilityReport};
use serde_json::Value;

const RULE_MISSING_SPEAK: &str = "missing-speak";
const RULE_IMAGE_MISSING_ALT: &str = "image-missing-altText";
const RULE_TEXT_NO_WRAP: &str = "text-missing-wrap";
const RULE_INPUT_MISSING_LABEL: &str = "input-missing-label";

/// Check a card for accessibility issues and return a scored report.
#[must_use]
pub fn check_accessibility(card: &Value) -> AccessibilityReport {
    let mut issues: Vec<A11yIssue> = Vec::new();
    let mut passes: Vec<String> = Vec::new();

    // Rule: AdaptiveCard should have a `speak` property describing the card.
    if card
        .get("speak")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        issues.push(A11yIssue {
            severity: A11ySeverity::Warning,
            rule: RULE_MISSING_SPEAK.to_string(),
            path: String::new(),
            message: "Card has no 'speak' property — screen readers rely on this".to_string(),
            fix_hint: "Add a 'speak' field at the root summarising the card content".to_string(),
        });
    } else {
        passes.push(RULE_MISSING_SPEAK.to_string());
    }

    // Walk body and report per-element issues.
    if let Some(body) = card.get("body").and_then(Value::as_array) {
        for (i, el) in body.iter().enumerate() {
            let seg = i.to_string();
            check_element(el, &["body", &seg], &mut issues, &mut passes);
        }
    }

    let score = compute_score(&issues);

    AccessibilityReport {
        score,
        issues,
        passes,
    }
}

fn check_element(
    element: &Value,
    path: &[&str],
    issues: &mut Vec<A11yIssue>,
    passes: &mut Vec<String>,
) {
    let ty = element.get("type").and_then(Value::as_str).unwrap_or("");

    match ty {
        "Image" => {
            let alt = element.get("altText").and_then(Value::as_str);
            if alt.is_none_or(str::is_empty) {
                issues.push(A11yIssue {
                    severity: A11ySeverity::Error,
                    rule: RULE_IMAGE_MISSING_ALT.to_string(),
                    path: build_pointer(path),
                    message: "Image is missing altText".to_string(),
                    fix_hint: "Add `altText` describing the image".to_string(),
                });
            } else {
                passes.push(RULE_IMAGE_MISSING_ALT.to_string());
            }
        }
        "TextBlock" => {
            let text_len = element
                .get("text")
                .and_then(Value::as_str)
                .map_or(0, str::len);
            let wrap = element
                .get("wrap")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if text_len > 60 && !wrap {
                issues.push(A11yIssue {
                    severity: A11ySeverity::Warning,
                    rule: RULE_TEXT_NO_WRAP.to_string(),
                    path: build_pointer(path),
                    message: "Long TextBlock should have wrap=true".to_string(),
                    fix_hint: "Set `wrap: true` to prevent text truncation".to_string(),
                });
            }
        }
        t if t.starts_with("Input.") => {
            let label = element.get("label").and_then(Value::as_str);
            if label.is_none_or(str::is_empty) {
                issues.push(A11yIssue {
                    severity: A11ySeverity::Error,
                    rule: RULE_INPUT_MISSING_LABEL.to_string(),
                    path: build_pointer(path),
                    message: format!("{t} is missing a label"),
                    fix_hint: "Add `label` describing the input's purpose".to_string(),
                });
            }
        }
        "ColumnSet" => {
            // Recurse into columns
            if let Some(columns) = element.get("columns").and_then(Value::as_array) {
                for (i, col) in columns.iter().enumerate() {
                    let mut new_path = path.to_vec();
                    let seg = format!("columns/{i}");
                    new_path.push(&seg);
                    check_container(col, &new_path, issues, passes);
                }
            }
        }
        "Container" => {
            check_container(element, path, issues, passes);
        }
        _ => {}
    }
}

fn check_container(
    container: &Value,
    path: &[&str],
    issues: &mut Vec<A11yIssue>,
    passes: &mut Vec<String>,
) {
    if let Some(items) = container.get("items").and_then(Value::as_array) {
        for (i, child) in items.iter().enumerate() {
            let mut new_path = path.to_vec();
            let seg = format!("items/{i}");
            new_path.push(&seg);
            check_element(child, &new_path, issues, passes);
        }
    }
}

fn compute_score(issues: &[A11yIssue]) -> u8 {
    let deduction: i32 = issues
        .iter()
        .map(|i| match i.severity {
            A11ySeverity::Error => 15,
            A11ySeverity::Warning => 5,
            A11ySeverity::Info => 1,
        })
        .sum();
    u8::try_from((100_i32 - deduction).clamp(0, 100)).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn perfect_card_scores_100() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "speak": "Hello world card with a text block",
            "body": [
                { "type": "TextBlock", "text": "Hi", "wrap": true }
            ]
        });
        let report = check_accessibility(&card);
        assert_eq!(report.score, 100);
        assert!(report.issues.is_empty());
    }

    #[test]
    fn missing_speak_warns() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{ "type": "TextBlock", "text": "Hi", "wrap": true }]
        });
        let report = check_accessibility(&card);
        assert!(report.issues.iter().any(|i| i.rule == RULE_MISSING_SPEAK));
        assert_eq!(report.score, 95);
    }

    #[test]
    fn image_without_alt_errors() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "speak": "photo",
            "body": [{ "type": "Image", "url": "x.png" }]
        });
        let report = check_accessibility(&card);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.rule == RULE_IMAGE_MISSING_ALT)
        );
        assert_eq!(report.score, 85);
    }

    #[test]
    fn long_text_without_wrap_warns() {
        let long = "a".repeat(100);
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "speak": "long text",
            "body": [{ "type": "TextBlock", "text": long }]
        });
        let report = check_accessibility(&card);
        assert!(report.issues.iter().any(|i| i.rule == RULE_TEXT_NO_WRAP));
    }

    #[test]
    fn input_without_label_errors() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "speak": "form",
            "body": [{ "type": "Input.Text", "id": "name" }]
        });
        let report = check_accessibility(&card);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.rule == RULE_INPUT_MISSING_LABEL)
        );
    }

    #[test]
    fn multiple_issues_cap_at_zero() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [
                { "type": "Image", "url": "a.png" },
                { "type": "Image", "url": "b.png" },
                { "type": "Image", "url": "c.png" },
                { "type": "Image", "url": "d.png" },
                { "type": "Image", "url": "e.png" },
                { "type": "Image", "url": "f.png" },
                { "type": "Image", "url": "g.png" }
            ]
        });
        let report = check_accessibility(&card);
        assert!(report.score <= 10);
    }
}
