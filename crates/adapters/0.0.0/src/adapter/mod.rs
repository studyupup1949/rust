//! The `Adapter` trait — the central interface for data schema validation,
//! serialization, deserialization, and representation mapping in the library.

use crate::deserializer::Deserialize;
use crate::error::Error;
#[allow(unused_imports)]
use crate::schema::{Schema, SchemaProvider, SchemaValidator};
use crate::serializer::Serialize;
use crate::value::Value;

/// Defines a post-construction validation behavior for concrete Rust instances.
///
/// While schema validation operates dynamically on un-typed [`Value`] objects
/// before instantiation, the `Validate` trait provides object-level constraints
/// on already constructed domain models.
pub trait Validate {
    /// Validates the instance, returning an error on the first constraint violation.
    fn validate(&self) -> Result<(), Error>;
}

/// The unified interface combining serialization, deserialization, schema definition,
/// and instance validation into a single, cohesive framework.
///
/// Any type implementing `Adapter` automatically inherits utility methods for:
/// - Parsing from/to JSON string representations
/// - Mapping from/to dynamic [`Value`] tree representation
/// - Self-validation sanity checks
///
/// Implementing types must satisfy the bounds: [`Serialize`], [`Deserialize`],
/// [`Validate`], and [`SchemaProvider`].
pub trait Adapter: Serialize + Deserialize + Validate + SchemaProvider + Sized {
    /// Parses a model from a JSON string.
    ///
    /// This follows a strict validation-deserialization protocol:
    /// 1. Parse JSON into a dynamic [`Value`]
    /// 2. Apply structured schema validation rules defined by [`SchemaProvider::schema`]
    /// 3. Deserialize validated fields into `Self`
    fn from_json(json: &str) -> Result<Self, Error> {
        let value = crate::json::parse(json)?;
        let schema = Self::schema();
        schema.validate(&value, "root")?;
        Self::deserialize(value)
    }

    /// Serializes the model instance directly into a compact JSON string.
    fn to_json(&self) -> Result<String, Error> {
        let value = self.serialize();
        crate::json::stringify(&value).map_err(Error::Json)
    }

    /// Parses a model from a generic, dynamic [`Value`].
    ///
    /// Validates the structure against [`SchemaProvider::schema`] before deserializing.
    fn from_value(value: Value) -> Result<Self, Error> {
        let schema = Self::schema();
        schema.validate(&value, "root")?;
        Self::deserialize(value)
    }

    /// Converts the model instance into a generic [`Value`].
    fn to_value(&self) -> Value {
        self.serialize()
    }

    /// Audits the validity of the current instance against its internal invariants.
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ValidationError;
    use crate::schema::{IntegerSchema, ObjectSchema, StringSchema};
    use std::collections::BTreeMap;

    #[derive(Debug, PartialEq)]
    struct User {
        name: String,
        age: i64,
    }

    impl Serialize for User {
        fn serialize(&self) -> Value {
            let mut m = BTreeMap::new();
            m.insert("name".into(), Value::String(self.name.clone()));
            m.insert("age".into(), Value::Int(self.age));
            Value::Object(m)
        }
    }

    impl Deserialize for User {
        fn deserialize(value: Value) -> Result<Self, Error> {
            let obj = value.as_object().ok_or_else(|| {
                crate::error::Error::Deserialization(crate::error::DeserializationError::new(
                    "expected object",
                ))
            })?;
            let name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    Error::Deserialization(crate::error::DeserializationError::new("missing name"))
                })?
                .to_string();
            let age = obj.get("age").and_then(|v| v.as_int()).ok_or_else(|| {
                Error::Deserialization(crate::error::DeserializationError::new("missing age"))
            })?;
            Ok(User { name, age })
        }
    }

    impl Validate for User {
        fn validate(&self) -> Result<(), Error> {
            if self.name.len() < 2 {
                return Err(ValidationError::new("name", "too short", "min_length").into());
            }
            Ok(())
        }
    }

    impl SchemaProvider for User {
        fn schema() -> Schema {
            Schema::Object(
                ObjectSchema::new()
                    .field("name", StringSchema::new().required().min_length(2))
                    .field("age", IntegerSchema::new().required().min(0)),
            )
        }
    }

    impl Adapter for User {}

    #[test]
    fn test_from_json_valid() {
        let u = User::from_json(r#"{"name":"alice","age":30}"#).unwrap();
        assert_eq!(u.name, "alice");
        assert_eq!(u.age, 30);
    }

    #[test]
    fn test_from_json_invalid_schema() {
        let result = User::from_json(r#"{"name":"alice","age":-1}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_json() {
        let u = User {
            name: "bob".into(),
            age: 25,
        };
        let json = u.to_json().unwrap();
        assert!(json.contains("bob"));
        assert!(json.contains("25"));
    }

    #[test]
    fn test_from_value() {
        let mut m = BTreeMap::new();
        m.insert("name".to_string(), Value::String("carol".into()));
        m.insert("age".to_string(), Value::Int(40));
        let u = User::from_value(Value::Object(m)).unwrap();
        assert_eq!(u.name, "carol");
    }

    #[test]
    fn test_to_value() {
        let u = User {
            name: "dan".into(),
            age: 20,
        };
        let v = u.to_value();
        assert_eq!(v.get("name"), Some(&Value::String("dan".into())));
    }

    #[test]
    fn test_is_valid() {
        let u = User {
            name: "alice".into(),
            age: 25,
        };
        assert!(u.is_valid());
        let bad = User {
            name: "x".into(),
            age: 25,
        };
        assert!(!bad.is_valid());
    }
}
