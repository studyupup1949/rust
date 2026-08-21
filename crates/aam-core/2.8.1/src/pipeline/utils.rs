use crate::error::AamlError;
use crate::pipeline::execution_descriptor::ExecutionContext;
use std::path::{Path, PathBuf};

/// Resolves a file path relative to a source directory.
///
/// If `file_path` is absolute, returns it as-is.
/// If `source_dir` is `Some`, joins it with the relative `file_path`.
/// Otherwise, returns `file_path` unchanged (relative to CWD).
///
/// # Errors
///
/// This function never returns an error; it always produces a `PathBuf`.
#[must_use]
pub fn resolve_relative_path(file_path: &str, source_dir: Option<&Path>) -> PathBuf {
    let path = Path::new(file_path);
    if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(dir) = source_dir {
        dir.join(path)
    } else {
        path.to_path_buf()
    }
}

/// Validates a value against a type, handling built-ins, registered custom types, and nested schemas.
/// Validates a value against a type, handling built-ins, registered custom types, and nested schemas.
///
/// # Errors
///
/// Returns `Err(AamlError)` when:
/// - The value does not match the expected inline object format for `schema`.
/// - The referenced schema is missing required fields, or a field value fails its type validation.
/// - The requested type is unknown.
pub fn validate_type_value(
    value: &str,
    type_name: &str,
    context: &ExecutionContext,
) -> Result<(), AamlError> {
    // 1. Registered custom type alias
    if let Some(custom_type) = context.types.get(type_name) {
        // Built-in defaults are registered as identity specs (e.g., string -> string).
        // Skip recursive alias expansion for self-mapped entries to avoid infinite recursion.
        if custom_type.spec != type_name {
            return validate_type_value(value, &custom_type.spec, context);
        }
    }

    // 2. "schema" type — a generic inline object type (no field-level validation)
    if type_name == "schema" {
        if !crate::aaml::parsing::is_inline_object(value) {
            return Err(AamlError::InvalidValue {
                details: format!(
                    "Value for type 'schema' must be an inline object '{{ k = v, ... }}', got: '{value}'"
                ),
                expected: "inline object format: { key = value, ... }".to_string(),
                diagnostics: None,
            });
        }
        return Ok(());
    }

    // 3. Nested schema — type_name matches a registered schema name
    if let Some(schema_info) = context.schemas.get(type_name) {
        return validate_inline_object_against_schema(value, schema_info, context);
    }

    // 4. Built-in types
    crate::types_aam::resolve_builtin(type_name).map_or_else(
        |_| {
            Err(AamlError::InvalidType {
                type_name: type_name.to_string(),
                details: format!("Unknown type '{type_name}'"),
                provided: value.to_string(),
                diagnostics: None,
            })
        },
        |validator| validator.validate(value, context),
    )
}

/// Validates an inline object string against a schema definition.
///
/// # Errors
///
/// Returns `Err(AamlError)` when:
/// - `value` is not a valid inline object string (`{ k = v, ... }`).
/// - A required field in the schema is missing.
/// - A field's value fails its type validation.
pub fn validate_inline_object_against_schema(
    value: &str,
    schema_info: &crate::pipeline::execution_descriptor::SchemaInfo,
    context: &ExecutionContext,
) -> Result<(), AamlError> {
    if !crate::aaml::parsing::is_inline_object(value) {
        return Err(AamlError::InvalidValue {
            details: format!(
                "Field typed as schema '{}' must be an inline object '{{ k = v, ... }}'",
                schema_info.name
            ),
            expected: "inline object format: { key = value, ... }".to_string(),
            diagnostics: None,
        });
    }

    let pairs = crate::aaml::parsing::parse_inline_object(value)?;

    let mut pair_map = std::collections::HashMap::new();
    for (k, v) in &pairs {
        pair_map.insert(k.as_str(), v.as_str());
    }

    for (field, (type_name, is_optional)) in &schema_info.fields {
        match pair_map.get(field.as_str()) {
            None => {
                if !is_optional {
                    return Err(AamlError::SchemaValidationError {
                        schema: schema_info.name.to_string(),
                        field: field.to_string(),
                        type_name: type_name.to_string(),
                        details: format!(
                            "Missing required field '{}' in inline object for schema '{}'",
                            field, schema_info.name
                        ),
                        diagnostics: None,
                    });
                }
            }
            Some(field_value) => {
                validate_type_value(field_value, type_name, context)?;
            }
        }
    }

    Ok(())
}
