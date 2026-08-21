# acroform

A high-level PDF form manipulation library using [lopdf](https://github.com/J-F-Liu/lopdf).

This crate provides a simple API for reading and filling PDF forms (AcroForms). It uses the official `lopdf` crate for PDF operations.

## Features

- Load PDF documents from files or bytes
- List all form fields with their properties
- Fill form fields with typed values
- Save filled PDFs to files or bytes
- Support for text, boolean, choice, and integer field types
- Automatic UTF-16BE encoding for text fields
- Hierarchical field name resolution

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
acroform = "0.1.0"
```

## Example

```rust
use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a PDF with form fields
    let mut doc = AcroFormDocument::from_pdf("form.pdf")?;

    // List all fields
    let fields = doc.fields()?;
    for field in &fields {
        println!("Field: {} ({})", field.name, field.field_type);
    }

    // Fill fields
    let mut values = HashMap::new();
    values.insert("name".to_string(), FieldValue::Text("John Doe".to_string()));
    values.insert("age".to_string(), FieldValue::Integer(30));
    values.insert("subscribe".to_string(), FieldValue::Boolean(true));

    // Save filled PDF
    doc.fill_and_save(values, "filled_form.pdf")?;
    
    Ok(())
}
```

## API

### `AcroFormDocument`

The main struct for working with PDF forms.

#### Methods

- `from_pdf(path: impl AsRef<Path>) -> Result<Self>` - Load a PDF from a file path
- `from_bytes(data: Vec<u8>) -> Result<Self>` - Load a PDF from bytes
- `fields(&self) -> Result<Vec<FormField>>` - Get all form fields
- `fill(&mut self, values: HashMap<String, FieldValue>) -> Result<Vec<u8>>` - Fill fields and return PDF bytes
- `fill_and_save(&mut self, values: HashMap<String, FieldValue>, output: impl AsRef<Path>) -> Result<()>` - Fill fields and save to file

### `FormField`

Represents a form field with its properties.

#### Fields

- `name: String` - The fully qualified field name (e.g., "parent.child.field")
- `field_type: FieldType` - The type of the field
- `current_value: Option<FieldValue>` - The current value
- `default_value: Option<FieldValue>` - The default value
- `flags: u32` - Field flags (bit field)
- `tooltip: Option<String>` - Tooltip text

### `FieldValue`

Typed values for form fields.

#### Variants

- `Text(String)` - Text string
- `Boolean(bool)` - Boolean value (for checkboxes)
- `Choice(String)` - Choice value (for radio buttons, list boxes, combo boxes)
- `Integer(i32)` - Integer value

### `FieldType`

PDF form field types.

#### Variants

- `Text` - Text field (/Tx)
- `Button` - Button field (/Btn) - includes checkboxes, radio buttons, and push buttons
- `Choice` - Choice field (/Ch) - includes list boxes and combo boxes
- `Signature` - Signature field (/Sig)
- `Unknown(String)` - Unknown or custom field type

## Implementation Details

### String Encoding

Text fields are automatically encoded as UTF-16BE with BOM when filling forms, which ensures proper Unicode support across PDF viewers.

### Field Hierarchy

Fields can be organized hierarchically in PDF forms. This library constructs fully qualified field names using dot notation (e.g., "parent.child.field").

### Appearance Updates

When filling forms, the library sets the `NeedAppearances` flag, which tells PDF viewers to regenerate field appearances. This ensures that filled values are properly displayed.

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
