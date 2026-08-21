use crate::error::AamlError;
use crate::pipeline::execution_descriptor::ExecutionContext;

/// Validates a value against a type, handling built-ins, registered custom types, and nested schemas.
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

    // 2. Nested schema — type_name matches a registered schema name
    if let Some(schema_info) = context.schemas.get(type_name) {
        return validate_inline_object_against_schema(value, schema_info, context);
    }

    // 3. Built-in types
    match crate::types_aam::resolve_builtin(type_name) {
        Ok(validator) => validator.validate(value, context),
        Err(_) => Err(AamlError::InvalidType {
            type_name: type_name.to_string(),
            details: format!("Unknown type '{}'", type_name),
            provided: value.to_string(),
            diagnostics: None,
        }),
    }
}

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
