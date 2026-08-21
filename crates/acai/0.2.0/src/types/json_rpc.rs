use std::collections::HashMap;

///////////////////////////////////////////// constants ////////////////////////////////////////////

pub fn default_jsonrpc() -> String {
    "2.0".to_string()
}

/////////////////////////////////////////// JsonRpcError ///////////////////////////////////////////

/// # JSON-RPC Error
///
/// Represents an error in a JSON-RPC response.
///
/// ## Example
/// ```json
/// {
///   "code": -32602,
///   "message": "Invalid parameters",
///   "data": {
///     "details": "The 'id' parameter is required"
///   }
/// }
/// ```
///
/// ## Standard Error Codes
/// - `-32700`: Parse error - Invalid JSON payload
/// - `-32600`: Invalid Request - Request payload validation error
/// - `-32601`: Method Not Found - Requested method does not exist
/// - `-32602`: Invalid Parameters - Invalid method parameters
/// - `-32603`: Internal Error - Internal JSON-RPC error
///
/// ## Custom Error Codes
/// - `-32001`: Task Not Found - The requested task wasn't found
/// - `-32002`: Task Cannot Be Canceled - The task cannot be canceled
/// - `-32003`: Push Notification Error - Push notification is not supported or error occurred
/// - `-32004`: Operation Not Supported - The requested operation is not supported
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, serde_json::Value>>,
}

impl JsonRpcError {
    /// Creates a parse error (code -32700)
    /// Use this for JSON parsing errors or other syntax errors
    pub fn parse_error<T: std::fmt::Display>(error: T) -> Self {
        Self {
            code: -32700,
            message: format!("Parse error: {}", error),
            data: None,
        }
    }

    /// Creates an invalid request error (code -32600)
    /// Use this when the JSON-RPC request doesn't follow protocol requirements
    pub fn invalid_request<T: std::fmt::Display>(error: T) -> Self {
        Self {
            code: -32600,
            message: format!("Invalid request: {}", error),
            data: None,
        }
    }

    /// Creates a method not found error (code -32601)
    /// Use this when the requested method does not exist
    pub fn method_not_found<T: std::fmt::Display>(method: T) -> Self {
        Self {
            code: -32601,
            message: format!("Method '{}' not found", method),
            data: None,
        }
    }

    /// Creates an invalid parameters error (code -32602)
    /// Use this when method parameters are invalid
    pub fn invalid_parameters<T: std::fmt::Display>(error: T) -> Self {
        Self {
            code: -32602,
            message: format!("Invalid parameters: {}", error),
            data: None,
        }
    }

    /// Creates an internal error (code -32603)
    /// Use this for general internal JSON-RPC errors
    pub fn internal_error<T: std::fmt::Display>(error: T) -> Self {
        Self {
            code: -32603,
            message: format!("Internal error: {}", error),
            data: None,
        }
    }

    /// Creates a serialization error (code -32603)
    /// Use this for errors during JSON serialization
    pub fn serialization<T: std::fmt::Display>(error: T) -> Self {
        Self {
            code: -32603,
            message: format!("Serialization error: {}", error),
            data: None,
        }
    }

    /// Creates a task error (code -32603)
    /// Use this for general task processing errors
    pub fn task<T: std::fmt::Display>(error: T) -> Self {
        Self {
            code: -32603,
            message: format!("Task error: {}", error),
            data: None,
        }
    }

    /// Creates a task not found error (code -32001)
    /// Use this when a requested task is not found
    pub fn task_not_found<T: std::fmt::Display>(task_id: T) -> Self {
        Self {
            code: -32001,
            message: format!("Task not found: {}", task_id),
            data: None,
        }
    }

    /// Creates a task cannot be canceled error (code -32002)
    /// Use this when a task cannot be canceled
    pub fn task_cannot_be_canceled<T: std::fmt::Display>(task_id: T) -> Self {
        Self {
            code: -32002,
            message: format!("Task cannot be canceled: {}", task_id),
            data: None,
        }
    }

    /// Creates a push notification error (code -32003)
    /// Use this for push notification errors
    pub fn push_notification<T: std::fmt::Display>(error: T) -> Self {
        Self {
            code: -32003,
            message: format!("Push notification error: {}", error),
            data: None,
        }
    }

    /// Creates a task not cancelable error (code -32002)
    /// Use this when a task is in a final state and cannot be canceled
    pub fn task_not_cancelable<T: std::fmt::Display>(error: T) -> Self {
        Self {
            code: -32002,
            message: format!("Task not cancelable: {}", error),
            data: None,
        }
    }

    /// Creates an unsupported operation error (code -32004)
    /// Use this when the requested operation is not supported
    pub fn unsupported_operation<T: std::fmt::Display>(error: T) -> Self {
        Self {
            code: -32004,
            message: format!("Unsupported operation: {}", error),
            data: None,
        }
    }

    /// Creates a content type not supported error (code -32005)
    /// Use this when there's a mismatch in supported content types
    pub fn content_type_not_supported<T: std::fmt::Display>(error: T) -> Self {
        Self {
            code: -32005,
            message: format!("Content type not supported: {}", error),
            data: None,
        }
    }

    /// Creates an operation not supported error (code -32004)
    /// Use this when a requested operation is not supported
    pub fn operation_not_supported<T: std::fmt::Display>(operation: T) -> Self {
        Self {
            code: -32004,
            message: format!("Operation not supported: {}", operation),
            data: None,
        }
    }
}

impl From<serde_json::Error> for JsonRpcError {
    fn from(error: serde_json::Error) -> Self {
        JsonRpcError::parse_error(error)
    }
}

////////////////////////////////////////// JsonRpcRequest //////////////////////////////////////////

/// # JSON-RPC Request
///
/// A JSON-RPC request message.
///
/// ## Example
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "id": "request-1",
///   "method": "tasks/get",
///   "params": {
///     "id": "task_01H9FHHSCN8Y5FVFQP66K2330K"
///   }
/// }
/// ```
///
/// # Generic Request Type
///
/// A generic request type that encapsulates common request structure.
/// Used for all JSON-RPC requests in the A2A protocol.
///
/// ## Usage Examples
///
/// The `JsonRpcRequest<P>` type provides two main ways to create requests:
///
/// ```
///
/// # use acai::{JsonRpcRequest, TaskIdParams};
/// let params = TaskIdParams { id: "task_123".to_string(), metadata: None };
/// // Method 1: Using the generic `new` method with a String method
/// let request1 = JsonRpcRequest::new(serde_json::json!("req-1"), "tasks/cancel", params.clone());
///
/// // Method 2: Using specialized methods on parameter types
/// let request2 = params.into_cancel_request(serde_json::json!("req-2"));
/// ```
///
/// ## JSON Example
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "id": "request-1",
///   "method": "tasks/send",
///   "params": { ... }
/// }
/// ```
///
/// ## Common A2A Request Methods
///
/// The A2A protocol defines the following standard request methods:
///
/// - `"tasks/send"` - Send a message to an agent for a specific task (params: `TaskSendParams`)
/// - `"tasks/get"` - Get information about a specific task (params: `TaskQueryParams`)
/// - `"tasks/cancel"` - Cancel a specific task (params: `TaskIdParams`)
/// - `"tasks/pushNotification/get"` - Get push notification config (params: `TaskIdParams`)
/// - `"tasks/pushNotification/set"` - Set push notification config (params: `TaskPushNotificationConfig`)
/// - `"tasks/sendSubscribe"` - Send a message with streaming response (params: `TaskSendParams`)
/// - `"tasks/resubscribe"` - Resubscribe to streaming updates (params: `TaskQueryParams`)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JsonRpcRequest<P> {
    #[serde(default = "default_jsonrpc")]
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: P,
}

impl<P> JsonRpcRequest<P> {
    /// Create a new request with the given method and parameters.
    ///
    /// # Example
    ///
    /// ```
    /// # use acai::{JsonRpcRequest, TaskIdParams};
    /// let params = TaskIdParams { id: "task_123".to_string(), metadata: None };
    /// let request = JsonRpcRequest::new(serde_json::json!("request-1"), "tasks/cancel".to_string(), params);
    /// ```
    pub fn new(id: serde_json::Value, method: impl Into<String>, params: P) -> Self {
        Self {
            jsonrpc: default_jsonrpc(),
            id: Some(id),
            method: method.into(),
            params,
        }
    }

    // Not providing a separate with_params method to align with CLAUDE.md guidance:
    // "Use type-specialized constructors instead of wrapper functions that just rearrange arguments"

    /// Get the method name
    pub fn method(&self) -> &str {
        &self.method
    }

    /// Get the ID
    pub fn id(&self) -> Option<&serde_json::Value> {
        self.id.as_ref()
    }

    /// Get the params
    pub fn params(&self) -> &P {
        &self.params
    }
}

////////////////////////////////////////// JsonRpcResponse /////////////////////////////////////////

/// # Generic response for any JSON-RPC method
///
/// A generic response type to be used for A2A protocol responses.
/// This allows strongly typed results.
///
/// ## Example
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "id": "request-1",
///   "result": { "value": 42 }
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound(
    serialize = "R: serde::Serialize",
    deserialize = "R: serde::de::DeserializeOwned"
))]
pub struct JsonRpcResponse<R = serde_json::Value> {
    #[serde(default = "default_jsonrpc")]
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<R>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl<R> JsonRpcResponse<R> {
    /// Create a new success response
    pub fn success(id: serde_json::Value, result: R) -> Self {
        Self {
            jsonrpc: default_jsonrpc(),
            id: Some(id),
            result: Some(result),
            error: None,
        }
    }

    /// Create a new error response with required ID
    pub fn error(id: serde_json::Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: default_jsonrpc(),
            id: Some(id),
            result: None,
            error: Some(error),
        }
    }

    /// Create a new error response with optional ID
    pub fn error_with_optional_id(id: Option<serde_json::Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: default_jsonrpc(),
            id,
            result: None,
            error: Some(error),
        }
    }

    /// Get the result
    pub fn result(&self) -> Option<&R> {
        self.result.as_ref()
    }

    /// Get the error
    pub fn get_error(&self) -> Option<&JsonRpcError> {
        self.error.as_ref()
    }

    /// Get the ID
    pub fn id(&self) -> Option<&serde_json::Value> {
        self.id.as_ref()
    }
}

/////////////////////////////////////////////// tests //////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::types::*;

    use super::*;

    #[test]
    fn request_new() {
        // Test the generic new method with a String method
        let params = TaskIdParams {
            id: "task_123".to_string(),
            metadata: None,
        };

        let request = JsonRpcRequest::new(
            serde_json::json!("request-1"),
            "tasks/cancel".to_string(),
            params,
        );

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tasks/cancel");
        assert_eq!(request.id, Some(serde_json::json!("request-1")));
    }

    #[test]
    fn request_with_id() {
        // Test adding an ID to a request
        let params = TaskIdParams {
            id: "task_123".to_string(),
            metadata: None,
        };

        let request = JsonRpcRequest::new(
            serde_json::json!("request-123"),
            "tasks/cancel".to_string(),
            params,
        );

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tasks/cancel");
        assert_eq!(
            request.id,
            Some(serde_json::Value::String("request-123".to_string()))
        );
    }

    #[test]
    fn request_send_task() {
        // Test the into_send_request method
        let message = Message {
            role: MessageRole::User,
            parts: vec![Part::Text {
                text: "Hello".to_string(),
                metadata: None,
            }],
            metadata: None,
        };

        let params = TaskSendParams {
            id: "task_123".to_string(),
            message,
            session_id: None,
            push_notification: None,
            history_length: None,
            metadata: None,
        };

        let request = params.into_send_request(serde_json::json!("request-id"));

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tasks/send");
        assert_eq!(request.id, Some(serde_json::json!("request-id")));
    }

    #[test]
    fn request_cancel_task() {
        // Test the into_cancel_request method
        let params = TaskIdParams {
            id: "task_123".to_string(),
            metadata: None,
        };

        let request = params.into_cancel_request(serde_json::json!("request-id"));

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tasks/cancel");
        assert_eq!(request.id, Some(serde_json::json!("request-id")));
    }

    #[test]
    fn request_get_task() {
        // Test the into_get_request method
        let params = TaskQueryParams {
            id: "task_123".to_string(),
            history_length: Some(5),
            metadata: None,
        };

        let request = params.into_get_request(serde_json::json!("request-id"));

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tasks/get");
        assert_eq!(request.id, Some(serde_json::json!("request-id")));
    }

    #[test]
    fn request_get_task_push_notification() {
        // Test the into_push_notification_get_request method
        let params = TaskIdParams {
            id: "task_123".to_string(),
            metadata: None,
        };

        let request = params.into_push_notification_get_request(serde_json::json!("request-id"));

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tasks/pushNotification/get");
        assert_eq!(request.id, Some(serde_json::json!("request-id")));
    }

    #[test]
    fn request_set_task_push_notification() {
        // Test the into_push_notification_set_request method
        let config = PushNotificationConfig {
            url: "https://example.com/webhook".to_string(),
            token: Some("secret-token".to_string()),
            authentication: None,
        };

        let params = TaskPushNotificationConfig {
            id: "task_123".to_string(),
            push_notification_config: config,
        };

        let request = params.into_push_notification_set_request(serde_json::json!("request-id"));

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tasks/pushNotification/set");
        assert_eq!(request.id, Some(serde_json::json!("request-id")));
    }

    #[test]
    fn request_send_subscribe_task() {
        // Test the into_send_subscribe_request method
        let message = Message {
            role: MessageRole::User,
            parts: vec![Part::Text {
                text: "Generate some code".to_string(),
                metadata: None,
            }],
            metadata: None,
        };

        let params = TaskSendParams {
            id: "task_123".to_string(),
            message,
            session_id: None,
            push_notification: None,
            history_length: None,
            metadata: None,
        };

        let request = params.into_send_subscribe_request(serde_json::json!("request-id"));

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tasks/sendSubscribe");
        assert_eq!(request.id, Some(serde_json::json!("request-id")));
    }

    #[test]
    fn request_resubscribe_task() {
        // Test the into_resubscribe_request method
        let params = TaskQueryParams {
            id: "task_123".to_string(),
            history_length: Some(5),
            metadata: None,
        };

        let request = params.into_resubscribe_request(serde_json::json!("request-id"));

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "tasks/resubscribe");
        assert_eq!(request.id, Some(serde_json::json!("request-id")));
    }

    #[test]
    fn request_serialization() {
        // Test that a request serializes to the expected JSON
        let params = TaskIdParams {
            id: "task_123".to_string(),
            metadata: None,
        };

        let request = params.into_cancel_request(serde_json::json!("request-123"));

        let json = serde_json::to_string(&request).unwrap();
        let expected = r#"{"jsonrpc":"2.0","id":"request-123","method":"tasks/cancel","params":{"id":"task_123"}}"#;

        assert_eq!(json, expected);

        // Test deserialization as well
        let deserialized: JsonRpcRequest<TaskIdParams> = serde_json::from_str(expected).unwrap();
        assert_eq!(deserialized.jsonrpc, "2.0");
        assert_eq!(deserialized.method, "tasks/cancel");
        assert_eq!(
            deserialized.id,
            Some(serde_json::Value::String("request-123".to_string()))
        );
        assert_eq!(deserialized.params.id, "task_123");
    }

    // Tests for JsonRpcResponse

    #[test]
    fn response_success() {
        // Test creating a success response with JsonRpcResponse::success
        let task = Task {
            id: "task_123".to_string(),
            session_id: None,
            status: TaskStatus {
                state: TaskState::Completed,
                message: None,
                timestamp: None,
            },
            artifacts: None,
            history: None,
            metadata: None,
        };

        let response = JsonRpcResponse::success("request-123".into(), task.clone());

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(
            response.id,
            Some(serde_json::Value::String("request-123".to_string()))
        );
        assert_eq!(response.result().unwrap().id, task.id);
        assert!(response.error.is_none());
    }

    #[test]
    fn response_error() {
        // Test creating an error response with JsonRpcResponse::error
        let error = JsonRpcError::task_not_found("task_123");

        let response = JsonRpcResponse::<Task>::error("request-123".into(), error.clone());

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(
            response.id,
            Some(serde_json::Value::String("request-123".to_string()))
        );
        assert!(response.result.is_none());
        assert_eq!(response.get_error().unwrap().code, error.code);
        assert_eq!(response.get_error().unwrap().message, error.message);
    }

    #[test]
    fn response_error_with_optional_id() {
        // Test creating an error response with optional ID
        let error = JsonRpcError::method_not_found("unknown_method");

        // With an ID
        let response = JsonRpcResponse::<()>::error_with_optional_id(
            Some("request-123".into()),
            error.clone(),
        );

        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(
            response.id,
            Some(serde_json::Value::String("request-123".to_string()))
        );
        assert!(response.result.is_none());
        assert_eq!(response.get_error().unwrap().code, error.code);

        // Without an ID
        let response = JsonRpcResponse::<()>::error_with_optional_id(None, error.clone());

        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.id.is_none());
        assert!(response.result.is_none());
        assert_eq!(response.get_error().unwrap().code, error.code);
    }

    #[test]
    fn response_serialization() {
        // Test serialization and deserialization of success response
        let task = Task {
            id: "task_123".to_string(),
            session_id: None,
            status: TaskStatus {
                state: TaskState::Working,
                message: None,
                timestamp: None,
            },
            artifacts: None,
            history: None,
            metadata: None,
        };

        let response = JsonRpcResponse::success("request-123".into(), task);

        let json = serde_json::to_string(&response).unwrap();
        let expected = r#"{"jsonrpc":"2.0","id":"request-123","result":{"id":"task_123","status":{"state":"working"}}}"#;

        assert_eq!(json, expected);

        // Test deserialization
        let deserialized: JsonRpcResponse<Task> = serde_json::from_str(expected).unwrap();
        assert_eq!(deserialized.jsonrpc, "2.0");
        assert_eq!(
            deserialized.id,
            Some(serde_json::Value::String("request-123".to_string()))
        );
        assert_eq!(deserialized.result.unwrap().id, "task_123");
    }

    #[test]
    fn response_error_serialization() {
        // Test serialization and deserialization of error response
        let error = JsonRpcError::parse_error("Invalid JSON");
        let response = JsonRpcResponse::<()>::error("request-123".into(), error);

        let json = serde_json::to_string(&response).unwrap();
        let expected = r#"{"jsonrpc":"2.0","id":"request-123","error":{"code":-32700,"message":"Parse error: Invalid JSON"}}"#;

        assert_eq!(json, expected);

        // Test deserialization
        let deserialized: JsonRpcResponse<()> = serde_json::from_str(expected).unwrap();
        assert_eq!(deserialized.jsonrpc, "2.0");
        assert_eq!(
            deserialized.id,
            Some(serde_json::Value::String("request-123".to_string()))
        );
        assert_eq!(deserialized.get_error().unwrap().code, -32700);
    }
}
