//! Integer schema — validates and describes integer-typed fields.
//!
//! Provides the [`IntegerSchema`] structure for integer validation constraints.

use super::SchemaValidator;
use crate::error::ValidationError;
use crate::validator::{
    MaxIntValidator, MinIntValidator, RequiredValidator, StrictTypeValidator, ValidatorChain,
};
use crate::value::Value;

/// Schema representing integer constraints and mapping metadata.
#[derive(Default)]
pub struct IntegerSchema {
    min: Option<i64>,
    max: Option<i64>,
    default: Option<i64>,
    required: bool,
    optional: bool,
    strict: bool,
    positive: bool,
    negative: bool,
    non_zero: bool,
}

impl IntegerSchema {
    /// Creates a new `IntegerSchema` with no validation constraints.
    pub fn new() -> Self {
        Default::default()
    }

    /// Constrains the integer to be greater than or equal to $N$.
    pub fn min(mut self, n: i64) -> Self {
        self.min = Some(n);
        self
    }

    /// Constrains the integer to be less than or equal to $M$.
    pub fn max(mut self, n: i64) -> Self {
        self.max = Some(n);
        self
    }

    /// Constrains the integer to be strictly positive (> 0).
    pub fn positive(mut self) -> Self {
        self.positive = true;
        self
    }

    /// Constrains the integer to be strictly negative (< 0).
    pub fn negative(mut self) -> Self {
        self.negative = true;
        self
    }

    /// Constrains the integer to be non-zero (!= 0).
    pub fn non_zero(mut self) -> Self {
        self.non_zero = true;
        self
    }

    /// Configures a fallback default value used when this field is missing.
    pub fn default(mut self, val: i64) -> Self {
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

    /// Opts into strict validation mode: non-integer inputs cause immediate failure instead of coercion.
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
            chain = chain.push_validator(StrictTypeValidator { expected: "int" });
        }
        if self.positive {
            chain = chain.push_validator(crate::validator::PositiveValidator);
        }
        if self.negative {
            chain = chain.push_validator(crate::validator::NegativeValidator);
        }
        if self.non_zero {
            chain = chain.push_validator(crate::validator::NonZeroValidator);
        }
        if let Some(n) = self.min {
            chain = chain.push_validator(MinIntValidator(n));
        }
        if let Some(n) = self.max {
            chain = chain.push_validator(MaxIntValidator(n));
        }
        chain
    }
}

impl SchemaValidator for IntegerSchema {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if self.optional && value.is_null() {
            return Ok(());
        }
        let effective = if !self.strict && !value.is_int() && !value.is_null() {
            if let Some(n) = value.coerce_to_int() {
                Value::Int(n)
            } else {
                value.clone()
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
        self.default.map(Value::Int)
    }

    fn schema_type(&self) -> &'static str {
        "integer"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_valid() {
        let s = IntegerSchema::new().min(0).max(100);
        assert!(s.validate(&Value::Int(50), "n").is_ok());
    }

    #[test]
    fn test_int_too_small() {
        let s = IntegerSchema::new().min(18);
        assert!(s.validate(&Value::Int(10), "age").is_err());
    }

    #[test]
    fn test_int_too_large() {
        let s = IntegerSchema::new().max(100);
        assert!(s.validate(&Value::Int(200), "n").is_err());
    }

    #[test]
    fn test_int_required_null() {
        let s = IntegerSchema::new().required();
        assert!(s.validate(&Value::Null, "n").is_err());
    }

    #[test]
    fn test_int_optional_null() {
        let s = IntegerSchema::new().optional();
        assert!(s.validate(&Value::Null, "n").is_ok());
    }

    #[test]
    fn test_int_strict_rejects_string() {
        let s = IntegerSchema::new().strict();
        assert!(s.validate(&Value::String("18".into()), "age").is_err());
    }

    #[test]
    fn test_int_nonstrict_coerces_string() {
        let s = IntegerSchema::new().min(18);
        assert!(s.validate(&Value::String("25".into()), "age").is_ok());
    }

    #[test]
    fn test_int_default() {
        let s = IntegerSchema::new().default(42);
        assert_eq!(s.default_value(), Some(Value::Int(42)));
    }

    #[test]
    fn test_int_positive() {
        let s = IntegerSchema::new().positive();
        assert!(s.validate(&Value::Int(10), "val").is_ok());
        assert!(s.validate(&Value::Int(0), "val").is_err());
        assert!(s.validate(&Value::Int(-1), "val").is_err());
    }

    #[test]
    fn test_int_negative() {
        let s = IntegerSchema::new().negative();
        assert!(s.validate(&Value::Int(-10), "val").is_ok());
        assert!(s.validate(&Value::Int(0), "val").is_err());
        assert!(s.validate(&Value::Int(5), "val").is_err());
    }

    #[test]
    fn test_int_non_zero() {
        let s = IntegerSchema::new().non_zero();
        assert!(s.validate(&Value::Int(10), "val").is_ok());
        assert!(s.validate(&Value::Int(-10), "val").is_ok());
        assert!(s.validate(&Value::Int(0), "val").is_err());
    }
}
