//! # JSON-RPC Response Module
//!
//! This module defines the structures for JSON-RPC responses.
//! A response is returned by the server upon receiving a request (except for notifications)
//! and contains either the successful result of the invocation or an error object.

use crate::{JsonRpcId, request::JsonRpcVersion};
use serde::{Deserialize, Serialize};

/// A standard JSON-RPC response object.
///
/// According to the specification, a response must contain either the `result`
/// or the `error` field, but not both. This implementation uses `Option<T>`
/// and `Option<JsonRpcError>` with `skip_serializing_if` to ensure that
/// missing fields are omitted from the serialized JSON output.
///
/// ### Example (Success)
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "result": "hello",
///   "id": 1
/// }
/// ```
///
/// ### Example (Error)
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "error": { "code": -32601, "message": "Method not found" },
///   "id": 1
/// }
/// ```

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    /// The version of the JSON-RPC protocol.
    pub jsonrpc: JsonRpcVersion,

    /// The result of the method invocation.
    /// This field is present only if the request succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,

    /// The error object.
    /// This field is present only if the request failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,

    /// The identifier matching the `id` of the corresponding request.
    pub id: JsonRpcId,
}

/// A structure representing a JSON-RPC error.
///
/// This object is included in the [`JsonRpcResponse`] when a method
/// call fails. It includes a numeric code and a descriptive message.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// A Number that indicates the error type that occurred.
    pub code: i64,

    /// A String providing a short description of the error.
    pub message: String,

    /// A Primitive or Structured value that contains additional information
    /// about the error. This field is omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}
