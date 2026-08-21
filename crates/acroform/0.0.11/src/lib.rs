/*!
# acroform - PDF Form Manipulation

A library for reading and filling PDF forms programmatically.

## Quick Start

### File-based API

```rust,no_run
use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

// Load a PDF with a form
let mut doc = AcroFormDocument::from_pdf("form.pdf").unwrap();

// List all form fields
for field in doc.fields().unwrap() {
    println!("Field: {} = {:?}", field.name, field.current_value);
}

// Fill in form fields and save to disk
let mut values = HashMap::new();
values.insert("firstName".to_string(), FieldValue::Text("John".to_string()));
values.insert("lastName".to_string(), FieldValue::Text("Doe".to_string()));
doc.fill_and_save(values, "filled_form.pdf").unwrap();
```

### In-Memory API

All operations can be performed in-memory without disk I/O:

```rust,no_run
use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

// Load from bytes
let pdf_data = std::fs::read("form.pdf").unwrap();
let mut doc = AcroFormDocument::from_bytes(pdf_data).unwrap();

// Fill fields and get result as bytes (no disk I/O!)
let mut values = HashMap::new();
values.insert("firstName".to_string(), FieldValue::Text("John".to_string()));
values.insert("lastName".to_string(), FieldValue::Text("Doe".to_string()));
let filled_pdf_bytes = doc.fill(values).unwrap();

// Now you can send filled_pdf_bytes over HTTP, store in a database, etc.
```

## Working with Form Fields

This library supports the following field types:
- **Text fields** - Use `FieldValue::Text(String)`
- **Checkboxes** - Use `FieldValue::Boolean(bool)`
- **Radio buttons and dropdowns** - Use `FieldValue::Choice(String)`
- **Number fields** - Use `FieldValue::Integer(i32)`

Field names are fully qualified (e.g., `"parent.child.field"`) and automatically
resolved for you, even in forms with nested field hierarchies.
*/

mod field;
mod api;

pub use api::{AcroFormDocument, FormField, FieldValue};
pub use field::{FieldDictionaryExt, InteractiveFormDictionaryExt};

// Re-export commonly used types from pdf crate
pub use pdf::error::PdfError;
pub use pdf::object::FieldType;
