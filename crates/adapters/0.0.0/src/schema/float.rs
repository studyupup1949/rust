//! Float schema — validates and describes float-typed fields.
//!
//! Provides the [`FloatSchema`] structure for float validation constraints.

use super::SchemaValidator;
use crate::error::ValidationError;
use crate::validator::{
    MaxFloatValidator, MinFloatValidator, RequiredValidator, StrictTypeValidator, ValidatorChain,
};
use crate::value::Value;

/// Schema representing float constraints and mapping metadata.
#[derive(Default)]
pub struct FloatSchema {
    min: Option<f64>,
    max: Option<f64>,
    default: Option<f64>,
    required: bool,
    optional: bool,
    strict: bool,
    positive: bool,
    negative: bool,
    non_zero: bool,
}

impl FloatSchema {
    /// Creates a new `FloatSchema` with no validation constraints.
    pub fn new() -> Self {
        Default::default()
    }

    /// Constrains the float to be greater than or equal to $N$.
    pub fn min(mut self, n: f64) -> Self {
        self.min = Some(n);
        self
    }

    /// Constrains the float to be less than or equal to $M$.
    pub fn max(mut self, n: f64) -> Self {
        self.max = Some(n);
        self
    }

    /// Constrains the float to be strictly positive (> 0.0).
    pub fn positive(mut self) -> Self {
        self.positive = true;
        self
    }

    /// Constrains the float to be strictly negative (< 0.0).
    pub fn negative(mut self) -> Self {
        self.negative = true;
        self
    }

    /// Constrains the float to be non-zero (!= 0.0).
    pub fn non_zero(mut self) -> Self {
        self.non_zero = true;
        self
    }

    /// Configures a fallback default value used when this field is missing.
    pub fn default(mut self, val: f64) -> Self {
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

    /// Opts into strict validation mode: non-float inputs cause immediate failure instead of coercion.
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
            chain = chain.push_validator(StrictTypeValidator { expected: "float" });
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
            chain = chain.push_validator(MinFloatValidator(n));
        }
        if let Some(n) = self.max {
            chain = chain.push_validator(MaxFloatValidator(n));
        }
        chain
    }
}

impl SchemaValidator for FloatSchema {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if self.optional && value.is_null() {
            return Ok(());
        }
        let effective = if !self.strict && !value.is_float() && !value.is_null() {
            if let Some(f) = value.coerce_to_float() {
                Value::Float(f)
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
        self.default.map(Value::Float)
    }

    fn schema_type(&self) -> &'static str {
        "float"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_float_valid() {
        let s = FloatSchema::new().min(0.0).max(100.0);
        assert!(s.validate(&Value::Float(50.5), "n").is_ok());
    }

    #[test]
    fn test_float_too_small() {
        let s = FloatSchema::new().min(1.0);
        assert!(s.validate(&Value::Float(0.5), "n").is_err());
    }

    #[test]
    fn test_float_strict_rejects_int() {
        let s = FloatSchema::new().strict();
        assert!(s.validate(&Value::Int(3), "n").is_err());
    }

    #[test]
    fn test_float_nonstrict_coerces_int() {
        let s = FloatSchema::new();
        assert!(s.validate(&Value::Int(3), "n").is_ok());
    }

    #[test]
    fn test_float_nonstrict_coerces_string() {
        let s = FloatSchema::new();
        assert!(s.validate(&Value::String("3.14".into()), "n").is_ok());
    }

    #[test]
    fn test_float_optional_null() {
        let s = FloatSchema::new().optional();
        assert!(s.validate(&Value::Null, "n").is_ok());
    }

    #[test]
    fn test_float_default() {
        let s = FloatSchema::new().default(1.5);
        assert_eq!(s.default_value(), Some(Value::Float(1.5)));
    }

    #[test]
    fn test_float_positive() {
        let s = FloatSchema::new().positive();
        assert!(s.validate(&Value::Float(3.15), "val").is_ok());
        assert!(s.validate(&Value::Float(0.0), "val").is_err());
        assert!(s.validate(&Value::Float(-1.5), "val").is_err());
    }

    #[test]
    fn test_float_negative() {
        let s = FloatSchema::new().negative();
        assert!(s.validate(&Value::Float(-3.15), "val").is_ok());
        assert!(s.validate(&Value::Float(0.0), "val").is_err());
        assert!(s.validate(&Value::Float(1.5), "val").is_err());
    }

    #[test]
    fn test_float_non_zero() {
        let s = FloatSchema::new().non_zero();
        assert!(s.validate(&Value::Float(3.15), "val").is_ok());
        assert!(s.validate(&Value::Float(-3.15), "val").is_ok());
        assert!(s.validate(&Value::Float(0.0), "val").is_err());
    }
}
