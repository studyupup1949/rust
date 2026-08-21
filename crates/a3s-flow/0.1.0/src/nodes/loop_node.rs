//! `"loop"` node — while-loop over an inline sub-flow.
//!
//! Repeatedly executes a sub-flow until either a `break_condition` is satisfied
//! or `max_iterations` is reached. The last iteration's collected output is
//! returned. Each iteration receives the previous one's output as a variable,
//! enabling accumulation and chaining patterns.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "flow":            { "nodes": [...], "edges": [...] },
//!   "output_selector": "step.result",
//!   "max_iterations":  10,
//!   "break_condition": { "from": "step", "path": "done", "op": "eq", "value": true }
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `flow` | object | ✅ | Inline sub-flow definition (`{ "nodes", "edges" }`) |
//! | `output_selector` | string | ✅ | Dot path into sub-flow outputs to collect each iteration (`"node_id"` or `"node_id.field"`) |
//! | `max_iterations` | integer | — | Safety cap (default `10`, minimum `1`) |
//! | `break_condition` | Condition | — | If provided, the loop stops when this condition evaluates to true against the sub-flow's outputs; without it the loop always runs `max_iterations` times |
//!
//! ## Variables injected into each iteration's sub-flow
//!
//! | Variable | Value |
//! |----------|-------|
//! | `iteration_index` | 0-based iteration counter |
//! | `loop_output` | The previous iteration's collected output (`null` for the first iteration) |
//!
//! # Output schema
//!
//! ```json
//! { "output": <last_output_selector_result>, "iterations": 3 }
//! ```
//!
//! # Example — retry until success
//!
//! ```json
//! {
//!   "id": "retry",
//!   "type": "loop",
//!   "data": {
//!     "max_iterations": 5,
//!     "output_selector": "check.ok",
//!     "break_condition": { "from": "check", "path": "ok", "op": "eq", "value": true },
//!     "flow": {
//!       "nodes": [
//!         { "id": "fetch", "type": "http-request", "data": { "url": "https://api.example.com/status" } },
//!         { "id": "check", "type": "if-else", "data": { "cases": [
//!           { "id": "ok", "conditions": [{ "from": "fetch", "path": "status", "op": "eq", "value": 200 }] }
//!         ]}}
//!       ],
//!       "edges": [{ "source": "fetch", "target": "check" }]
//!     }
//!   }
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::condition::{get_path, Condition};
use crate::error::{FlowError, Result};
use crate::graph::DagGraph;
use crate::node::{ExecContext, Node};
use crate::runner::FlowRunner;

/// Loop node — while-loop over a sub-flow.
pub struct LoopNode;

#[async_trait]
impl Node for LoopNode {
    fn node_type(&self) -> &str {
        "loop"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        // ── Parse config ──────────────────────────────────────────────────
        let sub_flow_def = ctx.data.get("flow").ok_or_else(|| {
            FlowError::InvalidDefinition("loop: missing data.flow".into())
        })?;

        let output_selector = ctx.data["output_selector"]
            .as_str()
            .ok_or_else(|| {
                FlowError::InvalidDefinition("loop: missing data.output_selector".into())
            })?
            .to_string();

        let max_iterations =
            ctx.data["max_iterations"].as_u64().unwrap_or(10).max(1) as usize;

        let break_condition: Option<Condition> = ctx
            .data
            .get("break_condition")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        // ── Parse sub-flow DAG once ───────────────────────────────────────
        let sub_dag = DagGraph::from_json(sub_flow_def)?;
        let registry = Arc::clone(&ctx.registry);
        let base_variables = ctx.variables.clone();

        let mut loop_output = Value::Null;
        let mut actual_iterations = 0usize;

        for i in 0..max_iterations {
            actual_iterations = i + 1;

            let mut vars = base_variables.clone();
            vars.insert("iteration_index".into(), json!(i));
            vars.insert("loop_output".into(), loop_output.clone());

            let runner =
                FlowRunner::with_arc_registry(sub_dag.clone(), Arc::clone(&registry));
            let sub_result = runner.run(vars).await?;

            // Collect this iteration's output via the selector.
            loop_output = resolve_selector(&sub_result.outputs, &output_selector)
                .cloned()
                .unwrap_or(Value::Null);

            // Evaluate break condition against sub-flow outputs.
            if let Some(ref cond) = break_condition {
                if cond.evaluate(&sub_result.outputs, &sub_result.skipped_nodes) {
                    break;
                }
            }
        }

        Ok(json!({
            "output":     loop_output,
            "iterations": actual_iterations,
        }))
    }
}

// ── Internal helpers ───────────────────────────────────────────────────────

/// Resolve `"node_id"` or `"node_id.field.subfield"` into the sub-flow outputs.
fn resolve_selector<'a>(
    outputs: &'a HashMap<String, Value>,
    selector: &str,
) -> Option<&'a Value> {
    let (node_id, rest) = match selector.find('.') {
        Some(pos) => (&selector[..pos], &selector[pos + 1..]),
        None => (selector, ""),
    };
    let node_out = outputs.get(node_id)?;
    if rest.is_empty() {
        Some(node_out)
    } else {
        get_path(node_out, rest)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_ctx(data: Value) -> ExecContext {
        ExecContext { data, ..Default::default() }
    }

    // ── Config validation ──────────────────────────────────────────────────

    #[tokio::test]
    async fn rejects_missing_flow() {
        let node = LoopNode;
        let err = node
            .execute(make_ctx(json!({ "output_selector": "n" })))
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn rejects_missing_output_selector() {
        let node = LoopNode;
        let err = node
            .execute(make_ctx(json!({
                "flow": { "nodes": [{ "id": "n", "type": "noop" }], "edges": [] }
            })))
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    // ── Execution ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn runs_max_iterations_without_break_condition() {
        let node = LoopNode;
        let out = node
            .execute(make_ctx(json!({
                "output_selector": "step.output",
                "max_iterations":  3,
                "flow": {
                    "nodes": [{
                        "id": "step", "type": "code",
                        "data": { "language": "rhai", "code": "variables.iteration_index" }
                    }],
                    "edges": []
                }
            })))
            .await
            .unwrap();

        // Ran exactly 3 times; last index is 2.
        assert_eq!(out["iterations"], json!(3));
        assert_eq!(out["output"], json!(2));
    }

    #[tokio::test]
    async fn defaults_to_ten_iterations() {
        // No max_iterations specified → defaults to 10.
        let node = LoopNode;
        let out = node
            .execute(make_ctx(json!({
                "output_selector": "step.output",
                "flow": {
                    "nodes": [{
                        "id": "step", "type": "code",
                        "data": { "language": "rhai", "code": "variables.iteration_index" }
                    }],
                    "edges": []
                }
            })))
            .await
            .unwrap();

        assert_eq!(out["iterations"], json!(10));
    }

    #[tokio::test]
    async fn break_condition_stops_loop_early() {
        // Sub-flow returns { "done": true } when iteration_index >= 2.
        // break_condition: done == true → loop ends after iteration 2 (index 2).
        let node = LoopNode;
        let out = node
            .execute(make_ctx(json!({
                "output_selector": "step.output",
                "max_iterations":  10,
                "break_condition": {
                    "from": "gate", "path": "output", "op": "eq", "value": true
                },
                "flow": {
                    "nodes": [
                        {
                            "id": "step", "type": "code",
                            "data": { "language": "rhai", "code": "variables.iteration_index" }
                        },
                        {
                            "id": "gate", "type": "code",
                            "data": { "language": "rhai", "code": "variables.iteration_index >= 2" }
                        }
                    ],
                    "edges": []
                }
            })))
            .await
            .unwrap();

        // Break fires when iteration_index == 2, so 3 iterations ran (0, 1, 2).
        assert_eq!(out["iterations"], json!(3));
        assert_eq!(out["output"], json!(2));
    }

    #[tokio::test]
    async fn loop_output_injected_into_next_iteration() {
        // Each iteration's output is its iteration_index.
        // loop_output for iteration N is the output of iteration N-1.
        // We collect loop_output from iteration index=2 (the 3rd and final run).
        let node = LoopNode;
        let out = node
            .execute(make_ctx(json!({
                "output_selector": "collect.output",
                "max_iterations":  3,
                "flow": {
                    "nodes": [{
                        // Return iteration_index as a proxy for the collected value.
                        "id": "collect", "type": "code",
                        "data": { "language": "rhai", "code": "variables.iteration_index" }
                    }],
                    "edges": []
                }
            })))
            .await
            .unwrap();

        // 3 iterations ran; final output = index of last = 2.
        assert_eq!(out["output"], json!(2));
    }

    #[tokio::test]
    async fn min_iterations_is_one() {
        // max_iterations: 0 is clamped to 1.
        let node = LoopNode;
        let out = node
            .execute(make_ctx(json!({
                "output_selector": "step.output",
                "max_iterations":  0,
                "flow": {
                    "nodes": [{
                        "id": "step", "type": "code",
                        "data": { "language": "rhai", "code": "42" }
                    }],
                    "edges": []
                }
            })))
            .await
            .unwrap();

        assert_eq!(out["iterations"], json!(1));
        assert_eq!(out["output"], json!(42));
    }
}
