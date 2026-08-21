//! Enum schema — validates that a value is one of a set of string variants.
//!
//! Provides the [`EnumSchema`] structure to represent string-restricted enumerations.

use super::SchemaValidator;
use crate::error::ValidationError;
use crate::value::Value;

/// Schema representing enumerated string variant constraints.
#[derive(Default)]
pub struct EnumSchema {
    variants: Vec<String>,
    required: bool,
    optional: bool,
}

impl EnumSchema {
    /// Creates a new `EnumSchema` with no structural variants configured.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a valid permissible string variant name.
    pub fn variant(mut self, name: &str) -> Self {
        self.variants.push(name.to_string());
        self
    }

    /// Configures the schema to strictly fail if the field is absent.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Registers the field as optional (permits `Null` values).
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

impl SchemaValidator for EnumSchema {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if self.optional && value.is_null() {
            return Ok(());
        }
        if self.required && value.is_null() {
            return Err(ValidationError::new(field, "field is required", "required"));
        }
        let s = match value.as_str() {
            Some(s) => s,
            None => {
                return Err(ValidationError::new(
                    field,
                    format!("expected string for enum, got {}", value.type_name()),
                    "type_mismatch",
                ));
            }
        };
        if !self.variants.is_empty() && !self.variants.iter().any(|v| v == s) {
            return Err(ValidationError::new(
                field,
                format!("'{}' is not one of: {}", s, self.variants.join(", ")),
                "enum_variant",
            ));
        }
        Ok(())
    }

    fn is_required(&self) -> bool {
        self.required
    }
    fn default_value(&self) -> Option<Value> {
        None
    }
    fn schema_type(&self) -> &'static str {
        "enum"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_valid() {
        let s = EnumSchema::new().variant("A").variant("B").variant("C");
        assert!(s.validate(&Value::String("B".into()), "status").is_ok());
    }

    #[test]
    fn test_enum_invalid_variant() {
        let s = EnumSchema::new().variant("A").variant("B");
        assert!(s.validate(&Value::String("Z".into()), "status").is_err());
    }

    #[test]
    fn test_enum_required_null_fails() {
        let s = EnumSchema::new().variant("A").required();
        assert!(s.validate(&Value::Null, "status").is_err());
    }

    #[test]
    fn test_enum_optional_null_passes() {
        let s = EnumSchema::new().variant("A").optional();
        assert!(s.validate(&Value::Null, "status").is_ok());
    }

    #[test]
    fn test_enum_wrong_type() {
        let s = EnumSchema::new().variant("A");
        assert!(s.validate(&Value::Int(1), "status").is_err());
    }
}
