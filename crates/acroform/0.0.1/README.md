# acroform - Minimal PDF Form Manipulation

A minimal, auditable PDF form manipulation library focusing on:
- Reading PDF forms
- Listing form fields
- Updating field values
- Saving modified PDFs

## Design Principles

- **Minimal**: Only form filling, no rendering or appearance generation
- **Auditable**: Small codebase, easy to review
- **Standards-compliant**: Relies on PDF viewers for appearance generation via NeedAppearances flag
- **Non-incremental**: Always writes complete PDF, not incremental updates
- **Separation of Concerns**: Built as a separate crate on top of the forked `pdf` crate

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
acroform = { path = "acroform" }
```

## Usage

### Three-Step API

```rust
use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

// Step 1: Load a PDF
let mut doc = AcroFormDocument::from_pdf("form.pdf")?;

// Step 2: List fields
for field in doc.fields()? {
    println!("Field: {} = {:?}", field.name, field.current_value);
}

// Step 3: Fill and save
let mut values = HashMap::new();
values.insert("firstName".to_string(), FieldValue::Text("John".to_string()));
values.insert("lastName".to_string(), FieldValue::Text("Doe".to_string()));

doc.fill_and_save(values, "filled_form.pdf")?;
```

### Field Types

The library supports the following field value types:

- `FieldValue::Text(String)` - Text fields
- `FieldValue::Boolean(bool)` - Checkboxes
- `FieldValue::Choice(String)` - Radio buttons and dropdowns
- `FieldValue::Integer(i32)` - Integer fields

### Field Names

Field names are automatically resolved with full hierarchical names (e.g., `parent.child.field`).
The library handles nested field structures internally and presents a flat list of terminal fields.

## Example

See `acroform/examples/simple_fill.rs` for a complete working example:

```bash
cargo run --example simple_fill -p acroform
```

## Architecture

The `acroform` crate is built as a separate layer on top of the forked `pdf` crate:

- **pdf/**: Forked PDF parsing and manipulation library (NOT MODIFIED)
- **acroform/**: Form-specific functionality (NEW)
  - `field.rs`: Extension traits for field traversal
  - `api.rs`: High-level form filling API
  - `lib.rs`: Public exports and documentation

This design allows the underlying `pdf` crate to remain unmodified, making it easy to merge upstream updates.

## Testing

Run the test suite:

```bash
cargo test -p acroform
```

Test files are located in `acroform_files/`:
- `af8.pdf` - Test PDF with a single text field

## Implementation Status

✅ Load PDF with AcroForm  
✅ List all fillable fields with names and types  
✅ Update text field values  
✅ Update checkbox/radio button states  
✅ Save modified PDF  
✅ Generated PDFs show updated values when opened  

## Non-Goals

The library explicitly does NOT support:
- PDF rendering or visual preview
- Incremental updates (linearized PDFs)
- Appearance stream generation
- Digital signature creation/validation
- XFA form support
- Interactive JavaScript evaluation
- PDF creation from scratch
- Form field creation/deletion

## License

MIT
