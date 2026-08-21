/// Test suite for PDF 1.5+ support
///
/// PDF 1.5 introduced compressed object streams as a space-saving feature.
/// This test ensures that the library can correctly update form fields in PDFs
/// that use compressed object streams.
///
/// Background: When a PDF uses compressed object streams, multiple objects are
/// stored together in a single compressed stream (XRef::Stream). The original
/// implementation would fail to update these objects correctly because it would
/// create a new object with a different ID instead of properly updating the
/// xref entry to point to the new uncompressed object.

use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

#[test]
fn test_pdf_16_field_update() {
    // af8_error.pdf is PDF 1.6 with compressed object streams
    let mut doc = AcroFormDocument::from_pdf("../acroform_files/af8_error.pdf")
        .expect("Failed to load PDF 1.6 file");
    
    // Find a text field to update
    let fields = doc.fields().expect("Failed to get fields");
    let test_field = fields.iter()
        .find(|f| f.name.contains("MbrName[1]"))
        .expect("Test field not found");
    
    let field_name = test_field.name.clone();
    
    // Update the field
    let mut values = HashMap::new();
    values.insert(field_name.clone(), FieldValue::Text("PDF_1.6_TEST".to_string()));
    
    let filled_bytes = doc.fill(values).expect("Failed to fill form");
    
    // Verify the update by re-opening
    let doc2 = AcroFormDocument::from_bytes(filled_bytes)
        .expect("Failed to reopen filled PDF");
    
    let fields2 = doc2.fields().expect("Failed to get fields from filled PDF");
    let updated_field = fields2.iter()
        .find(|f| f.name == field_name)
        .expect("Field not found in filled PDF");
    
    assert_eq!(
        updated_field.current_value,
        Some(FieldValue::Text("PDF_1.6_TEST".to_string())),
        "Field value was not updated correctly in PDF 1.6"
    );
}

#[test]
fn test_pdf_17_field_update() {
    // af8_clean.pdf is PDF 1.7 with compressed object streams
    let mut doc = AcroFormDocument::from_pdf("../acroform_files/af8_clean.pdf")
        .expect("Failed to load PDF 1.7 file");
    
    // Find a text field to update
    let fields = doc.fields().expect("Failed to get fields");
    let test_field = fields.iter()
        .find(|f| f.name.contains("MbrName[1]"))
        .expect("Test field not found");
    
    let field_name = test_field.name.clone();
    
    // Update the field
    let mut values = HashMap::new();
    values.insert(field_name.clone(), FieldValue::Text("PDF_1.7_TEST".to_string()));
    
    let filled_bytes = doc.fill(values).expect("Failed to fill form");
    
    // Verify the update by re-opening
    let doc2 = AcroFormDocument::from_bytes(filled_bytes)
        .expect("Failed to reopen filled PDF");
    
    let fields2 = doc2.fields().expect("Failed to get fields from filled PDF");
    let updated_field = fields2.iter()
        .find(|f| f.name == field_name)
        .expect("Field not found in filled PDF");
    
    assert_eq!(
        updated_field.current_value,
        Some(FieldValue::Text("PDF_1.7_TEST".to_string())),
        "Field value was not updated correctly in PDF 1.7"
    );
}

#[test]
fn test_pdf_13_backward_compatibility() {
    // af8.pdf is PDF 1.3 without compressed object streams
    // Ensure our fix doesn't break older PDFs
    let mut doc = AcroFormDocument::from_pdf("../acroform_files/af8.pdf")
        .expect("Failed to load PDF 1.3 file");
    
    // Find a text field to update
    let fields = doc.fields().expect("Failed to get fields");
    let test_field = fields.iter()
        .find(|f| f.name.contains("MbrName[1]"))
        .expect("Test field not found");
    
    let field_name = test_field.name.clone();
    
    // Update the field
    let mut values = HashMap::new();
    values.insert(field_name.clone(), FieldValue::Text("PDF_1.3_TEST".to_string()));
    
    let filled_bytes = doc.fill(values).expect("Failed to fill form");
    
    // Verify the update by re-opening
    let doc2 = AcroFormDocument::from_bytes(filled_bytes)
        .expect("Failed to reopen filled PDF");
    
    let fields2 = doc2.fields().expect("Failed to get fields from filled PDF");
    let updated_field = fields2.iter()
        .find(|f| f.name == field_name)
        .expect("Field not found in filled PDF");
    
    assert_eq!(
        updated_field.current_value,
        Some(FieldValue::Text("PDF_1.3_TEST".to_string())),
        "Field value was not updated correctly in PDF 1.3"
    );
}
