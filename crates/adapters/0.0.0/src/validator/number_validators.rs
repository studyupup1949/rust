//! Numeric validators for integer and float fields.
//!
//! Provides validation constraints specifically targeting numeric datatypes,
//! including min/max limits, positive/negative bounds, non-zero checks, and range validations.

use super::common_validators::ValidatorFn;
use crate::error::ValidationError;
use crate::value::Value;

/// A validator enforcing that an integer value is greater than or equal to $N$.
pub struct MinIntValidator(pub i64);

impl ValidatorFn for MinIntValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if let Some(n) = value.coerce_to_int()
            && n < self.0
        {
            return Err(ValidationError::new(
                field,
                format!("minimum value is {}, got {n}", self.0),
                "min",
            ));
        }
        Ok(())
    }
}

/// A validator enforcing that an integer value is less than or equal to $M$.
pub struct MaxIntValidator(pub i64);

impl ValidatorFn for MaxIntValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if let Some(n) = value.coerce_to_int()
            && n > self.0
        {
            return Err(ValidationError::new(
                field,
                format!("maximum value is {}, got {n}", self.0),
                "max",
            ));
        }
        Ok(())
    }
}

/// A validator enforcing that a float value is greater than or equal to $N$.
pub struct MinFloatValidator(pub f64);

impl ValidatorFn for MinFloatValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if let Some(f) = value.coerce_to_float()
            && f < self.0
        {
            return Err(ValidationError::new(
                field,
                format!("minimum value is {}, got {f}", self.0),
                "min",
            ));
        }
        Ok(())
    }
}

/// A validator enforcing that a float value is less than or equal to $M$.
pub struct MaxFloatValidator(pub f64);

impl ValidatorFn for MaxFloatValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if let Some(f) = value.coerce_to_float()
            && f > self.0
        {
            return Err(ValidationError::new(
                field,
                format!("maximum value is {}, got {f}", self.0),
                "max",
            ));
        }
        Ok(())
    }
}

/// A validator enforcing that a numeric value is strictly positive ($> 0$).
pub struct PositiveValidator;

impl ValidatorFn for PositiveValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if let Some(f) = value.coerce_to_float()
            && f <= 0.0
        {
            return Err(ValidationError::new(
                field,
                "value must be positive",
                "positive",
            ));
        }
        Ok(())
    }
}

/// A validator enforcing that a numeric value is strictly negative ($< 0$).
pub struct NegativeValidator;

impl ValidatorFn for NegativeValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if let Some(f) = value.coerce_to_float()
            && f >= 0.0
        {
            return Err(ValidationError::new(
                field,
                "value must be negative",
                "negative",
            ));
        }
        Ok(())
    }
}

/// A validator enforcing that a numeric value is not exactly equal to zero.
pub struct NonZeroValidator;

impl ValidatorFn for NonZeroValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if let Some(f) = value.coerce_to_float()
            && f == 0.0
        {
            return Err(ValidationError::new(
                field,
                "value must not be zero",
                "non_zero",
            ));
        }
        Ok(())
    }
}

/// A validator enforcing that a numeric value falls inclusively within the range `[min, max]`.
pub struct RangeValidator {
    /// The inclusive lower bound of the range.
    pub min: f64,
    /// The inclusive upper bound of the range.
    pub max: f64,
}

impl ValidatorFn for RangeValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if let Some(f) = value.coerce_to_float()
            && (f < self.min || f > self.max)
        {
            return Err(ValidationError::new(
                field,
                format!(
                    "value must be in range [{}, {}], got {f}",
                    self.min, self.max
                ),
                "range",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_int_pass() {
        assert!(MinIntValidator(5).validate(&Value::Int(10), "n").is_ok());
    }

    #[test]
    fn test_min_int_fail() {
        assert!(MinIntValidator(5).validate(&Value::Int(3), "n").is_err());
    }

    #[test]
    fn test_max_int_pass() {
        assert!(MaxIntValidator(100).validate(&Value::Int(50), "n").is_ok());
    }

    #[test]
    fn test_max_int_fail() {
        assert!(
            MaxIntValidator(100)
                .validate(&Value::Int(200), "n")
                .is_err()
        );
    }

    #[test]
    fn test_min_float_fail() {
        assert!(
            MinFloatValidator(1.0)
                .validate(&Value::Float(0.5), "f")
                .is_err()
        );
    }

    #[test]
    fn test_max_float_fail() {
        assert!(
            MaxFloatValidator(1.0)
                .validate(&Value::Float(1.5), "f")
                .is_err()
        );
    }

    #[test]
    fn test_positive_fail_zero() {
        assert!(PositiveValidator.validate(&Value::Int(0), "n").is_err());
    }

    #[test]
    fn test_positive_pass() {
        assert!(PositiveValidator.validate(&Value::Int(1), "n").is_ok());
    }

    #[test]
    fn test_negative_fail() {
        assert!(NegativeValidator.validate(&Value::Int(1), "n").is_err());
    }

    #[test]
    fn test_non_zero_fail() {
        assert!(NonZeroValidator.validate(&Value::Int(0), "n").is_err());
    }

    #[test]
    fn test_non_zero_pass() {
        assert!(NonZeroValidator.validate(&Value::Int(5), "n").is_ok());
    }

    #[test]
    fn test_range_pass() {
        assert!(
            RangeValidator {
                min: 0.0,
                max: 100.0
            }
            .validate(&Value::Int(50), "n")
            .is_ok()
        );
    }

    #[test]
    fn test_range_fail() {
        assert!(
            RangeValidator {
                min: 0.0,
                max: 10.0
            }
            .validate(&Value::Int(11), "n")
            .is_err()
        );
    }
}
