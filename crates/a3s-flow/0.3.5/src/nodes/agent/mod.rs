//! `"agent"` node — multi-turn LLM agent with tool calling.
//!
//! Maintains conversation history across turns via `ExecContext.context`,
//! supports OpenAI function-calling style tools, and loops until a stop
//! condition or `max_turns` is reached.

pub mod node;

pub use node::AgentNode;
