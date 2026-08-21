use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

#[test]
fn test_load_and_list_fields() {
    let doc = AcroFormDocument::from_pdf("../acroform_files/af8.pdf")
        .expect("Failed to load PDF");
    
    let fields = doc.fields().expect("Failed to get fields");
    
    // The test PDF should have at least one field
    assert!(!fields.is_empty(), "Expected at least one field in the PDF");
    
    // Check that we can find the expected field
    let field_names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
    assert!(
        field_names.contains(&"topmostSubform[0].Page1[0].P[0].MbrName[1]".to_string()),
        "Expected to find MbrName field"
    );
}

#[test]
fn test_fill_and_save() {
    let mut doc = AcroFormDocument::from_pdf("../acroform_files/af8.pdf")
        .expect("Failed to load PDF");
    
    // Get initial field value
    let fields = doc.fields().expect("Failed to get fields");
    let initial_field = fields.iter()
        .find(|f| f.name == "topmostSubform[0].Page1[0].P[0].MbrName[1]")
        .expect("Field not found");
    
    assert_eq!(
        initial_field.current_value,
        Some(FieldValue::Text("OLD_VALUE".to_string()))
    );
    
    // Update the field
    let mut values = HashMap::new();
    values.insert(
        "topmostSubform[0].Page1[0].P[0].MbrName[1]".to_string(),
        FieldValue::Text("TEST_VALUE".to_string()),
    );
    
    doc.fill_and_save(values, "/tmp/test_filled.pdf")
        .expect("Failed to save PDF");
    
    // Reload and verify
    let doc2 = AcroFormDocument::from_pdf("/tmp/test_filled.pdf")
        .expect("Failed to reload PDF");
    
    let fields2 = doc2.fields().expect("Failed to get fields");
    let updated_field = fields2.iter()
        .find(|f| f.name == "topmostSubform[0].Page1[0].P[0].MbrName[1]")
        .expect("Field not found after update");
    
    assert_eq!(
        updated_field.current_value,
        Some(FieldValue::Text("TEST_VALUE".to_string()))
    );
}

#[test]
fn test_nonexistent_field() {
    let mut doc = AcroFormDocument::from_pdf("../acroform_files/af8.pdf")
        .expect("Failed to load PDF");
    
    let mut values = HashMap::new();
    values.insert(
        "nonexistent.field".to_string(),
        FieldValue::Text("value".to_string()),
    );
    
    // Should not error when trying to update a nonexistent field
    // (it just won't update anything)
    doc.fill_and_save(values, "/tmp/test_nonexistent.pdf")
        .expect("Should not error on nonexistent field");
}
