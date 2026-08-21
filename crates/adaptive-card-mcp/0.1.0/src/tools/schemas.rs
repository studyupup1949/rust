//! JSON Schemas advertised for each tool in `tools/list` responses.
//!
//! Schemas are hand-authored (not derived) so we keep tight control over
//! descriptions and examples that the LLM sees. Each schema is a JSON
//! object; [`build_all`] returns one [`rmcp::model::Tool`] per tool.

use std::sync::Arc;

use rmcp::model::{JsonObject, Tool};
use serde_json::{Value, json};

use super::TOOL_NAMES;

/// Return all ten tools advertised by the server, in `TOOL_NAMES` order.
#[must_use]
pub fn build_all() -> Vec<Tool> {
    TOOL_NAMES.iter().map(|name| build_one(name)).collect()
}

fn build_one(name: &str) -> Tool {
    let (description, schema_json) = match name {
        "validate_card" => (
            "Validate an Adaptive Card against the v1.6 JSON Schema, the \
             accessibility rules, and (optionally) a target host's capabilities.",
            json!({
                "type": "object",
                "properties": {
                    "card": { "type": "object", "description": "The Adaptive Card JSON to validate." },
                    "host": { "type": "string", "description": "Optional host: teams, outlook, webchat, windows, viva, webex, generic." }
                },
                "required": ["card"]
            }),
        ),
        "analyze_card" => (
            "Report structural metrics for an Adaptive Card: element count, \
             action count, nesting depth, duplicate IDs, text length, etc.",
            json!({
                "type": "object",
                "properties": {
                    "card": { "type": "object", "description": "The Adaptive Card JSON to analyze." }
                },
                "required": ["card"]
            }),
        ),
        "check_accessibility" => (
            "Run accessibility rules over a card and return a 0-100 score plus \
             the full list of issues (errors, warnings, info) and passing checks.",
            json!({
                "type": "object",
                "properties": {
                    "card": { "type": "object", "description": "The Adaptive Card JSON to inspect." }
                },
                "required": ["card"]
            }),
        ),
        "optimize_card" => (
            "Apply accessibility, performance, and/or modernization optimizations \
             to an Adaptive Card. Returns the optimized card plus a changelog.",
            json!({
                "type": "object",
                "properties": {
                    "card": { "type": "object", "description": "The Adaptive Card JSON to optimize." },
                    "accessibility": { "type": "boolean", "default": false },
                    "performance":   { "type": "boolean", "default": false },
                    "modernize":     { "type": "boolean", "default": false },
                    "target_host":   { "type": "string", "description": "Optional host name to tune for." }
                },
                "required": ["card"]
            }),
        ),
        "transform_card" => (
            "Transform an Adaptive Card to a target version and/or host: downgrade \
             schema version, remove unsupported elements, trim excess actions, etc.",
            json!({
                "type": "object",
                "properties": {
                    "card":           { "type": "object" },
                    "target_version": { "type": "string", "description": "1.0-1.6" },
                    "target_host":    { "type": "string" },
                    "strict":         { "type": "boolean", "default": false, "description": "Error on lossy transforms." }
                },
                "required": ["card"]
            }),
        ),
        "template_card" => (
            "Convert a static Adaptive Card into a template by extracting literal \
             strings into ${expr} bindings and producing matching sample data.",
            json!({
                "type": "object",
                "properties": {
                    "card": { "type": "object" }
                },
                "required": ["card"]
            }),
        ),
        "data_to_card" => (
            "Auto-generate an Adaptive Card from raw data (list, table, or object). \
             Optionally force a presentation (table, factset, list, chart, auto).",
            json!({
                "type": "object",
                "properties": {
                    "data":         { "description": "Any JSON value to render." },
                    "title":        { "type": "string" },
                    "host":         { "type": "string", "default": "generic" },
                    "presentation": { "type": "string", "enum": ["table", "factset", "list", "chart", "auto"] }
                },
                "required": ["data"]
            }),
        ),
        "list_examples" => (
            "List examples from the knowledge base. Optionally filter by category \
             and limit the number of results.",
            json!({
                "type": "object",
                "properties": {
                    "category": { "type": "string" },
                    "limit":    { "type": "integer", "minimum": 1, "default": 20 }
                }
            }),
        ),
        "get_example" => (
            "Fetch a full knowledge base entry by id (card JSON + metadata + notes).",
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string" }
                },
                "required": ["id"]
            }),
        ),
        "suggest_layout" => (
            "Suggest knowledge base entries matching a free-text query using a \
             keyword scoring selector.",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "minimum": 1, "default": 5 }
                },
                "required": ["query"]
            }),
        ),
        other => (
            "(unknown tool)",
            json!({ "type": "object", "description": format!("no schema for {other}") }),
        ),
    };

    let input_schema: JsonObject = match schema_json {
        Value::Object(map) => map,
        _ => JsonObject::new(),
    };

    Tool::new(
        name.to_string(),
        description.to_string(),
        Arc::new(input_schema),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_ten_tools() {
        let tools = build_all();
        assert_eq!(tools.len(), 10);
    }

    #[test]
    fn every_tool_has_object_schema() {
        for tool in build_all() {
            let schema = &tool.input_schema;
            assert_eq!(
                schema.get("type").and_then(Value::as_str),
                Some("object"),
                "tool {} missing object schema",
                tool.name
            );
        }
    }

    #[test]
    fn validate_card_requires_card() {
        let tools = build_all();
        let tool = tools.iter().find(|t| t.name == "validate_card").unwrap();
        let required = tool
            .input_schema
            .get("required")
            .and_then(Value::as_array)
            .unwrap();
        assert!(required.iter().any(|v| v == "card"));
    }
}
