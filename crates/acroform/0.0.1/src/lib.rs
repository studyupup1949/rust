/*!
# acroform - Minimal PDF Form Manipulation

A minimal, auditable PDF form manipulation library focusing on:
- Reading PDF forms
- Listing form fields
- Updating field values
- Saving modified PDFs

## Example

```rust,no_run
use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

// Load a PDF
let mut doc = AcroFormDocument::from_pdf("form.pdf").unwrap();

// List fields
for field in doc.fields().unwrap() {
    println!("Field: {} = {:?}", field.name, field.current_value);
}

// Fill fields
let mut values = HashMap::new();
values.insert("firstName".to_string(), FieldValue::Text("John".to_string()));
values.insert("lastName".to_string(), FieldValue::Text("Doe".to_string()));

doc.fill_and_save(values, "filled_form.pdf").unwrap();
```

## Design Principles

- **Minimal**: Only form filling, no rendering or appearance generation
- **Auditable**: Small codebase, easy to review
- **Standards-compliant**: Relies on PDF viewers for appearance generation via NeedAppearances flag
- **Non-incremental**: Always writes complete PDF, not incremental updates
*/

mod field;
mod api;

pub use api::{AcroFormDocument, FormField, FieldValue};
pub use field::{FieldDictionaryExt, InteractiveFormDictionaryExt};

// Re-export commonly used types from pdf crate
pub use pdf::error::PdfError;
pub use pdf::object::FieldType;
