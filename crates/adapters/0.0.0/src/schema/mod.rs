//! Schema module — schema types, the `SchemaValidator` trait, and the unified `Schema` enum.
//!
//! Defines structural schemas representing various Rust and JSON datatypes
//! used to dynamically validate un-typed value representations.

pub mod array;
pub mod boolean;
pub mod enums;
pub mod float;
pub mod integer;
pub mod null;
pub mod object;
pub mod string;

pub use self::array::ArraySchema;
pub use self::boolean::BoolSchema;
pub use self::enums::EnumSchema;
pub use self::float::FloatSchema;
pub use self::integer::IntegerSchema;
pub use self::null::NullSchema;
pub use self::object::ObjectSchema;
pub use self::string::StringSchema;

use crate::error::ValidationError;
use crate::value::Value;

/// Defines validation rules and metadata introspection for a specific schema type.
pub trait SchemaValidator: Send + Sync {
    /// Validates the given [`Value`] against the structural rules of this schema.
    ///
    /// # Errors
    ///
    /// Returns a [`ValidationError`] if the value deviates from schema requirements.
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError>;

    /// Returns `true` if the field must be supplied and lacks a default value.
    fn is_required(&self) -> bool;

    /// Yields the fallback [`Value`] if this optional/default field is omitted.
    fn default_value(&self) -> Option<Value>;

    /// Static string representing the readable type name.
    fn schema_type(&self) -> &'static str;
}

/// A provider exposing the structural schema definition associated with a Rust structure.
pub trait SchemaProvider {
    /// Returns the static [`Schema`] representation.
    fn schema() -> Schema;
}

/// Unified runtime representation of any structural schema type.
///
/// Enables nested and dynamically composed schemas at runtime.
pub enum Schema {
    /// String schema.
    String(StringSchema),
    /// Integer schema.
    Integer(IntegerSchema),
    /// Floating-point schema.
    Float(FloatSchema),
    /// Boolean schema.
    Bool(BoolSchema),
    /// Array schema.
    Array(ArraySchema),
    /// Key-value object schema.
    Object(ObjectSchema),
    /// String-restricted enumeration schema.
    Enum(EnumSchema),
    /// Null-type schema.
    Null(NullSchema),
}

impl std::fmt::Debug for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Schema({})", self.schema_type())
    }
}

impl Schema {
    /// Constructs a new, empty string schema builder.
    pub fn string() -> StringSchema {
        StringSchema::new()
    }
    /// Constructs a new integer schema builder.
    pub fn integer() -> IntegerSchema {
        IntegerSchema::new()
    }
    /// Constructs a new float schema builder.
    pub fn float() -> FloatSchema {
        FloatSchema::new()
    }
    /// Constructs a new boolean schema builder.
    pub fn bool() -> BoolSchema {
        BoolSchema::new()
    }
    /// Constructs a new object/struct schema builder.
    pub fn object() -> ObjectSchema {
        ObjectSchema::new()
    }
    /// Constructs a new array schema builder over an item validator.
    pub fn array(item: impl SchemaValidator + 'static) -> ArraySchema {
        ArraySchema::new(item)
    }
    /// Constructs a new null-only schema.
    pub fn null() -> NullSchema {
        NullSchema::new()
    }

    /// Configures the schema to strictly fail if the field is absent.
    pub fn required(self) -> Self {
        match self {
            Schema::String(s) => Schema::String(s.required()),
            Schema::Integer(s) => Schema::Integer(s.required()),
            Schema::Float(s) => Schema::Float(s.required()),
            Schema::Bool(s) => Schema::Bool(s.required()),
            Schema::Array(s) => Schema::Array(s.required()),
            Schema::Object(s) => Schema::Object(s.required()),
            Schema::Enum(s) => Schema::Enum(s.required()),
            Schema::Null(s) => Schema::Null(s.required()),
        }
    }

    /// Registers the field as optional (permits `Null` values).
    pub fn optional(self) -> Self {
        match self {
            Schema::String(s) => Schema::String(s.optional()),
            Schema::Integer(s) => Schema::Integer(s.optional()),
            Schema::Float(s) => Schema::Float(s.optional()),
            Schema::Bool(s) => Schema::Bool(s.optional()),
            Schema::Array(s) => Schema::Array(s.optional()),
            Schema::Object(s) => Schema::Object(s.optional()),
            Schema::Enum(s) => Schema::Enum(s.optional()),
            Schema::Null(s) => Schema::Null(s.optional()),
        }
    }
}

impl SchemaValidator for Schema {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        match self {
            Schema::String(s) => s.validate(value, field),
            Schema::Integer(s) => s.validate(value, field),
            Schema::Float(s) => s.validate(value, field),
            Schema::Bool(s) => s.validate(value, field),
            Schema::Array(s) => s.validate(value, field),
            Schema::Object(s) => s.validate(value, field),
            Schema::Enum(s) => s.validate(value, field),
            Schema::Null(s) => s.validate(value, field),
        }
    }

    fn is_required(&self) -> bool {
        match self {
            Schema::String(s) => s.is_required(),
            Schema::Integer(s) => s.is_required(),
            Schema::Float(s) => s.is_required(),
            Schema::Bool(s) => s.is_required(),
            Schema::Array(s) => s.is_required(),
            Schema::Object(s) => s.is_required(),
            Schema::Enum(s) => s.is_required(),
            Schema::Null(s) => s.is_required(),
        }
    }

    fn default_value(&self) -> Option<Value> {
        match self {
            Schema::String(s) => s.default_value(),
            Schema::Integer(s) => s.default_value(),
            Schema::Float(s) => s.default_value(),
            Schema::Bool(s) => s.default_value(),
            Schema::Array(s) => s.default_value(),
            Schema::Object(s) => s.default_value(),
            Schema::Enum(s) => s.default_value(),
            Schema::Null(s) => s.default_value(),
        }
    }

    fn schema_type(&self) -> &'static str {
        match self {
            Schema::String(s) => s.schema_type(),
            Schema::Integer(s) => s.schema_type(),
            Schema::Float(s) => s.schema_type(),
            Schema::Bool(s) => s.schema_type(),
            Schema::Array(s) => s.schema_type(),
            Schema::Object(s) => s.schema_type(),
            Schema::Enum(s) => s.schema_type(),
            Schema::Null(s) => s.schema_type(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_string_builder() {
        let s = Schema::string().min_length(3);
        let schema = Schema::String(s);
        assert!(schema.validate(&Value::String("abc".into()), "f").is_ok());
        assert!(schema.validate(&Value::String("ab".into()), "f").is_err());
    }

    #[test]
    fn test_schema_integer_builder() {
        let s = Schema::integer().min(0).max(100);
        let schema = Schema::Integer(s);
        assert!(schema.validate(&Value::Int(50), "n").is_ok());
        assert!(schema.validate(&Value::Int(200), "n").is_err());
    }

    #[test]
    fn test_schema_object_builder() {
        let s = Schema::object().field("name", Schema::string().required());
        let schema = Schema::Object(s);
        let mut m = std::collections::BTreeMap::new();
        m.insert("name".to_string(), Value::String("alice".into()));
        assert!(schema.validate(&Value::Object(m), "root").is_ok());
    }

    #[test]
    fn test_schema_null() {
        let s = Schema::Null(Schema::null());
        assert!(s.validate(&Value::Null, "x").is_ok());
        assert!(s.validate(&Value::Int(1), "x").is_err());
    }

    #[test]
    fn test_schema_debug() {
        let s = Schema::String(Schema::string());
        let dbg = format!("{:?}", s);
        assert!(dbg.contains("string"));
    }
}
