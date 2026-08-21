//! Error types for the adapters library.
//!
//! Provides the primary [`Error`] union wrapping specific lexical, parsing,
//! schema configuration, validation, serialization, and deserialization errors.

use std::fmt;

/// A single field-level validation failure detailing structural or value violations.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// The dot-notation path to the invalid field.
    pub field: String,
    /// A human-readable description of the constraint failure.
    pub message: String,
    /// The structural path segments leading to the failed key.
    pub path: Vec<String>,
    /// Short machine-readable identification code for programmatic checking.
    pub code: &'static str,
}

impl ValidationError {
    /// Constructs a new `ValidationError` with field identity, message, and error code.
    pub fn new(field: impl Into<String>, message: impl Into<String>, code: &'static str) -> Self {
        let field = field.into();
        Self {
            path: vec![field.clone()],
            field,
            message: message.into(),
            code,
        }
    }

    /// Prepends a parent field path segment to preserve nested field path routing.
    pub fn prepend_path(mut self, parent: &str) -> Self {
        self.path.insert(0, parent.to_string());
        self.field = self.path.join(".");
        self
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {} (code: {})", self.field, self.message, self.code)
    }
}

impl std::error::Error for ValidationError {}

/// A collection representing all validation failures accumulated during validation.
#[derive(Debug, Clone, Default)]
pub struct ValidationErrors {
    /// List of validation failures.
    pub errors: Vec<ValidationError>,
}

impl ValidationErrors {
    /// Constructs a new, empty validation error collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a new [`ValidationError`] to the list.
    pub fn push(&mut self, e: ValidationError) {
        self.errors.push(e);
    }

    /// Returns `true` if no validation errors have been collected.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Converts the collection to a generic `Result` wrapping the first failure if non-empty.
    pub fn into_result(self) -> Result<(), Error> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(Error::Validation(self.errors[0].clone()))
        }
    }

    /// Converts the collection to a generic validation `Result`.
    pub fn into_error(self) -> Result<(), Error> {
        self.into_result()
    }

    /// Merges errors from another collection, prefixing the fields with a parent segment.
    pub fn extend_prefixed(&mut self, parent: &str, other: ValidationErrors) {
        for e in other.errors {
            self.push(e.prepend_path(parent));
        }
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, e) in self.errors.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{e}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

/// Error type indicating a failure in serializing structures into raw values.
#[derive(Debug, Clone)]
pub struct SerializationError {
    /// Human-readable explanation of why serialization failed.
    pub message: String,
}

impl SerializationError {
    /// Constructs a new `SerializationError`.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Serialization error: {}", self.message)
    }
}

impl std::error::Error for SerializationError {}

/// Error type indicating a failure in deserializing dynamic values into structures.
#[derive(Debug, Clone)]
pub struct DeserializationError {
    /// Human-readable explanation of the type or structural parsing mismatch.
    pub message: String,
    /// The structural field name that triggered the error, if applicable.
    pub field: Option<String>,
}

impl DeserializationError {
    /// Constructs a new generic `DeserializationError`.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: None,
        }
    }

    /// Constructs a new `DeserializationError` associated with a specific field.
    pub fn with_field(message: impl Into<String>, field: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: Some(field.into()),
        }
    }
}

impl fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.field {
            Some(field) => write!(f, "Deserialization error at '{}': {}", field, self.message),
            None => write!(f, "Deserialization error: {}", self.message),
        }
    }
}

impl std::error::Error for DeserializationError {}

/// Error type wrapping low-level JSON parser and lexer scanner syntax issues.
#[derive(Debug, Clone)]
pub struct JsonError {
    /// Human-readable explanation of the syntax anomaly.
    pub message: String,
    /// Stream offset position where the scanner reported the problem.
    pub position: Option<usize>,
}

impl JsonError {
    /// Constructs a new generic `JsonError`.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            position: None,
        }
    }

    /// Constructs a new `JsonError` at a specific source stream position.
    pub fn at(message: impl Into<String>, position: usize) -> Self {
        Self {
            message: message.into(),
            position: Some(position),
        }
    }
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.position {
            Some(pos) => write!(f, "JSON error at position {}: {}", pos, self.message),
            None => write!(f, "JSON error: {}", self.message),
        }
    }
}

impl std::error::Error for JsonError {}

/// Error type indicating a failure in schema verification or programmatic inspection.
#[derive(Debug, Clone)]
pub struct SchemaError {
    /// Human-readable description of the schema mismatch.
    pub message: String,
}

impl SchemaError {
    /// Constructs a new `SchemaError`.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Schema error: {}", self.message)
    }
}

impl std::error::Error for SchemaError {}

/// The top-level error enum encompassing all failures returned by this library.
#[derive(Debug, Clone)]
pub enum Error {
    /// A custom validation or datatype constraint rule was violated.
    Validation(ValidationError),
    /// Converting typed structures to general exchange types failed.
    Serialization(SerializationError),
    /// Re-constituting typed structures from values failed.
    Deserialization(DeserializationError),
    /// Native JSON engine lexer/parser error.
    Json(JsonError),
    /// Programmatic schema checking or configuration error.
    Schema(SchemaError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Validation(e) => write!(f, "{e}"),
            Error::Serialization(e) => write!(f, "{e}"),
            Error::Deserialization(e) => write!(f, "{e}"),
            Error::Json(e) => write!(f, "{e}"),
            Error::Schema(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Validation(e) => Some(e),
            Error::Serialization(e) => Some(e),
            Error::Deserialization(e) => Some(e),
            Error::Json(e) => Some(e),
            Error::Schema(e) => Some(e),
        }
    }
}

impl From<ValidationError> for Error {
    fn from(e: ValidationError) -> Self {
        Error::Validation(e)
    }
}

impl From<SerializationError> for Error {
    fn from(e: SerializationError) -> Self {
        Error::Serialization(e)
    }
}

impl From<DeserializationError> for Error {
    fn from(e: DeserializationError) -> Self {
        Error::Deserialization(e)
    }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Self {
        Error::Json(e)
    }
}

impl From<SchemaError> for Error {
    fn from(e: SchemaError) -> Self {
        Error::Schema(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_display() {
        let e = ValidationError::new("email", "invalid email format", "email");
        let s = e.to_string();
        assert!(s.contains("email"));
        assert!(s.contains("invalid email format"));
    }

    #[test]
    fn test_validation_error_prepend_path() {
        let e = ValidationError::new("city", "too short", "min_length");
        let e = e.prepend_path("address");
        assert_eq!(e.path, vec!["address", "city"]);
        assert_eq!(e.field, "address.city");
    }

    #[test]
    fn test_validation_errors_collect() {
        let mut errs = ValidationErrors::new();
        assert!(errs.is_empty());
        errs.push(ValidationError::new("name", "required", "required"));
        errs.push(ValidationError::new("email", "invalid", "email"));
        assert!(!errs.is_empty());
        assert_eq!(errs.errors.len(), 2);
    }

    #[test]
    fn test_validation_errors_into_result_empty() {
        let errs = ValidationErrors::new();
        assert!(errs.into_result().is_ok());
    }

    #[test]
    fn test_validation_errors_into_result_nonempty() {
        let mut errs = ValidationErrors::new();
        errs.push(ValidationError::new("x", "bad", "bad_code"));
        assert!(errs.into_result().is_err());
    }

    #[test]
    fn test_json_error_display_with_position() {
        let e = JsonError::at("unexpected token", 42);
        let s = e.to_string();
        assert!(s.contains("42"));
        assert!(s.contains("unexpected token"));
    }

    #[test]
    fn test_error_from_json() {
        let je = JsonError::new("bad json");
        let e: Error = je.into();
        assert!(matches!(e, Error::Json(_)));
    }

    #[test]
    fn test_error_from_validation() {
        let ve = ValidationError::new("f", "m", "c");
        let e: Error = ve.into();
        assert!(matches!(e, Error::Validation(_)));
    }

    #[test]
    fn test_error_from_deserialization() {
        let de = DeserializationError::new("bad");
        let e: Error = de.into();
        assert!(matches!(e, Error::Deserialization(_)));
    }

    #[test]
    fn test_error_from_serialization() {
        let se = SerializationError::new("bad");
        let e: Error = se.into();
        assert!(matches!(e, Error::Serialization(_)));
    }

    #[test]
    fn test_error_from_schema() {
        let se = SchemaError::new("bad schema");
        let e: Error = se.into();
        assert!(matches!(e, Error::Schema(_)));
    }

    #[test]
    fn test_deserialization_error_with_field() {
        let e = DeserializationError::with_field("expected i64", "age");
        let s = e.to_string();
        assert!(s.contains("age"));
        assert!(s.contains("expected i64"));
    }

    #[test]
    fn test_validation_errors_extend_prefixed() {
        let mut parent = ValidationErrors::new();
        let mut child = ValidationErrors::new();
        child.push(ValidationError::new("city", "required", "required"));
        parent.extend_prefixed("address", child);
        assert_eq!(parent.errors[0].field, "address.city");
    }
}
