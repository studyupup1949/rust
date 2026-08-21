//! acai: Agent2Agent Communication API Implementation
//!
//! A Rust implementation of the Agent2Agent (A2A) protocol, enabling standardized
//! communication between various AI agents and clients.
//!
//! This crate provides:
//! - Type definitions for all A2A protocol messages and data structures
//! - Serialization/deserialization support for JSON-RPC requests and responses
//! - Validation of protocol constraints
//! - HTTP client for making A2A requests
//! - HTTP server for handling A2A requests (supporting both HTTP/1.1 and HTTP/2)
//! - Structured logging via indicio

/// Global collector for structured logging
static COLLECTOR: indicio::Collector = indicio::Collector::new();

pub mod client;
pub mod error;
pub mod server;
mod types;
pub use crate::types::{
    AgentAuthentication, AgentCapabilities, AgentCard, AgentProvider, AgentSkill, Artifact,
    FileContent, FormField, FormSchema, JsonRpcError, JsonRpcRequest, JsonRpcResponse, Message,
    MessageRole, Part, PushNotificationConfig, StreamingResponseContent, Task,
    TaskArtifactUpdateEvent, TaskIdParams, TaskPushNotificationConfig, TaskQueryParams,
    TaskSendParams, TaskState, TaskStatus, TaskStatusUpdateEvent,
};

pub use client::{Client, ClientConfig, Error as ClientError};
pub use error::{
    TaskError, UrlValidationError, to_client_error, to_json_rpc_error, to_server_error,
    to_task_error, validate_url,
};
pub use reqwest::Error as ReqwestError;
pub use serde_json::Value;
pub use server::{
    Error as ServerError, MethodRouter, RequestHandler, Server, ServerConfig,
    jwks::{Claims, JwksError, JwksManager, TokenData},
};
