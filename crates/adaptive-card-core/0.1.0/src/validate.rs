//! Top-level `validate_card` wrapper combining schema + a11y + host compat.

use crate::analyze::accessibility::check_accessibility;
use crate::host::check_compatibility;
use crate::schema;
use crate::types::{Host, ValidationReport};
use serde_json::Value;

/// Validate an Adaptive Card against schema v1.6, accessibility rules, and
/// (optionally) host compatibility.
#[must_use]
pub fn validate_card(card: &Value, host: Option<Host>) -> ValidationReport {
    let schema_errors = schema::validate(card);
    let accessibility = check_accessibility(card);
    let host_compat = host.map(|h| check_compatibility(card, h));

    let valid = schema_errors.is_empty() && host_compat.as_ref().is_none_or(|r| r.compatible);

    let card_version = card
        .get("version")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let mut suggestions: Vec<String> = Vec::new();
    for issue in &accessibility.issues {
        suggestions.push(format!("[a11y/{}] {}", issue.rule, issue.fix_hint));
    }
    if let Some(report) = host_compat.as_ref() {
        for el in &report.unsupported_elements {
            suggestions.push(format!(
                "[host/{:?}] Remove or replace unsupported element: {el}",
                report.host
            ));
        }
        for act in &report.unsupported_actions {
            suggestions.push(format!(
                "[host/{:?}] Remove or replace unsupported action: {act}",
                report.host
            ));
        }
    }

    ValidationReport {
        valid,
        schema_errors,
        accessibility,
        host_compat,
        card_version,
        suggestions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn valid_card_no_host() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{ "type": "TextBlock", "text": "Hi" }]
        });
        let report = validate_card(&card, None);
        assert!(report.valid);
        assert_eq!(report.card_version.as_deref(), Some("1.6"));
    }

    #[test]
    fn invalid_card_reports_errors() {
        // `version` must be a string per the Microsoft v1.6 schema; passing a
        // number triggers a schema violation.
        let card = json!({ "type": "AdaptiveCard", "version": 16, "body": [] });
        let report = validate_card(&card, None);
        assert!(!report.valid);
        assert!(!report.schema_errors.is_empty());
    }

    #[test]
    fn valid_card_for_teams() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "speak": "Greeting",
            "body": [{ "type": "TextBlock", "text": "Hi", "wrap": true }]
        });
        let report = validate_card(&card, Some(Host::Teams));
        assert!(report.valid);
        assert_eq!(report.accessibility.score, 100);
    }

    #[test]
    fn v1_6_card_invalid_for_outlook() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "speak": "x",
            "body": [{ "type": "TextBlock", "text": "Hi", "wrap": true }]
        });
        let report = validate_card(&card, Some(Host::Outlook));
        assert!(!report.valid);
        assert!(!report.host_compat.as_ref().unwrap().version_ok);
    }

    #[test]
    fn suggestions_include_a11y_and_host_hints() {
        let card = json!({
            "type": "AdaptiveCard", "version": "1.6",
            "body": [{ "type": "Image", "url": "x.png" }]
        });
        let report = validate_card(&card, Some(Host::Webex));
        assert!(report.suggestions.iter().any(|s| s.contains("[a11y/")));
    }
}
