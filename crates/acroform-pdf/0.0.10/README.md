# acroform-rs

A minimal, auditable PDF form manipulation library forked from [pdf-rs](https://github.com/pdf-rs/pdf).

This repository contains:
- **pdf/**: Forked PDF parsing and manipulation library
- **acroform/**: High-level form filling API (NEW)

## Quick Start

### File-based API

```rust
use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

// Load a PDF
let mut doc = AcroFormDocument::from_pdf("form.pdf")?;

// List fields
for field in doc.fields()? {
    println!("Field: {} = {:?}", field.name, field.current_value);
}

// Fill and save
let mut values = HashMap::new();
values.insert("firstName".to_string(), FieldValue::Text("John".to_string()));
doc.fill_and_save(values, "filled_form.pdf")?;
```

### In-Memory API (NEW!)

The high-level API now performs all operations in-memory, returning byte vectors instead of writing to disk:

```rust
use acroform::{AcroFormDocument, FieldValue};
use std::collections::HashMap;

// Load from bytes
let pdf_data = std::fs::read("form.pdf")?;
let mut doc = AcroFormDocument::from_bytes(pdf_data)?;

// Fill fields and get result as bytes (no disk I/O!)
let mut values = HashMap::new();
values.insert("firstName".to_string(), FieldValue::Text("John".to_string()));
let filled_pdf_bytes = doc.fill(values)?;

// Use the bytes directly (e.g., send over HTTP, store in database, etc.)
// Or write to disk if needed
std::fs::write("filled_form.pdf", filled_pdf_bytes)?;
```

See [acroform/README.md](acroform/README.md) for detailed documentation.

## Original README (pdf-rs)

Read, alter and write PDF files.

Modifying and writing PDFs is still experimental.

One easy way you can contribute is to add different PDF files to `tests/files` and see if they pass the tests (`cargo test`).

Feel free to contribute with ideas, issues or code! Please join [us on Zulip](https://type.zulipchat.com/#narrow/stream/209232-pdf) if you have any questions or problems.

# Workspace
This repository uses a Cargo Workspace and default members. This means by default only the `pdf` library is build.
To build additional parts, pass `--package=read` to build the subcrate you are interested in (here the `read` example).

# Examples
Examples are located in `pdf/examples/` and can be executed using:

```
cargo run --example {content,metadata,names,read,text} -- <files/{choose a pdf}>
```

# Renderer and Viewer
A library for rendering PDFs via [Pathfinder](https://github.com/servo/pathfinder) and minimal viewer can be found [here](https://github.com/pdf-rs/pdf_render).

# Inspect
There is a tool for visualizing a PDF file as an interactive hierarchy of primitives at [inspect-prim](https://github.com/pdf-rs/inspect-prim). Just clone and `cargo run`.
