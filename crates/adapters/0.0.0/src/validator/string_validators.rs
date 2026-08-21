//! String-specific validators.
//!
//! Provides validation constraints specifically targeting string-like data structures,
//! including length restrictions, email and URL format validations, and regex checks.

use super::common_validators::ValidatorFn;
use crate::error::ValidationError;
use crate::value::Value;

/// A validator enforcing that the character count of a string is at least $N$.
pub struct MinLengthValidator(pub usize);

impl ValidatorFn for MinLengthValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if let Some(s) = value.as_str()
            && s.chars().count() < self.0
        {
            return Err(ValidationError::new(
                field,
                format!("minimum length is {}, got {}", self.0, s.chars().count()),
                "min_length",
            ));
        }
        Ok(())
    }
}

/// A validator enforcing that the character count of a string is at most $M$.
pub struct MaxLengthValidator(pub usize);

impl ValidatorFn for MaxLengthValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if let Some(s) = value.as_str()
            && s.chars().count() > self.0
        {
            return Err(ValidationError::new(
                field,
                format!("maximum length is {}, got {}", self.0, s.chars().count()),
                "max_length",
            ));
        }
        Ok(())
    }
}

/// Enforces that the evaluated string is formatted as a valid email address.
///
/// Applied rules:
/// - Exactly one `@` character
/// - Local part not empty and at most 64 characters long
/// - Domain contains at least one dot character
/// - Rejects spaces and consecutive dots anywhere
pub struct EmailValidator;

impl ValidatorFn for EmailValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        let s = match value.as_str() {
            Some(s) => s,
            None => return Ok(()),
        };
        if !is_valid_email(s) {
            return Err(ValidationError::new(
                field,
                format!("'{s}' is not a valid email address"),
                "email",
            ));
        }
        Ok(())
    }
}

/// Utility function checking standard, non-spaced email formatting rules.
pub fn is_valid_email(s: &str) -> bool {
    if s.contains(' ') {
        return false;
    }
    if s.contains("..") {
        return false;
    }
    let at_count = s.chars().filter(|&c| c == '@').count();
    if at_count != 1 {
        return false;
    }
    let (local, domain) = s.split_once('@').unwrap();
    if local.is_empty() || local.len() > 64 {
        return false;
    }
    if !domain.contains('.') {
        return false;
    }
    let dot_pos = domain.rfind('.').unwrap();
    if dot_pos == 0 || dot_pos == domain.len() - 1 {
        return false;
    }
    if domain.is_empty() {
        return false;
    }
    true
}

/// A validator confirming that the evaluated string matches a custom regular expression.
pub struct RegexValidator(pub String);

impl ValidatorFn for RegexValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        let s = match value.as_str() {
            Some(s) => s,
            None => return Ok(()),
        };
        let re = regex::Regex::new(&self.0).map_err(|e| {
            ValidationError::new(
                field,
                format!("invalid regex pattern: {e}"),
                "regex_invalid",
            )
        })?;
        if !re.is_match(s) {
            return Err(ValidationError::new(
                field,
                format!("value does not match pattern '{}'", self.0),
                "regex",
            ));
        }
        Ok(())
    }
}

/// A validator that fails if the evaluated string has a length of zero.
pub struct NonEmptyValidator;

impl ValidatorFn for NonEmptyValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if let Some(s) = value.as_str()
            && s.is_empty()
        {
            return Err(ValidationError::new(
                field,
                "value must not be empty",
                "non_empty",
            ));
        }
        Ok(())
    }
}

/// Enforces that the string contains exclusively letters and numeric digits.
pub struct AlphanumericValidator;

impl ValidatorFn for AlphanumericValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if let Some(s) = value.as_str()
            && !s.chars().all(|c| c.is_alphanumeric())
        {
            return Err(ValidationError::new(
                field,
                "value must be alphanumeric",
                "alphanumeric",
            ));
        }
        Ok(())
    }
}

/// Validates that a string format starts with standard `http://` or `https://` schemas.
pub struct UrlValidator;

impl ValidatorFn for UrlValidator {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if let Some(s) = value.as_str()
            && !is_valid_url(s)
        {
            return Err(ValidationError::new(
                field,
                format!("'{s}' is not a valid URL"),
                "url",
            ));
        }
        Ok(())
    }
}

fn is_valid_url(s: &str) -> bool {
    let s = if let Some(rest) = s.strip_prefix("https://") {
        rest
    } else if let Some(rest) = s.strip_prefix("http://") {
        rest
    } else {
        return false;
    };
    !s.is_empty() && !s.contains(' ')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_min_length_pass() {
        assert!(
            MinLengthValidator(3)
                .validate(&Value::String("abc".into()), "f")
                .is_ok()
        );
    }

    #[test]
    fn test_min_length_fail() {
        assert!(
            MinLengthValidator(5)
                .validate(&Value::String("ab".into()), "f")
                .is_err()
        );
    }

    #[test]
    fn test_max_length_pass() {
        assert!(
            MaxLengthValidator(10)
                .validate(&Value::String("hello".into()), "f")
                .is_ok()
        );
    }

    #[test]
    fn test_max_length_fail() {
        assert!(
            MaxLengthValidator(3)
                .validate(&Value::String("toolong".into()), "f")
                .is_err()
        );
    }

    #[test]
    fn test_email_valid() {
        assert!(
            EmailValidator
                .validate(&Value::String("alice@example.com".into()), "e")
                .is_ok()
        );
    }

    #[test]
    fn test_email_invalid_no_at() {
        assert!(
            EmailValidator
                .validate(&Value::String("nodomain.com".into()), "e")
                .is_err()
        );
    }

    #[test]
    fn test_email_invalid_no_dot() {
        assert!(
            EmailValidator
                .validate(&Value::String("alice@localhost".into()), "e")
                .is_err()
        );
    }

    #[test]
    fn test_email_invalid_space() {
        assert!(
            EmailValidator
                .validate(&Value::String("alice @example.com".into()), "e")
                .is_err()
        );
    }

    #[test]
    fn test_email_invalid_consecutive_dots() {
        assert!(
            EmailValidator
                .validate(&Value::String("alice@exam..ple.com".into()), "e")
                .is_err()
        );
    }

    #[test]
    fn test_non_empty_fail() {
        assert!(
            NonEmptyValidator
                .validate(&Value::String(String::new()), "f")
                .is_err()
        );
    }

    #[test]
    fn test_non_empty_pass() {
        assert!(
            NonEmptyValidator
                .validate(&Value::String("x".into()), "f")
                .is_ok()
        );
    }

    #[test]
    fn test_alphanumeric_pass() {
        assert!(
            AlphanumericValidator
                .validate(&Value::String("abc123".into()), "f")
                .is_ok()
        );
    }

    #[test]
    fn test_alphanumeric_fail() {
        assert!(
            AlphanumericValidator
                .validate(&Value::String("abc!".into()), "f")
                .is_err()
        );
    }

    #[test]
    fn test_url_valid() {
        assert!(
            UrlValidator
                .validate(&Value::String("https://example.com".into()), "u")
                .is_ok()
        );
    }

    #[test]
    fn test_url_invalid() {
        assert!(
            UrlValidator
                .validate(&Value::String("ftp://bad".into()), "u")
                .is_err()
        );
    }

    #[test]
    fn test_regex_pass() {
        assert!(
            RegexValidator(r"^\d+$".into())
                .validate(&Value::String("123".into()), "f")
                .is_ok()
        );
    }

    #[test]
    fn test_regex_fail() {
        assert!(
            RegexValidator(r"^\d+$".into())
                .validate(&Value::String("abc".into()), "f")
                .is_err()
        );
    }
}
