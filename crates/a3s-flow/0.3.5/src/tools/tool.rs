//! Core tool abstraction.

use async_trait::async_trait;
use serde_json::Value;

/// Unified trait for all tools that can be invoked by the Agent node.
///
/// Implement this trait to add a new built-in or custom tool.
/// For OpenAI function-calling integration, tools are described via
/// the `parameters_schema()` method which returns an OpenAI-compatible
/// function definition.
///
/// # Example
///
/// ```ignore
/// struct HttpFetchTool;
///
/// #[async_trait]
/// impl Tool for HttpFetchTool {
///     fn tool_name(&self) -> &str { "http_fetch" }
///     fn description(&self) -> &str { "Perform an HTTP GET or POST request" }
///     fn parameters_schema(&self) -> Value {
///         json!({
///             "type": "object",
///             "properties": {
///                 "url": { "type": "string", "description": "The URL to fetch" },
///                 "method": { "type": "string", "enum": ["GET", "POST"] }
///             },
///             "required": ["url"]
///         })
///     }
///
///     async fn invoke(&self, args: Value) -> Result<ToolOutput> {
///         // ... implementation
///     }
/// }
/// ```
pub trait Tool: Send + Sync {
    /// Unique name for this tool (e.g. `"http_fetch"`).
    fn tool_name(&self) -> &str;

    /// Human-readable description of what the tool does.
    fn description(&self) -> &str;

    /// OpenAI function-calling compatible JSON Schema for the tool's parameters.
    ///
    /// This is directly passed as the `parameters` field of an OpenAI
    /// function definition.
    fn parameters_schema(&self) -> Value;

    /// Invoke the tool with the given arguments.
    ///
    /// `args` is a JSON object conforming to `parameters_schema()`.
    async fn invoke(&self, args: Value) -> Result<ToolOutput>;
}

/// A single tool invocation request.
#[derive(Debug, Clone)]
pub struct ToolCall {
    /// Name of the tool to invoke.
    pub name: String,
    /// Tool-specific arguments (JSON object).
    pub arguments: Value,
}

/// The result of a tool invocation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolOutput {
    /// Text content returned by the tool.
    pub content: String,
    /// Optional error message. If `None`, the invocation succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolOutput {
    /// Create a successful tool output.
    pub fn ok(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            error: None,
        }
    }

    /// Create a failed tool output.
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            content: String::new(),
            error: Some(message.into()),
        }
    }
}

use crate::error::{FlowError, Result};

/// Extension trait for converting a `Tool` implementation into an OpenAI
/// function-calling compatible JSON value.
pub trait ToolExt {
    fn to_function_definition(&self) -> Value;
}

impl<T: Tool> ToolExt for T {
    fn to_function_definition(&self) -> Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.tool_name(),
                "description": self.description(),
                "parameters": self.parameters_schema(),
            }
        })
    }
}
