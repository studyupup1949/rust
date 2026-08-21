<div align="center">
  <img alt="rust logo" src="https://github.com/user-attachments/assets/79881f0c-dbed-4e76-b347-40da3e3ecd97" />
<h1>Adapters</h1>
<a href="https://muhammad-fiaz.github.io/adapters/"><img src="https://img.shields.io/badge/docs-muhammad--fiaz.github.io-blue" alt="Documentation"></a>
<a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-Edition%202024-orange.svg?logo=rust" alt="Rust Edition"></a>
<a href="https://github.com/muhammad-fiaz/adapters"><img src="https://img.shields.io/github/stars/muhammad-fiaz/adapters" alt="GitHub stars"></a>
<a href="https://github.com/muhammad-fiaz/adapters/issues"><img src="https://img.shields.io/github/issues/muhammad-fiaz/adapters" alt="GitHub issues"></a>
<a href="https://github.com/muhammad-fiaz/adapters/pulls"><img src="https://img.shields.io/github/issues-pr/muhammad-fiaz/adapters" alt="GitHub pull requests"></a>
<a href="https://github.com/muhammad-fiaz/adapters"><img src="https://img.shields.io/github/last-commit/muhammad-fiaz/adapters" alt="GitHub last commit"></a>
<a href="https://github.com/muhammad-fiaz/adapters"><img src="https://img.shields.io/github/license/muhammad-fiaz/adapters" alt="License"></a>
<a href="https://github.com/muhammad-fiaz/adapters/actions/workflows/ci.yml"><img src="https://github.com/muhammad-fiaz/adapters/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
<img src="https://img.shields.io/badge/platforms-linux%20%7C%20windows%20%7C%20macos-blue" alt="Supported Platforms">
<a href="https://github.com/muhammad-fiaz/adapters/actions/workflows/github-code-scanning/codeql"><img src="https://github.com/muhammad-fiaz/adapters/actions/workflows/github-code-scanning/codeql/badge.svg" alt="CodeQL"></a>
<a href="https://github.com/muhammad-fiaz/adapters/actions/workflows/release.yml"><img src="https://github.com/muhammad-fiaz/adapters/actions/workflows/release.yml/badge.svg" alt="Release"></a>
<a href="https://github.com/muhammad-fiaz/adapters/releases/latest"><img src="https://img.shields.io/github/v/release/muhammad-fiaz/adapters?label=Latest%20Release&style=flat-square" alt="Latest Release"></a>
<a href="https://pay.muhammadfiaz.com"><img src="https://img.shields.io/badge/Sponsor-pay.muhammadfiaz.com-ff69b4?style=flat&logo=heart" alt="Sponsor"></a>
<a href="https://github.com/sponsors/muhammad-fiaz"><img src="https://img.shields.io/badge/Sponsor-💖-pink?style=social&logo=github" alt="GitHub Sponsors"></a>
<a href="https://hits.sh/muhammad-fiaz/adapters/"><img src="https://hits.sh/muhammad-fiaz/adapters.svg?label=Visitors&extraCount=0&color=green" alt="Repo Visitors"></a>

<p><em>A fast, high-performance structured schema validation, serialization, and transformation library for Rust.</em></p>

<b><a href="https://muhammad-fiaz.github.io/adapters/">Documentation</a> |
<a href="https://muhammad-fiaz.github.io/adapters/api">API Reference</a> |
<a href="https://muhammad-fiaz.github.io/adapters/getting-started">Quick Start</a> |
<a href="CONTRIBUTING.md">Contributing</a></b>

</div>

A production-grade, high-performance schema and data transformations library for Rust, designed with a clean, intuitive, and developer-friendly API.

> [!NOTE]
> This Project aims to be production ready, while it is relatively new project you can find some interesting features which may simplify your Rust project structured validations and data transformations.

**⭐️ If you love `adapters`, make sure to give it a star! ⭐️**

---

<details>
<summary><strong>Table of Contents</strong> (click to expand)</summary>

- [Prerequisites](#prerequisites)
- [Supported Platforms](#supported-platforms)
- [Installation](#installation)
  - [Method 1: Cargo Add (Recommended)](#method-1-cargo-add-recommended)
  - [Method 2: Manual Cargo.toml Configuration](#method-2-manual-cargotoml-configuration)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
  - [Basic Declarative Validation](#basic-declarative-validation)
  - [Advanced Macro Validations](#advanced-macro-validations)
  - [Programmatic Builder Validation](#programmatic-builder-validation)
  - [Nested Model Validation](#nested-model-validation)
  - [Field Transformation Pipelines](#field-transformation-pipelines)
  - [Alias Key Remapping](#alias-key-remapping)
- [Declarative Macro Reference Table](#declarative-macro-reference-table)
- [Performance & Benchmarks](#performance--benchmarks)
- [Building & Testing](#building--testing)
- [Documentation](#documentation)
  - [Online Documentation](#online-documentation)
  - [Generating Local Documentation](#generating-local-documentation)
- [Contributing](#contributing)
- [License](#license)
- [Links](#links)

</details>

----

<details>
<summary><strong>Features of Adapters</strong> (click to expand)</summary>

| Feature | Description | Documentation |
|---------|-------------|---------------|
| **Declarative Derive Macro** | User-friendly data modeling interface via `#[derive(Schema)]` | [Docs](https://muhammad-fiaz.github.io/adapters/schema-validation) |
| **Comprehensive Field Rules** | email, url, min/max bounds, regex, non_empty, alphanumeric, positive, negative, and non_zero | [Docs](https://muhammad-fiaz.github.io/adapters/api) |
| **Coercion & Strict Mode** | Optional strict type enforcement to prevent implicit casting / type parsing | [Docs](https://muhammad-fiaz.github.io/adapters/schema-validation) |
| **Dynamic Schema Builders** | Build complex nested structural validations dynamically at runtime | [Docs](https://muhammad-fiaz.github.io/adapters/schema-validation) |
| **Robust Parsing Engine** | Native structured JSON parsing with precise escape seq bounds | [Docs](https://muhammad-fiaz.github.io/adapters/serialization) |
| **Functional Pipelines** | Chain value adjustments, defaults, transformations, and filters cleanly | [Docs](https://muhammad-fiaz.github.io/adapters/transformation) |
| **Nested Validation Error Paths** | Accumulates nested object failures using accurate dot-notation paths | [Docs](https://muhammad-fiaz.github.io/adapters/schema-validation) |

</details>

----

<details>
<summary><strong>Prerequisites & Supported Platforms</strong> (click to expand)</summary>

<br>

## Prerequisites

Before installing Adapters, ensure you have the following:

| Requirement | Version | Notes |
|-------------|---------|-------|
| **Rust** | Stable 1.85+ | 2024 Edition supported natively |
| **Operating System** | Windows, Linux, macOS | Cross-platform |

---

## Supported Platforms

Adapters supports a wide range of platforms and architectures:

| Platform | Architectures | Status |
|----------|---------------|--------|
| **Windows** | x86_64, aarch64, x86 | Full support |
| **Linux** | x86_64, aarch64, riscv64 | Full support |
| **macOS** | x86_64, aarch64 (Apple Silicon) | Full support |

</details>

---

## Installation

### Method 1: Cargo Add (Recommended)

Add **Adapters** to your cargo project dynamically:

```bash
cargo add adapters
```

### Method 2: Manual Cargo.toml Configuration

Add this directly to your `Cargo.toml` dependencies block:

```toml
[dependencies]
adapters = "0.0.0"
```

---

## Quick Start

```rust
use adapters::prelude::*;

#[derive(Schema, Debug)]
struct UserRegistration {
    #[schema(min_length = 3, max_length = 20, alphanumeric)]
    username: String,

    #[schema(email)]
    email: String,

    #[schema(positive)]
    age: u8,
}

fn main() -> Result<(), Error> {
    let raw_payload = r#"{
        "username": "Fiaz2026",
        "email": "contact@example.com",
        "age": 28
    }"#;

    // Direct structural deserialization & validation
    let user = UserRegistration::from_json(raw_payload)?;
    println!("Successfully validated user: {:?}", user);

    Ok(())
}
```

---

## Usage Examples

### Basic Declarative Validation

```rust
use adapters::prelude::*;

#[derive(Schema)]
struct SimpleUser {
    name: String,
    #[schema(optional, default = "Guest")]
    role: String,
}
```

### Advanced Macro Validations

```rust
use adapters::prelude::*;

#[derive(Schema)]
struct SecureAsset {
    #[schema(non_empty, alphanumeric)]
    serial_code: String,

    #[schema(positive)]
    quantity: i32,

    #[schema(negative)]
    balance_deficit: f64,

    #[schema(non_zero)]
    tracking_id: i64,
}
```

### Programmatic Builder Validation

```rust
use adapters::prelude::*;
use adapters::schema::{ObjectSchema, StringSchema, IntegerSchema};
use adapters::SchemaValidator;

let schema = ObjectSchema::new()
    .field("username", StringSchema::new().required().non_empty().alphanumeric())
    .field("age", IntegerSchema::new().required().min(18).positive());
```

### Nested Model Validation

```rust
use adapters::prelude::*;

#[derive(Schema)]
struct Address {
    city: String,
    country: String,
}

#[derive(Schema)]
struct Profile {
    name: String,
    address: Address, // Child schema validated recursively!
}
```

### Field Transformation Pipelines & Sanitization

Chaining data mapping and sanitization rules is highly expressive using functional pipelines. This is exceptionally useful when consuming legacy third-party Webhook payloads:

```rust
use adapters::prelude::*;
use adapters::transform::{Pipeline, FieldMapper};

// Create a field mapper to rename legacy API keys
let legacy_mapper = FieldMapper::new()
    .rename("txt_title", "title")
    .rename("usr_id", "user_id");

// Build a transformation pipeline
let pipeline = Pipeline::new()
    .push(legacy_mapper)
    .push(|mut value: Value| {
        // Custom sanitization: Force uppercase on the "title" key if present
        if let Some(obj) = value.as_object_mut() {
            if let Some(Value::String(s)) = obj.get_mut("title") {
                *s = s.to_uppercase();
            }
        }
        Ok(value)
    });

// Process a raw legacy payload
let legacy_payload = Value::Object(
    [
        ("txt_title".to_string(), Value::String("Adapters Guide".into())),
        ("usr_id".to_string(), Value::Int(99)),
    ]
    .into_iter()
    .collect()
);

let clean_value = pipeline.run(legacy_payload).unwrap();
println!("Transformed payload: {:?}", clean_value);
// Output will have "title" in uppercase, and "usr_id" renamed to "user_id"!
```

### Alias Key Remapping

If you don't need runtime pipelines and only require direct structural key mapping from your JSON API payloads, use the `#[schema(alias = "...")]` helper:

```rust
use adapters::prelude::*;

#[derive(Schema, Debug)]
struct ApiConfig {
    /// Remaps "serverPort" in JSON directly to "port" in Rust
    #[schema(alias = "serverPort")]
    port: u16,

    /// Remaps "db_host_name" in JSON directly to "host" in Rust
    #[schema(alias = "db_host_name")]
    host: String,
}
```

---

## Declarative Macro Reference Table

Configure your struct fields using the `#[schema(...)]` helper:

| Attribute Rule | Supported Types | Action Description |
| :--- | :--- | :--- |
| `min_length = <usize>` | `String` | Enforces a minimum string character count. |
| `max_length = <usize>` | `String` | Enforces a maximum string character count. |
| `non_empty` | `String` | Restricts string to be non-empty (minimum 1 character). |
| `alphanumeric` | `String` | Enforces only alphanumeric characters. |
| `email` | `String` | Matches the string value against standard RFC 5322 format. |
| `url` | `String` | Matches the string value against standard URL layout. |
| `regex = "<pattern>"` | `String` | Validates string matching using custom Rust regex. |
| `min = <number>` | All numbers | Restricts numbers to be greater than or equal to value. |
| `max = <number>` | All numbers | Restricts numbers to be less than or equal to value. |
| `positive` | All numbers | Checks if numbers are strictly positive ($>0$). |
| `negative` | All numbers | Checks if numbers are strictly negative ($<0$). |
| `non_zero` | All numbers | Restricts numbers to exclude exact $0$ value. |
| `optional` | All types | Declares the field is non-required and defaults to null. |
| `strict` | All types | Opts into strict validation: no implicit type coercions. |
| `default = <expr>` | All types | Populates field with expression value when key is absent. |

---

## Performance & Benchmarks

Adapters is engineered for extreme data validation and parsing throughput. It implements high-performance zero-allocation parsing routines, compact error trees, and static struct validation maps.

To run benchmarks locally:

```bash
cargo bench
```

---

## Building & Testing

```bash
# Check compiler requirements and clippy lints
cargo clippy --workspace --all-targets -- -D warnings

# Run all unit, integration, and doctests
cargo test --workspace --all-targets
```

---

## Documentation

### Online Documentation
Full structured guide and dynamic api walkthroughs are available natively at:
https://muhammad-fiaz.github.io/adapters

### Generating Local Documentation
To generate and view crate-level API documentation locally:

```bash
cargo doc --no-deps --open
```

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

## Links

- **Documentation**: https://muhammad-fiaz.github.io/adapters
- **Repository**: https://github.com/muhammad-fiaz/adapters
- **Issues**: https://github.com/muhammad-fiaz/adapters/issues
