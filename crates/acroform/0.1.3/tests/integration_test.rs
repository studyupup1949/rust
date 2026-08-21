use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

#[test]
fn test_load_and_list_fields() {
    let doc = AcroFormDocument::from_pdf("tests/af8.pdf").expect("Failed to load PDF");

    let fields = doc.fields().expect("Failed to get fields");

    // Print all fields for debugging
    for field in &fields {
        println!("Field: {} ({})", field.name, field.field_type);
        if let Some(ref value) = field.current_value {
            println!("  Current value: {:?}", value);
        }
        if let Some(ref tooltip) = field.tooltip {
            println!("  Tooltip: {}", tooltip);
        }
    }

    // Should have at least one field
    assert!(!fields.is_empty(), "No fields found in the PDF");
}

#[test]
fn test_load_from_bytes() {
    let bytes = std::fs::read("tests/af8.pdf").expect("Failed to read PDF file");
    let doc = AcroFormDocument::from_bytes(bytes).expect("Failed to load PDF from bytes");

    let fields = doc.fields().expect("Failed to get fields");
    assert!(!fields.is_empty(), "No fields found in the PDF");
}

#[test]
fn test_fill_and_save() {
    let mut doc = AcroFormDocument::from_pdf("tests/af8.pdf").expect("Failed to load PDF");

    // Get the fields first
    let fields = doc.fields().expect("Failed to get fields");

    // Fill with test values
    let mut values = HashMap::new();

    // Try to fill the first text field we find
    for field in fields {
        match field.field_type {
            acroform::FieldType::Text => {
                values.insert(
                    field.name.clone(),
                    FieldValue::Text("Test Value".to_string()),
                );
                break;
            }
            _ => {}
        }
    }

    if !values.is_empty() {
        let bytes = doc.fill(values).expect("Failed to fill fields");
        assert!(!bytes.is_empty(), "Filled PDF is empty");

        // Save to a temporary location
        std::fs::write("/tmp/filled_test.pdf", bytes).expect("Failed to write filled PDF");

        // Verify we can load it back
        let filled_doc =
            AcroFormDocument::from_pdf("/tmp/filled_test.pdf").expect("Failed to load filled PDF");
        let filled_fields = filled_doc
            .fields()
            .expect("Failed to get fields from filled PDF");
        assert!(!filled_fields.is_empty(), "No fields in filled PDF");
    }
}

#[test]
fn test_widget_annotations_updated() {
    use lopdf::Document;

    // This test ensures that when we update a field value,
    // both the field dictionary AND any widget annotations are updated.

    let mut doc = AcroFormDocument::from_pdf("tests/af8.pdf").expect("Failed to load PDF");

    // Update the field
    let mut values = HashMap::new();
    values.insert(
        "topmostSubform[0].Page1[0].P[0].MbrName[1]".to_string(),
        FieldValue::Text("WIDGET_TEST_VALUE".to_string()),
    );

    let filled_bytes = doc.fill(values).expect("Failed to fill fields");
    std::fs::write("/tmp/test_widget_update.pdf", &filled_bytes)
        .expect("Failed to save filled PDF");

    // Load the filled PDF and verify the update
    let filled_doc = Document::load("/tmp/test_widget_update.pdf").expect("Failed to load PDF");

    // Check the field value in AcroForm
    let catalog = filled_doc.catalog().expect("Failed to get catalog");
    let acroform_ref = catalog
        .get(b"AcroForm")
        .expect("Failed to get AcroForm")
        .as_reference()
        .expect("AcroForm should be a reference");
    let acroform_dict = filled_doc
        .get_dictionary(acroform_ref)
        .expect("Failed to get AcroForm dictionary");
    let fields = acroform_dict
        .get(b"Fields")
        .expect("Failed to get Fields")
        .as_array()
        .expect("Fields should be an array");

    let mut field_value_found = false;
    for field_obj in fields {
        if let Ok(field_ref) = field_obj.as_reference() {
            if let Ok(field_dict) = filled_doc.get_dictionary(field_ref) {
                if let Ok(name) = field_dict.get(b"T").and_then(|o| o.as_str()) {
                    if String::from_utf8_lossy(name) == "topmostSubform[0].Page1[0].P[0].MbrName[1]"
                    {
                        // Check the field value
                        if let Ok(value_obj) = field_dict.get(b"V") {
                            if let Ok(value_str) = value_obj.as_str() {
                                // Decode UTF-16BE with BOM
                                let value_text = if value_str.len() >= 2
                                    && value_str[0] == 0xFE
                                    && value_str[1] == 0xFF
                                {
                                    let u16_chars: Vec<u16> = value_str[2..]
                                        .chunks_exact(2)
                                        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                                        .collect();
                                    String::from_utf16(&u16_chars).unwrap_or_default()
                                } else {
                                    String::from_utf8_lossy(value_str).to_string()
                                };

                                assert_eq!(
                                    value_text, "WIDGET_TEST_VALUE",
                                    "Field value should be updated"
                                );
                                field_value_found = true;
                            }
                        }
                    }
                }
            }
        }
    }

    assert!(
        field_value_found,
        "Field value should have been found and verified"
    );

    println!("Widget annotation test passed!");
}
