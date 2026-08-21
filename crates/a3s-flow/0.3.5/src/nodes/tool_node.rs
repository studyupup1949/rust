//! `"tool"` node — invokes a built-in tool (HTTP Fetch, Calculator, etc.).
//!
//! This node acts as a generic tool invoker for built-in tools.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "tool_name": "http_fetch",
//!   "tool_parameters": {
//!     "url": "https://api.example.com/data"
//!   }
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `tool_name` | string | ✅ | Name of the tool to invoke |
//! | `tool_parameters` | object | ✅ | Tool-specific arguments |
//!
//! # Built-in tools
//!
//! | Tool | Description |
//! |------|-------------|
//! | `http_fetch` | Perform HTTP GET/POST/PUT/DELETE/PATCH requests |
//! | `calculator` | Evaluate mathematical expressions |
//!
//! # Output schema
//!
//! ```json
//! {
//!   "text": "...",
//!   "files": [],
//!   "json": { ... }
//! }
//! ```

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

/// Built-in tool invocation result.
struct ToolInvokeResult {
    text: String,
    json: Value,
}

/// Tool node — invokes a built-in tool and returns its output.
pub struct ToolNode;

impl ToolNode {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ToolNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Render a JSON value as a Jinja2 template string if it's a string,
/// otherwise return the value unchanged.
fn render_with_context(value: &Value, context: &HashMap<String, Value>) -> Result<Value> {
    match value {
        Value::String(s) => {
            let env = minijinja::Environment::new();
            let rendered = env
                .render_str(s, context)
                .map_err(|e| FlowError::InvalidDefinition(format!("Jinja2 render error: {}", e)))?;
            // Try to parse as JSON, fall back to string
            Ok(serde_json::from_str(&rendered).unwrap_or(Value::String(rendered)))
        }
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                new_map.insert(k.clone(), render_with_context(v, context)?);
            }
            Ok(Value::Object(new_map))
        }
        Value::Array(arr) => {
            let mut rendered = Vec::new();
            for v in arr {
                rendered.push(render_with_context(v, context)?);
            }
            Ok(Value::Array(rendered))
        }
        _ => Ok(value.clone()),
    }
}

#[async_trait]
impl Node for ToolNode {
    fn node_type(&self) -> &str {
        "tool"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        // Parse tool name
        let tool_name = ctx
            .data
            .get("tool_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                FlowError::InvalidDefinition("tool: missing required field 'tool_name'".into())
            })?
            .to_string();

        // Parse tool parameters
        let tool_parameters = ctx
            .data
            .get("tool_parameters")
            .cloned()
            .unwrap_or(Value::Object(serde_json::Map::new()));

        // Build context for tool invocation (merge variables and inputs)
        let mut context: HashMap<String, Value> = ctx.variables.into_iter().collect();
        for (k, v) in &ctx.inputs {
            context.insert(k.clone(), v.clone());
        }

        // Render tool parameters with Jinja2 template if needed
        let rendered_params = render_with_context(&tool_parameters, &context).map_err(|e| {
            FlowError::InvalidDefinition(format!("tool: failed to render parameters: {}", e))
        })?;

        // Invoke the appropriate tool
        let result = match tool_name.as_str() {
            "http_fetch" => invoke_http_fetch(&rendered_params).await,
            "calculator" => invoke_calculator(&rendered_params).await,
            _ => Err(FlowError::InvalidDefinition(format!(
                "tool: unknown tool '{}' (available: http_fetch, calculator)",
                tool_name
            ))),
        }?;

        Ok(json!({
            "text": result.text,
            "files": [],
            "json": result.json,
        }))
    }
}

/// Invoke the HTTP Fetch built-in tool.
async fn invoke_http_fetch(args: &Value) -> Result<ToolInvokeResult> {
    use reqwest::Client;

    let url = args["url"]
        .as_str()
        .ok_or_else(|| FlowError::InvalidDefinition("http_fetch: url is required".into()))?;

    let method = args["method"].as_str().unwrap_or("GET");
    let headers = args["headers"].as_object().cloned().unwrap_or_default();
    let body = args.get("body").cloned();

    let client = Client::new();
    let mut req = match method.to_ascii_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        other => {
            return Err(FlowError::InvalidDefinition(format!(
                "http_fetch: unsupported method '{}'",
                other
            )));
        }
    };

    for (name, val) in headers {
        if let Some(v) = val.as_str() {
            req = req.header(name.as_str(), v);
        }
    }

    if let Some(b) = body {
        if !b.is_null() {
            req = req.json(&b);
        }
    }

    let response = req
        .send()
        .await
        .map_err(|e| FlowError::Internal(format!("http_fetch: request failed: {}", e)))?;

    let status = response.status().as_u16();
    let body_text = response
        .text()
        .await
        .map_err(|e| FlowError::Internal(format!("http_fetch: failed to read response: {}", e)))?;

    let body_json: Value = serde_json::from_str(&body_text).unwrap_or(Value::String(body_text));

    let result = json!({
        "status": status,
        "body": body_json,
    });

    Ok(ToolInvokeResult {
        text: serde_json::to_string(&result).unwrap_or_default(),
        json: result,
    })
}

/// Invoke the Calculator built-in tool.
async fn invoke_calculator(args: &Value) -> Result<ToolInvokeResult> {
    let expression = args["expression"]
        .as_str()
        .ok_or_else(|| FlowError::InvalidDefinition("calculator: expression is required".into()))?;

    let result = meval::eval_str(expression)
        .map_err(|e| FlowError::InvalidDefinition(format!("calculator: {}", e)))?;

    let result_json = json!({ "result": result });

    Ok(ToolInvokeResult {
        text: result.to_string(),
        json: result_json,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with_data(data: Value) -> ExecContext {
        ExecContext {
            data,
            inputs: HashMap::new(),
            variables: HashMap::new(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn tool_node_requires_tool_name() {
        let node = ToolNode::new();
        let err = node
            .execute(ctx_with_data(json!({ "tool_parameters": {} })))
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn tool_node_unknown_tool() {
        let node = ToolNode::new();
        let err = node
            .execute(ctx_with_data(json!({
                "tool_name": "nonexistent_tool",
                "tool_parameters": {}
            })))
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn calculator_basic() {
        let node = ToolNode::new();
        let out = node
            .execute(ctx_with_data(json!({
                "tool_name": "calculator",
                "tool_parameters": { "expression": "2 + 3 * 4" }
            })))
            .await
            .unwrap();
        assert!(out["json"]["result"].as_f64().is_some());
        assert!((out["json"]["result"].as_f64().unwrap() - 14.0).abs() < 1e-9);
    }
}
