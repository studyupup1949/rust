//! Tool ecosystem for the a3s-flow workflow engine.
//!
//! Provides a unified `Tool` trait for built-in and custom tools,
//! and a `ToolRegistry` for managing available tools.
//!
//! # Built-in tools
//!
//! | Tool | Description |
//! |------|-------------|
//! | `http_fetch` | HTTP GET/POST request |
//! | `calculator` | Math expression evaluator |
//!
//! # Usage
//!
//! Tools are registered in `ToolRegistry` and can be looked up by name
//! when executing an `AgentNode`.

pub mod builtin;
pub mod tool;

pub use tool::{ToolCall, ToolOutput};
