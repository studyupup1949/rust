//! Schema validation methods for [`AAML`](super::AAML).

use std::collections::HashMap;
use crate::error::AamlError;
use crate::types::{resolve_builtin};
use crate::types::list::ListType;
use crate::aaml::parsing;
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
        };

        // 1. Registered custom type alias
        if let Some(type_def) = self.types.get(type_name) {
            return type_def.validate(value).map_err(|e| make_err(e.to_string()));
        }

        // 2. Nested schema — type_name matches a registered schema name
        if let Some(nested_schema) = self.schemas.get(type_name) {
            return self
                .validate_inline_object_against_schema(value, type_name, nested_schema.fields.clone())
                .map_err(|e| make_err(e.to_string()));
        }

        // 3. list<T>
        if let Some(inner_type) = ListType::parse_inner(type_name) {
            return self
                .validate_list_value(value, &inner_type)
                .map_err(|e| make_err(e.to_string()));
        }

        // 4. Built-in types
        match resolve_builtin(type_name) {
            Ok(type_def) => type_def.validate(value).map_err(|e| make_err(e.to_string())),
            Err(_) => Err(make_err(format!("Unknown type '{}'", type_name))),
        }
    }

    /// Validates a `[item, item, ...]` literal where each item is validated
    /// against `inner_type`.
    ///
    /// If `inner_type` names a registered schema the items are treated as
    /// inline objects `{ k = v, ... }` and validated against that schema.
    /// Validates a `[item, …]` literal where each item is checked against `inner_type`.
    /// Items are split respecting nested `{}` / `[]`, so `list<Schema>` works correctly.
    fn validate_list_value(&self, value: &str, inner_type: &str) -> Result<(), AamlError> {
        let items = ListType::parse_items(value).ok_or_else(|| {
            AamlError::InvalidValue(format!("Expected a list literal '[…]', got '{value}'"))
        })?;

        for item in &items {
            if let Some(nested_schema) = self.schemas.get(inner_type) {
                let fields = nested_schema.fields.clone();
                self.validate_inline_object_against_schema(item, inner_type, fields)?;
            } else if let Ok(builtin) = resolve_builtin(inner_type) {
                builtin.validate(item).map_err(|e| {
                    AamlError::InvalidValue(format!(
                        "List item '{item}' failed for type '{inner_type}': {e}"
                    ))
                })?;
            } else if let Some(type_def) = self.types.get(inner_type) {
                type_def.validate(item).map_err(|e| {
                    AamlError::InvalidValue(format!(
                        "List item '{item}' failed for type '{inner_type}': {e}"
                    ))
                })?;
            } else {
                return Err(AamlError::NotFound(format!(
                    "Unknown list element type '{inner_type}'"
                )));
            }
        }
        Ok(())
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
        schema_fields: HashMap<String, String>,
    ) -> Result<(), AamlError> {
        if !parsing::is_inline_object(value) {
            return Err(AamlError::InvalidValue(format!(
                "Field typed as schema '{}' must be an inline object '{{ k = v, ... }}', got: '{}'",
                schema_name, value
            )));
        }

        let pairs = parsing::parse_inline_object(value).map_err(|e| {
            AamlError::InvalidValue(format!(
                "Failed to parse inline object for schema '{}': {}",
                schema_name, e
            ))
        })?;

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

        for (field, type_name) in &schema_fields {
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

    /// Checks required fields only for the named schemas.
    /// Used by `@derive` to validate only child-defined schemas, not inherited ones.
    pub fn validate_schemas_completeness_for(&self, schema_names: &[&str]) -> Result<(), AamlError> {
        for name in schema_names {
            let Some(schema_def) = self.schemas.get(*name) else { continue };
            for (field, type_name) in &schema_def.fields {
                if schema_def.is_optional(field) { continue; }
                if !self.map.contains_key(field.as_str()) {
                    return Err(AamlError::SchemaValidationError {
                        schema: name.to_string(),
                        field: field.clone(),
                        type_name: type_name.clone(),
                        details: format!("Missing required field '{field}'"),
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
        let schema = self.schemas.get(schema_name).ok_or_else(|| {
            AamlError::NotFound(format!("Schema '{}' not found", schema_name))
        })?;

        let fields: Vec<(String, String)> = schema
            .fields
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let optional = schema.optional_fields.clone();

        for (field, type_name) in &fields {
            match data.get(field) {
                None => {
                    if !optional.contains(field.as_str()) {
                        return Err(AamlError::SchemaValidationError {
                            schema: schema_name.to_string(),
                            field: field.clone(),
                            type_name: type_name.clone(),
                            details: format!("Missing required field '{}'", field),
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

