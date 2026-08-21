//! acroform - High-level PDF form manipulation library
//!
//! This crate provides a simple, type-safe API for filling PDF forms.
//! It wraps the low-level `acroform-pdf` crate with a high-level interface.
//!
//! # Example
//!
//! ```no_run
//! use acroform::{AcroFormDocument, FieldValue};
//! use std::collections::HashMap;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load a PDF with forms
//! let mut doc = AcroFormDocument::from_pdf("input.pdf")?;
//!
//! // List available fields
//! for field in doc.fields() {
//!     println!("{}: {:?}", field.name, field.field_type);
//! }
//!
//! // Fill and save
//! let mut values = HashMap::new();
//! values.insert("name".to_string(), FieldValue::Text("John Doe".to_string()));
//! values.insert("agree".to_string(), FieldValue::Boolean(true));
//! doc.fill_and_save(values, "output.pdf")?;
//! # Ok(())
//! # }
//! ```

mod api;
mod field;
mod types;

pub use api::AcroFormDocument;
pub use field::{FieldTraversal, FormField};
pub use types::FieldValue;

// Re-export the pdf crate as `pdf` for backwards compatibility
pub use acroform_pdf as pdf;

// Re-export commonly needed types from pdf crate
pub use acroform_pdf::error::PdfError;
pub use acroform_pdf::object::FieldType;
