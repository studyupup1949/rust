//! Trigger nodes — workflow entry points that fire based on schedule or webhook.
//!
//! These nodes are the **root** of a workflow DAG. They are not triggered by
//! upstream nodes; instead, the SafeClaw app layer fires the workflow based on
//! either a cron schedule or an incoming HTTP webhook request.
//!
//! | Node type | Description |
//! |-----------|-------------|
//! | `trigger-schedule` | Fires on a cron schedule |
//! | `trigger-webhook` | Fires when an HTTP request is received |
//!
//! When a workflow is triggered, the app layer creates an execution with the
//! trigger payload injected as flow variables. The trigger node then validates
//! its configuration and emits the trigger payload as its output for downstream
//! nodes to consume.

pub mod protocol;
pub mod schedule;
pub mod webhook;

pub use schedule::TriggerScheduleNode;
pub use webhook::TriggerWebhookNode;
