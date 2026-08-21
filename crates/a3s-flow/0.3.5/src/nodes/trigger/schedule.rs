//! `"trigger-schedule"` node — fires the workflow on a cron schedule.
//!
//! This node acts as the entry point of a DAG. When a workflow containing this
//! node is triggered by the SafeClaw scheduler, this node emits the trigger
//! payload (timestamp, cron expression, timezone) as output.
//!
//! The scheduler lives in the SafeClaw app layer, not in a3s-flow. This node
//! only validates the cron configuration and produces the trigger payload.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "cron": "0 9 * * *",
//!   "timezone": "Asia/Shanghai"
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|----------|-------------|
//! | `cron` | string | ✅ | 5-field cron expression (standard cron format) |
//! | `timezone` | string | — | IANA timezone name (default: "UTC") |
//!
//! # Output schema
//!
//! ```json
//! {
//!   "cron": "0 9 * * *",
//!   "timezone": "Asia/Shanghai",
//!   "fired_at": 1710000000000,
//!   "description": "At 09:00"
//! }
//! ```

use async_trait::async_trait;
use serde_json::Value;
use std::str::FromStr;

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

use super::protocol::ScheduleTriggerPayload;

pub struct TriggerScheduleNode;

#[async_trait]
impl Node for TriggerScheduleNode {
    fn node_type(&self) -> &str {
        "trigger-schedule"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        // Support both Dify-style "cron" and a3s-style "cron_expression"
        let cron_expr = ctx
            .data
            .get("cron")
            .or_else(|| ctx.data.get("cron_expression"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                FlowError::InvalidDefinition(
                    "trigger-schedule: missing required field 'cron' (or 'cron_expression')"
                        .to_string(),
                )
            })?;

        // Validate the cron expression can be parsed.
        // The cron crate expects 6 fields (sec min hour day_of_month month day_of_week).
        // Standard cron is 5 fields (min hour day_of_month month day_of_week), so we prepend "0 " for seconds.
        let cron_with_seconds = if cron_expr.split_whitespace().count() == 5 {
            format!("0 {}", cron_expr)
        } else {
            cron_expr.to_string()
        };
        cron::Schedule::try_from(cron_with_seconds.as_str()).map_err(|e| {
            FlowError::InvalidDefinition(format!(
                "trigger-schedule: invalid cron expression '{}': {}",
                cron_expr, e
            ))
        })?;

        let timezone_str = ctx
            .data
            .get("timezone")
            .and_then(|v| v.as_str())
            .unwrap_or("UTC");

        // Validate timezone.
        let _tz = chrono_tz::Tz::from_str(timezone_str).map_err(|_| {
            FlowError::InvalidDefinition(format!(
                "trigger-schedule: unknown timezone '{}'",
                timezone_str
            ))
        })?;

        let fired_at = chrono::Utc::now().timestamp_millis();

        // Build a human-readable description from the cron expression.
        let description = describe_cron(cron_expr);

        let payload = ScheduleTriggerPayload {
            cron: cron_expr.to_string(),
            timezone: timezone_str.to_string(),
            fired_at,
            description,
        };

        Ok(serde_json::to_value(payload).expect("payload is serializable"))
    }
}

/// Returns a short human-readable description for a cron expression.
fn describe_cron(expr: &str) -> Option<String> {
    use std::fmt::Write;

    // Prepend "0 " for 5-field cron expressions (standard cron format)
    let cron_with_seconds = if expr.split_whitespace().count() == 5 {
        format!("0 {}", expr)
    } else {
        expr.to_string()
    };
    let schedule = cron::Schedule::try_from(cron_with_seconds.as_str()).ok()?;

    // Get the next few runs to give a sense of the schedule.
    let next: Vec<String> = schedule
        .upcoming(chrono::Utc)
        .take(3)
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .collect();

    if next.is_empty() {
        return None;
    }

    let mut desc = String::new();
    if next.len() == 1 {
        write!(&mut desc, "Next run: {}", next[0]).ok();
    } else {
        write!(&mut desc, "Next runs: {}", next.join(", ")).ok();
    }
    Some(desc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn ctx(data: Value) -> ExecContext {
        ExecContext {
            data,
            variables: HashMap::new(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn valid_cron_emits_payload() {
        let node = TriggerScheduleNode;
        let out = node
            .execute(ctx(
                json!({ "cron": "0 9 * * *", "timezone": "Asia/Shanghai" }),
            ))
            .await
            .unwrap();
        assert_eq!(out["cron"], "0 9 * * *");
        assert_eq!(out["timezone"], "Asia/Shanghai");
        assert!(out["fired_at"].is_number());
        assert!(out["description"].is_string());
    }

    #[tokio::test]
    async fn default_timezone_is_utc() {
        let node = TriggerScheduleNode;
        let out = node
            .execute(ctx(json!({ "cron": "0 9 * * *" })))
            .await
            .unwrap();
        assert_eq!(out["timezone"], "UTC");
    }

    #[tokio::test]
    async fn missing_cron_returns_error() {
        let node = TriggerScheduleNode;
        let result = node.execute(ctx(json!({}))).await;
        assert!(matches!(result, Err(FlowError::InvalidDefinition(_))));
    }

    #[tokio::test]
    async fn invalid_cron_returns_error() {
        let node = TriggerScheduleNode;
        let result = node.execute(ctx(json!({ "cron": "not a cron" }))).await;
        assert!(matches!(result, Err(FlowError::InvalidDefinition(_))));
    }

    #[tokio::test]
    async fn invalid_timezone_returns_error() {
        let node = TriggerScheduleNode;
        let result = node
            .execute(ctx(
                json!({ "cron": "0 9 * * *", "timezone": "Invalid/TZ" }),
            ))
            .await;
        assert!(matches!(result, Err(FlowError::InvalidDefinition(_))));
    }
}
