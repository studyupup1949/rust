use regex::Regex;
use serde_json::Value;

use crate::{ActflowError, Result, runtime::Context};

/// Regex pattern for output template variables
/// Format: `{{#nodeId.key#}}` or `{{#nodeId.key.subkey#}}`
const OUTPUT_TEMPLATE_PATTERN: &str = r"\{\{#([^.#]+)\.([^#]+)#\}\}";
/// Regex pattern for environment variables
/// Format: `{{$VAR_NAME$}}`
const ENV_TEMPLATE_PATTERN: &str = r"\{\{\$([^$]+)\$\}\}";

/// Resolve template variables in the format `{{#nodeId.key#}}` and `{{$VAR_NAME$}}`
/// Returns error if any template variable cannot be resolved
pub fn resolve_template(
    ctx: &Context,
    template: &str,
) -> Result<String> {
    let mut result = template.to_string();
    let mut errors: Vec<String> = Vec::new();

    // First, resolve environment variables from context
    let env_re = Regex::new(ENV_TEMPLATE_PATTERN).unwrap();
    for caps in env_re.captures_iter(template) {
        let full_match = &caps[0];
        let var_name = &caps[1];

        match ctx.env().get(&var_name.to_string()) {
            Some(value) => {
                result = result.replace(full_match, &value);
            }
            None => {
                errors.push(format!("env variable '{}' not found", var_name));
            }
        }
    }

    // Then, resolve output template variables
    let re = Regex::new(OUTPUT_TEMPLATE_PATTERN).unwrap();
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
    let re = Regex::new(OUTPUT_TEMPLATE_PATTERN).unwrap();

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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use super::*;
    use crate::common::Vars;
    use crate::runtime::Channel;

    fn create_test_context() -> Context {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let channel = Arc::new(Channel::new(Arc::new(runtime)));
        Context::new("test-pid".to_string(), channel)
    }

    // ==================== resolve_template tests ====================

    #[test]
    fn test_resolve_template_no_variables() {
        let ctx = create_test_context();
        let result = resolve_template(&ctx, "hello world").unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_resolve_template_simple_output() {
        let ctx = create_test_context();
        let mut vars = Vars::new();
        vars.set("message", "hello");
        ctx.add_output("node1".to_string(), vars);

        let result = resolve_template(&ctx, "{{#node1.message#}}").unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_resolve_template_nested_output() {
        let ctx = create_test_context();
        let mut vars = Vars::new();
        vars.set("data", json!({"user": {"name": "Alice"}}));
        ctx.add_output("node1".to_string(), vars);

        let result = resolve_template(&ctx, "{{#node1.data.user.name#}}").unwrap();
        assert_eq!(result, "Alice");
    }

    #[test]
    fn test_resolve_template_number_output() {
        let ctx = create_test_context();
        let mut vars = Vars::new();
        vars.set("count", 42);
        ctx.add_output("node1".to_string(), vars);

        let result = resolve_template(&ctx, "count: {{#node1.count#}}").unwrap();
        assert_eq!(result, "count: 42");
    }

    #[test]
    fn test_resolve_template_bool_output() {
        let ctx = create_test_context();
        let mut vars = Vars::new();
        vars.set("active", true);
        ctx.add_output("node1".to_string(), vars);

        let result = resolve_template(&ctx, "active: {{#node1.active#}}").unwrap();
        assert_eq!(result, "active: true");
    }

    #[test]
    fn test_resolve_template_multiple_outputs() {
        let ctx = create_test_context();

        let mut vars1 = Vars::new();
        vars1.set("name", "Alice");
        ctx.add_output("node1".to_string(), vars1);

        let mut vars2 = Vars::new();
        vars2.set("age", 30);
        ctx.add_output("node2".to_string(), vars2);

        let result = resolve_template(&ctx, "{{#node1.name#}} is {{#node2.age#}} years old").unwrap();
        assert_eq!(result, "Alice is 30 years old");
    }

    #[test]
    fn test_resolve_template_missing_node() {
        let ctx = create_test_context();
        let result = resolve_template(&ctx, "{{#unknown.value#}}");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_resolve_template_missing_key() {
        let ctx = create_test_context();
        let mut vars = Vars::new();
        vars.set("name", "Alice");
        ctx.add_output("node1".to_string(), vars);

        let result = resolve_template(&ctx, "{{#node1.unknown#}}");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_template_env_variable() {
        let ctx = create_test_context();
        ctx.env().set("TEST_VAR".to_string(), "test_value".to_string());

        let result = resolve_template(&ctx, "{{$TEST_VAR$}}").unwrap();
        assert_eq!(result, "test_value");
    }

    #[test]
    fn test_resolve_template_env_variable_in_text() {
        let ctx = create_test_context();
        ctx.env().set("API_KEY".to_string(), "secret123".to_string());

        let result = resolve_template(&ctx, "API Key: {{$API_KEY$}}").unwrap();
        assert_eq!(result, "API Key: secret123");
    }

    #[test]
    fn test_resolve_template_missing_env_variable() {
        let ctx = create_test_context();
        let result = resolve_template(&ctx, "{{$NONEXISTENT_VAR$}}");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("env variable"));
    }

    #[test]
    fn test_resolve_template_mixed_env_and_output() {
        let ctx = create_test_context();
        ctx.env().set("PREFIX".to_string(), "Hello".to_string());

        let mut vars = Vars::new();
        vars.set("name", "World");
        ctx.add_output("node1".to_string(), vars);

        let result = resolve_template(&ctx, "{{$PREFIX$}}, {{#node1.name#}}!").unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_resolve_template_multiple_env_variables() {
        let ctx = create_test_context();
        ctx.env().set("HOST".to_string(), "localhost".to_string());
        ctx.env().set("PORT".to_string(), "8080".to_string());

        let result = resolve_template(&ctx, "http://{{$HOST$}}:{{$PORT$}}/api").unwrap();
        assert_eq!(result, "http://localhost:8080/api");
    }

    // ==================== resolve_template_to_values tests ====================

    #[test]
    fn test_resolve_to_values_no_template() {
        let ctx = create_test_context();
        let result = resolve_template_to_values(&ctx, "plain text").unwrap();
        assert_eq!(result, vec![Value::String("plain text".to_string())]);
    }

    #[test]
    fn test_resolve_to_values_single() {
        let ctx = create_test_context();
        let mut vars = Vars::new();
        vars.set("data", json!({"key": "value"}));
        ctx.add_output("node1".to_string(), vars);

        let result = resolve_template_to_values(&ctx, "{{#node1.data#}}").unwrap();
        assert_eq!(result, vec![json!({"key": "value"})]);
    }

    #[test]
    fn test_resolve_to_values_multiple() {
        let ctx = create_test_context();

        let mut vars1 = Vars::new();
        vars1.set("a", 1);
        ctx.add_output("node1".to_string(), vars1);

        let mut vars2 = Vars::new();
        vars2.set("b", 2);
        ctx.add_output("node2".to_string(), vars2);

        let result = resolve_template_to_values(&ctx, "{{#node1.a#}} and {{#node2.b#}}").unwrap();
        assert_eq!(result, vec![json!(1), json!(2)]);
    }

    // ==================== resolve_json_value tests ====================

    #[test]
    fn test_resolve_json_value_string() {
        let ctx = create_test_context();
        let mut vars = Vars::new();
        vars.set("msg", "hello");
        ctx.add_output("node1".to_string(), vars);

        let input = Value::String("{{#node1.msg#}}".to_string());
        let result = resolve_json_value(&ctx, &input).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_resolve_json_value_array() {
        let ctx = create_test_context();
        let mut vars = Vars::new();
        vars.set("x", "a");
        vars.set("y", "b");
        ctx.add_output("node1".to_string(), vars);

        let input = json!(["{{#node1.x#}}", "{{#node1.y#}}"]);
        let result = resolve_json_value(&ctx, &input).unwrap();
        assert_eq!(result, json!(["a", "b"]));
    }

    #[test]
    fn test_resolve_json_value_object() {
        let ctx = create_test_context();
        let mut vars = Vars::new();
        vars.set("name", "Alice");
        vars.set("age", 25);
        ctx.add_output("node1".to_string(), vars);

        let input = json!({
            "user": "{{#node1.name#}}",
            "years": "{{#node1.age#}}"
        });
        let result = resolve_json_value(&ctx, &input).unwrap();
        assert_eq!(result, json!({"user": "Alice", "years": "25"}));
    }

    #[test]
    fn test_resolve_json_value_nested_object() {
        let ctx = create_test_context();
        let mut vars = Vars::new();
        vars.set("value", "test");
        ctx.add_output("node1".to_string(), vars);

        let input = json!({
            "level1": {
                "level2": {
                    "data": "{{#node1.value#}}"
                }
            }
        });
        let result = resolve_json_value(&ctx, &input).unwrap();
        assert_eq!(result, json!({"level1": {"level2": {"data": "test"}}}));
    }

    #[test]
    fn test_resolve_json_value_non_string_passthrough() {
        let ctx = create_test_context();

        // Numbers should pass through unchanged
        let result = resolve_json_value(&ctx, &json!(42)).unwrap();
        assert_eq!(result, json!(42));

        // Booleans should pass through unchanged
        let result = resolve_json_value(&ctx, &json!(true)).unwrap();
        assert_eq!(result, json!(true));

        // Null should pass through unchanged
        let result = resolve_json_value(&ctx, &Value::Null).unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_resolve_json_value_json_string_parsing() {
        let ctx = create_test_context();
        let mut vars = Vars::new();
        vars.set("obj", json!({"foo": "bar"}));
        ctx.add_output("node1".to_string(), vars);

        let input = Value::String("{{#node1.obj#}}".to_string());
        let result = resolve_json_value(&ctx, &input).unwrap();
        // Should parse JSON string back to object
        assert_eq!(result, json!({"foo": "bar"}));
    }
}
