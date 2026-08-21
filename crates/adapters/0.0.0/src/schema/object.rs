//! Object schema — validates structured objects with named fields.
//!
//! Provides the [`ObjectSchema`] structure to represent nested struct mappings and validate properties.

use super::SchemaValidator;
use crate::error::ValidationError;
use crate::value::Value;
use std::collections::BTreeMap;

/// Schema representing key-value object constraints and field mappings.
#[derive(Default)]
pub struct ObjectSchema {
    field_order: Vec<String>,
    field_map: BTreeMap<String, Box<dyn SchemaValidator>>,
    strict: bool,
    required: bool,
    optional: bool,
}

impl ObjectSchema {
    /// Creates a new `ObjectSchema` with no structural fields configured.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a new named property field associated with a schema validator.
    pub fn field(mut self, name: &str, schema: impl SchemaValidator + 'static) -> Self {
        self.field_order.push(name.to_string());
        self.field_map.insert(name.to_string(), Box::new(schema));
        self
    }

    /// Opts into strict validation mode: unrecognized keys trigger validation failures.
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Configures the schema to strictly fail if the field is absent.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Registers the field as optional (permits `Null` values).
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

impl SchemaValidator for ObjectSchema {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if self.optional && value.is_null() {
            return Ok(());
        }
        if self.required && value.is_null() {
            return Err(ValidationError::new(field, "field is required", "required"));
        }
        let obj = match value.as_object() {
            Some(o) => o,
            None => {
                return Err(ValidationError::new(
                    field,
                    format!("expected object, got {}", value.type_name()),
                    "type_mismatch",
                ));
            }
        };

        if self.strict {
            for key in obj.keys() {
                if !self.field_map.contains_key(key) {
                    return Err(ValidationError::new(
                        field,
                        format!("unknown field '{key}' (strict mode)"),
                        "unknown_field",
                    ));
                }
            }
        }

        for name in &self.field_order {
            let schema = &self.field_map[name];
            let child_field = if field == "root" || field.is_empty() {
                name.clone()
            } else {
                format!("{field}.{name}")
            };

            let val = match obj.get(name) {
                Some(v) => v.clone(),
                None => {
                    if let Some(def) = schema.default_value() {
                        def
                    } else if schema.is_required() {
                        return Err(ValidationError::new(
                            &child_field,
                            format!("required field '{name}' is missing"),
                            "required",
                        ));
                    } else {
                        Value::Null
                    }
                }
            };

            schema.validate(&val, &child_field)?;
        }
        Ok(())
    }

    fn is_required(&self) -> bool {
        self.required
    }
    fn default_value(&self) -> Option<Value> {
        None
    }
    fn schema_type(&self) -> &'static str {
        "object"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{IntegerSchema, StringSchema};

    fn simple_user_schema() -> ObjectSchema {
        ObjectSchema::new()
            .field("username", StringSchema::new().required().min_length(3))
            .field("age", IntegerSchema::new().required().min(18))
    }

    fn make_obj(fields: &[(&str, Value)]) -> Value {
        let mut m = BTreeMap::new();
        for (k, v) in fields {
            m.insert(k.to_string(), v.clone());
        }
        Value::Object(m)
    }

    #[test]
    fn test_object_valid() {
        let s = simple_user_schema();
        let v = make_obj(&[
            ("username", Value::String("alice".into())),
            ("age", Value::Int(25)),
        ]);
        assert!(s.validate(&v, "root").is_ok());
    }

    #[test]
    fn test_object_missing_required_field() {
        let s = simple_user_schema();
        let v = make_obj(&[("username", Value::String("alice".into()))]);
        assert!(s.validate(&v, "root").is_err());
    }

    #[test]
    fn test_object_field_validation_fails() {
        let s = simple_user_schema();
        let v = make_obj(&[
            ("username", Value::String("al".into())),
            ("age", Value::Int(25)),
        ]);
        let err = s.validate(&v, "root").unwrap_err();
        assert!(err.field.contains("username"), "field: {}", err.field);
    }

    #[test]
    fn test_object_strict_rejects_unknown() {
        let s = simple_user_schema().strict();
        let v = make_obj(&[
            ("username", Value::String("alice".into())),
            ("age", Value::Int(25)),
            ("extra", Value::Bool(true)),
        ]);
        assert!(s.validate(&v, "root").is_err());
    }

    #[test]
    fn test_object_not_an_object_fails() {
        let s = simple_user_schema();
        assert!(
            s.validate(&Value::String("not an object".into()), "root")
                .is_err()
        );
    }

    #[test]
    fn test_object_optional_null_passes() {
        let s = ObjectSchema::new().optional();
        assert!(s.validate(&Value::Null, "addr").is_ok());
    }

    #[test]
    fn test_nested_error_path() {
        let inner = ObjectSchema::new().field("city", StringSchema::new().required().min_length(3));
        let outer = ObjectSchema::new().field("address", inner);
        let bad_city = make_obj(&[("city", Value::String("ab".into()))]);
        let v = make_obj(&[("address", bad_city)]);
        let err = outer.validate(&v, "root").unwrap_err();
        assert!(err.field.contains("address"), "path: {}", err.field);
    }
}
