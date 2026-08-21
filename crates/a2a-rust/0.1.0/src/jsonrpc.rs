use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Required JSON-RPC version marker.
pub const JSONRPC_VERSION: &str = "2.0";
/// Required A2A protocol version header value.
pub const PROTOCOL_VERSION: &str = "1.0";

/// JSON-RPC parse error code.
pub const PARSE_ERROR: i32 = -32700;
/// JSON-RPC invalid request error code.
pub const INVALID_REQUEST: i32 = -32600;
/// JSON-RPC method not found error code.
pub const METHOD_NOT_FOUND: i32 = -32601;
/// JSON-RPC invalid params error code.
pub const INVALID_PARAMS: i32 = -32602;
/// JSON-RPC internal error code.
pub const INTERNAL_ERROR: i32 = -32603;

/// A2A task-not-found error code.
pub const TASK_NOT_FOUND: i32 = -32001;
/// A2A task-not-cancelable error code.
pub const TASK_NOT_CANCELABLE: i32 = -32002;
/// A2A push-notifications-not-supported error code.
pub const PUSH_NOTIFICATION_NOT_SUPPORTED: i32 = -32003;
/// A2A unsupported-operation error code.
pub const UNSUPPORTED_OPERATION: i32 = -32004;
/// A2A unsupported-content-type error code.
pub const CONTENT_TYPE_NOT_SUPPORTED: i32 = -32005;
/// A2A invalid-agent-response error code.
pub const INVALID_AGENT_RESPONSE: i32 = -32006;
/// A2A extended-agent-card-not-configured error code.
pub const EXTENDED_AGENT_CARD_NOT_CONFIGURED: i32 = -32007;
/// A2A extension-support-required error code.
pub const EXTENSION_SUPPORT_REQUIRED: i32 = -32008;
/// A2A version-not-supported error code.
pub const VERSION_NOT_SUPPORTED: i32 = -32009;

/// JSON-RPC method name for `SendMessage`.
pub const METHOD_SEND_MESSAGE: &str = "SendMessage";
/// JSON-RPC method name for `SendStreamingMessage`.
pub const METHOD_SEND_STREAMING_MESSAGE: &str = "SendStreamingMessage";
/// JSON-RPC method name for `GetTask`.
pub const METHOD_GET_TASK: &str = "GetTask";
/// JSON-RPC method name for `ListTasks`.
pub const METHOD_LIST_TASKS: &str = "ListTasks";
/// JSON-RPC method name for `CancelTask`.
pub const METHOD_CANCEL_TASK: &str = "CancelTask";
/// JSON-RPC method name for `SubscribeToTask`.
pub const METHOD_SUBSCRIBE_TO_TASK: &str = "SubscribeToTask";
/// JSON-RPC method name for `CreateTaskPushNotificationConfig`.
pub const METHOD_CREATE_TASK_PUSH_NOTIFICATION_CONFIG: &str = "CreateTaskPushNotificationConfig";
/// JSON-RPC method name for `GetTaskPushNotificationConfig`.
pub const METHOD_GET_TASK_PUSH_NOTIFICATION_CONFIG: &str = "GetTaskPushNotificationConfig";
/// JSON-RPC method name for `ListTaskPushNotificationConfigs`.
pub const METHOD_LIST_TASK_PUSH_NOTIFICATION_CONFIGS: &str = "ListTaskPushNotificationConfigs";
/// JSON-RPC method name for `DeleteTaskPushNotificationConfig`.
pub const METHOD_DELETE_TASK_PUSH_NOTIFICATION_CONFIG: &str = "DeleteTaskPushNotificationConfig";
/// JSON-RPC method name for `GetExtendedAgentCard`.
pub const METHOD_GET_EXTENDED_AGENT_CARD: &str = "GetExtendedAgentCard";

/// JSON-RPC 2.0 request envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    #[serde(default = "jsonrpc_version")]
    /// JSON-RPC protocol version, always `"2.0"`.
    pub jsonrpc: String,
    /// Method name to invoke on the remote peer.
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional method parameters encoded as a JSON object.
    pub params: Option<Value>,
    /// Request identifier echoed by the peer in the response.
    pub id: JsonRpcId,
}

/// JSON-RPC 2.0 response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    #[serde(default = "jsonrpc_version")]
    /// JSON-RPC protocol version, always `"2.0"`.
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Successful result payload.
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Error payload returned when the call fails.
    pub error: Option<JsonRpcError>,
    /// Response identifier copied from the request.
    pub id: JsonRpcId,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Numeric error code.
    pub code: i32,
    /// Human-readable error message.
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional protocol-specific error data.
    pub data: Option<Value>,
}

/// Allowed JSON-RPC request/response identifier forms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId {
    /// String request identifier.
    String(String),
    /// Numeric request identifier.
    Number(i64),
    /// Explicit null identifier.
    Null,
}

fn jsonrpc_version() -> String {
    JSONRPC_VERSION.to_owned()
}

#[cfg(test)]
mod tests {
    use super::JsonRpcId;

    #[test]
    fn jsonrpc_id_null_serializes_as_null() {
        let json = serde_json::to_string(&JsonRpcId::Null).expect("id should serialize");
        assert_eq!(json, "null");

        let round_trip: JsonRpcId = serde_json::from_str("null").expect("id should deserialize");
        assert!(matches!(round_trip, JsonRpcId::Null));
    }
}
