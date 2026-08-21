//! Validator module — reusable, composable validation building blocks.
//!
//! This module provides a flexible and modular validation subsystem based on the
//! [`ValidatorFn`] trait and [`ValidatorChain`] structure.

pub mod common_validators;
pub mod number_validators;
pub mod string_validators;

pub use common_validators::{
    CustomValidator, NullableValidator, OptionalValidator, RequiredValidator, StrictTypeValidator,
    ValidatorFn,
};
pub use number_validators::{
    MaxFloatValidator, MaxIntValidator, MinFloatValidator, MinIntValidator, NegativeValidator,
    NonZeroValidator, PositiveValidator, RangeValidator,
};
pub use string_validators::{
    AlphanumericValidator, EmailValidator, MaxLengthValidator, MinLengthValidator,
    NonEmptyValidator, RegexValidator, UrlValidator, is_valid_email,
};

use crate::error::ValidationErrors;
use crate::value::Value;

/// An ordered sequence of [`ValidatorFn`] instances evaluated together.
///
/// Unlike short-circuiting validators, `ValidatorChain` executes all validation
/// rules completely and accumulates all failures into a unified [`ValidationErrors`]
/// collection. This ensures a thorough report of all validation constraint issues.
#[derive(Default)]
pub struct ValidatorChain {
    validators: Vec<Box<dyn ValidatorFn>>,
}

impl ValidatorChain {
    /// Creates a new, empty validator chain.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a new validator to the end of the chain.
    pub fn push_validator(mut self, v: impl ValidatorFn + 'static) -> Self {
        self.validators.push(Box::new(v));
        self
    }

    /// Validates the given [`Value`] against all validators in the chain.
    ///
    /// # Errors
    ///
    /// Returns a [`ValidationErrors`] collection if one or more validators fail.
    pub fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationErrors> {
        let mut errs = ValidationErrors::new();
        for v in &self.validators {
            if let Err(e) = v.validate(value, field) {
                errs.push(e);
            }
        }
        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_collects_all_errors() {
        let chain = ValidatorChain::new()
            .push_validator(MinLengthValidator(10))
            .push_validator(EmailValidator);
        let result = chain.validate(&Value::String("hi".into()), "email");
        assert!(result.is_err());
        let errs = result.unwrap_err();
        assert_eq!(errs.errors.len(), 2);
    }

    #[test]
    fn test_chain_passes_when_all_valid() {
        let chain = ValidatorChain::new()
            .push_validator(MinLengthValidator(3))
            .push_validator(MaxLengthValidator(20));
        assert!(
            chain
                .validate(&Value::String("hello".into()), "name")
                .is_ok()
        );
    }

    #[test]
    fn test_chain_empty_always_passes() {
        let chain = ValidatorChain::new();
        assert!(chain.validate(&Value::Null, "x").is_ok());
    }

    #[test]
    fn test_required_validator_in_chain() {
        let chain = ValidatorChain::new().push_validator(RequiredValidator);
        assert!(chain.validate(&Value::Null, "x").is_err());
        assert!(chain.validate(&Value::Int(1), "x").is_ok());
    }
}
