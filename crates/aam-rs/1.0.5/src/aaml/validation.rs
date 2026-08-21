//! Schema validation methods for [`AAML`](super::AAML).

use std::collections::HashMap;
use crate::error::AamlError;
use crate::types::resolve_builtin;
use super::AAML;

impl AAML {
    /// Validates a single field value against any schema that declares it.
    ///
    /// If the field is not declared in any schema the function succeeds silently.
    pub(super) fn validate_against_schemas(
        &self,
        field: &str,
        value: &str,
    ) -> Result<(), AamlError> {
        for (schema_name, schema_def) in &self.schemas {
            if let Some(type_name) = schema_def.fields.get(field) {
                return self.validate_typed_field(type_name, value, schema_name, field);
            }
        }
        Ok(())
    }

    /// Validates `value` against `type_name`, checking registered types first,
    /// then built-in types. Returns a [`AamlError::SchemaValidationError`] on failure.
    fn validate_typed_field(
        &self,
        type_name: &str,
        value: &str,
        schema_name: &str,
        field: &str,
    ) -> Result<(), AamlError> {
        let make_err = |details: String| AamlError::SchemaValidationError {
            schema: schema_name.to_string(),
            field: field.to_string(),
            type_name: type_name.to_string(),
            details,
        };

        if let Some(type_def) = self.types.get(type_name) {
            return type_def.validate(value).map_err(|e| make_err(e.to_string()));
        }

        match resolve_builtin(type_name) {
            Ok(type_def) => type_def.validate(value).map_err(|e| make_err(e.to_string())),
            Err(_) => Err(make_err(format!("Unknown type '{}'", type_name))),
        }
    }

    /// Checks every field in every registered schema against the current map.
    ///
    /// Returns an error for the first schema field that has no value in the map.
    pub fn validate_schemas_completeness(&self) -> Result<(), AamlError> {
        for (schema_name, schema_def) in &self.schemas {
            for (field, type_name) in &schema_def.fields {
                if !self.map.contains_key(field.as_str()) {
                    return Err(AamlError::SchemaValidationError {
                        schema: schema_name.clone(),
                        field: field.clone(),
                        type_name: type_name.clone(),
                        details: format!("Missing required field '{}'", field),
                    });
                }
            }
        }
        Ok(())
    }

    /// Validates a complete `data` map against the named schema.
    ///
    /// For every field declared in the schema the method checks:
    /// 1. The key is present in `data`.
    /// 2. The value satisfies the declared type.
    pub fn apply_schema(
        &self,
        schema_name: &str,
        data: &HashMap<String, String>,
    ) -> Result<(), AamlError> {
        let schema = self.schemas.get(schema_name).ok_or_else(|| {
            AamlError::NotFound(format!("Schema '{}' not found", schema_name))
        })?;

        for (field, type_name) in &schema.fields {
            let value = data.get(field).ok_or_else(|| AamlError::SchemaValidationError {
                schema: schema_name.to_string(),
                field: field.clone(),
                type_name: type_name.clone(),
                details: format!("Missing required field '{}'", field),
            })?;
            self.validate_typed_field(type_name, value, schema_name, field)?;
        }

        Ok(())
    }
}

