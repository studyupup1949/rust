//! Array schema — validates arrays and each element against an item schema.
//!
//! Provides the [`ArraySchema`] structure for array validation constraints and inner element auditing.

use super::SchemaValidator;
use crate::error::ValidationError;
use crate::value::Value;

/// Schema representing array validation constraints and item schemas.
pub struct ArraySchema {
    item_schema: Box<dyn SchemaValidator>,
    min_items: Option<usize>,
    max_items: Option<usize>,
    required: bool,
    optional: bool,
}

impl ArraySchema {
    /// Creates a new `ArraySchema` verifying each item using the defined `item_schema`.
    pub fn new(item_schema: impl SchemaValidator + 'static) -> Self {
        Self {
            item_schema: Box::new(item_schema),
            min_items: None,
            max_items: None,
            required: false,
            optional: false,
        }
    }

    /// Enforces the array to hold a minimum of $N$ items.
    pub fn min_items(mut self, n: usize) -> Self {
        self.min_items = Some(n);
        self
    }

    /// Enforces the array to hold a maximum of $M$ items.
    pub fn max_items(mut self, n: usize) -> Self {
        self.max_items = Some(n);
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

impl SchemaValidator for ArraySchema {
    fn validate(&self, value: &Value, field: &str) -> Result<(), ValidationError> {
        if self.optional && value.is_null() {
            return Ok(());
        }
        if self.required && value.is_null() {
            return Err(ValidationError::new(field, "field is required", "required"));
        }
        let arr = match value.as_array() {
            Some(a) => a,
            None => {
                return Err(ValidationError::new(
                    field,
                    format!("expected array, got {}", value.type_name()),
                    "type_mismatch",
                ));
            }
        };
        if let Some(min) = self.min_items
            && arr.len() < min
        {
            return Err(ValidationError::new(
                field,
                format!("minimum items is {min}, got {}", arr.len()),
                "min_items",
            ));
        }
        if let Some(max) = self.max_items
            && arr.len() > max
        {
            return Err(ValidationError::new(
                field,
                format!("maximum items is {max}, got {}", arr.len()),
                "max_items",
            ));
        }
        for (i, item) in arr.iter().enumerate() {
            let item_field = format!("{field}[{i}]");
            self.item_schema.validate(item, &item_field)?;
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
        "array"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::StringSchema;

    #[test]
    fn test_array_valid() {
        let s = ArraySchema::new(StringSchema::new());
        let arr = Value::Array(vec![Value::String("a".into()), Value::String("b".into())]);
        assert!(s.validate(&arr, "tags").is_ok());
    }

    #[test]
    fn test_array_item_fails() {
        let s = ArraySchema::new(StringSchema::new().email());
        let arr = Value::Array(vec![Value::String("notanemail".into())]);
        assert!(s.validate(&arr, "emails").is_err());
    }

    #[test]
    fn test_array_min_items_fail() {
        let s = ArraySchema::new(StringSchema::new()).min_items(3);
        let arr = Value::Array(vec![Value::String("a".into())]);
        assert!(s.validate(&arr, "tags").is_err());
    }

    #[test]
    fn test_array_max_items_fail() {
        let s = ArraySchema::new(StringSchema::new()).max_items(2);
        let arr = Value::Array(vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
        ]);
        assert!(s.validate(&arr, "tags").is_err());
    }

    #[test]
    fn test_array_not_array_fails() {
        let s = ArraySchema::new(StringSchema::new());
        assert!(s.validate(&Value::String("bad".into()), "tags").is_err());
    }

    #[test]
    fn test_array_optional_null_passes() {
        let s = ArraySchema::new(StringSchema::new()).optional();
        assert!(s.validate(&Value::Null, "tags").is_ok());
    }

    #[test]
    fn test_array_error_has_indexed_path() {
        let s = ArraySchema::new(StringSchema::new().email());
        let arr = Value::Array(vec![Value::String("bad".into())]);
        let err = s.validate(&arr, "emails").unwrap_err();
        assert!(err.field.contains("[0]"), "field was: {}", err.field);
    }
}
