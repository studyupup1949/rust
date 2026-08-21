use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a PDF with form fields
    let mut doc = AcroFormDocument::from_pdf("tests/af8.pdf")?;

    // List all fields in the document
    println!("Fields in the document:");
    let fields = doc.fields()?;
    for field in &fields {
        println!("  - {} ({})", field.name, field.field_type);
        if let Some(ref value) = field.current_value {
            println!("    Current value: {:?}", value);
        }
    }

    // Fill fields with new values
    let mut values = HashMap::new();

    // Find and fill the first text field
    if let Some(field) = fields
        .iter()
        .find(|f| matches!(f.field_type, acroform::FieldType::Text))
    {
        println!("\nFilling field '{}' with new value", field.name);
        values.insert(
            field.name.clone(),
            FieldValue::Text("NEW_VALUE".to_string()),
        );
    }

    // Save the filled PDF
    if !values.is_empty() {
        doc.fill_and_save(values, "/tmp/filled_form.pdf")?;
        println!("\nFilled PDF saved to /tmp/filled_form.pdf");
    }

    Ok(())
}
