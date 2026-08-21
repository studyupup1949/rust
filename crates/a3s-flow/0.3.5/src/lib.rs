//! # a3s-flow
//!
//! A3S workflow engine — JSON DAG execution for agentic workflows.
//!
//! ## Architecture (Minimal Core + Extensions)
//!
//! **Core components:**
//! - [`FlowEngine`] — lifecycle API: start, pause, resume, terminate, query state
//! - [`DagGraph`] — parse + validate the JSON DAG, topological sort
//! - [`FlowRunner`] — wave-based concurrent execution engine
//! - [`NodeRegistry`] — maps type strings to [`Node`] implementations
//! - [`ExecContext`] — per-node runtime context (config + inputs + variables)
//!
//! **Extension point:** implement [`Node`] to add any node type.
//!
//! ## Built-in nodes (Dify-compatible)
//!
//! | Type string | Purpose |
//! |-------------|---------|
//! | `"noop"` | Pass inputs through (placeholder / fan-in join) |
//! | `"start"` | Dify-compatible entry point with typed input declaration |
//! | `"end"` | Dify-compatible output collector (JSON pointer paths) |
//! | `"http-request"` | HTTP GET / POST / PUT / DELETE / PATCH |
//! | `"if-else"` | Multi-case conditional routing → `{ "branch": "case_id" }` |
//! | `"template-transform"` | Jinja2 string rendering |
//! | `"variable-aggregator"` | First non-null fan-in from multiple branches |
//! | `"code"` | Sandboxed Rhai script execution |
//! | `"iteration"` | Concurrent or sequential sub-flow loop over an array |
//! | `"sub-flow"` | Execute a named flow as an inline step |
//! | `"llm"` | OpenAI-compatible chat completion with Jinja2 prompt templates |
//! | `"question-classifier"` | LLM-powered routing into N user-defined classes |
//! | `"assign"` | Write key-value pairs into the live flow variable scope |
//! | `"parameter-extractor"` | LLM-powered structured parameter extraction from natural language |
//! | `"loop"` | While-loop over inline sub-flow with break condition |
//! | `"list-operator"` | Filter / sort / deduplicate / limit a JSON array |
//!
//! ## Quick start — via `FlowEngine` (recommended)
//!
//! ```rust,no_run
//! use a3s_flow::{FlowEngine, NodeRegistry};
//! use serde_json::json;
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> a3s_flow::Result<()> {
//!     let engine = FlowEngine::new(NodeRegistry::with_defaults());
//!
//!     let def = json!({
//!         "nodes": [
//!             { "id": "start",   "type": "noop" },
//!             { "id": "process", "type": "noop" }
//!         ],
//!         "edges": [{ "source": "start", "target": "process" }]
//!     });
//!     let id = engine.start(&def, HashMap::new()).await?;
//!
//!     engine.pause(id).await?;
//!     engine.resume(id).await?;
//!     println!("{:?}", engine.state(id).await?);
//!     Ok(())
//! }
//! ```

pub mod capabilities;
pub mod condition;
pub mod engine;
pub mod error;
pub mod event;
pub mod execution;
pub mod flow_store;
pub mod graph;
pub mod node;
pub mod nodes;
pub mod registry;
pub mod result;
pub mod runner;
pub mod server;
pub mod service;
pub mod store;
pub mod validation;

pub use capabilities::FlowCapabilities;
pub use condition::{Case, CondOp, Condition, LogicalOp};
pub use engine::FlowEngine;
pub use error::{FlowError, Result};
pub use event::{EventEmitter, FlowEvent, NoopEventEmitter};
pub use execution::ExecutionState;
pub use flow_store::{FlowStore, MemoryFlowStore};
pub use graph::{DagGraph, EdgeDef, NodeDef};
pub use node::{ExecContext, Node, RetryPolicy};
pub use registry::{NodeDescriptor, NodeFieldDescriptor, NodeRegistry};
pub use result::FlowResult;
pub use runner::FlowRunner;
pub use server::{
    build_router as build_http_router,
    build_router_with_factories as build_http_router_with_factories,
    build_router_with_service as build_http_router_with_service, serve as serve_http,
    serve_listener as serve_http_listener, serve_with_factories as serve_http_with_factories,
    serve_with_service as serve_http_with_service,
};
pub use service::{FlowService, NodeFactory};
pub use store::{ExecutionStore, MemoryExecutionStore};
pub use validation::ValidationIssue;
