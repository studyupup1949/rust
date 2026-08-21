//! # JSON-RPC Request Module
//!
//! This module provides the structures and logic for constructing JSON-RPC requests.
//! It supports both versions 1.0 and 2.0 of the protocol and handles the
//! serialization of method calls and their associated parameters.

use crate::{JsonRpcId, RpcError};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Represents the supported JSON-RPC protocol versions.
///
/// This enum ensures that the `jsonrpc` field is serialized correctly as `"1.0"` or `"2.0"`.
/// It also provides validation during deserialization and string parsing.

#[derive(Debug, PartialEq, Eq)]
pub enum JsonRpcVersion {
    /// Version 1.0 of the JSON-RPC specification.
    V1_0,
    /// Version 2.0 of the JSON-RPC specification.
    V2_0,
}

impl Serialize for JsonRpcVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            JsonRpcVersion::V1_0 => serializer.serialize_str("1.0"),
            JsonRpcVersion::V2_0 => serializer.serialize_str("2.0"),
        }
    }
}

impl<'de> Deserialize<'de> for JsonRpcVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "1.0" => Ok(JsonRpcVersion::V1_0),
            "2.0" => Ok(JsonRpcVersion::V2_0),
            _ => Err(serde::de::Error::custom("invalid JSON-RPC version")),
        }
    }
}

impl FromStr for JsonRpcVersion {
    type Err = RpcError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1.0" => Ok(JsonRpcVersion::V1_0),
            "2.0" => Ok(JsonRpcVersion::V2_0),
            _ => Err(RpcError::InvalidJsonRpcVersion(s.to_string())),
        }
    }
}

/// A standard JSON-RPC request object.
///
/// `T` represents the type of the `params` field, which is typically a collection
/// (like a `Vec` or a `Tuple`) or a named object (struct).
#[derive(Debug, Serialize)]
pub struct JsonRpcRequest<T> {
    /// The version of the JSON-RPC protocol.
    pub jsonrpc: JsonRpcVersion,
    /// A string containing the name of the method to be invoked.
    pub method: String,
    /// A structured value that holds the parameter values to be used during the
    /// invocation of the method. This field is omitted if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<T>,
    /// An identifier established by the client.
    pub id: JsonRpcId,
}

impl<T> JsonRpcRequest<T> {
    /// Creates a new JSON-RPC 1.0 request with no parameters.
    pub fn new_v1(id: JsonRpcId, method: &str) -> Self {
        JsonRpcRequest {
            jsonrpc: JsonRpcVersion::V1_0,
            method: method.to_string(),
            params: None,
            id,
        }
    }

    /// Creates a new JSON-RPC 2.0 request with no parameters.
    pub fn new_v2(id: JsonRpcId, method: &str) -> Self {
        JsonRpcRequest {
            jsonrpc: JsonRpcVersion::V2_0,
            method: method.to_string(),
            params: None,
            id,
        }
    }

    /// Attaches parameters to the request.
    pub fn set_params(&mut self, params: T) {
        self.params = Some(params);
    }
}

impl JsonRpcRequest<Vec<serde_json::Value>> {
    /// Dynamically adds a parameter to a request that uses an array of values.
    ///
    /// This is a convenience method for building positional parameters one by one.
    ///
    /// # Example
    /// ```rust
    /// let mut req = JsonRpcRequest::new_v2(Id::from(1), "add");
    /// req.add_param(10);
    /// req.add_param(20);
    /// ```
    pub fn add_param<P>(&mut self, param: P)
    where
        P: Serialize,
        serde_json::Value: From<P>,
    {
        if self.params.is_none() {
            self.params = Some(vec![serde_json::Value::from(param)]);
            return;
        }
        self.params
            .as_mut()
            .unwrap()
            .push(serde_json::Value::from(param));
    }
}
