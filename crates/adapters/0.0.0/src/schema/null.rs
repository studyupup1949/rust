//! Null schema — validates that a value is exactly `null`.
//!
//! Provides the [`NullSchema`] structure representing null-only constraints.

use super::SchemaValidator;
use crate::error::ValidationError;
use crate::value::Value;

/// Schema representing null-only validation constraints.
#[derive(Default)]
pub struct NullSchema;

impl NullSchema {
    /// Creates a new `NullSchema`.
    pub fn new() -> Self {
        Self
    }

    /// Configures the schema to strictly fail if the field is absent.
    pub fn required(self) -> Self {
        self
    }

    /// Registers the field as optional (permits `Null` values).
    pub fn optional(self) -> Self {
        self
    }
}

impl SchemaValidator for NullSchema {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if value.is_null() {
            Ok(())
        } else {
            Err(ValidationError::new(
                field,
                format!("expected null, got {}", value.type_name()),
                "type_mismatch",
            ))
        }
    }

    fn is_required(&self) -> bool {
        false
    }
    fn default_value(&self) -> Option<Value> {
        Some(Value::Null)
    }
    fn schema_type(&self) -> &'static str {
        "null"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_accepts_null() {
        assert!(NullSchema.validate(&Value::Null, "x").is_ok());
    }

    #[test]
    fn test_null_rejects_non_null() {
        assert!(NullSchema.validate(&Value::Int(1), "x").is_err());
    }
}
