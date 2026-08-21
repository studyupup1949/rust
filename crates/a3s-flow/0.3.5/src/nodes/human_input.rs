//! `"human-input"` node — pauses workflow execution to wait for human input.
//!
//! This node suspends workflow execution and waits for human confirmation or
//! form input before proceeding. It supports timeout handling.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "inputs": [],
//!   "user_actions": [],
//!   "timeout": 3,
//!   "timeout_unit": "day",
//!   "form_content": ""
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `inputs` | array | — | Input field definitions for the form |
//! | `user_actions` | array | — | Available action buttons |
//! | `timeout` | number | — | Timeout value (default: 3) |
//! | `timeout_unit` | string | — | Time unit: second, minute, hour, day (default: day) |
//! | `form_content` | string | — | Form content description |
//!
//! # Output schema
//!
//! ```json
//! {
//!   "__action_id": "approve_001",
//!   "__rendered_content": "...",
//!   ...<input fields>
//! }
//! ```

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::error::Result;
use crate::node::{ExecContext, Node};

/// Human Input node — pauses workflow to wait for human confirmation or input.
pub struct HumanInputNode;

#[async_trait]
impl Node for HumanInputNode {
    fn node_type(&self) -> &str {
        "human-input"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        // Parse timeout configuration
        let timeout = ctx
            .data
            .get("timeout")
            .and_then(|v| v.as_i64())
            .unwrap_or(3);

        let timeout_unit = ctx
            .data
            .get("timeout_unit")
            .and_then(|v| v.as_str())
            .unwrap_or("day");

        // Parse input fields
        let inputs = ctx
            .data
            .get("inputs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // Parse user actions
        let user_actions = ctx
            .data
            .get("user_actions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // Form content
        let form_content = ctx
            .data
            .get("form_content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Calculate timeout in seconds
        let timeout_seconds = match timeout_unit {
            "second" => timeout as u64,
            "minute" => timeout as u64 * 60,
            "hour" => timeout as u64 * 3600,
            "day" => timeout as u64 * 86400,
            _ => timeout as u64 * 86400, // default to days
        };

        // Build lookup from variables and inputs
        let mut lookup: HashMap<String, Value> = ctx.variables.into_iter().collect();
        for (k, v) in &ctx.inputs {
            lookup.insert(k.clone(), v.clone());
        }

        // Collect input field values from the lookup
        let mut input_values: HashMap<String, Value> = HashMap::new();
        for input_name in &inputs {
            if let Some(value) = lookup.get(input_name) {
                input_values.insert(input_name.clone(), value.clone());
            } else {
                input_values.insert(input_name.clone(), Value::Null);
            }
        }

        // Generate a unique action ID for this human input instance
        let action_id = format!(
            "action_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        );

        // Build output — includes action_id, form_content, and input values
        let mut output: HashMap<String, Value> = HashMap::new();
        output.insert("__action_id".to_string(), json!(action_id));
        output.insert("__rendered_content".to_string(), json!(form_content));
        output.insert("__timeout_seconds".to_string(), json!(timeout_seconds));
        output.insert("__user_actions".to_string(), json!(user_actions));

        // Merge input field values
        for (k, v) in input_values {
            output.insert(k, v);
        }

        Ok(json!({ "output": Value::Object(output.into_iter().collect()) }))
    }
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
    async fn human_input_returns_action_id() {
        let node = HumanInputNode;
        let out = node
            .execute(ctx_with_data(json!({
                "timeout": 1,
                "timeout_unit": "day",
                "inputs": ["name", "email"],
                "user_actions": ["approve", "reject"],
                "form_content": "Please confirm your details"
            })))
            .await
            .unwrap();

        let output_obj = out["output"].as_object().unwrap();
        assert!(output_obj.contains_key("__action_id"));
        assert_eq!(
            output_obj.get("__rendered_content").unwrap(),
            "Please confirm your details"
        );
        assert_eq!(output_obj.get("__timeout_seconds").unwrap(), &json!(86400)); // 1 day in seconds
        assert_eq!(
            output_obj.get("__user_actions").unwrap(),
            &json!(["approve", "reject"])
        );
        assert!(output_obj.contains_key("name"));
        assert!(output_obj.contains_key("email"));
    }

    #[tokio::test]
    async fn human_input_default_timeout() {
        let node = HumanInputNode;
        let out = node.execute(ctx_with_data(json!({}))).await.unwrap();

        let output_obj = out["output"].as_object().unwrap();
        assert_eq!(output_obj.get("__timeout_seconds").unwrap(), &json!(259200));
        // 3 days in seconds
    }

    #[tokio::test]
    async fn timeout_unit_conversion() {
        let node = HumanInputNode;

        // Test seconds
        let out = node
            .execute(ctx_with_data(
                json!({ "timeout": 30, "timeout_unit": "second" }),
            ))
            .await
            .unwrap();
        assert_eq!(out["output"]["__timeout_seconds"], json!(30));

        // Test minutes
        let out = node
            .execute(ctx_with_data(
                json!({ "timeout": 5, "timeout_unit": "minute" }),
            ))
            .await
            .unwrap();
        assert_eq!(out["output"]["__timeout_seconds"], json!(300));

        // Test hours
        let out = node
            .execute(ctx_with_data(
                json!({ "timeout": 2, "timeout_unit": "hour" }),
            ))
            .await
            .unwrap();
        assert_eq!(out["output"]["__timeout_seconds"], json!(7200));
    }
}
