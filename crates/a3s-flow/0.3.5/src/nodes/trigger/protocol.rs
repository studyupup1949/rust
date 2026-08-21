//! Shared types for trigger nodes.

use serde::{Deserialize, Serialize};

/// Payload emitted by a `"trigger-schedule"` node when its workflow fires.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTriggerPayload {
    /// The cron expression that triggered this run.
    pub cron: String,
    /// The timezone used to evaluate the cron expression.
    pub timezone: String,
    /// Unix timestamp in milliseconds when the trigger fired.
    pub fired_at: i64,
    /// Human-readable description of the cron expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Payload emitted by a `"trigger-webhook"` node when its workflow fires.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookTriggerPayload {
    /// HTTP method of the incoming webhook request.
    pub method: String,
    /// URL path that was called.
    pub path: String,
    /// Request headers.
    pub headers: std::collections::HashMap<String, String>,
    /// Parsed JSON body, or raw text if not JSON.
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub body: serde_json::Value,
    /// Raw body string (available for non-JSON bodies).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_body: Option<String>,
}
