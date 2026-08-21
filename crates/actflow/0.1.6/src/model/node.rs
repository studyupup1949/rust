//! Node model for workflow definition.

use serde::{Deserialize, Serialize};

/// Represents a node in a workflow definition.
///
/// Nodes are the building blocks of a workflow, each performing a specific action
/// such as HTTP requests, code execution, conditional branching, or agent calls.
///
/// # Example
///
/// ```rust
/// use actflow::NodeModel;
/// use serde_json::json;
///
/// let node = NodeModel {
///     id: "http_node".to_string(),
///     title: "Fetch Data".to_string(),
///     desc: "Fetch data from API".to_string(),
///     uses: "http_request".to_string(),
///     action: json!({
///         "url": "https://api.example.com/data",
///         "method": "GET"
///     }),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeModel {
    /// Unique identifier for this node within the workflow.
    pub id: String,
    /// Human-readable title for display purposes.
    pub title: String,
    /// Description of what this node does.
    pub desc: String,
    /// Action type to use (e.g., "start", "http_request", "code", "if_else", "agent", "end").
    pub uses: String,
    /// Error handling strategy: "fail_branch" or "continue".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_strategy: Option<String>,
    /// Retry configuration for failed executions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<serde_json::Value>,
    /// Execution timeout in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Action-specific configuration parameters.
    pub action: serde_json::Value,
}
