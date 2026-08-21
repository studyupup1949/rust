//! Schema validation methods for [`AAML`](AAML).

use super::AAML;
use crate::aaml::parsing;
use crate::error::AamlError;
use crate::types::resolve_builtin;
use std::collections::HashMap;

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

    /// Validates `value` against `type_name`, checking:
    /// 1. Registered custom types.
    /// 2. Nested schema types (type_name matches a registered schema name).
    /// 3. `list<T>` — validates every element of a `[...]` literal against `T`.
    /// 4. Built-in module types (`math::`, `time::`, `physics::`, primitives).
    ///
    /// Returns a [`AamlError::SchemaValidationError`] on failure.
    pub(crate) fn validate_typed_field(
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
            diagnostics: None,
        };

        // 1. Registered custom type alias
        if let Some(type_def) = self.types.get(type_name) {
            return type_def
                .validate(value, self)
                .map_err(|e| make_err(e.to_string()));
        }

        // 2. Nested schema — type_name matches a registered schema name
        if let Some(nested_schema) = self.schemas.get(type_name) {
            let fields = nested_schema.fields.clone();
            return self
                .validate_inline_object_against_schema(value, type_name, &fields)
                .map_err(|e| make_err(e.to_string()));
        }

        // 3. Built-in types
        match resolve_builtin(type_name) {
            Ok(type_def) => type_def
                .validate(value, self)
                .map_err(|e| make_err(e.to_string())),
            Err(_) => Err(make_err(format!("Unknown type '{}'", type_name))),
        }
    }

    /// Validates an inline object literal `{ key = val, ... }` against the
    /// fields of the named nested schema.
    ///
    /// - Required fields (not marked `*`) declared in the schema must be present.
    /// - Optional fields (marked `*`) may be absent; if present they are validated.
    /// - Each value is validated against its declared type (recursively).
    fn validate_inline_object_against_schema(
        &self,
        value: &str,
        schema_name: &str,
        schema_fields: &HashMap<String, String>,
    ) -> Result<(), AamlError> {
        if !parsing::is_inline_object(value) {
            return Err(AamlError::InvalidValue {
                details: format!(
                    "Field typed as schema '{}' must be an inline object '{{ k = v, ... }}'",
                    schema_name
                ),
                expected: "inline object format: { key = value, ... }".to_string(),
                diagnostics: Some(crate::error::ErrorDiagnostics::new(
                    "Schema field must be an object",
                    format!(
                        "Expected inline object for schema '{}', got: '{}'",
                        schema_name, value
                    ),
                    "Use format: { key1 = value1, key2 = value2 }",
                )),
            });
        }

        let pairs = parsing::parse_inline_object(value)?;

        let pair_map: HashMap<&str, &str> = pairs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        // Fetch optional set from the registered schema (if still available).
        let optional_fields = self
            .schemas
            .get(schema_name)
            .map(|s| s.optional_fields.clone())
            .unwrap_or_default();

        for (field, type_name) in schema_fields.iter() {
            match pair_map.get(field.as_str()) {
                None => {
                    // Missing field — only an error for required fields
                    if !optional_fields.contains(field.as_str()) {
                        return Err(AamlError::SchemaValidationError {
                            schema: schema_name.to_string(),
                            field: field.clone(),
                            type_name: type_name.clone(),
                            details: format!(
                                "Missing field '{}' in inline object for schema '{}'",
                                field, schema_name
                            ),
                            diagnostics: Some(crate::error::ErrorDiagnostics::new(
                                "Missing required field",
                                format!(
                                    "Field '{}' is required in schema '{}' but not provided",
                                    field, schema_name
                                ),
                                format!("Add field: {} = <value>", field),
                            )),
                        });
                    }
                }
                Some(field_value) => {
                    self.validate_typed_field(type_name, field_value, schema_name, field)?;
                }
            }
        }

        Ok(())
    }

    /// Checks every **required** field in every registered schema against the current map.
    /// Optional fields (declared with `*`) are skipped.
    pub fn validate_schemas_completeness(&self) -> Result<(), AamlError> {
        let names: Vec<&str> = self.schemas.keys().map(|s| s.as_str()).collect();
        self.validate_schemas_completeness_for(&names)
    }

    // Medium Complexity
    /// Checks required fields only for the named schemas.
    /// Used by `@derive` to validate only child-defined schemas, not inherited ones.
    pub fn validate_schemas_completeness_for(
        &self,
        schema_names: &[&str],
    ) -> Result<(), AamlError> {
        for name in schema_names {
            let Some(schema_def) = self.schemas.get(*name) else {
                continue;
            };
            for (field, type_name) in &schema_def.fields {
                if schema_def.is_optional(field) {
                    continue;
                }
                if !self.map.contains_key(field.as_str()) {
                    return Err(AamlError::SchemaValidationError {
                        schema: name.to_string(),
                        field: field.clone(),
                        type_name: type_name.clone(),
                        details: format!("Missing required field '{field}'"),
                        diagnostics: Some(crate::error::ErrorDiagnostics::new(
                            "Missing required field in schema",
                            format!(
                                "Schema '{}' requires field '{}' of type '{}'",
                                name, field, type_name
                            ),
                            format!(
                                "Define the field in your configuration: {} = <value>",
                                field
                            ),
                        )),
                    });
                }
            }
        }
        Ok(())
    }

    /// Validates a complete `data` map against the named schema.
    ///
    /// For every **required** field declared in the schema the method checks:
    /// 1. The key is present in `data`.
    /// 2. The value satisfies the declared type (including nested schemas and lists).
    ///
    /// Optional fields (declared with `*`) are only validated when they are
    /// present in `data`; their absence is not an error.
    pub fn apply_schema(
        &self,
        schema_name: &str,
        data: &HashMap<String, String>,
    ) -> Result<(), AamlError> {
        let schema = self
            .schemas
            .get(schema_name)
            .ok_or_else(|| AamlError::NotFound {
                key: schema_name.to_string(),
                context: "schema registry".to_string(),
                diagnostics: Some(crate::error::ErrorDiagnostics::new(
                    "Schema not found",
                    format!("Schema '{}' does not exist", schema_name),
                    "Check your @schema definitions",
                )),
            })?;

        for (field, type_name) in &schema.fields {
            match data.get(field) {
                None => {
                    if !schema.optional_fields.contains(field.as_str()) {
                        return Err(AamlError::SchemaValidationError {
                            schema: schema_name.to_string(),
                            field: field.clone(),
                            type_name: type_name.clone(),
                            details: format!("Missing required field '{}'", field),
                            diagnostics: None,
                        });
                    }
                }
                Some(value) => {
                    self.validate_typed_field(type_name, value, schema_name, field)?;
                }
            }
        }

        Ok(())
    }
}
