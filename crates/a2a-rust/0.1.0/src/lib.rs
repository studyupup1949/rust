//! Rust SDK for the A2A Protocol v1.0.
//!
//! This crate provides a protocol-accurate type layer plus optional server and
//! client implementations behind feature flags.

/// HTTP client for discovering and calling remote A2A agents.
#[cfg(feature = "client")]
pub mod client;
/// Transport-neutral error type shared across the crate.
pub mod error;
/// JSON-RPC 2.0 envelope types and A2A method/code constants.
pub mod jsonrpc;
/// Axum-based server framework for exposing an A2A agent.
#[cfg(feature = "server")]
pub mod server;
/// Task persistence traits and the in-memory store implementation.
#[cfg(feature = "server")]
pub mod store;
/// Protocol request, response, and resource types.
pub mod types;

#[cfg(feature = "client")]
pub use crate::client::{
    A2AClient, A2AClientConfig, A2AClientStream, AgentCardDiscovery, AgentCardDiscoveryConfig,
};
pub use crate::error::A2AError;
#[cfg(feature = "server")]
pub use crate::store::{InMemoryTaskStore, TaskStore};
pub use crate::types::*;
