//! # Actflow Agent SDK
//!
//! A Rust SDK for building agents for Actflow workflow engine.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use actflow_agent_sdk::{Agent, AgentServer, Context, Inputs, LogSender};
//! use actflow_agent_sdk::{AgentOutput, NodeExecutionStatus};
//!
//! struct MyAgent;
//!
//! impl Agent for MyAgent {
//!     async fn run(
//!         &self,
//!         nid: String,
//!         ctx: Context,
//!         inputs: Inputs,
//!         log: LogSender,
//!     ) -> AgentOutput {
//!         log.send("Starting agent...").await;
//!         // Your agent logic here
//!         AgentOutput::success(serde_json::json!({"result": "done"}))
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     AgentServer::new(MyAgent)
//!         .serve("0.0.0.0:50051")
//!         .await
//!         .unwrap();
//! }
//! ```

mod proto {
    tonic::include_proto!("agent");
}

mod agent;
mod server;
mod types;

pub use agent::{Agent, LogSender};
pub use server::AgentServer;
pub use types::*;

// Re-export proto types for advanced usage
pub mod pb {
    pub use crate::proto::*;
}
