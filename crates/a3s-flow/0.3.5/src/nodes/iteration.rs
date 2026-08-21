//! Built-in `"iteration"` node — runs a sub-flow for every element of an
//! input array, collecting per-iteration outputs.
//!
//! Mirrors Dify's Iteration node. Each element is passed to the sub-flow as a
//! flow variable named `"item"` (plus an `"index"` variable with the 0-based
//! position).
//!
//! Two execution modes are available via `data["mode"]`:
//! - `"parallel"` *(default)* — all iterations run concurrently via Tokio tasks.
//! - `"sequential"` — iterations run one-at-a-time in order; each iteration
//!   receives the previous iteration's collected output as `"prev_output"` in
//!   its variable scope (`null` for the first item).
//!
//! # Config schema
//!
//! ```json
//! {
//!   "input_selector":  "fetch.body.items",
//!   "output_selector": "summarize.output",
//!   "mode":            "sequential",
//!   "flow": { ... }
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `input_selector` | string | ✅ | Dot path into `inputs` to reach the array |
//! | `output_selector` | string | ✅ | Dot path into sub-flow outputs to collect |
//! | `flow` | object | ✅ | Inline sub-flow definition |
//! | `mode` | string | — | `"parallel"` (default) or `"sequential"` |
//!
//! # Variables injected into each iteration's sub-flow
//!
//! | Variable | Value |
//! |----------|-------|
//! | `item` | The current array element |
//! | `index` | The 0-based position of the element |
//! | `prev_output` | *(sequential only)* The previous iteration's collected output (`null` for the first item) |
//!
//! # Output schema
//!
//! ```json
//! { "output": [ <value from output_selector for iteration 0>, ... ] }
//! ```
//!
//! Results are always returned in the original array order. A `null` is placed
//! for any iteration whose `output_selector` path resolves to nothing.
//!
//! # Example
//!
//! ```json
//! {
//!   "nodes": [
//!     { "id": "fetch", "type": "http-request", "data": { "url": "..." } },
//!     {
//!       "id": "process_all",
//!       "type": "iteration",
//!       "data": {
//!         "input_selector":  "fetch.body.items",
//!         "output_selector": "process.output",
//!         "flow": {
//!           "nodes": [
//!             { "id": "process", "type": "code", "data": { "language": "rhai", "code": "item" } }
//!           ],
//!           "edges": []
//!         }
//!       }
//!     }
//!   ],
//!   "edges": [{ "source": "fetch", "target": "process_all" }]
//! }
//! ```

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::task::JoinSet;

use crate::error::{FlowError, Result};
use crate::graph::DagGraph;
use crate::node::{ExecContext, Node};
use crate::runner::FlowRunner;

/// Iteration node — runs a sub-flow for each element of an array (Dify-compatible).
pub struct IterationNode;

/// Resolves a dot-separated path into a JSON value.
///
/// `"a.b.c"` into `{"a": {"b": {"c": 42}}}` returns `Some(42)`.
/// An empty string returns the root value.
fn resolve_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() {
        return Some(root);
    }
    let mut cur = root;
    for segment in path.split('.') {
        cur = cur.get(segment)?;
    }
    Some(cur)
}

/// Resolves `selector` of the form `"<node_id>.<field>.<subfield>..."` into
/// `outputs["node_id"]["field"]["subfield"]...`.
///
/// If the selector has no dot, it is treated as a node ID and the whole output
/// for that node is returned.
fn resolve_selector<'a>(
    outputs: &'a std::collections::HashMap<String, Value>,
    selector: &str,
) -> Option<&'a Value> {
    let (node_id, rest) = match selector.find('.') {
        Some(pos) => (&selector[..pos], &selector[pos + 1..]),
        None => (selector, ""),
    };
    let node_out = outputs.get(node_id)?;
    resolve_path(node_out, rest)
}

#[async_trait]
impl Node for IterationNode {
    fn node_type(&self) -> &str {
        "iteration"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        // ── Parse data ────────────────────────────────────────────────────
        let input_selector = ctx.data["input_selector"]
            .as_str()
            .ok_or_else(|| {
                FlowError::InvalidDefinition("iteration: missing data.input_selector".into())
            })?
            .to_string();

        let output_selector = ctx.data["output_selector"]
            .as_str()
            .ok_or_else(|| {
                FlowError::InvalidDefinition("iteration: missing data.output_selector".into())
            })?
            .to_string();

        let sub_flow_def = ctx
            .data
            .get("flow")
            .ok_or_else(|| FlowError::InvalidDefinition("iteration: missing data.flow".into()))?;

        // ── Parse and validate the sub-flow DAG once ──────────────────────
        let sub_dag = DagGraph::from_json(sub_flow_def)?;

        // ── Resolve the input array ───────────────────────────────────────
        // input_selector is relative to the combined inputs map, e.g. "fetch.body.items".
        // We split on the first dot to get the node_id, then path into its output.
        let items: Vec<Value> = {
            let (node_id, rest) = match input_selector.find('.') {
                Some(pos) => (&input_selector[..pos], &input_selector[pos + 1..]),
                None => (input_selector.as_str(), ""),
            };
            let node_out = ctx.inputs.get(node_id).ok_or_else(|| {
                FlowError::InvalidDefinition(format!(
                    "iteration: input_selector '{input_selector}' references unknown node '{node_id}'"
                ))
            })?;
            let arr = resolve_path(node_out, rest).ok_or_else(|| {
                FlowError::InvalidDefinition(format!(
                    "iteration: path '{rest}' not found in node '{node_id}' output"
                ))
            })?;
            arr.as_array()
                .ok_or_else(|| {
                    FlowError::InvalidDefinition(format!(
                        "iteration: input_selector '{input_selector}' must point to a JSON array"
                    ))
                })?
                .clone()
        };

        if items.is_empty() {
            return Ok(json!({ "output": [] }));
        }

        let mode = ctx.data["mode"].as_str().unwrap_or("parallel");
        let registry = Arc::clone(&ctx.registry);
        let base_variables = ctx.variables.clone();

        if mode == "sequential" {
            // ── Sequential: process items one-at-a-time in order ──────────
            let mut results = Vec::with_capacity(items.len());
            let mut prev_output = Value::Null;

            for (index, item) in items.into_iter().enumerate() {
                let mut vars = base_variables.clone();
                vars.insert("item".into(), item);
                vars.insert("index".into(), json!(index));
                vars.insert("prev_output".into(), prev_output.clone());

                let runner = FlowRunner::with_arc_registry(sub_dag.clone(), Arc::clone(&registry));
                let sub_result = runner.run(vars).await?;

                let value = resolve_selector(&sub_result.outputs, &output_selector)
                    .cloned()
                    .unwrap_or(Value::Null);
                prev_output = value.clone();
                results.push(value);
            }

            Ok(json!({ "output": results }))
        } else {
            // ── Parallel (default): launch all items concurrently ─────────
            let n = items.len();
            let mut join_set: JoinSet<(usize, Result<std::collections::HashMap<String, Value>>)> =
                JoinSet::new();

            for (index, item) in items.into_iter().enumerate() {
                let dag = sub_dag.clone();
                let reg = Arc::clone(&registry);
                let mut vars = base_variables.clone();
                vars.insert("item".into(), item);
                vars.insert("index".into(), json!(index));

                join_set.spawn(async move {
                    let runner = FlowRunner::with_arc_registry(dag, reg);
                    let result: crate::error::Result<_> = runner.run(vars).await.map(|r| r.outputs);
                    (index, result)
                });
            }

            // Collect results in order.
            let mut results: Vec<Option<Value>> = vec![None; n];

            while let Some(task) = join_set.join_next().await {
                match task {
                    Ok((index, Ok(outputs))) => {
                        let value = resolve_selector(&outputs, &output_selector).cloned();
                        results[index] = value;
                    }
                    Ok((_, Err(e))) => return Err(e),
                    Err(e) if e.is_cancelled() => return Err(FlowError::Terminated),
                    Err(e) => return Err(FlowError::Internal(e.to_string())),
                }
            }

            let output: Vec<Value> = results
                .into_iter()
                .map(|v| v.unwrap_or(Value::Null))
                .collect();

            Ok(json!({ "output": output }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ctx(data: Value) -> ExecContext {
        ExecContext {
            data,
            inputs: HashMap::new(),
            variables: HashMap::new(),
            ..Default::default()
        }
    }

    fn ctx_with_inputs(data: Value, inputs: HashMap<String, Value>) -> ExecContext {
        ExecContext {
            data,
            inputs,
            variables: HashMap::new(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn iterates_over_array_and_collects_outputs() {
        // Sub-flow: single "code" node that returns { output: item * 2 }
        // `item` is injected as a flow variable, accessible via `variables.item` in Rhai.
        let node = IterationNode;
        let out = node
            .execute(ctx_with_inputs(
                json!({
                    "input_selector":  "src.items",
                    "output_selector": "double.output",
                    "flow": {
                        "nodes": [
                            {
                                "id": "double",
                                "type": "code",
                                "data": { "language": "rhai", "code": "variables.item * 2" }
                            }
                        ],
                        "edges": []
                    }
                }),
                HashMap::from([("src".into(), json!({ "items": [1, 2, 3] }))]),
            ))
            .await
            .unwrap();

        let arr = out["output"].as_array().unwrap();
        assert_eq!(arr.len(), 3);
        // Order is preserved.
        assert_eq!(arr[0], json!(2));
        assert_eq!(arr[1], json!(4));
        assert_eq!(arr[2], json!(6));
    }

    #[tokio::test]
    async fn empty_array_returns_empty_output() {
        let node = IterationNode;
        let out = node
            .execute(ctx_with_inputs(
                json!({
                    "input_selector":  "src",
                    "output_selector": "noop",
                    "flow": { "nodes": [{ "id": "noop", "type": "noop" }], "edges": [] }
                }),
                HashMap::from([("src".into(), json!([]))]),
            ))
            .await
            .unwrap();
        assert_eq!(out["output"], json!([]));
    }

    #[tokio::test]
    async fn index_variable_injected() {
        // `index` is injected as a flow variable, accessible via `variables.index` in Rhai.
        let node = IterationNode;
        let out = node
            .execute(ctx_with_inputs(
                json!({
                    "input_selector":  "src",
                    "output_selector": "idx.output",
                    "flow": {
                        "nodes": [
                            {
                                "id": "idx",
                                "type": "code",
                                "data": { "language": "rhai", "code": "variables.index" }
                            }
                        ],
                        "edges": []
                    }
                }),
                HashMap::from([("src".into(), json!(["a", "b", "c"]))]),
            ))
            .await
            .unwrap();

        let arr = out["output"].as_array().unwrap();
        assert_eq!(arr[0], json!(0));
        assert_eq!(arr[1], json!(1));
        assert_eq!(arr[2], json!(2));
    }

    #[tokio::test]
    async fn rejects_missing_input_selector() {
        let node = IterationNode;
        let err = node
            .execute(ctx(json!({
                "output_selector": "x",
                "flow": { "nodes": [{ "id": "n", "type": "noop" }], "edges": [] }
            })))
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn rejects_missing_output_selector() {
        let node = IterationNode;
        let err = node
            .execute(ctx(json!({
                "input_selector": "src",
                "flow": { "nodes": [{ "id": "n", "type": "noop" }], "edges": [] }
            })))
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn rejects_non_array_input() {
        let node = IterationNode;
        let err = node
            .execute(ctx_with_inputs(
                json!({
                    "input_selector":  "src",
                    "output_selector": "n",
                    "flow": { "nodes": [{ "id": "n", "type": "noop" }], "edges": [] }
                }),
                HashMap::from([("src".into(), json!("not an array"))]),
            ))
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    // ── Sequential mode ────────────────────────────────────────────────────

    #[tokio::test]
    async fn sequential_mode_processes_in_order() {
        let node = IterationNode;
        let out = node
            .execute(ctx_with_inputs(
                json!({
                    "input_selector":  "src",
                    "output_selector": "step.output",
                    "mode": "sequential",
                    "flow": {
                        "nodes": [
                            {
                                "id": "step",
                                "type": "code",
                                "data": { "language": "rhai", "code": "variables.item * 10" }
                            }
                        ],
                        "edges": []
                    }
                }),
                HashMap::from([("src".into(), json!([1, 2, 3]))]),
            ))
            .await
            .unwrap();

        let arr = out["output"].as_array().unwrap();
        assert_eq!(arr, &[json!(10), json!(20), json!(30)]);
    }

    #[tokio::test]
    async fn sequential_mode_injects_prev_output() {
        // Each step receives `prev_output` from the previous iteration.
        // Step returns index + 1; prev_output for step 1 = 0 (null → 0 in Rhai).
        let node = IterationNode;
        let out = node
            .execute(ctx_with_inputs(
                json!({
                    "input_selector":  "src",
                    "output_selector": "step.output",
                    "mode": "sequential",
                    "flow": {
                        "nodes": [
                            {
                                "id": "step",
                                "type": "code",
                                "data": {
                                    "language": "rhai",
                                    // Return the index as a simple marker.
                                    "code": "variables.index"
                                }
                            }
                        ],
                        "edges": []
                    }
                }),
                HashMap::from([("src".into(), json!(["a", "b", "c"]))]),
            ))
            .await
            .unwrap();

        let arr = out["output"].as_array().unwrap();
        assert_eq!(arr, &[json!(0), json!(1), json!(2)]);
    }

    #[tokio::test]
    async fn sequential_mode_empty_array_returns_empty() {
        let node = IterationNode;
        let out = node
            .execute(ctx_with_inputs(
                json!({
                    "input_selector":  "src",
                    "output_selector": "n",
                    "mode": "sequential",
                    "flow": { "nodes": [{ "id": "n", "type": "noop" }], "edges": [] }
                }),
                HashMap::from([("src".into(), json!([]))]),
            ))
            .await
            .unwrap();
        assert_eq!(out["output"], json!([]));
    }

    #[tokio::test]
    async fn unknown_mode_defaults_to_parallel() {
        // Any unrecognised mode string falls back to parallel.
        let node = IterationNode;
        let out = node
            .execute(ctx_with_inputs(
                json!({
                    "input_selector":  "src",
                    "output_selector": "step.output",
                    "mode": "turbo",
                    "flow": {
                        "nodes": [
                            {
                                "id": "step",
                                "type": "code",
                                "data": { "language": "rhai", "code": "variables.item" }
                            }
                        ],
                        "edges": []
                    }
                }),
                HashMap::from([("src".into(), json!([7, 8]))]),
            ))
            .await
            .unwrap();

        let mut arr = out["output"].as_array().unwrap().clone();
        arr.sort_by(|a, b| a.as_i64().cmp(&b.as_i64()));
        assert_eq!(arr, &[json!(7), json!(8)]);
    }
}
