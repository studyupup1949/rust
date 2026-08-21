//! Node registry — maps node type strings to [`Node`] implementations.
//!
//! [`NodeRegistry`] is the extension point for adding custom node types.
//! The default registry ships with all built-in Dify-compatible nodes.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{FlowError, Result};
use crate::node::Node;
use crate::nodes::assign::AssignNode;
use crate::nodes::code::CodeNode;
use crate::nodes::list_operator::ListOperatorNode;
use crate::nodes::loop_node::LoopNode;
use crate::nodes::mcp::McpNode;
use crate::nodes::parameter_extractor::ParameterExtractorNode;
use crate::nodes::cond::IfElseNode;
use crate::nodes::http::HttpRequestNode;
use crate::nodes::end::EndNode;
use crate::nodes::iteration::IterationNode;
use crate::nodes::llm::LlmNode;
use crate::nodes::noop::NoopNode;
use crate::nodes::question_classifier::QuestionClassifierNode;
use crate::nodes::start::StartNode;
use crate::nodes::subflow::SubFlowNode;
use crate::nodes::template_transform::TemplateTransformNode;
use crate::nodes::variable_aggregator::VariableAggregatorNode;

/// Registry mapping node type strings to [`Node`] implementations.
///
/// `Clone` is cheap — all values are `Arc`-wrapped.
#[derive(Clone)]
pub struct NodeRegistry {
    nodes: HashMap<String, Arc<dyn Node>>,
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
    /// | `"iteration"` | [`IterationNode`] — sub-flow loop over an array |
    /// | `"sub-flow"` | [`SubFlowNode`] — execute a named flow as an inline step |
    /// | `"llm"` | [`LlmNode`] — OpenAI-compatible chat completion |
    /// | `"question-classifier"` | [`QuestionClassifierNode`] — LLM-powered routing |
    /// | `"assign"` | [`AssignNode`] — write key-value pairs into the flow's variable scope |
    /// | `"parameter-extractor"` | [`ParameterExtractorNode`] — LLM-powered structured parameter extraction |
    /// | `"loop"` | [`LoopNode`] — while-loop over a sub-flow with break condition |
    /// | `"list-operator"` | [`ListOperatorNode`] — filter / sort / deduplicate / limit a JSON array |
    /// | `"mcp"` | [`McpNode`] — call a tool on a Model Context Protocol server |
    pub fn with_defaults() -> Self {
        let mut r = Self {
            nodes: HashMap::new(),
        };
        r.register(Arc::new(NoopNode));
        r.register(Arc::new(StartNode));
        r.register(Arc::new(EndNode));
        r.register(Arc::new(HttpRequestNode));
        r.register(Arc::new(IfElseNode));
        r.register(Arc::new(TemplateTransformNode));
        r.register(Arc::new(VariableAggregatorNode));
        r.register(Arc::new(CodeNode));
        r.register(Arc::new(IterationNode));
        r.register(Arc::new(SubFlowNode));
        r.register(Arc::new(LlmNode));
        r.register(Arc::new(QuestionClassifierNode));
        r.register(Arc::new(AssignNode));
        r.register(Arc::new(ParameterExtractorNode));
        r.register(Arc::new(LoopNode));
        r.register(Arc::new(ListOperatorNode));
        r.register(Arc::new(McpNode));
        r
    }

    /// Register a custom node implementation.
    ///
    /// The node's [`Node::node_type`] is used as the lookup key.
    /// Overwrites any existing registration for the same type string.
    pub fn register(&mut self, node: Arc<dyn Node>) {
        self.nodes.insert(node.node_type().to_string(), node);
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
}

impl Default for NodeRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}
