//! Boolean schema — validates and describes bool-typed fields.
//!
//! Provides the [`BoolSchema`] structure for boolean validation constraints.

use super::SchemaValidator;
use crate::error::ValidationError;
use crate::validator::{RequiredValidator, StrictTypeValidator, ValidatorChain};
use crate::value::Value;

/// Schema representing boolean constraints and mapping metadata.
#[derive(Default)]
pub struct BoolSchema {
    default: Option<bool>,
    required: bool,
    optional: bool,
    strict: bool,
}

impl BoolSchema {
    /// Creates a new `BoolSchema` with no validation constraints.
    pub fn new() -> Self {
        Default::default()
    }

    /// Configures a fallback default value used when this field is missing.
    pub fn default(mut self, val: bool) -> Self {
        self.default = Some(val);
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

    /// Opts into strict validation mode: non-boolean inputs cause immediate failure instead of coercion.
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    fn build_chain(&self) -> ValidatorChain {
        let mut chain = ValidatorChain::new();
        if self.required {
            chain = chain.push_validator(RequiredValidator);
        }
        if self.strict {
            chain = chain.push_validator(StrictTypeValidator { expected: "bool" });
        }
        chain
    }
}

impl SchemaValidator for BoolSchema {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if self.optional && value.is_null() {
            return Ok(());
        }
        let effective = if !self.strict && !value.is_bool() && !value.is_null() {
            if let Some(b) = value.coerce_to_bool() {
                Value::Bool(b)
            } else {
                return Err(ValidationError::new(
                    field,
                    format!("cannot coerce {} to bool", value.type_name()),
                    "type_mismatch",
                ));
            }
        } else {
            value.clone()
        };
        self.build_chain()
            .validate(&effective, field)
            .map_err(|e| e.errors[0].clone())
    }

    fn is_required(&self) -> bool {
        self.required && self.default.is_none()
    }

    fn default_value(&self) -> Option<Value> {
        self.default.map(Value::Bool)
    }

    fn schema_type(&self) -> &'static str {
        "bool"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_valid() {
        let s = BoolSchema::new();
        assert!(s.validate(&Value::Bool(true), "flag").is_ok());
    }

    #[test]
    fn test_bool_strict_rejects_string() {
        let s = BoolSchema::new().strict();
        assert!(s.validate(&Value::String("true".into()), "flag").is_err());
    }

    #[test]
    fn test_bool_nonstrict_coerces_string() {
        let s = BoolSchema::new();
        assert!(s.validate(&Value::String("true".into()), "flag").is_ok());
    }

    #[test]
    fn test_bool_nonstrict_coerces_int() {
        let s = BoolSchema::new();
        assert!(s.validate(&Value::Int(1), "flag").is_ok());
    }

    #[test]
    fn test_bool_nonstrict_fails_invalid_string() {
        let s = BoolSchema::new();
        assert!(s.validate(&Value::String("maybe".into()), "flag").is_err());
    }

    #[test]
    fn test_bool_optional_null() {
        let s = BoolSchema::new().optional();
        assert!(s.validate(&Value::Null, "flag").is_ok());
    }

    #[test]
    fn test_bool_required_null_fails() {
        let s = BoolSchema::new().required();
        assert!(s.validate(&Value::Null, "flag").is_err());
    }

    #[test]
    fn test_bool_default() {
        let s = BoolSchema::new().default(false);
        assert_eq!(s.default_value(), Some(Value::Bool(false)));
    }
}
