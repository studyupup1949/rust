use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load PDF from disk as bytes
    let pdf_data = std::fs::read("acroform_files/af8.pdf")?;
    println!("Loaded {} bytes from disk", pdf_data.len());
    
    // Parse PDF from bytes (in-memory)
    let mut doc = AcroFormDocument::from_bytes(pdf_data)?;
    
    println!("\nFields in the PDF:");
    for field in doc.fields()? {
        println!("  Name: {}", field.name);
        println!("    Type: {:?}", field.field_type);
        println!("    Value: {:?}", field.current_value);
        println!();
    }
    
    // Fill the form (in-memory)
    let mut values = HashMap::new();
    values.insert(
        "topmostSubform[0].Page1[0].P[0].MbrName[1]".to_string(),
        FieldValue::Text("FILLED_IN_MEMORY".to_string()),
    );
    
    // Get filled PDF as bytes (no disk I/O!)
    let filled_bytes = doc.fill(values)?;
    println!("Filled PDF is {} bytes", filled_bytes.len());
    
    // At this point, you could:
    // 1. Send filled_bytes over HTTP
    // 2. Store in a database
    // 3. Process further in-memory
    // 4. Or write to disk if needed
    
    // For demonstration, let's write to disk and verify
    std::fs::write("/tmp/af8_filled_in_memory.pdf", &filled_bytes)?;
    println!("Saved filled PDF to /tmp/af8_filled_in_memory.pdf");
    
    // Verify by loading again from bytes
    let verification_doc = AcroFormDocument::from_bytes(filled_bytes)?;
    let fields = verification_doc.fields()?;
    let updated_field = fields.iter()
        .find(|f| f.name == "topmostSubform[0].Page1[0].P[0].MbrName[1]")
        .expect("Field not found");
    
    println!("\nVerification:");
    println!("  Field value after round-trip: {:?}", updated_field.current_value);
    
    Ok(())
}
