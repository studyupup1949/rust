# API Configs & Reference

This page provides a comprehensive breakdown of the core traits, types, custom configurations, and errors exposed by the Adapters public API.

---

## Unified Configurations & Traits

### `Adapter`
The central trait combining validation, serialization, deserialization, and schema introspection.

```rust
pub trait Adapter: Serialize + Deserialize + Validate + SchemaProvider + Sized {
    fn from_json(json: &str) -> Result<Self, Error>;
    fn to_json(&self) -> Result<String, Error>;
    fn from_value(value: Value) -> Result<Self, Error>;
    fn to_value(&self) -> Value;
    fn is_valid(&self) -> bool;
}
```

### `SchemaProvider`
Allows types to expose their structural validation specifications dynamically at runtime.

```rust
pub trait SchemaProvider {
    fn schema() -> Schema;
}
```

---

## Dynamic Type Reference: `Value`

The `Value` enum represents dynamic, JSON-compatible, or runtime-defined data structures:

```rust
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}
```

### Utility Type-Safe Extractors
- `as_str() -> Option<&str>`
- `as_int() -> Option<i64>`
- `as_float() -> Option<f64>`
- `as_bool() -> Option<bool>`
- `as_array() -> Option<&Vec<Value>>`
- `as_object() -> Option<&BTreeMap<String, Value>>`
- `is_null() -> bool`

---

## Complete Macro Reference

The procedural macro `#[derive(Schema)]` automatically implements `SchemaProvider`, `Serialize`, `Deserialize`, `Validate`, and `Adapter` on target structures.

### Field Attributes Listing
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

## Error Handling Types

Every fallible operation returns a `Result<T, Error>`. The top-level `Error` enum covers:

```rust
pub enum Error {
    /// Validation constraint violated.
    Validation(ValidationError),
    /// Failure during struct-to-Value serialization.
    Serialization(SerializationError),
    /// Failure during Value-to-struct deserialization.
    Deserialization(DeserializationError),
    /// Lexing or parsing failures inside the native JSON engine.
    Json(JsonError),
    /// Structural Schema error.
    Schema(SchemaError),
}
```
