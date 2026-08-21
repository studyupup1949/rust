//! `"list-operator"` node — pure JSON array operations.
//!
//! Applies a configurable pipeline of operations to an input array in a fixed
//! order: **filter → sort → deduplicate → limit**. Every operation is optional.
//!
//! All operations are pure in-process JSON logic — no LLM or network calls.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "input_selector": "fetch.body.items",
//!   "filter":          { "path": "active", "op": "eq", "value": true },
//!   "sort_by":         "name",
//!   "sort_order":      "asc",
//!   "deduplicate_by":  "id",
//!   "limit":           10
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `input_selector` | string | ✅ | Dot path into upstream inputs: `"node_id"` or `"node_id.field.subfield"` |
//! | `filter` | object | — | Keep items where condition is true. Fields: `path` (dot path into each element), `op` (`"eq"`, `"ne"`, `"gt"`, `"lt"`, `"gte"`, `"lte"`, `"contains"`), `value` |
//! | `sort_by` | string | — | Dot path into each element to sort by. Numbers and strings are both supported; nulls sort last |
//! | `sort_order` | string | — | `"asc"` (default) or `"desc"` |
//! | `deduplicate_by` | string | — | Dot path into each element; keeps the first occurrence of each unique value at that path. Empty string deduplicates by full element equality |
//! | `limit` | integer | — | Keep only the first N elements (applied last) |
//!
//! Operations are applied in this fixed order: filter → sort → deduplicate → limit.
//!
//! # Output schema
//!
//! ```json
//! { "output": [ ... ] }
//! ```
//!
//! # Example
//!
//! ```json
//! {
//!   "id": "clean",
//!   "type": "list-operator",
//!   "data": {
//!     "input_selector":  "fetch.body.users",
//!     "filter":          { "path": "active", "op": "eq", "value": true },
//!     "sort_by":         "name",
//!     "sort_order":      "asc",
//!     "deduplicate_by":  "email",
//!     "limit":           100
//!   }
//! }
//! ```

use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::condition::{get_path, CondOp};
use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

/// List operator node — filter / sort / deduplicate / limit a JSON array.
pub struct ListOperatorNode;

#[async_trait]
impl Node for ListOperatorNode {
    fn node_type(&self) -> &str {
        "list-operator"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        // ── Resolve input array ───────────────────────────────────────────
        let input_selector = ctx.data["input_selector"].as_str().ok_or_else(|| {
            FlowError::InvalidDefinition("list-operator: missing data.input_selector".into())
        })?;

        let mut items = resolve_input_array(&ctx.inputs, input_selector)?;

        // ── Apply operations in fixed order ───────────────────────────────
        if !ctx.data["filter"].is_null() {
            items = apply_filter(items, &ctx.data["filter"])?;
        }
        if let Some(key) = ctx.data["sort_by"].as_str() {
            let order = ctx.data["sort_order"].as_str().unwrap_or("asc");
            items = apply_sort(items, key, order);
        }
        if let Some(key) = ctx.data["deduplicate_by"].as_str() {
            items = apply_deduplicate(items, key);
        }
        if let Some(n) = ctx.data["limit"].as_u64() {
            items.truncate(n as usize);
        }

        Ok(json!({ "output": items }))
    }
}

// ── Operation implementations ──────────────────────────────────────────────

/// Resolve `"node_id"` or `"node_id.field.subfield"` from upstream inputs.
fn resolve_input_array(inputs: &HashMap<String, Value>, selector: &str) -> Result<Vec<Value>> {
    let (node_id, rest) = match selector.find('.') {
        Some(pos) => (&selector[..pos], &selector[pos + 1..]),
        None => (selector, ""),
    };

    let node_out = inputs.get(node_id).ok_or_else(|| {
        FlowError::InvalidDefinition(format!(
            "list-operator: input_selector '{selector}' references unknown node '{node_id}'"
        ))
    })?;

    let value = if rest.is_empty() {
        node_out
    } else {
        get_path(node_out, rest).ok_or_else(|| {
            FlowError::InvalidDefinition(format!(
                "list-operator: path '{rest}' not found in node '{node_id}' output"
            ))
        })?
    };

    value
        .as_array()
        .ok_or_else(|| {
            FlowError::InvalidDefinition(format!(
                "list-operator: input_selector '{selector}' must point to a JSON array"
            ))
        })
        .map(|a| a.clone())
}

/// Keep elements where the filter condition evaluates to true.
fn apply_filter(items: Vec<Value>, filter: &Value) -> Result<Vec<Value>> {
    let path = filter["path"].as_str().unwrap_or("");
    let op: CondOp = serde_json::from_value(filter["op"].clone()).map_err(|e| {
        FlowError::InvalidDefinition(format!("list-operator: invalid filter.op: {e}"))
    })?;
    let expected = &filter["value"];

    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let actual = if path.is_empty() {
            &item
        } else {
            match get_path(&item, path) {
                Some(v) => v,
                None => continue, // path missing → filter out
            }
        };
        if compare_values(actual, &op, expected) {
            out.push(item);
        }
    }
    Ok(out)
}

/// Sort elements by a dot-path field.
///
/// Numbers compare numerically, strings lexicographically, nulls sort last.
/// Mixed types (number vs string) sort numbers before strings.
fn apply_sort(mut items: Vec<Value>, key: &str, order: &str) -> Vec<Value> {
    let descending = order == "desc";
    items.sort_by(|a, b| {
        let av = if key.is_empty() {
            Some(a)
        } else {
            get_path(a, key)
        };
        let bv = if key.is_empty() {
            Some(b)
        } else {
            get_path(b, key)
        };
        let ord = compare_sort_values(av, bv);
        if descending {
            ord.reverse()
        } else {
            ord
        }
    });
    items
}

/// Remove duplicate elements by a dot-path key (first occurrence wins).
///
/// Empty key deduplicates by full element equality (serialized to JSON string).
fn apply_deduplicate(items: Vec<Value>, key: &str) -> Vec<Value> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let fingerprint = if key.is_empty() {
            item.to_string()
        } else {
            get_path(&item, key)
                .map(|v| v.to_string())
                .unwrap_or_default()
        };
        if seen.insert(fingerprint) {
            out.push(item);
        }
    }
    out
}

// ── Comparison helpers ─────────────────────────────────────────────────────

fn compare_values(actual: &Value, op: &CondOp, expected: &Value) -> bool {
    match op {
        CondOp::Eq => actual == expected,
        CondOp::Ne => actual != expected,
        CondOp::Gt => numeric_cmp(actual, expected)
            .map(|o| o.is_gt())
            .unwrap_or(false),
        CondOp::Lt => numeric_cmp(actual, expected)
            .map(|o| o.is_lt())
            .unwrap_or(false),
        CondOp::Gte => numeric_cmp(actual, expected)
            .map(|o| o.is_ge())
            .unwrap_or(false),
        CondOp::Lte => numeric_cmp(actual, expected)
            .map(|o| o.is_le())
            .unwrap_or(false),
        CondOp::Contains => match (actual, expected) {
            (Value::String(s), Value::String(sub)) => s.contains(sub.as_str()),
            (Value::Array(arr), v) => arr.contains(v),
            _ => false,
        },
    }
}

fn numeric_cmp(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    a.as_f64()?.partial_cmp(&b.as_f64()?)
}

fn compare_sort_values(a: Option<&Value>, b: Option<&Value>) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater, // null sorts last
        (Some(_), None) => Ordering::Less,
        (Some(av), Some(bv)) => {
            // Both numeric → numeric comparison.
            if let (Some(an), Some(bn)) = (av.as_f64(), bv.as_f64()) {
                return an.partial_cmp(&bn).unwrap_or(Ordering::Equal);
            }
            // Both strings → lexicographic.
            if let (Some(as_), Some(bs)) = (av.as_str(), bv.as_str()) {
                return as_.cmp(bs);
            }
            // Mixed / other → fall back to string representation.
            av.to_string().cmp(&bv.to_string())
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ctx_with(data: Value, input_node: &str, array: Value) -> ExecContext {
        ExecContext {
            data,
            inputs: HashMap::from([(input_node.to_string(), array)]),
            ..Default::default()
        }
    }

    // ── input resolution ───────────────────────────────────────────────────

    #[tokio::test]
    async fn resolves_root_array() {
        let node = ListOperatorNode;
        let out = node
            .execute(ctx_with(
                json!({ "input_selector": "src" }),
                "src",
                json!([1, 2, 3]),
            ))
            .await
            .unwrap();
        assert_eq!(out["output"], json!([1, 2, 3]));
    }

    #[tokio::test]
    async fn resolves_nested_array() {
        let node = ListOperatorNode;
        let out = node
            .execute(ctx_with(
                json!({ "input_selector": "src.items" }),
                "src",
                json!({ "items": [4, 5, 6] }),
            ))
            .await
            .unwrap();
        assert_eq!(out["output"], json!([4, 5, 6]));
    }

    #[tokio::test]
    async fn rejects_missing_input_selector() {
        let node = ListOperatorNode;
        let err = node
            .execute(ExecContext {
                data: json!({}),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn rejects_non_array_input() {
        let node = ListOperatorNode;
        let err = node
            .execute(ctx_with(
                json!({ "input_selector": "src" }),
                "src",
                json!("not an array"),
            ))
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    // ── filter ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn filter_eq_keeps_matching_items() {
        let node = ListOperatorNode;
        let out = node
            .execute(ctx_with(
                json!({
                    "input_selector": "src",
                    "filter": { "path": "active", "op": "eq", "value": true }
                }),
                "src",
                json!([
                    { "name": "Alice", "active": true },
                    { "name": "Bob",   "active": false },
                    { "name": "Carol", "active": true }
                ]),
            ))
            .await
            .unwrap();
        let arr = out["output"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], json!("Alice"));
        assert_eq!(arr[1]["name"], json!("Carol"));
    }

    #[tokio::test]
    async fn filter_gt_keeps_numeric_matches() {
        let node = ListOperatorNode;
        let out = node
            .execute(ctx_with(
                json!({
                    "input_selector": "src",
                    "filter": { "path": "score", "op": "gt", "value": 5 }
                }),
                "src",
                json!([
                    { "score": 3 },
                    { "score": 7 },
                    { "score": 10 }
                ]),
            ))
            .await
            .unwrap();
        let arr = out["output"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[tokio::test]
    async fn filter_contains_string() {
        let node = ListOperatorNode;
        let out = node
            .execute(ctx_with(
                json!({
                    "input_selector": "src",
                    "filter": { "path": "tag", "op": "contains", "value": "rust" }
                }),
                "src",
                json!([
                    { "tag": "rust-2024" },
                    { "tag": "python" },
                    { "tag": "rust-async" }
                ]),
            ))
            .await
            .unwrap();
        assert_eq!(out["output"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn filter_on_missing_path_excludes_item() {
        let node = ListOperatorNode;
        let out = node
            .execute(ctx_with(
                json!({
                    "input_selector": "src",
                    "filter": { "path": "missing_field", "op": "eq", "value": true }
                }),
                "src",
                json!([{ "x": 1 }, { "x": 2 }]),
            ))
            .await
            .unwrap();
        assert_eq!(out["output"], json!([]));
    }

    // ── sort ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn sort_strings_ascending() {
        let node = ListOperatorNode;
        let out = node
            .execute(ctx_with(
                json!({ "input_selector": "src", "sort_by": "name" }),
                "src",
                json!([{ "name": "Charlie" }, { "name": "Alice" }, { "name": "Bob" }]),
            ))
            .await
            .unwrap();
        let names: Vec<_> = out["output"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["Alice", "Bob", "Charlie"]);
    }

    #[tokio::test]
    async fn sort_numbers_descending() {
        let node = ListOperatorNode;
        let out = node
            .execute(ctx_with(
                json!({ "input_selector": "src", "sort_by": "score", "sort_order": "desc" }),
                "src",
                json!([{ "score": 3 }, { "score": 9 }, { "score": 1 }]),
            ))
            .await
            .unwrap();
        let scores: Vec<_> = out["output"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["score"].as_i64().unwrap())
            .collect();
        assert_eq!(scores, [9, 3, 1]);
    }

    #[tokio::test]
    async fn sort_null_values_sort_last() {
        let node = ListOperatorNode;
        let out = node
            .execute(ctx_with(
                json!({ "input_selector": "src", "sort_by": "x" }),
                "src",
                json!([{ "x": 3 }, {}, { "x": 1 }]),
            ))
            .await
            .unwrap();
        let arr = out["output"].as_array().unwrap();
        assert_eq!(arr[0]["x"], json!(1));
        assert_eq!(arr[1]["x"], json!(3));
        assert!(arr[2].get("x").is_none()); // null-keyed item last
    }

    // ── deduplicate ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn deduplicate_by_field_keeps_first() {
        let node = ListOperatorNode;
        let out = node
            .execute(ctx_with(
                json!({ "input_selector": "src", "deduplicate_by": "id" }),
                "src",
                json!([
                    { "id": 1, "v": "a" },
                    { "id": 2, "v": "b" },
                    { "id": 1, "v": "c" }  // duplicate id:1 — dropped
                ]),
            ))
            .await
            .unwrap();
        let arr = out["output"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["v"], json!("a")); // first occurrence kept
    }

    #[tokio::test]
    async fn deduplicate_empty_key_uses_full_equality() {
        let node = ListOperatorNode;
        let out = node
            .execute(ctx_with(
                json!({ "input_selector": "src", "deduplicate_by": "" }),
                "src",
                json!([1, 2, 1, 3, 2]),
            ))
            .await
            .unwrap();
        let arr = out["output"].as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    // ── limit ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn limit_truncates_to_n() {
        let node = ListOperatorNode;
        let out = node
            .execute(ctx_with(
                json!({ "input_selector": "src", "limit": 2 }),
                "src",
                json!([10, 20, 30, 40]),
            ))
            .await
            .unwrap();
        assert_eq!(out["output"], json!([10, 20]));
    }

    #[tokio::test]
    async fn limit_larger_than_array_keeps_all() {
        let node = ListOperatorNode;
        let out = node
            .execute(ctx_with(
                json!({ "input_selector": "src", "limit": 100 }),
                "src",
                json!([1, 2]),
            ))
            .await
            .unwrap();
        assert_eq!(out["output"], json!([1, 2]));
    }

    // ── combined pipeline ──────────────────────────────────────────────────

    #[tokio::test]
    async fn filter_sort_limit_combined() {
        let node = ListOperatorNode;
        let out = node
            .execute(ctx_with(
                json!({
                    "input_selector": "src",
                    "filter": { "path": "active", "op": "eq", "value": true },
                    "sort_by": "score",
                    "sort_order": "desc",
                    "limit": 2
                }),
                "src",
                json!([
                    { "name": "A", "score": 5,  "active": true  },
                    { "name": "B", "score": 10, "active": false },
                    { "name": "C", "score": 8,  "active": true  },
                    { "name": "D", "score": 3,  "active": true  },
                    { "name": "E", "score": 12, "active": true  }
                ]),
            ))
            .await
            .unwrap();
        let arr = out["output"].as_array().unwrap();
        // Active: A(5), C(8), D(3), E(12). Sorted desc: E(12), C(8), A(5), D(3). Limit 2: E, C.
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["name"], json!("E"));
        assert_eq!(arr[1]["name"], json!("C"));
    }
}
