//! High-performance, schema-driven data validation, serialization,
//! deserialization, and structural transformation library for Rust.
//!
//! This crate provides a unified and extensible API for defining data schemas
//! at runtime or compile-time (via derive macros), parsing and validating JSON
//! natively without external parser dependencies, and transforming data dynamically.
//!
//! # Unified Schema & Validation Layer
//!
//! Instead of validating data *after* parsing it into structured models (which can cause panics
//! or silent errors on invalid types), `adapters` defines a declarative, dynamic schema tree.
//! Inbound payloads are verified at the dynamic level first, matching strict type names,
//! number ranges, string lengths, custom regexes, and complex formats (e.g., Email or URLs).
//!
//! ## Nested Schema Validation
//!
//! `adapters` natively supports recursive validation of complex nested structures. When a structure
//! derives `Schema`, its schema definition incorporates the schema of any sub-structures that
//! also implement `SchemaProvider`.
//!
//! For example, when validating a parent struct like `User`, any nested objects (e.g., `Address`)
//! will be fully validated against their own schemas. Any validation failures in the nested child
//! are reported with correct dot-notation paths (e.g., `address.city` or `address.zip_code`).
//!
//! ```rust
//! use adapters::prelude::*;
//! use adapters::Schema;
//!
//! #[derive(Schema, Debug)]
//! struct Address {
//!     #[schema(min_length = 3)]
//!     city: String,
//!     country: String,
//! }
//!
//! #[derive(Schema, Debug)]
//! struct User {
//!     name: String,
//!     address: Address, // Automatically delegates validation to Address::schema()!
//! }
//! ```
//!
//! # High-Performance Serialization & Deserialization
//!
//! `adapters` features highly optimized and fully type-safe serialization and deserialization traits.
//! These traits define how memory structures are converted to and from intermediate [`Value`] dynamic trees.
//!
//! ## Safety and Strict Numeric Bounds
//!
//! Unlike naive decoders that might cause silent overflows, `adapters` performs type-safe checks during deserialization:
//! - If you deserialize a value of `300` into a `u8` field, it will return a clean `DeserializationError` explaining that `300` overflows the bounds of `u8`.
//! - If an unsigned integer type (e.g., `u32`) receives a negative value (e.g., `-10`), it is caught and rejected immediately.
//!
//! # Functional Data Transformation
//!
//! Domain models often diverge between different contexts (e.g., Database Models vs. API Presentation Models).
//! Using `Pipeline` and `FieldMapper` classes, you can map, rename, and transform data trees
//! programmatically in a highly functional manner.
//!
//! ```rust
//! use adapters::prelude::*;
//!
//! let transform_pipeline = Pipeline::new()
//!     .step(|val| match val {
//!         Value::Int(n) => Ok(Value::Int(n * 2)),
//!         other => Ok(other),
//!     });
//!
//! let result = transform_pipeline.run(Value::Int(5)).unwrap();
//! assert_eq!(result, Value::Int(10));
//! ```
//!
//! # Complete Macro Reference
//!
//! Configure your struct fields using the `#[schema(...)]` helper:
//!
//! | Attribute Rule | Supported Types | Action Description |
//! | :--- | :--- | :--- |
//! | `min_length = <usize>` | `String` | Enforces a minimum string character count. |
//! | `max_length = <usize>` | `String` | Enforces a maximum string character count. |
//! | `non_empty` | `String` | Restricts string to be non-empty (minimum 1 character). |
//! | `alphanumeric` | `String` | Enforces only alphanumeric characters. |
//! | `email` | `String` | Matches the string value against standard RFC 5322 format. |
//! | `url` | `String` | Matches the string value against standard URL layout. |
//! | `regex = "<pattern>"` | `String` | Validates string matching using custom Rust regex. |
//! | `min = <number>` | All numbers | Restricts numbers to be greater than or equal to value. |
//! | `max = <number>` | All numbers | Restricts numbers to be less than or equal to value. |
//! | `positive` | All numbers | Checks if numbers are strictly positive ($>0$). |
//! | `negative` | All numbers | Checks if numbers are strictly negative ($<0$). |
//! | `non_zero` | All numbers | Restricts numbers to exclude exact $0$ value. |
//! | `optional` | All types | Declares the field is non-required and defaults to null. |
//! | `strict` | All types | Opts into strict validation: no implicit type coercions. |
//! | `default = <expr>` | All types | Populates field with expression value when key is absent. |
//!

pub mod adapter;
pub mod deserializer;
pub mod error;
pub mod json;
pub mod schema;
pub mod serializer;
pub mod transform;
pub mod validator;
pub mod value;

pub use adapter::{Adapter, Validate};
pub use deserializer::Deserialize;
pub use error::{Error, ValidationError, ValidationErrors};
pub use schema::{
    ArraySchema, BoolSchema, EnumSchema, FloatSchema, IntegerSchema, NullSchema, ObjectSchema,
    Schema, SchemaProvider, SchemaValidator, StringSchema,
};
pub use serializer::Serialize;
pub use transform::{Adapt, FieldMapper, Pipeline};
pub use value::Value;

pub use adapters_macros::Schema;

/// The prelude module re-exports the most commonly used traits and types
/// for ergonomic integration across projects.
pub mod prelude {
    pub use crate::{
        Adapt, Adapter, Deserialize, Error, FieldMapper, Pipeline, Schema, Serialize, Validate,
        ValidationError, ValidationErrors, Value,
    };
}

/// Utility module providing native, highly optimized JSON parsing and serialization functions.
pub mod json_utils {
    pub use crate::json::{parse, stringify, stringify_pretty};
}
