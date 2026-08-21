# acroform-rs [![test](https://github.com/pdf-rs/pdf/actions/workflows/test.yml/badge.svg)](https://github.com/pdf-rs/pdf/actions/workflows/test.yml) 
Read, alter and write PDF files with AcroForm support.

This repository contains a fork of [pdf-rs](https://github.com/pdf-rs/pdf) (published as `acroform-pdf`) with added high-level AcroForm (PDF form) manipulation capabilities.

## Features

- **Low-level PDF manipulation** via the `acroform-pdf` crate (fork of pdf-rs with bug fixes)
- **High-level form filling** via the new `acroform` crate
- Simple three-step API: load, list fields, fill and save
- Type-safe field values
- Minimal dependencies and auditable code (KISS principle)

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
acroform = "0.0.3"
```

For low-level PDF manipulation:

```toml
[dependencies]
acroform-pdf = "0.9.0"  # Low-level PDF library
acroform = "0.0.3"       # High-level form filling
```

### Basic Example

```rust
use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a PDF with forms
    let mut doc = AcroFormDocument::from_pdf("input.pdf")?;

    // List available fields
    for field in doc.fields() {
        println!("{}: {:?}", field.name, field.field_type);
    }

    // Fill fields
    let mut values = HashMap::new();
    values.insert("name".to_string(), FieldValue::Text("John Doe".to_string()));
    values.insert("age".to_string(), FieldValue::Integer(30));
    values.insert("agree".to_string(), FieldValue::Boolean(true));

    // Save the filled PDF
    doc.fill_and_save(values, "output.pdf")?;
    Ok(())
}
```

## Architecture

This repository contains two main publishable crates:

1. **`acroform-pdf`** (in `pdf/` directory) - Fork of pdf-rs with bug fixes, provides low-level PDF parsing and writing
2. **`acroform`** (in `acroform/` directory) - NEW high-level API for form manipulation (depends on `acroform-pdf`)

This separation allows:
- Publishing both crates to crates.io
- Easy merging of upstream pdf-rs updates
- Clean separation between forked and new code
- Use of either low-level or high-level APIs as needed

## Examples

Run the simple form filling example:

```bash
cargo run --bin simple_fill -- input.pdf output.pdf
```

See more examples in `examples/src/bin/`:
- `simple_fill.rs` - High-level form filling API
- `form.rs` - Low-level form manipulation example

## Testing

Run tests:

```bash
cargo test                         # Run all tests
cargo test -p acroform             # Run only acroform tests
cargo test -p acroform-pdf         # Run only acroform-pdf tests
```

## Documentation

See [`designs/PLAN.md`](designs/PLAN.md) for detailed architecture and design decisions.

## Limitations

- Non-incremental PDF writing only (full rewrite)
- No appearance stream generation (relies on viewer's `/NeedAppearances` support)
- No digital signature support
- No XFA form support

## Contributing

Modifying and writing PDFs is still experimental.

One easy way you can contribute is to add different PDF files to `tests/files` and see if they pass the tests (`cargo test`).

Feel free to contribute with ideas, issues or code! Please join [us on Zulip](https://type.zulipchat.com/#narrow/stream/209232-pdf) if you have any questions or problems.

# Workspace
This repository uses a Cargo Workspace. The workspace contains:
- `acroform-pdf` - Low-level PDF library (fork of pdf-rs)
- `acroform-pdf-derive` - Derive macros for acroform-pdf
- `acroform` - High-level form filling API
- `examples` - Example applications

To build specific crates:
```bash
cargo build --package acroform-pdf
cargo build --package acroform
```

# Examples
Examples are located in `pdf/examples/` and `examples/src/bin/` and can be executed using:

```
cargo run --example {content,metadata,names,read,text} -- <files/{choose a pdf}>
cargo run --bin {simple_fill,form} -- <input.pdf> [output.pdf]
```

# Renderer and Viewer
A library for rendering PDFs via [Pathfinder](https://github.com/servo/pathfinder) and minimal viewer can be found [here](https://github.com/pdf-rs/pdf_render).

# Inspect
There is a tool for visualizing a PDF file as an interactive hierarchy of primitives at [inspect-prim](https://github.com/pdf-rs/inspect-prim). Just clone and `cargo run`.
