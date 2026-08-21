//! String schema — validates and describes string-typed fields.
//!
//! Provides the [`StringSchema`] structure for string validation constraints.

use super::SchemaValidator;
use crate::error::ValidationError;
use crate::validator::{
    EmailValidator, MaxLengthValidator, MinLengthValidator, RegexValidator, RequiredValidator,
    StrictTypeValidator, ValidatorChain,
};
use crate::value::Value;

/// Schema representing string constraints and mapping metadata.
#[derive(Default)]
pub struct StringSchema {
    min_length: Option<usize>,
    max_length: Option<usize>,
    email: bool,
    url: bool,
    regex_pattern: Option<String>,
    default: Option<String>,
    alias: Option<String>,
    required: bool,
    optional: bool,
    strict: bool,
    non_empty: bool,
    alphanumeric: bool,
}

impl StringSchema {
    /// Creates a new `StringSchema` with no validation constraints.
    pub fn new() -> Self {
        Default::default()
    }

    /// Constrains the string to have a minimum character count of $N$.
    pub fn min_length(mut self, n: usize) -> Self {
        self.min_length = Some(n);
        self
    }

    /// Constrains the string to have a maximum character count of $M$.
    pub fn max_length(mut self, n: usize) -> Self {
        self.max_length = Some(n);
        self
    }

    /// Constrains the string to be non-empty.
    pub fn non_empty(mut self) -> Self {
        self.non_empty = true;
        self
    }

    /// Constrains the string to contain only alphanumeric characters.
    pub fn alphanumeric(mut self) -> Self {
        self.alphanumeric = true;
        self
    }

    /// Restricts the string format to standard email addresses.
    pub fn email(mut self) -> Self {
        self.email = true;
        self
    }

    /// Restricts the string format to absolute URLs.
    pub fn url(mut self) -> Self {
        self.url = true;
        self
    }

    /// Matches the string value against a custom Rust regular expression pattern.
    pub fn regex(mut self, pattern: &str) -> Self {
        self.regex_pattern = Some(pattern.to_string());
        self
    }

    /// Configures a fallback default value used when this field is missing.
    pub fn default(mut self, val: &str) -> Self {
        self.default = Some(val.to_string());
        self
    }

    /// Registers a field key rename mapping for parsing input payloads.
    pub fn alias(mut self, name: &str) -> Self {
        self.alias = Some(name.to_string());
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

    /// Opts into strict validation mode: non-string inputs cause immediate failure instead of coercion.
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Returns the registered key alias name if defined.
    pub fn get_alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    fn build_chain(&self) -> ValidatorChain {
        let mut chain = ValidatorChain::new();
        if self.required {
            chain = chain.push_validator(RequiredValidator);
        }
        if self.strict {
            chain = chain.push_validator(StrictTypeValidator { expected: "string" });
        }
        if self.non_empty {
            chain = chain.push_validator(crate::validator::NonEmptyValidator);
        }
        if self.alphanumeric {
            chain = chain.push_validator(crate::validator::AlphanumericValidator);
        }
        if let Some(n) = self.min_length {
            chain = chain.push_validator(MinLengthValidator(n));
        }
        if let Some(n) = self.max_length {
            chain = chain.push_validator(MaxLengthValidator(n));
        }
        if self.email {
            chain = chain.push_validator(EmailValidator);
        }
        if self.url {
            chain = chain.push_validator(crate::validator::UrlValidator);
        }
        if let Some(ref pat) = self.regex_pattern {
            chain = chain.push_validator(RegexValidator(pat.clone()));
        }
        chain
    }
}

impl SchemaValidator for StringSchema {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if self.optional && value.is_null() {
            return Ok(());
        }
        let effective = if !self.strict && !value.is_string() && !value.is_null() {
            if let Some(s) = value.coerce_to_string() {
                Value::String(s)
            } else {
                value.clone()
            }
        } else {
            value.clone()
        };
        self.build_chain()
            .validate(&effective, field)
            .map_err(|errs| errs.errors[0].clone())
    }

    fn is_required(&self) -> bool {
        self.required && self.default.is_none()
    }

    fn default_value(&self) -> Option<Value> {
        self.default.as_ref().map(|s| Value::String(s.clone()))
    }

    fn schema_type(&self) -> &'static str {
        "string"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_valid() {
        let s = StringSchema::new().min_length(2).max_length(10);
        assert!(s.validate(&Value::String("hello".into()), "name").is_ok());
    }

    #[test]
    fn test_string_too_short() {
        let s = StringSchema::new().min_length(5);
        assert!(s.validate(&Value::String("hi".into()), "name").is_err());
    }

    #[test]
    fn test_string_too_long() {
        let s = StringSchema::new().max_length(3);
        assert!(
            s.validate(&Value::String("toolong".into()), "name")
                .is_err()
        );
    }

    #[test]
    fn test_string_email_valid() {
        let s = StringSchema::new().email();
        assert!(
            s.validate(&Value::String("a@b.com".into()), "email")
                .is_ok()
        );
    }

    #[test]
    fn test_string_email_invalid() {
        let s = StringSchema::new().email();
        assert!(
            s.validate(&Value::String("notanemail".into()), "email")
                .is_err()
        );
    }

    #[test]
    fn test_string_required_fails_null() {
        let s = StringSchema::new().required();
        assert!(s.validate(&Value::Null, "name").is_err());
    }

    #[test]
    fn test_string_optional_passes_null() {
        let s = StringSchema::new().optional();
        assert!(s.validate(&Value::Null, "bio").is_ok());
    }

    #[test]
    fn test_string_strict_rejects_int() {
        let s = StringSchema::new().strict();
        assert!(s.validate(&Value::Int(42), "name").is_err());
    }

    #[test]
    fn test_string_nonstrict_coerces_int() {
        let s = StringSchema::new();
        assert!(s.validate(&Value::Int(42), "name").is_ok());
    }

    #[test]
    fn test_string_default() {
        let s = StringSchema::new().default("India");
        assert_eq!(s.default_value(), Some(Value::String("India".into())));
    }

    #[test]
    fn test_string_non_empty() {
        let s = StringSchema::new().non_empty();
        assert!(s.validate(&Value::String("hello".into()), "val").is_ok());
        assert!(s.validate(&Value::String("".into()), "val").is_err());
    }

    #[test]
    fn test_string_alphanumeric() {
        let s = StringSchema::new().alphanumeric();
        assert!(
            s.validate(&Value::String("hello123WORLD".into()), "val")
                .is_ok()
        );
        assert!(
            s.validate(&Value::String("hello-world".into()), "val")
                .is_err()
        );
    }
}
