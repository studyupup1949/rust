//! Node registry — maps node type strings to [`Node`] implementations.
//!
//! [`NodeRegistry`] is the extension point for adding custom node types.
//! The default registry ships with all built-in Dify-compatible nodes.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{FlowError, Result};
use crate::node::Node;
use crate::nodes::assign::AssignNode;
use crate::nodes::code::CodeNode;
use crate::nodes::cond::IfElseNode;
use crate::nodes::context_get::ContextGetNode;
use crate::nodes::context_set::ContextSetNode;
use crate::nodes::csv_parse::CsvParseNode;
use crate::nodes::end::EndNode;
use crate::nodes::http::HttpRequestNode;
use crate::nodes::iteration::IterationNode;
use crate::nodes::list_operator::ListOperatorNode;
use crate::nodes::llm::LlmNode;
use crate::nodes::loop_node::LoopNode;
use crate::nodes::noop::NoopNode;
use crate::nodes::parameter_extractor::ParameterExtractorNode;
use crate::nodes::question_classifier::QuestionClassifierNode;
use crate::nodes::start::StartNode;
use crate::nodes::subflow::SubFlowNode;
use crate::nodes::template_transform::TemplateTransformNode;
use crate::nodes::variable_aggregator::VariableAggregatorNode;

/// Static capability descriptor for a registered node type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeDescriptor {
    /// Stable type string used in flow definitions.
    pub node_type: String,
    /// Human-friendly display name suitable for UI lists.
    pub display_name: String,
    /// Short category label for grouping similar node types.
    pub category: String,
    /// Single-sentence summary of what the node does.
    pub summary: String,
    /// Suggested default config payload for editor-side node creation.
    #[serde(default = "default_node_config")]
    pub default_data: Value,
    /// Optional field hints for editors and capability discovery UIs.
    #[serde(default)]
    pub fields: Vec<NodeFieldDescriptor>,
}

/// Field-level hint for a node configuration shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeFieldDescriptor {
    pub key: String,
    pub kind: String,
    pub required: bool,
    pub description: String,
}

fn default_node_config() -> Value {
    json!({})
}

/// Registry mapping node type strings to [`Node`] implementations.
///
/// `Clone` is cheap — all values are `Arc`-wrapped.
#[derive(Clone)]
pub struct NodeRegistry {
    nodes: HashMap<String, Arc<dyn Node>>,
    descriptors: HashMap<String, NodeDescriptor>,
    builtin_types: HashSet<String>,
}

impl NodeRegistry {
    /// Create a registry pre-loaded with all built-in node types.
    ///
    /// | Type string | Node |
    /// |-------------|------|
    /// | `"noop"` | [`NoopNode`] — passes inputs through |
    /// | `"start"` | [`StartNode`] — Dify-compatible input declaration with defaults |
    /// | `"end"` | [`EndNode`] — gathers upstream values via JSON pointer paths |
    /// | `"http-request"` | [`HttpRequestNode`] — HTTP GET/POST/PUT/DELETE/PATCH |
    /// | `"if-else"` | [`IfElseNode`] — multi-case conditional routing |
    /// | `"template-transform"` | [`TemplateTransformNode`] — Jinja2 rendering |
    /// | `"variable-aggregator"` | [`VariableAggregatorNode`] — first non-null fan-in |
    /// | `"code"` | [`CodeNode`] — sandboxed Rhai script |
    /// | `"csv-parse"` | [`CsvParseNode`] — parse CSV text into JSON array |
    /// | `"iteration"` | [`IterationNode`] — sub-flow loop over an array |
    /// | `"sub-flow"` | [`SubFlowNode`] — execute a named flow as an inline step |
    /// | `"llm"` | [`LlmNode`] — OpenAI-compatible chat completion |
    /// | `"question-classifier"` | [`QuestionClassifierNode`] — LLM-powered routing |
    /// | `"assign"` | [`AssignNode`] — write key-value pairs into the flow's variable scope |
    /// | `"context-get"` | [`ContextGetNode`] — read keys from the shared execution context |
    /// | `"context-set"` | [`ContextSetNode`] — write key-value pairs into the shared execution context |
    /// | `"parameter-extractor"` | [`ParameterExtractorNode`] — LLM-powered structured parameter extraction |
    /// | `"loop"` | [`LoopNode`] — while-loop over a sub-flow with break condition |
    /// | `"list-operator"` | [`ListOperatorNode`] — filter / sort / deduplicate / limit a JSON array |
    pub fn with_defaults() -> Self {
        let mut r = Self {
            nodes: HashMap::new(),
            descriptors: HashMap::new(),
            builtin_types: HashSet::new(),
        };
        r.register_builtin(
            Arc::new(NoopNode),
            "No-op",
            "utility",
            "Pass inputs through unchanged for placeholder or fan-in flows.",
            json!({}),
            vec![],
        );
        r.register_builtin(
            Arc::new(StartNode),
            "Start",
            "control",
            "Declares workflow inputs and default values at the entry point.",
            json!({ "inputs": [] }),
            vec![field(
                "inputs",
                "array",
                false,
                "Input variable declarations.",
            )],
        );
        r.register_builtin(
            Arc::new(EndNode),
            "End",
            "control",
            "Collects final outputs from upstream nodes.",
            json!({ "outputs": {} }),
            vec![field("outputs", "object", false, "Output field mapping.")],
        );
        r.register_builtin(
            Arc::new(HttpRequestNode),
            "HTTP Request",
            "integration",
            "Calls external HTTP APIs with configurable method, headers, and body.",
            json!({ "method": "GET", "url": "", "headers": {} }),
            vec![
                field("method", "string", true, "HTTP method."),
                field("url", "string", true, "Request URL."),
            ],
        );
        r.register_builtin(
            Arc::new(IfElseNode),
            "If/Else",
            "logic",
            "Routes execution to a branch based on evaluated conditions.",
            json!({ "cases": [{ "id": "case_1", "logical_operator": "and", "conditions": [] }] }),
            vec![field(
                "cases",
                "array",
                true,
                "Branch definitions keyed by case id.",
            )],
        );
        r.register_builtin(
            Arc::new(TemplateTransformNode),
            "Template Transform",
            "transform",
            "Renders structured or text output from a Jinja-style template.",
            json!({ "template": "" }),
            vec![field(
                "template",
                "string",
                true,
                "Jinja-style template body.",
            )],
        );
        r.register_builtin(
            Arc::new(VariableAggregatorNode),
            "Variable Aggregator",
            "transform",
            "Selects the first non-null value from multiple upstream branches.",
            json!({ "mode": "first_non_null" }),
            vec![field("mode", "string", false, "Aggregation strategy.")],
        );
        r.register_builtin(
            Arc::new(CodeNode),
            "Code",
            "compute",
            "Executes sandboxed Rhai code against the current flow state.",
            json!({ "script": "" }),
            vec![field("script", "string", true, "Rhai script body.")],
        );
        r.register_builtin(
            Arc::new(CsvParseNode),
            "CSV Parse",
            "transform",
            "Parses CSV text into structured JSON rows.",
            json!({ "text": "" }),
            vec![field("text", "string", true, "Raw CSV text input.")],
        );
        r.register_builtin(
            Arc::new(IterationNode),
            "Iteration",
            "control",
            "Runs a sub-flow for each item in an input array.",
            json!({ "input_selector": "", "output_selector": "", "mode": "parallel", "flow": { "nodes": [], "edges": [] } }),
            vec![
                field("input_selector", "string", true, "Selector for input array."),
                field("flow", "object", true, "Nested flow definition."),
            ],
        );
        r.register_builtin(
            Arc::new(SubFlowNode),
            "Sub-flow",
            "control",
            "Invokes another named flow as a nested step.",
            json!({ "flow_name": "" }),
            vec![field("flow_name", "string", true, "Target named flow.")],
        );
        r.register_builtin(
            Arc::new(LlmNode),
            "LLM",
            "ai",
            "Sends a prompt to an OpenAI-compatible chat completion model.",
            json!({ "model": "gpt-4o-mini", "system_prompt": "", "user_prompt": "", "api_base": "https://api.openai.com/v1", "api_key": "", "temperature": 0.7 }),
            vec![
                field("model", "string", true, "Model identifier."),
                field("user_prompt", "string", true, "User prompt template."),
            ],
        );
        r.register_builtin(
            Arc::new(QuestionClassifierNode),
            "Question Classifier",
            "ai",
            "Classifies user intent into predefined categories with an LLM.",
            json!({ "model": "gpt-4o-mini", "question": "", "classes": [], "api_base": "https://api.openai.com/v1", "api_key": "", "temperature": 0 }),
            vec![
                field("question", "string", true, "Input question to classify."),
                field("classes", "array", true, "Target label list."),
            ],
        );
        r.register_builtin(
            Arc::new(AssignNode),
            "Assign",
            "context",
            "Writes key-value pairs into the flow variable scope.",
            json!({ "assigns": {} }),
            vec![field("assigns", "object", true, "Key-value assignments.")],
        );
        r.register_builtin(
            Arc::new(ContextGetNode),
            "Context Get",
            "context",
            "Reads values from shared execution context.",
            json!({ "keys": [] }),
            vec![field("keys", "array", true, "Context keys to read.")],
        );
        r.register_builtin(
            Arc::new(ContextSetNode),
            "Context Set",
            "context",
            "Writes values into shared execution context.",
            json!({ "values": {} }),
            vec![field("values", "object", true, "Context values to write.")],
        );
        r.register_builtin(
            Arc::new(ParameterExtractorNode),
            "Parameter Extractor",
            "ai",
            "Extracts structured parameters from natural language with an LLM.",
            json!({ "model": "gpt-4o-mini", "query": "", "parameters": [], "api_base": "https://api.openai.com/v1", "api_key": "", "temperature": 0 }),
            vec![
                field("query", "string", true, "Natural language query."),
                field("parameters", "array", true, "Parameter definitions."),
            ],
        );
        r.register_builtin(
            Arc::new(LoopNode),
            "Loop",
            "control",
            "Repeats a sub-flow until a break condition is met.",
            json!({ "output_selector": "", "max_iterations": 10, "flow": { "nodes": [], "edges": [] } }),
            vec![
                field("max_iterations", "number", false, "Maximum loop iterations."),
                field("flow", "object", true, "Loop body flow definition."),
            ],
        );
        r.register_builtin(
            Arc::new(ListOperatorNode),
            "List Operator",
            "transform",
            "Filters, sorts, deduplicates, or limits a JSON array.",
            json!({ "operation": "limit", "input": "", "limit": 10 }),
            vec![field("operation", "string", true, "List operation kind.")],
        );
        r
    }

    /// Register a custom node implementation.
    ///
    /// The node's [`Node::node_type`] is used as the lookup key.
    /// Overwrites any existing registration for the same type string.
    pub fn register(&mut self, node: Arc<dyn Node>) {
        let node_type = node.node_type().to_string();
        self.nodes.insert(node_type.clone(), node);
        self.descriptors
            .entry(node_type.clone())
            .or_insert_with(|| NodeDescriptor {
                node_type: node_type.clone(),
                display_name: node_type.clone(),
                category: "custom".to_string(),
                summary: "Custom node registered at runtime.".to_string(),
                default_data: default_node_config(),
                fields: Vec::new(),
            });
    }

    /// Register a node implementation with explicit discovery metadata.
    pub fn register_with_descriptor(&mut self, node: Arc<dyn Node>, descriptor: NodeDescriptor) {
        let node_type = node.node_type().to_string();
        self.nodes.insert(node_type.clone(), node);
        self.descriptors.insert(
            node_type.clone(),
            NodeDescriptor {
                node_type,
                ..descriptor
            },
        );
    }

    /// Remove a registered node type and its discovery metadata.
    ///
    /// Returns `true` if the node type existed and was removed, `false`
    /// otherwise.
    pub fn unregister(&mut self, node_type: &str) -> Result<bool> {
        if self.is_builtin(node_type) {
            return Err(FlowError::ProtectedNodeType(node_type.to_string()));
        }
        let removed_node = self.nodes.remove(node_type).is_some();
        let removed_descriptor = self.descriptors.remove(node_type).is_some();
        Ok(removed_node || removed_descriptor)
    }

    /// Return whether a node type is part of the built-in catalog.
    pub fn is_builtin(&self, node_type: &str) -> bool {
        self.builtin_types.contains(node_type)
    }

    /// Look up a node implementation by type string.
    pub fn get(&self, node_type: &str) -> Result<Arc<dyn Node>> {
        self.nodes.get(node_type).cloned().ok_or_else(|| {
            FlowError::InvalidDefinition(format!("unknown node type: '{node_type}'"))
        })
    }

    /// Return all registered node type strings, sorted alphabetically.
    pub fn list_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self.nodes.keys().cloned().collect();
        types.sort();
        types
    }

    /// Return node descriptors sorted by node type.
    pub fn list_descriptors(&self) -> Vec<NodeDescriptor> {
        let mut descriptors: Vec<NodeDescriptor> = self.descriptors.values().cloned().collect();
        descriptors.sort_by(|a, b| a.node_type.cmp(&b.node_type));
        descriptors
    }

    fn register_builtin(
        &mut self,
        node: Arc<dyn Node>,
        display_name: &str,
        category: &str,
        summary: &str,
        default_data: Value,
        fields: Vec<NodeFieldDescriptor>,
    ) {
        let node_type = node.node_type().to_string();
        self.register_with_descriptor(
            node,
            NodeDescriptor {
                node_type: String::new(),
                display_name: display_name.to_string(),
                category: category.to_string(),
                summary: summary.to_string(),
                default_data,
                fields,
            },
        );
        self.builtin_types.insert(node_type);
    }
}

fn field(key: &str, kind: &str, required: bool, description: &str) -> NodeFieldDescriptor {
    NodeFieldDescriptor {
        key: key.to_string(),
        kind: kind.to_string(),
        required,
        description: description.to_string(),
    }
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}
