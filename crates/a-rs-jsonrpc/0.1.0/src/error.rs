//! # Error Handling Module
//!
//! This module defines the [`RpcError`] enum, the central error type used throughout
//! the library for both client and server operations. It also provides automatic
//! conversion into the standard JSON-RPC error format.

use crate::response::JsonRpcError;
use thiserror::Error;

/// The primary error type for JSON-RPC operations.
///
/// This enum wraps underlying transport errors (IO, HTTP), serialization errors (Serde),
/// and protocol-specific violations (Invalid Version, Method Not Found).
///
/// It implements [`thiserror::Error`] for convenient error propagation and
/// provides a mapping to [`JsonRpcError`] for standardized API responses.

#[derive(Debug, Error)]
pub enum RpcError {
    /// Errors occurring during low-level I/O operations.
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    /// Errors occurring during HTTP transport, handled by the `reqwest` crate.
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    /// Errors occurring during JSON serialization or deserialization.
    #[error("serialize/deserialize error: {0}")]
    SerdeError(#[from] serde_json::Error),

    /// Triggered when the `jsonrpc` field in a request does not match the expected version.
    #[error("invalid json rpc version: {0}")]
    InvalidJsonRpcVersion(String),

    /// Standard JSON-RPC error (-32601) indicating the requested method does not exist.
    #[error("json rpc method not found")]
    MethodNotFound,

    /// General-purpose error for custom business logic failures.
    #[error("custom error: {0}")]
    CustomError(String),

    /// Standard JSON-RPC error (-32602) indicating invalid or malformed arguments.
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
}

impl From<RpcError> for JsonRpcError {
    /// Converts an internal [`RpcError`] into a [`JsonRpcError`] suitable for
    /// transmission over the wire.
    ///
    /// The conversion maps specific variants to standard JSON-RPC codes:
    /// - `InvalidJsonRpcVersion` -> `-32600` (Invalid Request)
    /// - `MethodNotFound` -> `-32601`
    /// - `InvalidParams` -> `-32602`
    /// - Internal errors (IO/Reqwest/Serde) -> `-32000` to `-32002` (Server Error range)
    fn from(err: RpcError) -> Self {
        match err {
            RpcError::IoError(e) => JsonRpcError {
                code: -32000,
                message: e.to_string(),
                data: None,
            },
            RpcError::ReqwestError(e) => JsonRpcError {
                code: -32001,
                message: e.to_string(),
                data: None,
            },
            RpcError::SerdeError(e) => JsonRpcError {
                code: -32002,
                message: e.to_string(),
                data: None,
            },
            RpcError::InvalidJsonRpcVersion(v) => JsonRpcError {
                code: -32600,
                message: format!("Invalid JSON-RPC version: {}", v),
                data: None,
            },
            RpcError::MethodNotFound => JsonRpcError {
                code: -32601,
                message: "method not found".to_string(),
                data: None,
            },
            RpcError::CustomError(msg) => JsonRpcError {
                code: -32003,
                message: msg,
                data: None,
            },
            RpcError::InvalidParams(msg) => JsonRpcError {
                code: -32602,
                message: format!("Invalid parameters: {}", msg),
                data: None,
            },
        }
    }
}
