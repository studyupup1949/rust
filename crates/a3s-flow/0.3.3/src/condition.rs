//! Condition types shared by `run_if` (in [`NodeDef`]) and the `"if-else"` built-in node.
//!
//! - [`Condition`] — single comparison (`from`, `path`, `op`, `value`)
//! - [`Case`] — named branch with one or more conditions joined by [`LogicalOp`]
//!
//! The evaluation logic lives here once and is reused by the runner and nodes.
//!
//! [`NodeDef`]: crate::graph::NodeDef

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── Operators ──────────────────────────────────────────────────────────────

/// Comparison operator for a [`Condition`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CondOp {
    /// Equal (`==`)
    Eq,
    /// Not equal (`!=`)
    Ne,
    /// Greater than (`>`, numbers only)
    Gt,
    /// Less than (`<`, numbers only)
    Lt,
    /// Greater than or equal (`>=`, numbers only)
    Gte,
    /// Less than or equal (`<=`, numbers only)
    Lte,
    /// Substring (string) or element membership (array)
    Contains,
}

/// Logical operator combining multiple conditions inside a [`Case`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogicalOp {
    /// All conditions must be true (default, matches Dify default).
    #[default]
    And,
    /// At least one condition must be true.
    Or,
}

// ── Condition ──────────────────────────────────────────────────────────────

/// A single comparison against an upstream node's output.
///
/// Used in two places:
/// - `run_if` on a [`NodeDef`] — guard that skips the node when false
/// - inside a [`Case`] in the `"if-else"` node config
///
/// [`NodeDef`]: crate::graph::NodeDef
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// ID of the upstream node whose output to inspect.
    pub from: String,
    /// Dot-separated path into the upstream output (e.g. `"status"`, `"data.count"`).
    /// Use `""` to compare the root value.
    pub path: String,
    /// Comparison operator.
    pub op: CondOp,
    /// Right-hand side value.
    pub value: Value,
}

impl Condition {
    /// Evaluate against the full output map, respecting the skipped set.
    ///
    /// Returns `false` when the source node was skipped, is missing, the path
    /// does not resolve, or the comparison fails.
    pub fn evaluate(&self, outputs: &HashMap<String, Value>, skipped: &HashSet<String>) -> bool {
        if skipped.contains(&self.from) {
            return false;
        }
        let from_output = match outputs.get(&self.from) {
            Some(v) => v,
            None => return false,
        };
        self.evaluate_on_value(from_output)
    }

    /// Evaluate directly against a single upstream output value.
    ///
    /// Used by the `"if-else"` node, which already has the value in `ctx.inputs`.
    pub fn evaluate_on_value(&self, from_output: &Value) -> bool {
        if from_output.is_null() {
            return false;
        }
        let actual = match get_path(from_output, &self.path) {
            Some(v) => v,
            None => return false,
        };
        compare(actual, &self.op, &self.value)
    }
}

// ── Case ───────────────────────────────────────────────────────────────────

/// A named branch inside an `"if-else"` node.
///
/// A case is satisfied when its conditions evaluate to true according to
/// `logical_operator`. Cases are tested in order; the first match wins.
///
/// # Example
///
/// ```json
/// {
///   "id": "is_ok",
///   "logical_operator": "and",
///   "conditions": [
///     { "from": "fetch", "path": "status", "op": "eq", "value": 200 }
///   ]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    /// Branch identifier — returned as `"branch"` in the node output.
    pub id: String,
    /// How multiple conditions are combined (default: `"and"`).
    #[serde(default)]
    pub logical_operator: LogicalOp,
    /// Conditions that must hold for this case to match.
    pub conditions: Vec<Condition>,
}

impl Case {
    /// Evaluate all conditions against `inputs`, combining with `logical_operator`.
    ///
    /// Returns `false` for an empty conditions list.
    pub fn evaluate(&self, inputs: &HashMap<String, Value>) -> bool {
        if self.conditions.is_empty() {
            return false;
        }
        let results = self.conditions.iter().map(|cond| {
            inputs
                .get(&cond.from)
                .map(|v| cond.evaluate_on_value(v))
                .unwrap_or(false)
        });
        match self.logical_operator {
            LogicalOp::And => results.into_iter().all(|b| b),
            LogicalOp::Or => results.into_iter().any(|b| b),
        }
    }
}

// ── Helpers ──────��─────────────────────────────────────────────────────────

/// Navigate a dot-separated path into a JSON value.
///
/// Returns `None` if any key is missing. Empty path returns the root.
pub(crate) fn get_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() {
        return Some(value);
    }
    let mut current = value;
    for key in path.split('.') {
        current = current.get(key)?;
    }
    Some(current)
}

fn compare(actual: &Value, op: &CondOp, expected: &Value) -> bool {
    match op {
        CondOp::Eq => actual == expected,
        CondOp::Ne => actual != expected,
        CondOp::Gt => numeric_cmp(actual, expected) == Some(Ordering::Greater),
        CondOp::Lt => numeric_cmp(actual, expected) == Some(Ordering::Less),
        CondOp::Gte => matches!(
            numeric_cmp(actual, expected),
            Some(Ordering::Greater | Ordering::Equal)
        ),
        CondOp::Lte => matches!(
            numeric_cmp(actual, expected),
            Some(Ordering::Less | Ordering::Equal)
        ),
        CondOp::Contains => match (actual, expected) {
            (Value::String(s), Value::String(sub)) => s.contains(sub.as_str()),
            (Value::Array(arr), v) => arr.contains(v),
            _ => false,
        },
    }
}

fn numeric_cmp(a: &Value, b: &Value) -> Option<Ordering> {
    a.as_f64()?.partial_cmp(&b.as_f64()?)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn outputs(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    fn inputs(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
        outputs(pairs)
    }

    #[test]
    fn condition_eq_passes() {
        let cond = Condition {
            from: "a".into(),
            path: "status".into(),
            op: CondOp::Eq,
            value: json!(200),
        };
        assert!(cond.evaluate(&outputs(&[("a", json!({"status": 200}))]), &HashSet::new()));
    }

    #[test]
    fn condition_eq_fails() {
        let cond = Condition {
            from: "a".into(),
            path: "status".into(),
            op: CondOp::Eq,
            value: json!(200),
        };
        assert!(!cond.evaluate(&outputs(&[("a", json!({"status": 404}))]), &HashSet::new()));
    }

    #[test]
    fn skipped_upstream_propagates() {
        let cond = Condition {
            from: "a".into(),
            path: "".into(),
            op: CondOp::Eq,
            value: json!(true),
        };
        let mut skipped = HashSet::new();
        skipped.insert("a".to_string());
        assert!(!cond.evaluate(&outputs(&[("a", json!(null))]), &skipped));
    }

    #[test]
    fn null_upstream_returns_false() {
        let cond = Condition {
            from: "a".into(),
            path: "x".into(),
            op: CondOp::Eq,
            value: json!(1),
        };
        assert!(!cond.evaluate_on_value(&json!(null)));
    }

    #[test]
    fn gt_numeric() {
        let cond = Condition {
            from: "a".into(),
            path: "count".into(),
            op: CondOp::Gt,
            value: json!(5),
        };
        assert!(cond.evaluate(&outputs(&[("a", json!({"count": 10}))]), &HashSet::new()));
    }

    #[test]
    fn contains_string() {
        let cond = Condition {
            from: "a".into(),
            path: "msg".into(),
            op: CondOp::Contains,
            value: json!("hello"),
        };
        assert!(cond.evaluate(
            &outputs(&[("a", json!({"msg": "say hello world"}))]),
            &HashSet::new()
        ));
    }

    #[test]
    fn case_and_all_pass() {
        let case = Case {
            id: "ok".into(),
            logical_operator: LogicalOp::And,
            conditions: vec![
                Condition {
                    from: "a".into(),
                    path: "x".into(),
                    op: CondOp::Eq,
                    value: json!(1),
                },
                Condition {
                    from: "a".into(),
                    path: "y".into(),
                    op: CondOp::Eq,
                    value: json!(2),
                },
            ],
        };
        assert!(case.evaluate(&inputs(&[("a", json!({"x": 1, "y": 2}))])));
    }

    #[test]
    fn case_and_one_fails() {
        let case = Case {
            id: "ok".into(),
            logical_operator: LogicalOp::And,
            conditions: vec![
                Condition {
                    from: "a".into(),
                    path: "x".into(),
                    op: CondOp::Eq,
                    value: json!(1),
                },
                Condition {
                    from: "a".into(),
                    path: "y".into(),
                    op: CondOp::Eq,
                    value: json!(99),
                },
            ],
        };
        assert!(!case.evaluate(&inputs(&[("a", json!({"x": 1, "y": 2}))])));
    }

    #[test]
    fn case_or_one_passes() {
        let case = Case {
            id: "ok".into(),
            logical_operator: LogicalOp::Or,
            conditions: vec![
                Condition {
                    from: "a".into(),
                    path: "x".into(),
                    op: CondOp::Eq,
                    value: json!(99),
                },
                Condition {
                    from: "a".into(),
                    path: "y".into(),
                    op: CondOp::Eq,
                    value: json!(2),
                },
            ],
        };
        assert!(case.evaluate(&inputs(&[("a", json!({"x": 1, "y": 2}))])));
    }

    #[test]
    fn case_empty_conditions_returns_false() {
        let case = Case {
            id: "ok".into(),
            logical_operator: LogicalOp::And,
            conditions: vec![],
        };
        assert!(!case.evaluate(&inputs(&[])));
    }
}
