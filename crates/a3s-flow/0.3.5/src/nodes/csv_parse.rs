//! `"csv-parse"` node — parse CSV text into a JSON array.
//!
//! Parses CSV text from an upstream node output or variable into a JSON array.
//! Supports configurable delimiter and header row handling.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "input_selector": "fetch.body",
//!   "delimiter": ",",
//!   "has_header": true
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `input_selector` | string | ✅ | Dot path into upstream inputs: `"node_id"` or `"node_id.field.subfield"` |
//! | `delimiter` | string | — | Field delimiter (default: `","`) |
//! | `has_header` | boolean | — | Whether the first row is a header (default: `true`) |
//!
//! # Output schema
//!
//! **With header (`has_header: true`, default):**
//! ```json
//! {
//!   "output": [
//!     { "name": "Alice", "age": "30", "city": "NYC" },
//!     { "name": "Bob", "age": "25", "city": "LA" }
//!   ]
//! }
//! ```
//!
//! **Without header (`has_header: false`):**
//! ```json
//! {
//!   "output": [
//!     ["Alice", "30", "NYC"],
//!     ["Bob", "25", "LA"]
//!   ]
//! }
//! ```
//!
//! # Example
//!
//! ```json
//! {
//!   "id": "parse_csv",
//!   "type": "csv-parse",
//!   "data": {
//!     "input_selector": "fetch.body",
//!     "delimiter": ",",
//!     "has_header": true
//!   }
//! }
//! ```

use async_trait::async_trait;
use csv::ReaderBuilder;
use serde_json::{json, Value};

use crate::condition::get_path;
use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

/// CSV parse node — converts CSV text into a JSON array.
pub struct CsvParseNode;

#[async_trait]
impl Node for CsvParseNode {
    fn node_type(&self) -> &str {
        "csv-parse"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        // ── Resolve input CSV text ────────────────────────────────────────
        let input_selector = ctx.data["input_selector"].as_str().ok_or_else(|| {
            FlowError::InvalidDefinition("csv-parse: missing data.input_selector".into())
        })?;

        let csv_text = resolve_input_text(&ctx.inputs, input_selector)?;

        // ── Parse configuration ───────────────────────────────────────────
        let delimiter = ctx.data["delimiter"]
            .as_str()
            .and_then(|s| s.chars().next())
            .unwrap_or(',');

        let has_header = ctx.data["has_header"].as_bool().unwrap_or(true);

        // ── Parse CSV ─────────────────────────────────────────────────────
        let output = if has_header {
            parse_csv_with_header(&csv_text, delimiter)?
        } else {
            parse_csv_without_header(&csv_text, delimiter)?
        };

        Ok(json!({ "output": output }))
    }
}

// ── Helper functions ──────────────────────────────────────────────────────

/// Resolve the input CSV text from upstream inputs using a dot-separated path.
fn resolve_input_text(
    inputs: &std::collections::HashMap<String, Value>,
    selector: &str,
) -> Result<String> {
    let parts: Vec<&str> = selector.split('.').collect();
    if parts.is_empty() {
        return Err(FlowError::InvalidDefinition(
            "csv-parse: input_selector cannot be empty".into(),
        ));
    }

    let node_id = parts[0];
    let node_output = inputs.get(node_id).ok_or_else(|| {
        FlowError::Internal(format!(
            "csv-parse: upstream node '{}' not found in inputs",
            node_id
        ))
    })?;

    let value = if parts.len() == 1 {
        node_output
    } else {
        let path = parts[1..].join(".");
        get_path(node_output, &path).unwrap_or(&Value::Null)
    };

    value.as_str().map(|s| s.to_string()).ok_or_else(|| {
        FlowError::InvalidDefinition(format!(
            "csv-parse: input at '{}' is not a string",
            selector
        ))
    })
}

/// Parse CSV with header row — returns array of objects.
fn parse_csv_with_header(csv_text: &str, delimiter: char) -> Result<Vec<Value>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .from_reader(csv_text.as_bytes());

    let headers = reader
        .headers()
        .map_err(|e| FlowError::Internal(format!("csv-parse: failed to read headers: {}", e)))?
        .iter()
        .map(|h| h.to_string())
        .collect::<Vec<_>>();

    let mut output = Vec::new();

    for result in reader.records() {
        let record = result
            .map_err(|e| FlowError::Internal(format!("csv-parse: failed to read record: {}", e)))?;

        let mut obj = serde_json::Map::new();
        for (i, field) in record.iter().enumerate() {
            let key = headers.get(i).map(|s| s.as_str()).unwrap_or("");
            obj.insert(key.to_string(), Value::String(field.to_string()));
        }
        output.push(Value::Object(obj));
    }

    Ok(output)
}

/// Parse CSV without header row — returns array of arrays.
fn parse_csv_without_header(csv_text: &str, delimiter: char) -> Result<Vec<Value>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .has_headers(false)
        .from_reader(csv_text.as_bytes());

    let mut output = Vec::new();

    for result in reader.records() {
        let record = result
            .map_err(|e| FlowError::Internal(format!("csv-parse: failed to read record: {}", e)))?;

        let row: Vec<Value> = record
            .iter()
            .map(|field| Value::String(field.to_string()))
            .collect();

        output.push(Value::Array(row));
    }

    Ok(output)
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ctx_with_input(data: Value, input_node: &str, input_value: Value) -> ExecContext {
        let mut inputs = HashMap::new();
        inputs.insert(input_node.to_string(), input_value);
        ExecContext {
            data,
            inputs,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn parses_csv_with_header() {
        let csv_text = "name,age,city\nAlice,30,NYC\nBob,25,LA";
        let node = CsvParseNode;
        let out = node
            .execute(ctx_with_input(
                json!({ "input_selector": "fetch" }),
                "fetch",
                json!(csv_text),
            ))
            .await
            .unwrap();

        let output = out["output"].as_array().unwrap();
        assert_eq!(output.len(), 2);
        assert_eq!(output[0]["name"], "Alice");
        assert_eq!(output[0]["age"], "30");
        assert_eq!(output[0]["city"], "NYC");
        assert_eq!(output[1]["name"], "Bob");
    }

    #[tokio::test]
    async fn parses_csv_without_header() {
        let csv_text = "Alice,30,NYC\nBob,25,LA";
        let node = CsvParseNode;
        let out = node
            .execute(ctx_with_input(
                json!({ "input_selector": "fetch", "has_header": false }),
                "fetch",
                json!(csv_text),
            ))
            .await
            .unwrap();

        let output = out["output"].as_array().unwrap();
        assert_eq!(output.len(), 2);
        assert_eq!(output[0], json!(["Alice", "30", "NYC"]));
        assert_eq!(output[1], json!(["Bob", "25", "LA"]));
    }

    #[tokio::test]
    async fn parses_csv_with_custom_delimiter() {
        let csv_text = "name;age;city\nAlice;30;NYC\nBob;25;LA";
        let node = CsvParseNode;
        let out = node
            .execute(ctx_with_input(
                json!({ "input_selector": "fetch", "delimiter": ";" }),
                "fetch",
                json!(csv_text),
            ))
            .await
            .unwrap();

        let output = out["output"].as_array().unwrap();
        assert_eq!(output.len(), 2);
        assert_eq!(output[0]["name"], "Alice");
        assert_eq!(output[0]["age"], "30");
    }

    #[tokio::test]
    async fn parses_csv_from_nested_path() {
        let csv_text = "name,age\nAlice,30";
        let node = CsvParseNode;
        let out = node
            .execute(ctx_with_input(
                json!({ "input_selector": "fetch.body" }),
                "fetch",
                json!({ "body": csv_text }),
            ))
            .await
            .unwrap();

        let output = out["output"].as_array().unwrap();
        assert_eq!(output.len(), 1);
        assert_eq!(output[0]["name"], "Alice");
    }

    #[tokio::test]
    async fn empty_csv_returns_empty_array() {
        let csv_text = "name,age\n";
        let node = CsvParseNode;
        let out = node
            .execute(ctx_with_input(
                json!({ "input_selector": "fetch" }),
                "fetch",
                json!(csv_text),
            ))
            .await
            .unwrap();

        let output = out["output"].as_array().unwrap();
        assert_eq!(output.len(), 0);
    }

    #[tokio::test]
    async fn missing_input_selector_returns_error() {
        let node = CsvParseNode;
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
    async fn non_string_input_returns_error() {
        let node = CsvParseNode;
        let err = node
            .execute(ctx_with_input(
                json!({ "input_selector": "fetch" }),
                "fetch",
                json!(123),
            ))
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn missing_upstream_node_returns_error() {
        let node = CsvParseNode;
        let err = node
            .execute(ExecContext {
                data: json!({ "input_selector": "nonexistent" }),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::Internal(_)));
    }
}
