//! Validation utilities.

/// Re-export of the `Validator` trait from the extract module.
pub use crate::extract::validation::Validator;

/// Validate a value and return the value if valid, or an error message.
pub fn validate_or_error<T: Validator>(value: T) -> Result<T, String> {
    value.validate()?;
    Ok(value)
}

/// Assert that a string is not empty.
pub fn not_empty(s: &str, field: &str) -> Result<(), String> {
    if s.is_empty() {
        Err(format!("{} cannot be empty", field))
    } else {
        Ok(())
    }
}

/// Assert that a value is within a range.
pub fn in_range<T: PartialOrd + std::fmt::Debug>(
    value: T,
    min: T,
    max: T,
    field: &str,
) -> Result<(), String> {
    if value < min || value > max {
        Err(format!("{} must be between {:?} and {:?}", field, min, max))
    } else {
        Ok(())
    }
}

/// Assert that a string matches an email pattern (simple check).
pub fn is_email(s: &str, field: &str) -> Result<(), String> {
    if !s.contains('@') || !s.contains('.') {
        Err(format!("{} must be a valid email", field))
    } else {
        Ok(())
    }
}
