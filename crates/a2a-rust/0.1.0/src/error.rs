use std::collections::BTreeMap;

use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::jsonrpc;
use crate::jsonrpc::JsonRpcError;

/// Type URL used for structured `ErrorInfo` entries.
pub const ERROR_INFO_TYPE_URL: &str = "type.googleapis.com/google.rpc.ErrorInfo";
/// Domain used for SDK-generated structured error details.
pub const ERROR_INFO_DOMAIN: &str = "a2a-protocol.org";

/// Structured protocol error detail modeled after `google.rpc.ErrorInfo`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Type URL identifying the detail payload.
    #[serde(rename = "@type", default = "error_info_type_url")]
    pub type_url: String,
    /// Stable machine-readable reason string.
    pub reason: String,
    /// Domain that defined the reason.
    pub domain: String,
    /// Additional structured metadata for the error.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

/// Structured HTTP error payload using RFC 9457-style problem details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProblemDetails {
    /// Stable type URI for the problem kind.
    #[serde(rename = "type")]
    pub type_url: String,
    /// Short human-readable problem title.
    pub title: String,
    /// HTTP status code.
    pub status: u16,
    /// Human-readable error detail message.
    pub detail: String,
    /// Optional stable machine-readable reason string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Optional domain associated with the reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Additional structured problem metadata.
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: BTreeMap<String, Value>,
}

/// Unified error type for A2A protocol, HTTP, and serialization failures.
#[derive(Debug, Error)]
pub enum A2AError {
    /// The requested task identifier does not exist.
    #[error("task not found: {0}")]
    TaskNotFound(String),
    /// The requested task cannot transition to canceled.
    #[error("task not cancelable: {0}")]
    TaskNotCancelable(String),
    /// Push notifications are disabled or unsupported for this agent.
    #[error("push notification not supported: {0}")]
    PushNotificationNotSupported(String),
    /// The requested operation is not implemented.
    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),
    /// The request content type is not supported by the peer.
    #[error("content type not supported: {0}")]
    ContentTypeNotSupported(String),
    /// The remote agent returned an invalid response payload.
    #[error("invalid agent response: {0}")]
    InvalidAgentResponse(String),
    /// Extended agent card retrieval is not configured for the agent.
    #[error("extended agent card not configured: {0}")]
    ExtendedAgentCardNotConfigured(String),
    /// A required extension is not supported by the peer.
    #[error("extension support required: {0}")]
    ExtensionSupportRequired(String),
    /// The peer rejected the requested A2A protocol version.
    #[error("version not supported: {0}")]
    VersionNotSupported(String),
    /// The request body could not be parsed as valid JSON-RPC or JSON.
    #[error("parse error: {0}")]
    ParseError(String),
    /// The request shape is structurally invalid.
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    /// The requested method or route does not exist.
    #[error("method not found: {0}")]
    MethodNotFound(String),
    /// The supplied parameters could not be deserialized or validated.
    #[error("invalid params: {0}")]
    InvalidParams(String),
    /// An internal SDK or server error occurred.
    #[error("internal error: {0}")]
    Internal(String),
    /// JSON serialization or deserialization failed locally.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[cfg(feature = "client")]
    /// The underlying HTTP client returned an error.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
}

impl A2AError {
    /// Return the stable structured reason name for this error.
    pub fn reason(&self) -> &'static str {
        match self {
            Self::TaskNotFound(_) => "TASK_NOT_FOUND",
            Self::TaskNotCancelable(_) => "TASK_NOT_CANCELABLE",
            Self::PushNotificationNotSupported(_) => "PUSH_NOTIFICATION_NOT_SUPPORTED",
            Self::UnsupportedOperation(_) => "UNSUPPORTED_OPERATION",
            Self::ContentTypeNotSupported(_) => "CONTENT_TYPE_NOT_SUPPORTED",
            Self::InvalidAgentResponse(_) => "INVALID_AGENT_RESPONSE",
            Self::ExtendedAgentCardNotConfigured(_) => "EXTENDED_AGENT_CARD_NOT_CONFIGURED",
            Self::ExtensionSupportRequired(_) => "EXTENSION_SUPPORT_REQUIRED",
            Self::VersionNotSupported(_) => "VERSION_NOT_SUPPORTED",
            Self::ParseError(_) => "PARSE_ERROR",
            Self::InvalidRequest(_) => "INVALID_REQUEST",
            Self::MethodNotFound(_) => "METHOD_NOT_FOUND",
            Self::InvalidParams(_) => "INVALID_PARAMS",
            Self::Internal(_) | Self::Serialization(_) => "INTERNAL",
            #[cfg(feature = "client")]
            Self::Http(_) => "HTTP",
        }
    }

    /// Return the JSON-RPC error code associated with this error.
    pub fn code(&self) -> i32 {
        match self {
            Self::TaskNotFound(_) => jsonrpc::TASK_NOT_FOUND,
            Self::TaskNotCancelable(_) => jsonrpc::TASK_NOT_CANCELABLE,
            Self::PushNotificationNotSupported(_) => jsonrpc::PUSH_NOTIFICATION_NOT_SUPPORTED,
            Self::UnsupportedOperation(_) => jsonrpc::UNSUPPORTED_OPERATION,
            Self::ContentTypeNotSupported(_) => jsonrpc::CONTENT_TYPE_NOT_SUPPORTED,
            Self::InvalidAgentResponse(_) => jsonrpc::INVALID_AGENT_RESPONSE,
            Self::ExtendedAgentCardNotConfigured(_) => jsonrpc::EXTENDED_AGENT_CARD_NOT_CONFIGURED,
            Self::ExtensionSupportRequired(_) => jsonrpc::EXTENSION_SUPPORT_REQUIRED,
            Self::VersionNotSupported(_) => jsonrpc::VERSION_NOT_SUPPORTED,
            Self::ParseError(_) => jsonrpc::PARSE_ERROR,
            Self::InvalidRequest(_) => jsonrpc::INVALID_REQUEST,
            Self::MethodNotFound(_) => jsonrpc::METHOD_NOT_FOUND,
            Self::InvalidParams(_) => jsonrpc::INVALID_PARAMS,
            Self::Internal(_) => jsonrpc::INTERNAL_ERROR,
            Self::Serialization(_) => jsonrpc::INTERNAL_ERROR,
            #[cfg(feature = "client")]
            Self::Http(_) => jsonrpc::INTERNAL_ERROR,
        }
    }

    /// Convert this error into a JSON-RPC error object.
    pub fn to_jsonrpc_error(&self) -> JsonRpcError {
        JsonRpcError {
            code: self.code(),
            message: self.to_string(),
            data: Some(
                serde_json::to_value(self.to_error_info()).expect("error details should serialize"),
            ),
        }
    }

    /// Convert this error into an RFC 9457-style HTTP problem payload.
    pub fn to_problem_details(&self) -> ProblemDetails {
        let status_code = self.status_code();
        ProblemDetails {
            type_url: self.problem_type_url().to_owned(),
            title: self.problem_title().to_owned(),
            status: status_code.as_u16(),
            detail: self.to_string(),
            reason: Some(self.reason().to_owned()),
            domain: Some(ERROR_INFO_DOMAIN.to_owned()),
            extensions: self
                .metadata()
                .into_iter()
                .map(|(key, value)| (key, Value::String(value)))
                .collect(),
        }
    }

    /// Return the HTTP status code associated with this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::TaskNotFound(_) => StatusCode::NOT_FOUND,
            Self::TaskNotCancelable(_) => StatusCode::CONFLICT,
            Self::PushNotificationNotSupported(_) => StatusCode::BAD_REQUEST,
            Self::UnsupportedOperation(_) => StatusCode::BAD_REQUEST,
            Self::ContentTypeNotSupported(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::InvalidAgentResponse(_) => StatusCode::BAD_GATEWAY,
            Self::ExtendedAgentCardNotConfigured(_) => StatusCode::BAD_REQUEST,
            Self::ExtensionSupportRequired(_) => StatusCode::BAD_REQUEST,
            Self::VersionNotSupported(_) => StatusCode::BAD_REQUEST,
            Self::ParseError(_) => StatusCode::BAD_REQUEST,
            Self::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            Self::MethodNotFound(_) => StatusCode::NOT_FOUND,
            Self::InvalidParams(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Serialization(_) => StatusCode::INTERNAL_SERVER_ERROR,
            #[cfg(feature = "client")]
            Self::Http(_) => StatusCode::BAD_GATEWAY,
        }
    }

    /// Convert this error into a structured `ErrorInfo` detail.
    pub fn to_error_info(&self) -> ErrorInfo {
        ErrorInfo {
            type_url: error_info_type_url(),
            reason: self.reason().to_owned(),
            domain: ERROR_INFO_DOMAIN.to_owned(),
            metadata: self.metadata(),
        }
    }

    /// Reconstruct an `A2AError` from HTTP problem details when possible.
    pub fn from_problem_details(problem: &ProblemDetails) -> Self {
        let reason = problem
            .reason
            .clone()
            .unwrap_or_else(|| problem_reason(problem.type_url.as_str()).to_owned());
        let info = ErrorInfo {
            type_url: error_info_type_url(),
            reason: reason.clone(),
            domain: problem
                .domain
                .clone()
                .unwrap_or_else(|| ERROR_INFO_DOMAIN.to_owned()),
            metadata: problem
                .extensions
                .iter()
                .filter_map(|(key, value)| match value {
                    Value::String(value) => Some((key.clone(), value.clone())),
                    Value::Number(value) => Some((key.clone(), value.to_string())),
                    Value::Bool(value) => Some((key.clone(), value.to_string())),
                    _ => None,
                })
                .collect(),
        };

        Self::from_error_info(reason_code(reason.as_str()), &problem.detail, Some(&info))
    }

    /// Reconstruct an `A2AError` from structured error details when possible.
    pub fn from_error_info(error_code: i32, message: &str, info: Option<&ErrorInfo>) -> Self {
        let fallback_detail = info
            .and_then(|info| info.metadata.get("detail").cloned())
            .unwrap_or_else(|| message.to_owned());

        let reason = info.map(|info| info.reason.as_str()).unwrap_or("");
        let metadata = info.map(|info| &info.metadata);

        match (error_code, reason) {
            (jsonrpc::TASK_NOT_FOUND, "TASK_NOT_FOUND") => Self::TaskNotFound(
                metadata
                    .and_then(|metadata| metadata.get("taskId").cloned())
                    .unwrap_or(fallback_detail),
            ),
            (jsonrpc::TASK_NOT_CANCELABLE, "TASK_NOT_CANCELABLE") => Self::TaskNotCancelable(
                metadata
                    .and_then(|metadata| metadata.get("taskId").cloned())
                    .unwrap_or(fallback_detail),
            ),
            (jsonrpc::PUSH_NOTIFICATION_NOT_SUPPORTED, _) => {
                Self::PushNotificationNotSupported(fallback_detail)
            }
            (jsonrpc::UNSUPPORTED_OPERATION, _) => Self::UnsupportedOperation(fallback_detail),
            (jsonrpc::CONTENT_TYPE_NOT_SUPPORTED, _) => {
                Self::ContentTypeNotSupported(fallback_detail)
            }
            (jsonrpc::INVALID_AGENT_RESPONSE, _) => Self::InvalidAgentResponse(fallback_detail),
            (jsonrpc::EXTENDED_AGENT_CARD_NOT_CONFIGURED, _) => {
                Self::ExtendedAgentCardNotConfigured(fallback_detail)
            }
            (jsonrpc::EXTENSION_SUPPORT_REQUIRED, _) => {
                Self::ExtensionSupportRequired(fallback_detail)
            }
            (jsonrpc::VERSION_NOT_SUPPORTED, _) => Self::VersionNotSupported(fallback_detail),
            (jsonrpc::PARSE_ERROR, _) => Self::ParseError(fallback_detail),
            (jsonrpc::INVALID_REQUEST, _) => Self::InvalidRequest(fallback_detail),
            (jsonrpc::METHOD_NOT_FOUND, _) => Self::MethodNotFound(fallback_detail),
            (jsonrpc::INVALID_PARAMS, _) => Self::InvalidParams(fallback_detail),
            (jsonrpc::INTERNAL_ERROR, _) => Self::Internal(fallback_detail),
            _ => Self::Internal(fallback_detail),
        }
    }

    fn problem_type_url(&self) -> &'static str {
        match self {
            Self::TaskNotFound(_) => "https://a2a-protocol.org/errors/task-not-found",
            Self::TaskNotCancelable(_) => "https://a2a-protocol.org/errors/task-not-cancelable",
            Self::PushNotificationNotSupported(_) => {
                "https://a2a-protocol.org/errors/push-notification-not-supported"
            }
            Self::UnsupportedOperation(_) => {
                "https://a2a-protocol.org/errors/unsupported-operation"
            }
            Self::ContentTypeNotSupported(_) => {
                "https://a2a-protocol.org/errors/content-type-not-supported"
            }
            Self::InvalidAgentResponse(_) => {
                "https://a2a-protocol.org/errors/invalid-agent-response"
            }
            Self::ExtendedAgentCardNotConfigured(_) => {
                "https://a2a-protocol.org/errors/extended-agent-card-not-configured"
            }
            Self::ExtensionSupportRequired(_) => {
                "https://a2a-protocol.org/errors/extension-support-required"
            }
            Self::VersionNotSupported(_) => "https://a2a-protocol.org/errors/version-not-supported",
            Self::ParseError(_) => "about:blank",
            Self::InvalidRequest(_) => "about:blank",
            Self::MethodNotFound(_) => "about:blank",
            Self::InvalidParams(_) => "about:blank",
            Self::Internal(_) | Self::Serialization(_) => "about:blank",
            #[cfg(feature = "client")]
            Self::Http(_) => "about:blank",
        }
    }

    fn problem_title(&self) -> &'static str {
        match self {
            Self::TaskNotFound(_) => "Task not found",
            Self::TaskNotCancelable(_) => "Task not cancelable",
            Self::PushNotificationNotSupported(_) => "Push notifications not supported",
            Self::UnsupportedOperation(_) => "Unsupported operation",
            Self::ContentTypeNotSupported(_) => "Content type not supported",
            Self::InvalidAgentResponse(_) => "Invalid agent response",
            Self::ExtendedAgentCardNotConfigured(_) => "Extended agent card not configured",
            Self::ExtensionSupportRequired(_) => "Extension support required",
            Self::VersionNotSupported(_) => "Version not supported",
            Self::ParseError(_) => "Bad Request",
            Self::InvalidRequest(_) => "Bad Request",
            Self::MethodNotFound(_) => "Not Found",
            Self::InvalidParams(_) => "Bad Request",
            Self::Internal(_) | Self::Serialization(_) => "Internal Server Error",
            #[cfg(feature = "client")]
            Self::Http(_) => "Bad Gateway",
        }
    }

    fn metadata(&self) -> BTreeMap<String, String> {
        let mut metadata = BTreeMap::new();

        match self {
            Self::TaskNotFound(task_id) | Self::TaskNotCancelable(task_id) => {
                metadata.insert("taskId".to_owned(), task_id.clone());
            }
            Self::PushNotificationNotSupported(detail)
            | Self::UnsupportedOperation(detail)
            | Self::ContentTypeNotSupported(detail)
            | Self::InvalidAgentResponse(detail)
            | Self::ExtendedAgentCardNotConfigured(detail)
            | Self::ExtensionSupportRequired(detail)
            | Self::VersionNotSupported(detail)
            | Self::ParseError(detail)
            | Self::InvalidRequest(detail)
            | Self::MethodNotFound(detail)
            | Self::InvalidParams(detail)
            | Self::Internal(detail) => {
                metadata.insert("detail".to_owned(), detail.clone());
            }
            Self::Serialization(error) => {
                metadata.insert("detail".to_owned(), error.to_string());
            }
            #[cfg(feature = "client")]
            Self::Http(error) => {
                metadata.insert("detail".to_owned(), error.to_string());
            }
        }

        metadata
    }
}

impl ProblemDetails {
    /// Convert this HTTP error payload back into an `A2AError`.
    pub fn to_a2a_error(&self) -> A2AError {
        A2AError::from_problem_details(self)
    }
}

impl JsonRpcError {
    /// Return the first structured `ErrorInfo` entry from `data`, when present.
    pub fn first_error_info(&self) -> Option<ErrorInfo> {
        match self.data.as_ref()? {
            Value::Array(details) => details
                .iter()
                .find_map(|detail| serde_json::from_value::<ErrorInfo>(detail.clone()).ok()),
            Value::Object(_) => serde_json::from_value::<ErrorInfo>(self.data.clone()?).ok(),
            _ => None,
        }
    }
}

fn error_info_type_url() -> String {
    ERROR_INFO_TYPE_URL.to_owned()
}

fn problem_code(type_url: &str) -> i32 {
    match type_url {
        "https://a2a-protocol.org/errors/task-not-found" => jsonrpc::TASK_NOT_FOUND,
        "https://a2a-protocol.org/errors/task-not-cancelable" => jsonrpc::TASK_NOT_CANCELABLE,
        "https://a2a-protocol.org/errors/push-notification-not-supported" => {
            jsonrpc::PUSH_NOTIFICATION_NOT_SUPPORTED
        }
        "https://a2a-protocol.org/errors/unsupported-operation" => jsonrpc::UNSUPPORTED_OPERATION,
        "https://a2a-protocol.org/errors/content-type-not-supported" => {
            jsonrpc::CONTENT_TYPE_NOT_SUPPORTED
        }
        "https://a2a-protocol.org/errors/invalid-agent-response" => jsonrpc::INVALID_AGENT_RESPONSE,
        "https://a2a-protocol.org/errors/extended-agent-card-not-configured" => {
            jsonrpc::EXTENDED_AGENT_CARD_NOT_CONFIGURED
        }
        "https://a2a-protocol.org/errors/extension-support-required" => {
            jsonrpc::EXTENSION_SUPPORT_REQUIRED
        }
        "https://a2a-protocol.org/errors/version-not-supported" => jsonrpc::VERSION_NOT_SUPPORTED,
        _ => jsonrpc::INTERNAL_ERROR,
    }
}

fn problem_reason(type_url: &str) -> &'static str {
    match problem_code(type_url) {
        jsonrpc::TASK_NOT_FOUND => "TASK_NOT_FOUND",
        jsonrpc::TASK_NOT_CANCELABLE => "TASK_NOT_CANCELABLE",
        jsonrpc::PUSH_NOTIFICATION_NOT_SUPPORTED => "PUSH_NOTIFICATION_NOT_SUPPORTED",
        jsonrpc::UNSUPPORTED_OPERATION => "UNSUPPORTED_OPERATION",
        jsonrpc::CONTENT_TYPE_NOT_SUPPORTED => "CONTENT_TYPE_NOT_SUPPORTED",
        jsonrpc::INVALID_AGENT_RESPONSE => "INVALID_AGENT_RESPONSE",
        jsonrpc::EXTENDED_AGENT_CARD_NOT_CONFIGURED => "EXTENDED_AGENT_CARD_NOT_CONFIGURED",
        jsonrpc::EXTENSION_SUPPORT_REQUIRED => "EXTENSION_SUPPORT_REQUIRED",
        jsonrpc::VERSION_NOT_SUPPORTED => "VERSION_NOT_SUPPORTED",
        jsonrpc::PARSE_ERROR => "PARSE_ERROR",
        jsonrpc::INVALID_REQUEST => "INVALID_REQUEST",
        jsonrpc::METHOD_NOT_FOUND => "METHOD_NOT_FOUND",
        jsonrpc::INVALID_PARAMS => "INVALID_PARAMS",
        _ => "INTERNAL",
    }
}

fn reason_code(reason: &str) -> i32 {
    match reason {
        "TASK_NOT_FOUND" => jsonrpc::TASK_NOT_FOUND,
        "TASK_NOT_CANCELABLE" => jsonrpc::TASK_NOT_CANCELABLE,
        "PUSH_NOTIFICATION_NOT_SUPPORTED" => jsonrpc::PUSH_NOTIFICATION_NOT_SUPPORTED,
        "UNSUPPORTED_OPERATION" => jsonrpc::UNSUPPORTED_OPERATION,
        "CONTENT_TYPE_NOT_SUPPORTED" => jsonrpc::CONTENT_TYPE_NOT_SUPPORTED,
        "INVALID_AGENT_RESPONSE" => jsonrpc::INVALID_AGENT_RESPONSE,
        "EXTENDED_AGENT_CARD_NOT_CONFIGURED" => jsonrpc::EXTENDED_AGENT_CARD_NOT_CONFIGURED,
        "EXTENSION_SUPPORT_REQUIRED" => jsonrpc::EXTENSION_SUPPORT_REQUIRED,
        "VERSION_NOT_SUPPORTED" => jsonrpc::VERSION_NOT_SUPPORTED,
        "PARSE_ERROR" => jsonrpc::PARSE_ERROR,
        "INVALID_REQUEST" => jsonrpc::INVALID_REQUEST,
        "METHOD_NOT_FOUND" => jsonrpc::METHOD_NOT_FOUND,
        "INVALID_PARAMS" => jsonrpc::INVALID_PARAMS,
        _ => jsonrpc::INTERNAL_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use super::{A2AError, ERROR_INFO_DOMAIN, ERROR_INFO_TYPE_URL};

    #[test]
    fn jsonrpc_error_uses_structured_error_info_object() {
        let error = A2AError::TaskNotFound("task-1".to_owned()).to_jsonrpc_error();

        assert_eq!(error.code, crate::jsonrpc::TASK_NOT_FOUND);
        assert_eq!(
            error.data,
            Some(serde_json::json!({
                "@type": ERROR_INFO_TYPE_URL,
                "reason": "TASK_NOT_FOUND",
                "domain": ERROR_INFO_DOMAIN,
                "metadata": {
                    "taskId": "task-1",
                }
            }))
        );
    }

    #[test]
    fn problem_details_round_trip_to_a2a_error() {
        let error = A2AError::ExtensionSupportRequired("missing extension".to_owned());
        let problem = error.to_problem_details();

        assert_eq!(
            A2AError::from_problem_details(&problem).to_string(),
            error.to_string()
        );
    }
}
