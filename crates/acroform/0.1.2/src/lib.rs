//! # acroform
//!
//! A high-level PDF form manipulation library using lopdf.
//!
//! This crate provides a simple API for reading and filling PDF forms (AcroForms).
//! It uses the official `lopdf` crate for PDF operations.
//!
//! ## Example
//!
//! ```no_run
//! use acroform::{AcroFormDocument, FieldValue};
//! use std::collections::HashMap;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load a PDF with form fields
//! let mut doc = AcroFormDocument::from_pdf("form.pdf")?;
//!
//! // List all fields
//! let fields = doc.fields()?;
//! for field in &fields {
//!     println!("Field: {} ({})", field.name, field.field_type);
//! }
//!
//! // Fill fields
//! let mut values = HashMap::new();
//! values.insert("name".to_string(), FieldValue::Text("John Doe".to_string()));
//! values.insert("age".to_string(), FieldValue::Integer(30));
//!
//! // Save filled PDF
//! doc.fill_and_save(values, "filled_form.pdf")?;
//! # Ok(())
//! # }
//! ```

mod api;
mod field;

pub use api::{AcroFormDocument, FieldValue, FormField};
pub use field::FieldType;

/// Result type alias for acroform operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for acroform operations
#[derive(Debug)]
pub enum Error {
    /// Error from lopdf library
    Lopdf(lopdf::Error),
    /// Field not found
    MissingField { field: String },
    /// Invalid field type
    InvalidFieldType { expected: String, found: String },
    /// IO error
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Lopdf(e) => write!(f, "PDF error: {}", e),
            Error::MissingField { field } => write!(f, "Field not found: {}", field),
            Error::InvalidFieldType { expected, found } => {
                write!(
                    f,
                    "Invalid field type: expected {}, found {}",
                    expected, found
                )
            }
            Error::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Lopdf(e) => Some(e),
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<lopdf::Error> for Error {
    fn from(err: lopdf::Error) -> Self {
        Error::Lopdf(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
