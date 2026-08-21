//! Built-in node implementations.
//!
//! Each module provides a node type registered in [`NodeRegistry::with_defaults`].
//!
//! | Module | Node type string | Dify equivalent |
//! |--------|-----------------|-----------------|
//! | [`noop`] | `"noop"` | — |
//! | [`start`] | `"start"` | Start |
//! | [`end`] | `"end"` | End |
//! | [`http`] | `"http-request"` | HTTP Request |
//! | [`cond`] | `"if-else"` | IF/ELSE |
//! | [`template_transform`] | `"template-transform"` | Template |
//! | [`variable_aggregator`] | `"variable-aggregator"` | Variable Aggregator |
//! | [`code`] | `"code"` | Code |
//! | [`iteration`] | `"iteration"` | Iteration |
//! | [`subflow`] | `"sub-flow"` | Sub-flow |
//! | [`llm`] | `"llm"` | LLM |
//! | [`question_classifier`] | `"question-classifier"` | Question Classifier |
//! | [`assign`] | `"assign"` | Variable Assigner |
//! | [`parameter_extractor`] | `"parameter-extractor"` | Parameter Extractor |
//! | [`loop_node`] | `"loop"` | Loop |
//! | [`list_operator`] | `"list-operator"` | List Operator |
//! | [`mcp`] | `"mcp"` | — (external tool call) |
//!
//! [`NodeRegistry::with_defaults`]: crate::registry::NodeRegistry::with_defaults

pub mod assign;
pub mod code;
pub mod list_operator;
pub mod loop_node;
pub mod mcp;
pub mod parameter_extractor;
pub mod cond;
pub mod end;
pub mod http;
pub mod iteration;
pub mod llm;
pub mod noop;
pub mod question_classifier;
pub mod start;
pub mod subflow;
pub mod template_transform;
pub mod variable_aggregator;
