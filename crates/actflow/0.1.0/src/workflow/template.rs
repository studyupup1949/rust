use regex::Regex;
use serde_json::Value;

use crate::{ActflowError, Result, runtime::Context};

/// Resolve template variables in the format `{{#nodeId.key#}}`
/// Returns error if any template variable cannot be resolved
pub fn resolve_template(
    ctx: &Context,
    template: &str,
) -> Result<String> {
    // Regex pattern: {{#nodeId.key#}} or {{#nodeId.key.subkey#}}
    let re = Regex::new(r"\{\{#([^.#]+)\.([^#]+)#\}\}").unwrap();

    let mut result = template.to_string();
    let mut errors: Vec<String> = Vec::new();

    for caps in re.captures_iter(template) {
        let full_match = &caps[0];
        let node_id = &caps[1];
        let key_path = &caps[2];

        // Get outputs for the node
        let resolved_value = if let Some(node_outputs) = ctx.outputs().get(&node_id.to_string()) {
            // Handle nested keys like "result.data.value"
            let keys: Vec<&str> = key_path.split('.').collect();
            let mut current_value: Option<Value> = None;

            // Get the first key
            if let Some(first_key) = keys.first() {
                current_value = node_outputs.get::<Value>(first_key);

                // Traverse nested keys
                for key in keys.iter().skip(1) {
                    if let Some(ref val) = current_value {
                        current_value = val.get(key).cloned();
                    } else {
                        break;
                    }
                }
            }

            // Convert value to string
            match current_value {
                Some(Value::String(s)) => Some(s),
                Some(Value::Number(n)) => Some(n.to_string()),
                Some(Value::Bool(b)) => Some(b.to_string()),
                Some(Value::Null) => Some("null".to_string()),
                Some(v) => Some(v.to_string()), // For objects/arrays, use JSON string
                None => None,
            }
        } else {
            None
        };

        match resolved_value {
            Some(value) => {
                result = result.replace(full_match, &value);
            }
            None => {
                errors.push(format!("variable '{}' not found", full_match));
            }
        }
    }

    if !errors.is_empty() {
        return Err(ActflowError::Runtime(errors.join(", ")));
    }

    Ok(result)
}

/// Resolve template variables and return all matched values as a Vec
/// Format: `{{#nodeId.key#}}` or `{{#nodeId.key.subkey#}}`
pub fn resolve_template_to_values(
    ctx: &Context,
    template: &str,
) -> Result<Vec<Value>> {
    let re = Regex::new(r"\{\{#([^.#]+)\.([^#]+)#\}\}").unwrap();

    let mut values: Vec<Value> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for caps in re.captures_iter(template) {
        let full_match = &caps[0];
        let node_id = &caps[1];
        let key_path = &caps[2];

        let resolved_value = if let Some(node_outputs) = ctx.outputs().get(&node_id.to_string()) {
            let keys: Vec<&str> = key_path.split('.').collect();
            let mut current_value: Option<Value> = None;

            if let Some(first_key) = keys.first() {
                current_value = node_outputs.get::<Value>(first_key);

                for key in keys.iter().skip(1) {
                    if let Some(ref val) = current_value {
                        current_value = val.get(key).cloned();
                    } else {
                        break;
                    }
                }
            }
            current_value
        } else {
            None
        };

        match resolved_value {
            Some(value) => values.push(value),
            None => errors.push(format!("variable '{}' not found", full_match)),
        }
    }

    if !errors.is_empty() {
        return Err(ActflowError::Runtime(errors.join(", ")));
    }

    // If no template variables found, return the original template as a Value
    if values.is_empty() {
        return Ok(vec![Value::String(template.to_string())]);
    }

    Ok(values)
}

/// Resolve template variables in a JSON Value recursively
pub fn resolve_json_value(
    ctx: &Context,
    value: &Value,
) -> Result<Value> {
    match value {
        Value::String(s) => {
            let resolved = resolve_template(ctx, s)?;
            // Try to parse as JSON if the resolved string looks like JSON
            if resolved.starts_with('{') || resolved.starts_with('[') {
                Ok(serde_json::from_str(&resolved).unwrap_or(Value::String(resolved)))
            } else {
                Ok(Value::String(resolved))
            }
        }
        Value::Array(arr) => {
            let resolved: Result<Vec<Value>> = arr.iter().map(|v| resolve_json_value(ctx, v)).collect();
            Ok(Value::Array(resolved?))
        }
        Value::Object(obj) => {
            let resolved: Result<serde_json::Map<String, Value>> = obj.iter().map(|(k, v)| resolve_json_value(ctx, v).map(|rv| (k.clone(), rv))).collect();
            Ok(Value::Object(resolved?))
        }
        _ => Ok(value.clone()),
    }
}
