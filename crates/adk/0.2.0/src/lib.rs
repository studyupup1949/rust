// Re-export the proc macros so users don't need to import adk-macros
pub use adk_macros::*;

pub mod agent;
pub mod error;
pub mod openai;
pub mod tool;
pub mod types;

pub use agent::Agent;
pub use error::AgentError;
pub use openai::Model;
pub use tool::{Tool, ToolResult};

/// Re-export common types for convenience
pub mod prelude {
    pub use super::Agent;
    pub use super::AgentError;
    pub use super::tool_fn;
    pub use super::types::*; // Also expose tool_fn in the prelude
}
