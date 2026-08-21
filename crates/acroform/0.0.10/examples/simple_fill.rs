use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the test PDF
    let mut doc = AcroFormDocument::from_pdf("acroform_files/af8.pdf")?;
    
    println!("Fields in the PDF:");
    for field in doc.fields()? {
        println!("  Name: {}", field.name);
        println!("    Type: {:?}", field.field_type);
        println!("    Value: {:?}", field.current_value);
        println!();
    }
    
    // Update the field value
    let mut values = HashMap::new();
    values.insert(
        "topmostSubform[0].Page1[0].P[0].MbrName[1]".to_string(),
        FieldValue::Text("NEW_VALUE".to_string()),
    );
    
    doc.fill_and_save(values, "/tmp/af8_filled.pdf")?;
    println!("Saved filled PDF to /tmp/af8_filled.pdf");
    
    Ok(())
}
