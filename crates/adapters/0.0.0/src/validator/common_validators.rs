//! Common validation building blocks.
//!
//! Provides the primary [`ValidatorFn`] trait along with standard structural
//! validators like required, optional, strict type checks, and custom validators.

use crate::error::ValidationError;
use crate::value::Value;

/// A single reusable validation rule applied to dynamic values.
pub trait ValidatorFn: Send + Sync {
    /// Validates the given [`Value`] under the scope of a field name.
    ///
    /// # Errors
    ///
    /// Returns a [`ValidationError`] if the value violates the constraint rule.
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError>;
}

/// A validator that fails if the checked value is null.
pub struct RequiredValidator;

impl ValidatorFn for RequiredValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if value.is_null() {
            Err(ValidationError::new(field, "field is required", "required"))
        } else {
            Ok(())
        }
    }
}

/// A structural validator representing an optional field constraint.
///
/// Always evaluates to success regardless of the value provided.
pub struct OptionalValidator;

impl ValidatorFn for OptionalValidator {
    fn validate(&self, _value: &Value, _field: &str) -> Result<(), ValidationError> {
        Ok(())
    }
}

/// A structural validator representing a nullable field constraint.
///
/// Always evaluates to success. Nullability rules are checked prior in the schema validation sequence.
pub struct NullableValidator;

impl ValidatorFn for NullableValidator {
    fn validate(&self, _value: &Value, _field: &str) -> Result<(), ValidationError> {
        Ok(())
    }
}

/// Restricts the value to strictly match the expected dynamic type name.
pub struct StrictTypeValidator {
    /// The expected dynamic type name (e.g. `"int"`, `"string"`, `"bool"`).
    pub expected: &'static str,
}

impl ValidatorFn for StrictTypeValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if value.type_name() != self.expected {
            Err(ValidationError::new(
                field,
                format!(
                    "strict mode: expected type '{}', got '{}'",
                    self.expected,
                    value.type_name()
                ),
                "strict_type",
            ))
        } else {
            Ok(())
        }
    }
}

type CustomFn = Box<dyn Fn(&Value, &str) -> Result<(), ValidationError> + Send + Sync>;

/// A validation rule backed by a user-supplied validation closure.
pub struct CustomValidator {
    /// The backing closure to evaluate.
    pub func: CustomFn,
}

impl ValidatorFn for CustomValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        (self.func)(value, field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_fails_on_null() {
        let r = RequiredValidator.validate(&Value::Null, "x");
        assert!(r.is_err());
    }

    #[test]
    fn test_required_passes_on_value() {
        let r = RequiredValidator.validate(&Value::Int(1), "x");
        assert!(r.is_ok());
    }

    #[test]
    fn test_optional_always_passes() {
        assert!(OptionalValidator.validate(&Value::Null, "x").is_ok());
        assert!(OptionalValidator.validate(&Value::Int(99), "x").is_ok());
    }

    #[test]
    fn test_strict_type_fails() {
        let v = StrictTypeValidator { expected: "int" };
        assert!(v.validate(&Value::String("3".into()), "n").is_err());
    }

    #[test]
    fn test_strict_type_passes() {
        let v = StrictTypeValidator { expected: "bool" };
        assert!(v.validate(&Value::Bool(true), "flag").is_ok());
    }

    #[test]
    fn test_custom_validator() {
        let cv = CustomValidator {
            func: Box::new(|val, field| {
                if val.as_int() == Some(42) {
                    Ok(())
                } else {
                    Err(ValidationError::new(field, "must be 42", "must_be_42"))
                }
            }),
        };
        assert!(cv.validate(&Value::Int(42), "n").is_ok());
        assert!(cv.validate(&Value::Int(1), "n").is_err());
    }
}
