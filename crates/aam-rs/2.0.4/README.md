# AAM (Abstract Alias Mapping)

A robust and lightweight configuration library for Rust built around the new pipeline-backed `AAM` API.
It parses `.aam` files (`key = value`), supports directives (`@import`, `@derive`, `@schema`, `@type`), and provides
fast query and formatting utilities.

> **The Origin Story:** AAM was born out of necessity during the development of [rustgames](https://github.com/ininids/rustgames). I needed a configuration format that lived entirely outside the codebase, supported high-speed bidirectional search, and was optimized for extreme performance. What started as a specialized tool for game engine internals eventually evolved into a robust configuration ecosystem with schemas, imports, and AOT compilation.
> 
## Why AAM?

AAM was designed to solve the "configuration fatigue" in large-scale Rust projects. While formats like TOML are great for simple key-value pairs, they often fall short when your config grows. AAM introduces:

* **Type Safety & Schemas:** Define `@type` aliases and `@schema` structures directly in the config. No more "guessing" what a value should be.
* **Modular Architecture:** Use `@import` to split massive configs into clean, manageable modules.
* **AOT Performance:** Optional Ahead-of-Time compilation "cooks" your `.aam` files into a binary format for near-instant loading.
* **Developer-Centric:** Built-in LSP support and formatting help you catch errors *before* you run the code.

### AAM vs TOML

| Feature | AAM | TOML |
| :--- | :--- | :--- |
| **Schema Validation** | Native (`@schema`) | External tools only |
| **Modular Imports** | Native (`@import`) | Not supported |
| **Type Aliasing** | Native (`@type`) | No |
| **Performance** | High (AOT/Binary) | Standard (Text parsing) |
| **Extensibility** | Pipeline-backed | Static |

## What changed in 2.x

- `AAM` is now the primary API.
- Parsing/loading returns `Result<_, Vec<AamlError>>` to preserve full diagnostics.
- Query methods are now centered around `get`, `find`, `find_by`, `deep_search`, and `reverse_search`.
- Pipeline formatter and LSP helper methods are available directly on `AAM`.
- `AAML` remains available for backward compatibility, but is deprecated.

## Features

- Simple `key = value` syntax.
- Directive support: `@import`, `@derive`, `@schema`, `@type`.
- Schema/type validation via pipeline.
- Search helpers for key lookup, reverse lookup, and predicate filtering.
- Formatter utilities (`format`, `format_range`) and LSP assist (`lsp_assist`).
- Optional AOT loading (`.aam.bin`) for fast startup (`aot` feature; enabled by default).
- Fluent config generation with `AAMBuilder`.

## Format

You can find syntax documentation and examples at:
https://aam.ininids.in.rs/

## Installation

Add the crate to your `Cargo.toml`:

<!-- x-release-please-start-version -->
```toml
[dependencies]
aam-rs = "2.0.4"
```
<!-- x-release-please-start-version -->

## Configuration syntax (`.aam`)

```aam
# Comments are supported
host = "localhost"
port = 8080

@import database.aam
@import theme.aam

@type port_t = i32
@schema Service {
    host: string
    port: port_t
}
```

## Usage (new `AAM` API)

### 1) Parse and load

```rust
use aam_rs::aam::AAM;
use aam_rs::error::AamlError;

fn print_errors(errors: &[AamlError]) {
    for err in errors {
        eprintln!("{err}");
    }
}

fn main() {
    let inline = "host = localhost\nport = 8080";

    match AAM::parse(inline) {
        Ok(cfg) => {
            if let Some(host) = cfg.get("host") {
                println!("host = {host}");
            }
        }
        Err(errors) => {
            print_errors(&errors);
        }
    }
}
```

### 2) Lookups and queries

```rust
use aam_rs::aam::AAM;

let cfg = AAM::parse(
    "
app_mode = production
backup_mode = production
api_port = 8080
"
).unwrap();

// O(1) key lookup
assert_eq!(cfg.get("api_port"), Some("8080"));

// key-first lookup, then reverse lookup by value
let found = cfg.find("production");
assert_eq!(found.len(), 2);

// reverse lookup only
let keys = cfg.reverse_search("production");
assert_eq!(keys.len(), 2);

// key pattern search
let deep = cfg.deep_search("mode");
assert_eq!(deep.len(), 2);

// custom predicate
let filtered = cfg.find_by(|k, v| k.ends_with("_port") && v == "8080");
assert_eq!(filtered, vec![("api_port", "8080")]);
```

### 3) Iteration and extraction

```rust
use aam_rs::aam::AAM;

let cfg = AAM::parse("a = 1\nb = 2").unwrap();

for (k, v) in cfg.iter() {
    println!("{k} = {v}");
}

let keys = cfg.keys();
let map = cfg.to_map();
assert!(keys.contains(&"a"));
assert_eq!(map.get("b"), Some(&"2".to_string()));
```

### 4) Formatting and LSP helper

```rust
use aam_rs::aam::AAM;
use aam_rs::pipeline::{FormatRange, FormattingOptions};

let cfg = AAM::new();
let src = "host=localhost\nport=8080";

let formatted = cfg.format(src, &FormattingOptions::default()).unwrap();
let _range = cfg
    .format_range(
        src,
        FormatRange { start_line: 1, end_line: 1 },
        &FormattingOptions::default(),
    )
    .unwrap();

let assist = AAM::lsp_assist(&formatted, &FormattingOptions::default());
assert!(assist.diagnostics.is_empty());
```

### 5) Builder (`AAMBuilder`)

```rust
use aam_rs::builder::{AAMBuilder, SchemaField};

let mut builder = AAMBuilder::new();
builder
    .comment("Server configuration")
    .type_alias("port_t", "i32")
    .schema("Server", [
        SchemaField::required("host", "string"),
        SchemaField::required("port", "port_t"),
        SchemaField::optional("debug", "bool"),
    ])
    .add_line("host", "2.0.4.1")
    .add_line("port", "8080");

println!("{}", builder.as_string());
builder.to_file("generated_config.aam").unwrap();
```

## Legacy API (`AAML`, deprecated)

`AAML` is still available for compatibility and supports methods such as:

- `parse`, `load`
- `merge_content`, `merge_file`
- `find_obj`, `find_key`, `find_deep`
- `validate_value`, `apply_schema`, `validate_schemas_completeness`

For migration guidance, see `docs/AAML_TO_AAM_MIGRATION.md`.

## Bindings

### Node.js / N-API

```bash
npm install aam-nodejs
```

### C# / .NET

```bash
dotnet add package aam-csharp
```

## Quick verification commands

```bash
cargo test
cargo run --example standard
cargo run --example advanced
```

## API reference (high-level)

### `AAM`

- Constructors/loaders: `new`, `parse`, `load`, `from_pipeline`
- Query: `get`, `find`, `find_by`, `deep_search`, `reverse_search`
- Iteration/export: `iter`, `keys`, `to_map`
- Introspection: `schemas`, `get_schema`, `types`, `get_type`
- Formatting/LSP: `format`, `format_range`, `lsp_assist`
- AOT (feature-gated): `cook`, `load_fast`

### `AAMBuilder`

- `new`, `with_capacity`
- `add_line`, `comment`
- `schema`, `schema_multiline`, `derive`, `import`, `type_alias`
- `to_file`, `build`, `as_string`

### `AamlError`

Typed errors used across parser, validator, and runtime paths.

## Ecosystem Tooling

### AAM CLI
For managing, formatting, and "cooking" your configuration files, check out the [aam-cli](https://github.com/ininids/aam-cli).

**Installation:**
```bash
cargo install aam-cli
```
Key features:
 * Cook: Convert .aam to binary .aam.bin for AOT loading.
 * Format: Keep your config files clean and consistent.
 * Check: Validate syntax and schema integrity from the terminal.

### AAM Examples
If you need more examples, check [aam-examples repository](https://github.com/ininids/aam-examples)

## License

See `LICENSE-MIT` and `LICENSE-APACHE`.

## Full documentation

- Docs.rs: https://docs.rs/aam-rs/
- Project docs: https://aam.ininids.in.rs/

