//! `"http_fetch"` built-in tool — performs HTTP GET/POST requests.

use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::error::{FlowError, Result};
use crate::tools::tool::{Tool, ToolOutput};

/// HTTP Fetch tool — supports GET, POST, PUT, DELETE, PATCH.
pub struct HttpFetchTool;

impl HttpFetchTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HttpFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for HttpFetchTool {
    fn tool_name(&self) -> &str {
        "http_fetch"
    }

    fn description(&self) -> &str {
        "Perform an HTTP GET, POST, PUT, DELETE, or PATCH request and return the response status and body."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to request"
                },
                "method": {
                    "type": "string",
                    "enum": ["GET", "POST", "PUT", "DELETE", "PATCH"],
                    "description": "HTTP method (default: GET)"
                },
                "headers": {
                    "type": "object",
                    "description": "Request headers as key-value pairs"
                },
                "body": {
                    "type": "object",
                    "description": "JSON request body (for POST/PUT/PATCH)"
                }
            },
            "required": ["url"]
        })
    }

    async fn invoke(&self, args: Value) -> Result<ToolOutput> {
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

        let body_json: Value =
            serde_json::from_str(&body_text).unwrap_or(Value::String(body_text));

        let result = json!({
            "status": status,
            "body": body_json,
        });

        Ok(ToolOutput::ok(
            serde_json::to_string(&result)
                .map_err(|e| FlowError::Internal(format!(
                    "http_fetch: failed to serialize result: {}",
                    e
                )))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn http_fetch_rejects_missing_url() {
        let tool = HttpFetchTool::new();
        let result = tool.invoke(json!({})).await;
        assert!(result.is_err());
    }
}
