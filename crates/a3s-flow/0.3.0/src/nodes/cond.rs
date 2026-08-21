//! Built-in `"if-else"` node — evaluates an ordered list of named cases and
//! emits `{ "branch": "<case_id>" }` for the first matching case, or
//! `{ "branch": "else" }` when none match.
//!
//! This mirrors Dify's IF/ELSE node. Downstream nodes use `run_if` on
//! `branch` to determine which path executes.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "cases": [
//!     {
//!       "id": "is_ok",
//!       "logical_operator": "and",
//!       "conditions": [
//!         { "from": "fetch", "path": "status", "op": "eq", "value": 200 }
//!       ]
//!     },
//!     {
//!       "id": "is_error",
//!       "conditions": [
//!         { "from": "fetch", "path": "status", "op": "gte", "value": 400 }
//!       ]
//!     }
//!   ]
//! }
//! ```
//!
//! | Field | Type | Description |
//! |-------|------|-------------|
//! | `cases` | array | Ordered list of named cases; first match wins |
//! | `cases[].id` | string | Branch identifier (returned as `"branch"`) |
//! | `cases[].logical_operator` | `"and"` \| `"or"` | How to combine conditions (default: `"and"`) |
//! | `cases[].conditions` | array | One or more [`Condition`] objects |
//!
//! # Output schema
//!
//! ```json
//! { "branch": "is_ok" }
//! ```
//!
//! The implicit ELSE branch is always `"else"`.
//!
//! # Routing downstream nodes
//!
//! ```json
//! {
//!   "id": "notify",
//!   "type": "http-request",
//!   "data": { "run_if": { "from": "route", "path": "branch", "op": "eq", "value": "is_ok" } }
//! }
//! ```
//!
//! [`Condition`]: crate::condition::Condition

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::condition::Case;
use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

#[derive(Debug, Deserialize, Serialize)]
struct IfElseConfig {
    cases: Vec<Case>,
}

/// IF/ELSE routing node (Dify-compatible).
pub struct IfElseNode;

#[async_trait]
impl Node for IfElseNode {
    fn node_type(&self) -> &str {
        "if-else"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let config: IfElseConfig = serde_json::from_value(ctx.data.clone())
            .map_err(|e| FlowError::InvalidDefinition(format!("if-else: invalid data: {e}")))?;

        if config.cases.is_empty() {
            return Err(FlowError::InvalidDefinition(
                "if-else: at least one case is required".into(),
            ));
        }

        for case in &config.cases {
            if case.evaluate(&ctx.inputs) {
                return Ok(json!({ "branch": case.id }));
            }
        }

        Ok(json!({ "branch": "else" }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ctx(inputs: HashMap<String, Value>, data: Value) -> ExecContext {
        ExecContext {
            data,
            inputs,
            variables: HashMap::new(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn first_matching_case_wins() {
        let node = IfElseNode;
        let c = ctx(
            HashMap::from([("fetch".into(), json!({ "status": 200 }))]),
            json!({
                "cases": [
                    { "id": "is_ok",    "conditions": [{ "from": "fetch", "path": "status", "op": "eq", "value": 200 }] },
                    { "id": "is_error", "conditions": [{ "from": "fetch", "path": "status", "op": "gte", "value": 400 }] }
                ]
            }),
        );
        let out = node.execute(c).await.unwrap();
        assert_eq!(out["branch"], "is_ok");
    }

    #[tokio::test]
    async fn falls_through_to_else() {
        let node = IfElseNode;
        let c = ctx(
            HashMap::from([("fetch".into(), json!({ "status": 302 }))]),
            json!({
                "cases": [
                    { "id": "is_ok",    "conditions": [{ "from": "fetch", "path": "status", "op": "eq", "value": 200 }] },
                    { "id": "is_error", "conditions": [{ "from": "fetch", "path": "status", "op": "gte", "value": 500 }] }
                ]
            }),
        );
        let out = node.execute(c).await.unwrap();
        assert_eq!(out["branch"], "else");
    }

    #[tokio::test]
    async fn or_logical_operator() {
        let node = IfElseNode;
        let c = ctx(
            HashMap::from([("a".into(), json!({ "x": 1, "y": 99 }))]),
            json!({
                "cases": [{
                    "id": "hit",
                    "logical_operator": "or",
                    "conditions": [
                        { "from": "a", "path": "x", "op": "eq", "value": 1 },
                        { "from": "a", "path": "y", "op": "eq", "value": 2 }
                    ]
                }]
            }),
        );
        let out = node.execute(c).await.unwrap();
        assert_eq!(out["branch"], "hit");
    }

    #[tokio::test]
    async fn and_logical_operator_all_must_pass() {
        let node = IfElseNode;
        let c = ctx(
            HashMap::from([("a".into(), json!({ "x": 1, "y": 99 }))]),
            json!({
                "cases": [{
                    "id": "hit",
                    "logical_operator": "and",
                    "conditions": [
                        { "from": "a", "path": "x", "op": "eq", "value": 1 },
                        { "from": "a", "path": "y", "op": "eq", "value": 2 }
                    ]
                }]
            }),
        );
        let out = node.execute(c).await.unwrap();
        assert_eq!(out["branch"], "else");
    }

    #[tokio::test]
    async fn rejects_empty_cases() {
        let node = IfElseNode;
        let c = ctx(HashMap::new(), json!({ "cases": [] }));
        assert!(matches!(
            node.execute(c).await,
            Err(FlowError::InvalidDefinition(_))
        ));
    }

    #[tokio::test]
    async fn rejects_invalid_config() {
        let node = IfElseNode;
        let c = ctx(HashMap::new(), json!("not an object"));
        assert!(matches!(
            node.execute(c).await,
            Err(FlowError::InvalidDefinition(_))
        ));
    }
}
