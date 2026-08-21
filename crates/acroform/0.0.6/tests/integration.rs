//! Integration tests for the acroform crate

use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

#[test]
fn test_api_compiles() {
    // Basic compilation test
}

#[test]
fn test_field_value_display() {
    let text = FieldValue::Text("Hello".to_string());
    assert_eq!(text.to_string(), "Hello");

    let boolean = FieldValue::Boolean(true);
    assert_eq!(boolean.to_string(), "true");

    let integer = FieldValue::Integer(42);
    assert_eq!(integer.to_string(), "42");

    let choice = FieldValue::Choice("Option1".to_string());
    assert_eq!(choice.to_string(), "Option1");
}

#[test]
fn test_field_value_equality() {
    let text1 = FieldValue::Text("Hello".to_string());
    let text2 = FieldValue::Text("Hello".to_string());
    assert_eq!(text1, text2);

    let text3 = FieldValue::Text("World".to_string());
    assert_ne!(text1, text3);
}

// Note: Testing with real PDFs would require sample form PDFs in the repo
// For now, we keep tests minimal to demonstrate the API works
#[test]
#[ignore] // Ignored until we have a sample PDF with forms
fn test_load_pdf_with_forms() {
    // This test would load a real PDF and verify fields can be read
    // Example:
    // let doc = AcroFormDocument::from_pdf("../files/sample_form.pdf").unwrap();
    // let fields = doc.fields();
    // assert!(!fields.is_empty());
}

#[test]
#[ignore] // Ignored until we have a sample PDF with forms
fn test_fill_and_save() {
    // This test would fill a form and save it
    // Example:
    // let mut doc = AcroFormDocument::from_pdf("../files/sample_form.pdf").unwrap();
    // let mut values = HashMap::new();
    // values.insert("field_name".to_string(), FieldValue::Text("value".to_string()));
    // doc.fill_and_save(values, "/tmp/output.pdf").unwrap();
}
