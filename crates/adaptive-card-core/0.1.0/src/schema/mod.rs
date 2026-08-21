//! Adaptive Cards v1.6 JSON Schema loading and validation.

pub mod paths;

use crate::types::SchemaError;
use serde_json::Value;
use std::sync::LazyLock;

/// Embedded v1.6 schema JSON (sourced from Microsoft `AdaptiveCards` under MIT).
const V1_6_SCHEMA_SOURCE: &str = include_str!("../../data/adaptive-card-v1.6.schema.json");

static V1_6_SCHEMA: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(V1_6_SCHEMA_SOURCE).expect("embedded v1.6 schema must be valid JSON")
});

static V1_6_VALIDATOR: LazyLock<jsonschema::Validator> = LazyLock::new(|| {
    jsonschema::options()
        .build(&V1_6_SCHEMA)
        .expect("embedded v1.6 schema must compile as jsonschema")
});

/// Return the embedded v1.6 schema as a borrowed `serde_json::Value`.
#[must_use]
pub fn v1_6_schema() -> &'static Value {
    &V1_6_SCHEMA
}

/// Validate a card against the v1.6 schema.
/// Returns a list of [`SchemaError`] — empty if the card is valid.
#[must_use]
pub fn validate(card: &Value) -> Vec<SchemaError> {
    V1_6_VALIDATOR
        .iter_errors(card)
        .map(|e| SchemaError {
            path: e.instance_path().to_string(),
            message: e.to_string(),
            keyword: e.kind().keyword().to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn embedded_schema_loads() {
        let schema = v1_6_schema();
        assert!(schema.is_object());
    }

    #[test]
    fn valid_minimal_card_passes() {
        let card = json!({
            "type": "AdaptiveCard",
            "version": "1.6",
            "body": [
                { "type": "TextBlock", "text": "Hello" }
            ]
        });
        let errors = validate(&card);
        assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    }

    #[test]
    fn wrong_version_type_produces_error() {
        // Microsoft's v1.6 schema declares `version` as `type: "string"`,
        // so a numeric version should fail validation.
        let card = json!({
            "type": "AdaptiveCard",
            "version": 16,
            "body": []
        });
        let errors = validate(&card);
        assert!(!errors.is_empty());
    }

    #[test]
    fn wrong_type_value_produces_error() {
        let card = json!({
            "type": "NotACard",
            "version": "1.6",
            "body": []
        });
        let errors = validate(&card);
        assert!(!errors.is_empty());
    }
}
