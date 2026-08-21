# AAML -> AAM Migration Guide

This guide explains how to migrate application code from legacy `AAML` usage to the pipeline-backed `AAM` API, and how to migrate string-based errors to `AamlError`.

## 1) Imports

Before:

```rust
use aam_rs::aaml::AAML;
```

After:

```rust
use aam_rs::aam::AAM;
```

## 2) Parse and load

Before:

```rust
let cfg = AAML::parse(src)?;
let cfg = AAML::load("config.aam")?;
```

After (`AAM::parse`/`AAM::load` return `Result<_, Vec<AamlError>>`):

```rust
use aam_rs::error::AamlError;

fn first_error(errors: Vec<AamlError>) -> AamlError {
    errors.into_iter().next().unwrap_or(AamlError::ParseError {
        line: 1,
        content: String::new(),
        details: "unknown parse error".to_string(),
        diagnostics: None,
    }) // It's highly recommended to handle all errors properly instead of just taking the first one, but this is a simple helper for quick migration.
}

let cfg = AAM::parse(src).map_err(first_error)?;
let cfg = AAM::load("config.aam").map_err(first_error)?;
```

## 3) Lookups

These methods are supported on `AAM`:

- `find_obj(key)`
- `find_key(value)`
- `find_deep(key)`
- `get(key)`
- `find(query)`
- `deep_search(pattern)`
- `reverse_search(value)`

Example:

```rust
if let Some(v) = cfg.find_obj("host") {
    println!("{}", v.as_str());
}
```

## 4) Schema and type validation

`AAM` supports:

- `validate_value(type_name, value)`
- `validate_schemas_completeness()`
- `apply_schema(schema_name, data)`

## 5) Merge and formatting

`AAM` supports:

- `merge_content(text)`
- `merge_file(path)`
- `format(content, &FormattingOptions::default())`

## 6) Error migration: `Err`/`String` -> `AamlError`

If you have internal helpers returning `Result<T, String>`, migrate them to `Result<T, AamlError>`.

Before:

```rust
fn validate_x(v: &str) -> Result<(), String> {
    if v.is_empty() {
        return Err("empty value".to_string());
    }
    Ok(())
}
```

After:

```rust
use aam_rs::error::AamlError;

fn validate_x(v: &str) -> Result<(), AamlError> {
    if v.is_empty() {
        return Err(AamlError::InvalidValue {
            details: "empty value".to_string(),
            expected: "non-empty string".to_string(),
            diagnostics: None,
        });
    }
    Ok(())
}
```

## 7) Bindings migration

Bindings should use `aam_rs::aam::AAM` as backend type directly.

- Python: `src/python.rs`
- C FFI: `src/ffi.rs`
- JNI: `src/jni.rs`
- JS/WASM/Ruby/C# wrappers should expose `AAM` as primary class

If backward compatibility is required, keep legacy aliases for one deprecation cycle.

