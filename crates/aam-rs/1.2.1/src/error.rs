//! Error types for the AAML parser and validation pipeline.

use std::fmt;
use std::io;

/// All errors that can be produced while parsing or validating an AAML document.
#[derive(Debug)]
pub enum AamlError {
    /// An I/O error occurred while reading a file.
    IoError(io::Error),

    /// A line could not be parsed as a valid AAML statement.
    ParseError {
        /// 1-based line number in the source file.
        line: usize,
        /// Raw content of the offending line.
        content: String,
        /// Human-readable explanation of why parsing failed.
        details: String,
    },

    /// A key or type name was not found in the registry or map.
    NotFound(String),

    /// A value does not satisfy a basic type constraint (not schema-specific).
    InvalidValue(String),

    /// A value failed validation against a registered or built-in type.
    InvalidType {
        /// Name of the type that rejected the value.
        type_name: String,
        /// Details from the type validator.
        details: String,
    },

    /// A directive (`@import`, `@derive`, …) encountered an error in its arguments.
    DirectiveError(String, String),

    /// A schema constraint was violated during parsing or explicit validation.
    ///
    /// Produced by:
    /// - Automatic per-field validation when `key = value` is parsed and `key`
    ///   belongs to an active schema.
    /// - [`AAML::apply_schema`](crate::aaml::AAML::apply_schema) when called explicitly.
    /// - [`AAML::validate_schemas_completeness`](crate::aaml::AAML::validate_schemas_completeness)
    ///   after `@derive` to ensure all required fields are present.
    SchemaValidationError {
        /// Name of the schema that declared the field.
        schema: String,
        /// Name of the field that failed validation.
        field: String,
        /// Declared type of the field.
        type_name: String,
        /// Human-readable description of the failure.
        details: String,
    },
}

impl fmt::Display for AamlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AamlError::IoError(err) => write!(f, "IO Error: {}", err),
            AamlError::ParseError { line, content, details } => {
                write!(f, "Parse Error at line {}: '{}'. Reason: {}", line, content, details)
            }
            AamlError::NotFound(key) => write!(f, "Key not found: '{}'", key),
            AamlError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            AamlError::InvalidType { type_name, details } => {
                write!(f, "Invalid type '{}': {}", type_name, details)
            }
            AamlError::DirectiveError(cmd, msg) => {
                write!(f, "Directive '@{}' error: {}", cmd, msg)
            }
            AamlError::SchemaValidationError { schema, field, type_name, details } => {
                write!(
                    f,
                    "Schema '{}' validation error: field '{}' (type '{}') — {}",
                    schema, field, type_name, details
                )
            }
        }
    }
}

impl std::error::Error for AamlError {}

impl From<io::Error> for AamlError {
    fn from(err: io::Error) -> Self {
        AamlError::IoError(err)
    }
}