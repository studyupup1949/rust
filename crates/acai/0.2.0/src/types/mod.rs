use std::collections::HashMap;
use std::convert::TryFrom;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, de};

mod json_rpc;

pub use json_rpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, default_jsonrpc};

/// # Agent Authentication information
///
/// Specifies the authentication schemes and optional credentials for an agent.
///
/// ## Example
/// ```json
/// {
///   "schemes": ["api_key"],
///   "credentials": "optional-credential-string"
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentAuthentication {
    pub schemes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credentials: Option<String>,
}

/// # Agent Capabilities
///
/// Defines the capabilities supported by an agent.
///
/// ## Example
/// ```json
/// {
///   "streaming": true,
///   "pushNotifications": false,
///   "stateTransitionHistory": true
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentCapabilities {
    #[serde(default)]
    pub streaming: bool,
    #[serde(default, rename = "pushNotifications")]
    pub push_notifications: bool,
    #[serde(default, rename = "stateTransitionHistory")]
    pub state_transition_history: bool,
}

/// # Agent Provider
///
/// Information about the organization that provides the agent.
///
/// ## Example
/// ```json
/// {
///   "organization": "Example Corp",
///   "url": "https://example.com"
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentProvider {
    pub organization: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// # Agent Skill
///
/// Defines a specific capability or skill that an agent provides.
///
/// ## Example
/// ```json
/// {
///   "id": "code-review",
///   "name": "Code Review",
///   "description": "Reviews code for bugs and improvements",
///   "tags": ["programming", "review", "quality"],
///   "examples": ["Review this pull request", "Find bugs in this code"],
///   "inputModes": ["text", "file"],
///   "outputModes": ["text"]
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentSkill {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "inputModes"
    )]
    pub input_modes: Option<Vec<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "outputModes"
    )]
    pub output_modes: Option<Vec<String>>,
}

/// # Agent Card
///
/// Contains metadata about an agent, including its capabilities, skills, and authentication requirements.
///
/// ## Example
/// ```json
/// {
///   "name": "CodeHelper",
///   "description": "An AI assistant for coding tasks",
///   "url": "https://api.example.com/agents/codehelper",
///   "provider": {
///     "organization": "Example Corp",
///     "url": "https://example.com"
///   },
///   "version": "1.0.0",
///   "documentationUrl": "https://docs.example.com/codehelper",
///   "capabilities": {
///     "streaming": true,
///     "pushNotifications": false,
///     "stateTransitionHistory": true
///   },
///   "authentication": {
///     "schemes": ["api_key"]
///   },
///   "defaultInputModes": ["text", "file"],
///   "defaultOutputModes": ["text"],
///   "skills": [
///     {
///       "id": "code-review",
///       "name": "Code Review",
///       "description": "Reviews code for bugs and improvements"
///     }
///   ]
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentCard {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<AgentProvider>,
    pub version: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "documentationUrl"
    )]
    pub documentation_url: Option<String>,
    pub capabilities: AgentCapabilities,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authentication: Option<AgentAuthentication>,
    #[serde(default = "default_input_modes", rename = "defaultInputModes")]
    pub default_input_modes: Vec<String>,
    #[serde(default = "default_output_modes", rename = "defaultOutputModes")]
    pub default_output_modes: Vec<String>,
    pub skills: Vec<AgentSkill>,
}

fn default_input_modes() -> Vec<String> {
    vec!["text".to_string()]
}

fn default_output_modes() -> Vec<String> {
    vec!["text".to_string()]
}

/// # File Content
///
/// Represents the content of a file, either as base64 encoded bytes or a URI.
///
/// ## Example
/// ```json
/// {
///   "name": "example.txt",
///   "mimeType": "text/plain",
///   "bytes": "SGVsbG8gd29ybGQ="
/// }
/// ```
///
/// Or with a URI:
/// ```json
/// {
///   "name": "example.txt",
///   "mimeType": "text/plain",
///   "uri": "https://example.com/files/example.txt"
/// }
/// ```
///
/// Ensures that either 'bytes' or 'uri' is provided, but not both.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "raw::FileContent")]
pub struct FileContent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

impl TryFrom<raw::FileContent> for FileContent {
    type Error = String;

    fn try_from(raw: raw::FileContent) -> Result<Self, Self::Error> {
        match (raw.bytes.is_some(), raw.uri.is_some()) {
            (true, true) => Err("FileContent cannot have both 'bytes' and 'uri' set".to_string()),
            (false, false) => Err("FileContent must have either 'bytes' or 'uri' set".to_string()),
            _ => Ok(FileContent {
                name: raw.name,
                mime_type: raw.mime_type,
                bytes: raw.bytes,
                uri: raw.uri,
            }),
        }
    }
}

/// Raw module for deserializing and validating
mod raw {
    // Explicitly import only what's needed
    use std::option::Option;
    use std::string::String;

    #[derive(Debug, Clone, serde::Deserialize)]
    pub struct FileContent {
        #[serde(default)]
        pub name: Option<String>,
        #[serde(default, rename = "mimeType")]
        pub mime_type: Option<String>,
        #[serde(default)]
        pub bytes: Option<String>,
        #[serde(default)]
        pub uri: Option<String>,
    }
}

// Helper functions for deserializing potentially malformed timestamps
fn deserialize_timestamp_option<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    // First try to deserialize as a string, then parse it
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            // Try to normalize the timestamp format
            let normalized = normalize_timestamp(&s);
            match DateTime::parse_from_rfc3339(&normalized) {
                Ok(dt) => Ok(Some(dt.with_timezone(&Utc))),
                Err(_) => {
                    // Try without normalization as fallback
                    match DateTime::parse_from_rfc3339(&s) {
                        Ok(dt) => Ok(Some(dt.with_timezone(&Utc))),
                        Err(_) => {
                            // If all parsing attempts fail, return an error
                            Err(de::Error::custom(format!(
                                "Failed to parse timestamp: {}",
                                s
                            )))
                        }
                    }
                }
            }
        }
        None => Ok(None),
    }
}

// Helper to normalize timestamp formats to RFC3339
fn normalize_timestamp(ts: &str) -> String {
    // Remove trailing Z if present
    let ts = ts.trim_end_matches('Z');

    // Split by decimal point to handle microseconds
    let parts: Vec<&str> = ts.split('.').collect();

    if parts.len() == 1 {
        // No fractional part, add .000000Z
        return format!("{}.000000Z", parts[0]);
    } else if parts.len() == 2 {
        // Has fractional part, ensure it's 6 digits (microseconds)
        let base = parts[0];
        let mut micros = parts[1].to_string();

        // Pad or truncate to exactly 6 digits
        match micros.len().cmp(&6) {
            std::cmp::Ordering::Less => {
                // Pad with zeros
                micros = format!("{:0<6}", micros);
            }
            std::cmp::Ordering::Greater => {
                // Truncate to 6 digits
                micros = micros[..6].to_string();
            }
            std::cmp::Ordering::Equal => {}
        }

        return format!("{}.{}Z", base, micros);
    }

    // If the format is unexpected, return with Z appended
    format!("{}Z", ts)
}

/// # Message Part
///
/// Represents different types of content that can be included in a message.
///
/// ## Examples
///
/// ### Text Part
/// ```json
/// {
///   "type": "text",
///   "text": "This is a text message",
///   "metadata": {
///     "format": "markdown"
///   }
/// }
/// ```
///
/// ### File Part
/// ```json
/// {
///   "type": "file",
///   "file": {
///     "name": "example.txt",
///     "mimeType": "text/plain",
///     "bytes": "SGVsbG8gd29ybGQ="
///   },
///   "metadata": {
///     "description": "An example file"
///   }
/// }
/// ```
///
/// ### Data Part
/// ```json
/// {
///   "type": "data",
///   "data": {
///     "type": "chart",
///     "values": [1, 2, 3, 4, 5],
///     "labels": ["A", "B", "C", "D", "E"]
///   },
///   "metadata": {
///     "format": "bar-chart"
///   }
/// }
/// ```
/// # Form Field Definition
///
/// Represents a field in a form, with properties like type, validation, and UI hints.
///
/// ## Example
/// ```json
/// {
///   "title": "Full Name",
///   "format": "text",
///   "required": true,
///   "default": "John Doe",
///   "description": "Please enter your full legal name"
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FormField {
    /// Field title/label shown to users
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Field format/input type (text, email, number, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// Whether this field is required
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub required: bool,

    /// Default value for the field
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    /// Field description or help text
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Validation rules or constraints
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<HashMap<String, serde_json::Value>>,

    /// Additional field properties
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// # Form Schema
///
/// Defines a structured form with properties and validation rules.
///
/// ## Example
/// ```json
/// {
///   "properties": {
///     "name": {
///       "title": "Full Name",
///       "format": "text",
///       "required": true
///     },
///     "email": {
///       "title": "Email Address",
///       "format": "email"
///     }
///   },
///   "required": ["name", "email"]
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FormSchema {
    /// Form field definitions
    pub properties: HashMap<String, FormField>,

    /// List of required field names
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,

    /// Additional schema properties
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Part {
    #[serde(rename = "text")]
    Text {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
    #[serde(rename = "file")]
    File {
        file: FileContent,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
    #[serde(rename = "data")]
    Data {
        data: HashMap<String, serde_json::Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
    },
}

/// # Message Role
///
/// Defines the role of the sender of a message.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Agent, // "Agent" is new way to say "Assistant"
    System,
}

/// # Message
///
/// Represents a message in a conversation between a user and an agent.
///
/// ## Example
/// ```json
/// {
///   "role": "user",
///   "parts": [
///     {
///       "type": "text",
///       "text": "Hello, I need help with my code"
///     },
///     {
///       "type": "file",
///       "file": {
///         "name": "code.js",
///         "mimeType": "application/javascript",
///         "bytes": "Y29uc3QgaGVsbG8gPSAoKSA9PiAiSGVsbG8sIFdvcmxkISI7"
///       }
///     }
///   ],
///   "metadata": {
///     "timestamp": "2023-10-25T15:30:00Z"
///   }
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub parts: Vec<Part>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// # Artifact
///
/// Represents a file or piece of content generated by an agent.
///
/// ## Example
/// ```json
/// {
///   "name": "Generated Code",
///   "description": "JavaScript function to calculate fibonacci numbers",
///   "parts": [
///     {
///       "type": "text",
///       "text": "function fibonacci(n) {\n  if (n <= 1) return n;\n  return fibonacci(n-1) + fibonacci(n-2);\n}"
///     }
///   ],
///   "index": 0,
///   "append": false,
///   "lastChunk": true,
///   "metadata": {
///     "language": "javascript"
///   }
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Artifact {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parts: Vec<Part>,
    #[serde(default)]
    pub index: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub append: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "lastChunk")]
    pub last_chunk: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// # Authentication Info
///
/// Authentication information for an API endpoint.
///
/// ## Example
/// ```json
/// {
///   "schemes": ["bearer"],
///   "credentials": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
///   "additional_property": "custom value"
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthenticationInfo {
    pub schemes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credentials: Option<String>,
    #[serde(flatten)]
    pub additional_properties: HashMap<String, serde_json::Value>,
}

/// # Push Notification Configuration
///
/// Configuration for receiving push notifications about task status changes.
///
/// ## Example
/// ```json
/// {
///   "url": "https://example.com/webhook",
///   "token": "secret-webhook-token",
///   "authentication": {
///     "schemes": ["bearer"],
///     "credentials": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
///   }
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PushNotificationConfig {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authentication: Option<AuthenticationInfo>,
}

/// # Task State
///
/// The current state of a task in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskState {
    /// Task has been submitted but processing hasn't started
    Submitted,
    /// Task is being processed
    Working,
    /// Task requires additional input to continue
    InputRequired,
    /// Task has been successfully completed
    Completed,
    /// Task was canceled before completion
    Canceled,
    /// Task processing encountered an error and failed
    Failed,
    /// Task is in an unknown state
    Unknown,
}

/// # Task Status
///
/// Information about the current status of a task.
///
/// ## Example
/// ```json
/// {
///   "state": "working",
///   "message": {
///     "role": "agent",
///     "parts": [
///       {
///         "type": "text",
///         "text": "I'm analyzing your code..."
///       }
///     ]
///   },
///   "timestamp": "2023-10-25T15:32:10Z"
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskStatus {
    pub state: TaskState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_timestamp_option"
    )]
    pub timestamp: Option<DateTime<Utc>>,
}

///
/// # Task
///
/// Represents a task being processed by an agent.
///
/// ## Example
/// ```json
/// {
///   "id": "task_01H9FHHSCN8Y5FVFQP66K2330K",
///   "sessionId": "session_01H9FH8N3EHY3YF6K56JCFJ33F",
///   "status": {
///     "state": "working",
///     "timestamp": "2023-10-25T15:32:10Z"
///   },
///   "artifacts": [
///     {
///       "name": "Analysis Results",
///       "parts": [
///         {
///           "type": "text",
///           "text": "Here are the issues I found in your code..."
///         }
///       ],
///       "index": 0
///     }
///   ],
///   "history": [
///     {
///       "role": "user",
///       "parts": [
///         {
///           "type": "text",
///           "text": "Please review my code"
///         }
///       ]
///     },
///     {
///       "role": "agent",
///       "parts": [
///         {
///           "type": "text",
///           "text": "I'll analyze your code now"
///         }
///       ]
///     }
///   ],
///   "metadata": {
///     "priority": "high"
///   }
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "sessionId",
        alias = "sessionId"
    )]
    pub session_id: Option<String>,
    pub status: TaskStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<Artifact>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<Message>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// # Task Push Notification Configuration
///
/// Configuration for push notifications for a specific task.
///
/// ## Example
/// ```json
/// {
///   "id": "task_01H9FHHSCN8Y5FVFQP66K2330K",
///   "pushNotificationConfig": {
///     "url": "https://example.com/webhook",
///     "token": "secret-webhook-token",
///     "authentication": {
///       "schemes": ["bearer"],
///       "credentials": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
///     }
///   }
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskPushNotificationConfig {
    pub id: String,
    #[serde(rename = "pushNotificationConfig")]
    pub push_notification_config: PushNotificationConfig,
}

impl TaskPushNotificationConfig {
    /// Convert parameters into a tasks/pushNotification/set request
    ///
    /// # Example
    ///
    /// ```
    /// # use acai::{TaskPushNotificationConfig, PushNotificationConfig, JsonRpcRequest};
    /// let push_config = PushNotificationConfig {
    ///     url: "https://example.com/webhook".to_string(),
    ///     token: Some("secret-token".to_string()),
    ///     authentication: None,
    /// };
    /// let params = TaskPushNotificationConfig {
    ///     id: "task_123".to_string(),
    ///     push_notification_config: push_config,
    /// };
    /// let request = params.into_push_notification_set_request(serde_json::json!("request-1"));
    /// ```
    pub fn into_push_notification_set_request(self, id: serde_json::Value) -> JsonRpcRequest<Self> {
        JsonRpcRequest {
            jsonrpc: default_jsonrpc(),
            id: Some(id),
            method: "tasks/pushNotification/set".to_string(),
            params: self,
        }
    }
}

/// # Task ID Parameters
///
/// Parameters for requests that require a task ID.
///
/// ## Example
/// ```json
/// {
///   "id": "task_01H9FHHSCN8Y5FVFQP66K2330K",
///   "metadata": {
///     "client_id": "web-client-123"
///   }
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskIdParams {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl TaskIdParams {
    /// Convert parameters into a tasks/cancel request
    ///
    /// # Example
    ///
    /// ```
    /// # use acai::{TaskIdParams, JsonRpcRequest};
    /// let params = TaskIdParams { id: "task_123".to_string(), metadata: None };
    /// let request = params.into_cancel_request(serde_json::json!("request-1"));
    /// ```
    pub fn into_cancel_request(self, id: serde_json::Value) -> JsonRpcRequest<Self> {
        JsonRpcRequest {
            jsonrpc: default_jsonrpc(),
            id: Some(id),
            method: "tasks/cancel".to_string(),
            params: self,
        }
    }

    /// Convert parameters into a tasks/pushNotification/get request
    ///
    /// # Example
    ///
    /// ```
    /// # use acai::{TaskIdParams, JsonRpcRequest};
    /// let params = TaskIdParams { id: "task_123".to_string(), metadata: None };
    /// let request = params.into_push_notification_get_request(serde_json::json!("request-1"));
    /// ```
    pub fn into_push_notification_get_request(self, id: serde_json::Value) -> JsonRpcRequest<Self> {
        JsonRpcRequest {
            jsonrpc: default_jsonrpc(),
            id: Some(id),
            method: "tasks/pushNotification/get".to_string(),
            params: self,
        }
    }
}

/// # Task Query Parameters
///
/// Parameters for querying an existing task.
///
/// ## Fields
/// - `id`: The unique identifier for the task.
/// - `history_length`: Optional limit on the number of historical messages to include.
/// - `metadata`: Optional key-value metadata associated with the task query.
///
/// ## Example
/// ```json
/// {
///   "id": "task_123",
///   "historyLength": 10
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskQueryParams {
    pub id: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "historyLength"
    )]
    pub history_length: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl TaskQueryParams {
    /// Create a new TaskQueryParams with just an ID
    ///
    /// # Example
    ///
    /// ```
    /// # use acai::TaskQueryParams;
    /// let params = TaskQueryParams::from_id("task_123");
    /// assert_eq!(params.id, "task_123");
    /// assert_eq!(params.history_length, None);
    /// assert_eq!(params.metadata, None);
    /// ```
    pub fn from_id(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            history_length: None,
            metadata: None,
        }
    }

    /// Convert parameters into a tasks/get request
    ///
    /// # Example
    ///
    /// ```
    /// # use acai::{TaskQueryParams, JsonRpcRequest};
    /// let params = TaskQueryParams {
    ///     id: "task_123".to_string(),
    ///     history_length: Some(10),
    ///     metadata: None
    /// };
    /// let request = params.into_get_request(serde_json::json!("request-1"));
    /// ```
    pub fn into_get_request(self, id: serde_json::Value) -> JsonRpcRequest<Self> {
        JsonRpcRequest {
            jsonrpc: default_jsonrpc(),
            id: Some(id),
            method: "tasks/get".to_string(),
            params: self,
        }
    }

    /// Convert parameters into a tasks/resubscribe request
    ///
    /// # Example
    ///
    /// ```
    /// # use acai::{TaskQueryParams, JsonRpcRequest};
    /// let params = TaskQueryParams {
    ///     id: "task_123".to_string(),
    ///     history_length: Some(10),
    ///     metadata: None
    /// };
    /// let request = params.into_resubscribe_request(serde_json::json!("request-1"));
    /// ```
    pub fn into_resubscribe_request(self, id: serde_json::Value) -> JsonRpcRequest<Self> {
        JsonRpcRequest {
            jsonrpc: default_jsonrpc(),
            id: Some(id),
            method: "tasks/resubscribe".to_string(),
            params: self,
        }
    }
}

/// # Task Send Parameters
///
/// Parameters for sending a message to a task.
///
/// ## Fields
/// - `id`: The unique identifier for the task.
/// - `message`: The message to send to the task.
/// - `session_id`: Optional session identifier for stateful conversations.
/// - `push_notification`: Optional configuration for push notifications.
/// - `history_length`: Optional limit on the number of historical messages to include in the response.
///
/// ## Example
/// ```json
/// {
///   "id": "task_123",
///   "message": {
///     "role": "user",
///     "content": "Hello, assistant!"
///   },
///   "sessionId": "session_456",
///   "historyLength": 10
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskSendParams {
    pub id: String,
    pub message: Message,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sessionId")]
    pub session_id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "pushNotification"
    )]
    pub push_notification: Option<PushNotificationConfig>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "historyLength"
    )]
    pub history_length: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl TaskSendParams {
    /// Convert parameters into a tasks/send request
    ///
    /// # Example
    ///
    /// ```
    /// # use acai::{TaskSendParams, JsonRpcRequest, Message, MessageRole, Part};
    /// # let message = Message {
    /// #     role: MessageRole::User,
    /// #     parts: vec![Part::Text {
    /// #         text: "Hello".to_string(),
    /// #         metadata: None,
    /// #     }],
    /// #     metadata: None,
    /// # };
    /// let params = TaskSendParams {
    ///     id: "task_123".to_string(),
    ///     message,
    ///     session_id: None,
    ///     push_notification: None,
    ///     history_length: None,
    ///     metadata: None,
    /// };
    /// let request = params.into_send_request(serde_json::json!("request-1"));
    /// ```
    pub fn into_send_request(self, id: serde_json::Value) -> JsonRpcRequest<Self> {
        JsonRpcRequest {
            jsonrpc: default_jsonrpc(),
            id: Some(id),
            method: "tasks/send".to_string(),
            params: self,
        }
    }

    /// Convert parameters into a tasks/sendSubscribe request
    ///
    /// # Example
    ///
    /// ```
    /// # use acai::{TaskSendParams, JsonRpcRequest, Message, MessageRole, Part};
    /// # let message = Message {
    /// #     role: MessageRole::User,
    /// #     parts: vec![Part::Text {
    /// #         text: "Hello".to_string(),
    /// #         metadata: None,
    /// #     }],
    /// #     metadata: None,
    /// # };
    /// let params = TaskSendParams {
    ///     id: "task_123".to_string(),
    ///     message,
    ///     session_id: None,
    ///     push_notification: None,
    ///     history_length: None,
    ///     metadata: None,
    /// };
    /// let request = params.into_send_subscribe_request(serde_json::json!("request-1"));
    /// ```
    pub fn into_send_subscribe_request(self, id: serde_json::Value) -> JsonRpcRequest<Self> {
        JsonRpcRequest {
            jsonrpc: default_jsonrpc(),
            id: Some(id),
            method: "tasks/sendSubscribe".to_string(),
            params: self,
        }
    }
}

/// # Task Status Update Event
///
/// Event sent when a task's status changes.
///
/// ## Example
/// ```json
/// {
///   "id": "task_01H9FHHSCN8Y5FVFQP66K2330K",
///   "status": {
///     "state": "completed",
///     "message": {
///       "role": "agent",
///       "parts": [
///         {
///           "type": "text",
///           "text": "I've completed the analysis of your code."
///         }
///       ]
///     },
///     "timestamp": "2023-10-25T15:35:00Z"
///   },
///   "final": true,
///   "metadata": {
///     "processingTime": "3.2s"
///   }
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskStatusUpdateEvent {
    pub id: String,
    pub status: TaskStatus,
    #[serde(default, rename = "final")]
    pub final_status: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// # Task Artifact Update Event
///
/// Event sent when a task generates or updates an artifact.
///
/// ## Example
/// ```json
/// {
///   "id": "task_01H9FHHSCN8Y5FVFQP66K2330K",
///   "artifact": {
///     "name": "Analysis Results",
///     "parts": [
///       {
///         "type": "text",
///         "text": "Additional analysis details..."
///       }
///     ],
///     "index": 1,
///     "append": true,
///     "lastChunk": true
///   },
///   "metadata": {
///     "artifactId": "artifact_01H9FI2A4S8P9R3M5G7T6V0W3X"
///   }
/// }
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskArtifactUpdateEvent {
    pub id: String,
    pub artifact: Artifact,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// A streaming response result content type.
///
/// This enum represents the different types of streaming response content that can be received
/// during a task subscription. It is used as the result type for streaming JSON-RPC responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum StreamingResponseContent {
    /// A status update event
    StatusUpdate(TaskStatusUpdateEvent),

    /// An artifact update event
    ArtifactUpdate(TaskArtifactUpdateEvent),
}

/// Functions for creating streaming responses with StreamingResponseContent
impl JsonRpcResponse<StreamingResponseContent> {
    /// Create a new status update response
    pub fn status_update(id: Option<serde_json::Value>, result: TaskStatusUpdateEvent) -> Self {
        JsonRpcResponse {
            jsonrpc: default_jsonrpc(),
            id,
            result: Some(StreamingResponseContent::StatusUpdate(result)),
            error: None,
        }
    }

    /// Create a new artifact update response
    pub fn artifact_update(id: Option<serde_json::Value>, result: TaskArtifactUpdateEvent) -> Self {
        JsonRpcResponse {
            jsonrpc: default_jsonrpc(),
            id,
            result: Some(StreamingResponseContent::ArtifactUpdate(result)),
            error: None,
        }
    }

    /// Create a new streaming error response
    pub fn streaming_error(id: Option<serde_json::Value>, error: JsonRpcError) -> Self {
        JsonRpcResponse {
            jsonrpc: default_jsonrpc(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_state_serialization() {
        // Test that TaskState serializes and deserializes correctly using kebab-case
        let states = vec![
            (TaskState::Submitted, "\"submitted\""),
            (TaskState::Working, "\"working\""),
            (TaskState::InputRequired, "\"input-required\""),
            (TaskState::Completed, "\"completed\""),
            (TaskState::Canceled, "\"canceled\""),
            (TaskState::Failed, "\"failed\""),
            (TaskState::Unknown, "\"unknown\""),
        ];

        for (state, expected_json) in states {
            // Test serialization
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(json, expected_json);

            // Test deserialization
            let deserialized: TaskState = serde_json::from_str(expected_json).unwrap();
            assert_eq!(deserialized, state);
        }
    }

    // Tests for TaskStatus

    #[test]
    fn task_status_serialization() {
        // Create a task status with all fields
        let now = Utc::now();
        let message = Message {
            role: MessageRole::Agent,
            parts: vec![Part::Text {
                text: "Processing your request".to_string(),
                metadata: None,
            }],
            metadata: None,
        };

        let status = TaskStatus {
            state: TaskState::Working,
            message: Some(message.clone()),
            timestamp: Some(now),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&status).unwrap();

        // Deserialize back and verify fields
        let deserialized: TaskStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.state, TaskState::Working);
        assert_eq!(deserialized.message.as_ref().unwrap().role, message.role);
        assert_eq!(
            deserialized.message.as_ref().unwrap().parts.len(),
            message.parts.len()
        );
        assert!(deserialized.timestamp.is_some());
    }

    #[test]
    fn task_status_minimal() {
        // Test with only required field (state)
        let status = TaskStatus {
            state: TaskState::Submitted,
            message: None,
            timestamp: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        let expected = r#"{"state":"submitted"}"#;
        assert_eq!(json, expected);

        let deserialized: TaskStatus = serde_json::from_str(expected).unwrap();
        assert_eq!(deserialized.state, TaskState::Submitted);
        assert!(deserialized.message.is_none());
        assert!(deserialized.timestamp.is_none());
    }

    // Tests for Task

    #[test]
    fn task_serialization() {
        // Create a task with all fields
        let task = Task {
            id: "task_123".to_string(),
            session_id: Some("session_456".to_string()),
            status: TaskStatus {
                state: TaskState::Working,
                message: None,
                timestamp: Some(Utc::now()),
            },
            artifacts: Some(vec![Artifact {
                name: Some("Result".to_string()),
                description: Some("Analysis results".to_string()),
                parts: vec![Part::Text {
                    text: "Analysis content".to_string(),
                    metadata: None,
                }],
                index: 0,
                append: None,
                last_chunk: Some(true),
                metadata: None,
            }]),
            history: Some(vec![
                Message {
                    role: MessageRole::User,
                    parts: vec![Part::Text {
                        text: "Initial request".to_string(),
                        metadata: None,
                    }],
                    metadata: None,
                },
                Message {
                    role: MessageRole::Agent,
                    parts: vec![Part::Text {
                        text: "Processing request".to_string(),
                        metadata: None,
                    }],
                    metadata: None,
                },
            ]),
            metadata: {
                let mut map = HashMap::new();
                map.insert("priority".to_string(), "high".into());
                Some(map)
            },
        };

        // Serialize to JSON
        let json = serde_json::to_string(&task).unwrap();

        // Deserialize back and verify fields
        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "task_123");
        assert_eq!(deserialized.session_id, Some("session_456".to_string()));
        assert_eq!(deserialized.status.state, TaskState::Working);
        assert!(deserialized.artifacts.is_some());
        assert_eq!(deserialized.artifacts.as_ref().unwrap().len(), 1);
        assert_eq!(
            deserialized.artifacts.as_ref().unwrap()[0].name,
            Some("Result".to_string())
        );
        assert!(deserialized.history.is_some());
        assert_eq!(deserialized.history.as_ref().unwrap().len(), 2);
        assert_eq!(
            deserialized.history.as_ref().unwrap()[0].role,
            MessageRole::User
        );
        assert!(deserialized.metadata.is_some());
        assert_eq!(
            deserialized
                .metadata
                .as_ref()
                .unwrap()
                .get("priority")
                .unwrap(),
            &serde_json::Value::String("high".to_string())
        );
    }

    #[test]
    fn task_minimal() {
        // Test with only required fields
        let task = Task {
            id: "task_123".to_string(),
            session_id: None,
            status: TaskStatus {
                state: TaskState::Submitted,
                message: None,
                timestamp: None,
            },
            artifacts: None,
            history: None,
            metadata: None,
        };

        let json = serde_json::to_string(&task).unwrap();

        // Ensure optional fields are not included
        assert!(!json.contains("sessionId"));
        assert!(!json.contains("artifacts"));
        assert!(!json.contains("history"));
        assert!(!json.contains("metadata"));

        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "task_123");
        assert_eq!(deserialized.status.state, TaskState::Submitted);
        assert!(deserialized.session_id.is_none());
        assert!(deserialized.artifacts.is_none());
        assert!(deserialized.history.is_none());
        assert!(deserialized.metadata.is_none());
    }

    // Tests for Agent Card and related structures

    #[test]
    fn agent_capabilities_serialization() {
        // Test minimal capabilities
        let capabilities = AgentCapabilities {
            streaming: false,
            push_notifications: false,
            state_transition_history: false,
        };

        let json = serde_json::to_string(&capabilities).unwrap();
        let expected =
            r#"{"streaming":false,"pushNotifications":false,"stateTransitionHistory":false}"#;
        assert_eq!(json, expected);

        // Test with all capabilities enabled
        let capabilities = AgentCapabilities {
            streaming: true,
            push_notifications: true,
            state_transition_history: true,
        };

        let json = serde_json::to_string(&capabilities).unwrap();
        let expected =
            r#"{"streaming":true,"pushNotifications":true,"stateTransitionHistory":true}"#;
        assert_eq!(json, expected);

        // Test deserialization
        let deserialized: AgentCapabilities = serde_json::from_str(expected).unwrap();
        assert!(deserialized.streaming);
        assert!(deserialized.push_notifications);
        assert!(deserialized.state_transition_history);
    }

    #[test]
    fn agent_skill_serialization() {
        // Test with all fields
        let skill = AgentSkill {
            id: "code-review".to_string(),
            name: "Code Review".to_string(),
            description: Some("Reviews code for bugs and improvements".to_string()),
            tags: Some(vec!["coding".to_string(), "review".to_string()]),
            examples: Some(vec!["Review my code".to_string()]),
            input_modes: Some(vec!["text".to_string(), "file".to_string()]),
            output_modes: Some(vec!["text".to_string()]),
        };

        let json = serde_json::to_string(&skill).unwrap();

        // Check specific fields in the JSON output
        assert!(json.contains(r#""id":"code-review""#));
        assert!(json.contains(r#""name":"Code Review""#));
        assert!(json.contains(r#""description":"Reviews code for bugs and improvements""#));
        assert!(json.contains(r#""tags":["coding","review"]"#));
        assert!(json.contains(r#""examples":["Review my code"]"#));
        assert!(json.contains(r#""inputModes":["text","file"]"#));
        assert!(json.contains(r#""outputModes":["text"]"#));

        // Test deserialization
        let deserialized: AgentSkill = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "code-review");
        assert_eq!(deserialized.name, "Code Review");
        assert_eq!(
            deserialized.description,
            Some("Reviews code for bugs and improvements".to_string())
        );
        assert_eq!(
            deserialized.tags,
            Some(vec!["coding".to_string(), "review".to_string()])
        );
        assert_eq!(
            deserialized.examples,
            Some(vec!["Review my code".to_string()])
        );
    }

    #[test]
    fn agent_skill_minimal() {
        // Test with only required fields
        let skill = AgentSkill {
            id: "chat".to_string(),
            name: "Chat".to_string(),
            description: None,
            tags: None,
            examples: None,
            input_modes: None,
            output_modes: None,
        };

        let json = serde_json::to_string(&skill).unwrap();

        // Ensure optional fields are not included
        assert!(!json.contains("description"));
        assert!(!json.contains("tags"));
        assert!(!json.contains("examples"));
        assert!(!json.contains("inputModes"));
        assert!(!json.contains("outputModes"));

        // Test deserialization
        let deserialized: AgentSkill = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "chat");
        assert_eq!(deserialized.name, "Chat");
        assert!(deserialized.description.is_none());
        assert!(deserialized.tags.is_none());
        assert!(deserialized.examples.is_none());
        assert!(deserialized.input_modes.is_none());
        assert!(deserialized.output_modes.is_none());
    }

    #[test]
    fn agent_card_serialization() {
        // Test with all fields
        let card = AgentCard {
            name: "Code Assistant".to_string(),
            description: Some("AI assistant for code review and generation".to_string()),
            url: "https://example.com/agent".to_string(),
            provider: Some(AgentProvider {
                organization: "acai Project".to_string(),
                url: Some("https://github.com/rescrv/acai".to_string()),
            }),
            version: "1.0.0".to_string(),
            documentation_url: Some("https://example.com/docs".to_string()),
            capabilities: AgentCapabilities {
                streaming: true,
                push_notifications: true,
                state_transition_history: false,
            },
            authentication: Some(AgentAuthentication {
                schemes: vec!["bearer".to_string()],
                credentials: Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...".to_string()),
            }),
            default_input_modes: vec!["text".to_string()],
            default_output_modes: vec!["text".to_string()],
            skills: vec![
                AgentSkill {
                    id: "code-review".to_string(),
                    name: "Code Review".to_string(),
                    description: Some("Review code for issues".to_string()),
                    tags: None,
                    examples: None,
                    input_modes: None,
                    output_modes: None,
                },
                AgentSkill {
                    id: "code-gen".to_string(),
                    name: "Code Generation".to_string(),
                    description: None,
                    tags: None,
                    examples: None,
                    input_modes: None,
                    output_modes: None,
                },
            ],
        };

        let json = serde_json::to_string(&card).unwrap();

        // Verify key fields are in the serialized JSON
        assert!(json.contains(r#""name":"Code Assistant""#));
        assert!(json.contains(r#""url":"https://example.com/agent""#));
        assert!(json.contains(r#""version":"1.0.0""#));
        assert!(json.contains(r#""streaming":true"#));
        assert!(json.contains(r#""skills":[{"id":"code-review""#));

        // Test deserialization
        let deserialized: AgentCard = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Code Assistant");
        assert_eq!(deserialized.url, "https://example.com/agent");
        assert_eq!(deserialized.version, "1.0.0");
        assert!(deserialized.capabilities.streaming);
        assert_eq!(deserialized.skills.len(), 2);
        assert_eq!(deserialized.skills[0].id, "code-review");
        assert_eq!(deserialized.skills[1].id, "code-gen");
    }

    #[test]
    fn agent_card_minimal() {
        // Test with minimal required fields
        let card = AgentCard {
            name: "Minimal Agent".to_string(),
            description: None,
            url: "https://example.com/agent".to_string(),
            provider: None,
            version: "1.0.0".to_string(),
            documentation_url: None,
            capabilities: AgentCapabilities {
                streaming: false,
                push_notifications: false,
                state_transition_history: false,
            },
            authentication: None,
            default_input_modes: vec!["text".to_string()],
            default_output_modes: vec!["text".to_string()],
            skills: vec![AgentSkill {
                id: "chat".to_string(),
                name: "Chat".to_string(),
                description: None,
                tags: None,
                examples: None,
                input_modes: None,
                output_modes: None,
            }],
        };

        let json = serde_json::to_string(&card).unwrap();

        // Check that optional fields are not included
        assert!(!json.contains("description"));
        assert!(!json.contains("provider"));
        assert!(!json.contains("documentationUrl"));
        assert!(!json.contains("authentication"));

        // Test deserialization
        let deserialized: AgentCard = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "Minimal Agent");
        assert_eq!(deserialized.url, "https://example.com/agent");
        assert_eq!(deserialized.version, "1.0.0");
        assert!(!deserialized.capabilities.streaming);
        assert_eq!(deserialized.skills.len(), 1);
        assert_eq!(deserialized.skills[0].id, "chat");
        assert!(deserialized.provider.is_none());
        assert!(deserialized.documentation_url.is_none());
        assert!(deserialized.authentication.is_none());
    }

    // Tests for Message, MessageRole, and Part

    #[test]
    fn message_role_serialization() {
        // Test that MessageRole serializes to lowercase strings
        let roles = vec![
            (MessageRole::User, "\"user\""),
            (MessageRole::Agent, "\"agent\""),
        ];

        for (role, expected_json) in roles {
            // Test serialization
            let json = serde_json::to_string(&role).unwrap();
            assert_eq!(json, expected_json);

            // Test deserialization
            let deserialized: MessageRole = serde_json::from_str(expected_json).unwrap();
            assert_eq!(deserialized, role);
        }
    }

    #[test]
    fn part_text_serialization() {
        // Test Text part
        let part = Part::Text {
            text: "Hello, world!".to_string(),
            metadata: None,
        };

        let json = serde_json::to_string(&part).unwrap();
        let expected = r#"{"type":"text","text":"Hello, world!"}"#;
        assert_eq!(json, expected);

        // Test deserialization
        let deserialized: Part = serde_json::from_str(expected).unwrap();
        match deserialized {
            Part::Text { text, metadata } => {
                assert_eq!(text, "Hello, world!");
                assert!(metadata.is_none());
            }
            _ => panic!("Deserialized to wrong variant"),
        }

        // Test Text part with metadata
        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "markdown".into());

        let part = Part::Text {
            text: "**Bold text**".to_string(),
            metadata: Some(metadata),
        };

        let json = serde_json::to_string(&part).unwrap();

        // Check JSON contains expected fields
        assert!(json.contains(r#""type":"text""#));
        assert!(json.contains(r#""text":"**Bold text**""#));
        assert!(json.contains(r#""metadata":{"format":"markdown"}"#));

        // Test deserialization with metadata
        let deserialized: Part = serde_json::from_str(&json).unwrap();
        match deserialized {
            Part::Text { text, metadata } => {
                assert_eq!(text, "**Bold text**");
                assert!(metadata.is_some());
                let meta = metadata.unwrap();
                assert_eq!(
                    meta.get("format").unwrap(),
                    &serde_json::Value::String("markdown".to_string())
                );
            }
            _ => panic!("Deserialized to wrong variant"),
        }
    }

    #[test]
    fn part_data_serialization() {
        // Test Data part
        let mut data = HashMap::new();
        data.insert("name".to_string(), "John Doe".into());
        data.insert("age".to_string(), 30.into());

        let part = Part::Data {
            data,
            metadata: None,
        };

        let json = serde_json::to_string(&part).unwrap();

        // Check JSON format
        assert!(json.contains(r#""type":"data""#));
        assert!(json.contains(r#""data":{"#));
        assert!(json.contains(r#""name":"John Doe""#));
        assert!(json.contains(r#""age":30"#));

        // Test deserialization
        let deserialized: Part = serde_json::from_str(&json).unwrap();
        match deserialized {
            Part::Data { data, metadata } => {
                assert_eq!(
                    data.get("name").unwrap(),
                    &serde_json::Value::String("John Doe".to_string())
                );
                assert_eq!(
                    data.get("age").unwrap(),
                    &serde_json::Value::Number(30.into())
                );
                assert!(metadata.is_none());
            }
            _ => panic!("Deserialized to wrong variant"),
        }
    }

    #[test]
    fn part_file_serialization() {
        // Test File part
        let file_content = FileContent {
            name: Some("example.txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            bytes: Some("SGVsbG8gd29ybGQ=".to_string()), // Base64 for "Hello world"
            uri: None,
        };

        let part = Part::File {
            file: file_content,
            metadata: None,
        };

        let json = serde_json::to_string(&part).unwrap();

        // Check JSON format
        assert!(json.contains(r#""type":"file""#));
        assert!(json.contains(r#""file":{"#));
        assert!(json.contains(r#""name":"example.txt""#));
        assert!(json.contains(r#""mimeType":"text/plain""#));
        assert!(json.contains(r#""bytes":"SGVsbG8gd29ybGQ=""#));

        // Test deserialization
        let deserialized: Part = serde_json::from_str(&json).unwrap();
        match deserialized {
            Part::File { file, metadata } => {
                assert_eq!(file.name, Some("example.txt".to_string()));
                assert_eq!(file.mime_type, Some("text/plain".to_string()));
                assert_eq!(file.bytes, Some("SGVsbG8gd29ybGQ=".to_string()));
                assert!(file.uri.is_none());
                assert!(metadata.is_none());
            }
            _ => panic!("Deserialized to wrong variant"),
        }
    }

    #[test]
    fn message_serialization() {
        // Test a message with multiple parts
        let message = Message {
            role: MessageRole::User,
            parts: vec![
                Part::Text {
                    text: "Here's my code:".to_string(),
                    metadata: None,
                },
                Part::File {
                    file: FileContent {
                        name: Some("code.js".to_string()),
                        mime_type: Some("application/javascript".to_string()),
                        bytes: Some("Y29uc3QgaGVsbG8gPSAoKSA9PiAiSGVsbG8sIFdvcmxkISI7".to_string()),
                        uri: None,
                    },
                    metadata: None,
                },
            ],
            metadata: Some({
                let mut meta = HashMap::new();
                meta.insert("timestamp".to_string(), "2023-10-25T15:30:00Z".into());
                meta
            }),
        };

        let json = serde_json::to_string(&message).unwrap();

        // Check JSON format
        assert!(json.contains(r#""role":"user""#));
        assert!(json.contains(r#""parts":[{"type":"text""#));
        assert!(json.contains(r#"{"type":"file""#));
        assert!(json.contains(r#""metadata":{"timestamp":"2023-10-25T15:30:00Z"}"#));

        // Test deserialization
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, MessageRole::User);
        assert_eq!(deserialized.parts.len(), 2);
        assert!(matches!(deserialized.parts[0], Part::Text { .. }));
        assert!(matches!(deserialized.parts[1], Part::File { .. }));
        assert!(deserialized.metadata.is_some());
    }

    #[test]
    fn message_minimal() {
        // Test message with minimal fields
        let message = Message {
            role: MessageRole::Agent,
            parts: vec![Part::Text {
                text: "Hello, how can I help you?".to_string(),
                metadata: None,
            }],
            metadata: None,
        };

        let json = serde_json::to_string(&message).unwrap();

        // Ensure metadata is not included
        assert!(!json.contains("metadata"));

        // Test deserialization
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.role, MessageRole::Agent);
        assert_eq!(deserialized.parts.len(), 1);
        match &deserialized.parts[0] {
            Part::Text { text, metadata } => {
                assert_eq!(text, "Hello, how can I help you?");
                assert!(metadata.is_none());
            }
            _ => panic!("Wrong part type"),
        }
        assert!(deserialized.metadata.is_none());
    }

    // Tests for FileContent

    #[test]
    fn file_content_with_bytes() {
        // Create a FileContent with bytes
        let content = FileContent {
            name: Some("document.pdf".to_string()),
            mime_type: Some("application/pdf".to_string()),
            bytes: Some("JVBERi0xLjUKJYCBgoMK".to_string()), // Start of a base64 encoded PDF
            uri: None,
        };

        let json = serde_json::to_string(&content).unwrap();

        // Check JSON format
        assert!(json.contains(r#""name":"document.pdf""#));
        assert!(json.contains(r#""mimeType":"application/pdf""#));
        assert!(json.contains(r#""bytes":"JVBERi0xLjUKJYCBgoMK""#));
        assert!(!json.contains("uri"));

        // Test deserialization
        let deserialized: FileContent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, Some("document.pdf".to_string()));
        assert_eq!(deserialized.mime_type, Some("application/pdf".to_string()));
        assert_eq!(deserialized.bytes, Some("JVBERi0xLjUKJYCBgoMK".to_string()));
        assert!(deserialized.uri.is_none());
    }

    #[test]
    fn file_content_with_uri() {
        // Create a FileContent with URI
        let content = FileContent {
            name: Some("image.jpg".to_string()),
            mime_type: Some("image/jpeg".to_string()),
            bytes: None,
            uri: Some("https://example.com/images/photo.jpg".to_string()),
        };

        let json = serde_json::to_string(&content).unwrap();

        // Check JSON format
        assert!(json.contains(r#""name":"image.jpg""#));
        assert!(json.contains(r#""mimeType":"image/jpeg""#));
        assert!(json.contains(r#""uri":"https://example.com/images/photo.jpg""#));
        assert!(!json.contains("bytes"));

        // Test deserialization
        let deserialized: FileContent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, Some("image.jpg".to_string()));
        assert_eq!(deserialized.mime_type, Some("image/jpeg".to_string()));
        assert_eq!(
            deserialized.uri,
            Some("https://example.com/images/photo.jpg".to_string())
        );
        assert!(deserialized.bytes.is_none());
    }

    #[test]
    fn file_content_minimal() {
        // Test with minimal required fields - just bytes
        let content = FileContent {
            name: None,
            mime_type: None,
            bytes: Some("SGVsbG8gd29ybGQ=".to_string()),
            uri: None,
        };

        let json = serde_json::to_string(&content).unwrap();

        // Check that optional fields are not included
        assert!(!json.contains("name"));
        assert!(!json.contains("mimeType"));
        assert!(!json.contains("uri"));

        // Test deserialization
        let deserialized: FileContent = serde_json::from_str(&json).unwrap();
        assert!(deserialized.name.is_none());
        assert!(deserialized.mime_type.is_none());
        assert_eq!(deserialized.bytes, Some("SGVsbG8gd29ybGQ=".to_string()));
        assert!(deserialized.uri.is_none());

        // Minimal with just URI
        let content = FileContent {
            name: None,
            mime_type: None,
            bytes: None,
            uri: Some("https://example.com/file.txt".to_string()),
        };

        let json = serde_json::to_string(&content).unwrap();

        // Check that optional fields are not included
        assert!(!json.contains("name"));
        assert!(!json.contains("mimeType"));
        assert!(!json.contains("bytes"));

        // Test deserialization
        let deserialized: FileContent = serde_json::from_str(&json).unwrap();
        assert!(deserialized.name.is_none());
        assert!(deserialized.mime_type.is_none());
        assert!(deserialized.bytes.is_none());
        assert_eq!(
            deserialized.uri,
            Some("https://example.com/file.txt".to_string())
        );
    }

    #[test]
    fn file_content_validation() {
        // Test the validation that requires either bytes or uri
        let json = r#"{"name":"test.txt"}"#;
        let result = serde_json::from_str::<FileContent>(json);
        assert!(result.is_err());

        // Test validation that forbids both bytes and uri
        let json = r#"{
            "name": "test.txt",
            "bytes": "dGVzdA==",
            "uri": "https://example.com/test.txt"
        }"#;
        let result = serde_json::from_str::<FileContent>(json);
        assert!(result.is_err());
    }

    // Tests for FormField

    #[test]
    fn form_field_serialization() {
        // Test FormField with all fields
        let mut validation = HashMap::new();
        validation.insert("minLength".to_string(), 3.into());
        validation.insert("maxLength".to_string(), 50.into());

        let mut additional_props = HashMap::new();
        additional_props.insert("placeholder".to_string(), "Enter your name".into());
        additional_props.insert("style".to_string(), "bold".into());

        let field = FormField {
            title: Some("Full Name".to_string()),
            format: Some("text".to_string()),
            required: true,
            default: Some("John Doe".into()),
            description: Some("Please enter your full legal name".to_string()),
            validation: Some(validation),
            additional_properties: additional_props,
        };

        let json = serde_json::to_string(&field).unwrap();

        // Check JSON format
        assert!(json.contains(r#""title":"Full Name""#));
        assert!(json.contains(r#""format":"text""#));
        assert!(json.contains(r#""required":true"#));
        assert!(json.contains(r#""default":"John Doe""#));
        assert!(json.contains(r#""description":"Please enter your full legal name""#));
        assert!(json.contains(r#""validation":{""#));
        assert!(json.contains(r#""minLength":3"#));
        assert!(json.contains(r#""maxLength":50"#));
        assert!(json.contains(r#""placeholder":"Enter your name""#));
        assert!(json.contains(r#""style":"bold""#));

        // Test deserialization
        let deserialized: FormField = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, Some("Full Name".to_string()));
        assert_eq!(deserialized.format, Some("text".to_string()));
        assert!(deserialized.required);
        assert_eq!(
            deserialized.default,
            Some(serde_json::Value::String("John Doe".to_string()))
        );
        assert_eq!(
            deserialized.description,
            Some("Please enter your full legal name".to_string())
        );

        // Check validation properties
        let validation = deserialized.validation.unwrap();
        assert_eq!(
            validation.get("minLength").unwrap(),
            &serde_json::Value::Number(3.into())
        );
        assert_eq!(
            validation.get("maxLength").unwrap(),
            &serde_json::Value::Number(50.into())
        );

        // Check additional properties
        assert_eq!(
            deserialized
                .additional_properties
                .get("placeholder")
                .unwrap(),
            &serde_json::Value::String("Enter your name".to_string())
        );
    }

    #[test]
    fn form_field_minimal() {
        // Test FormField with minimal fields
        let field = FormField {
            title: None,
            format: None,
            required: false,
            default: None,
            description: None,
            validation: None,
            additional_properties: HashMap::new(),
        };

        let json = serde_json::to_string(&field).unwrap();

        // Check that optional fields are not included
        assert!(!json.contains("title"));
        assert!(!json.contains("format"));
        assert!(!json.contains("required"));
        assert!(!json.contains("default"));
        assert!(!json.contains("description"));
        assert!(!json.contains("validation"));

        // Should be an empty JSON object
        assert_eq!(json, "{}");

        // Test deserialization
        let deserialized: FormField = serde_json::from_str(&json).unwrap();
        assert!(deserialized.title.is_none());
        assert!(deserialized.format.is_none());
        assert!(!deserialized.required);
        assert!(deserialized.default.is_none());
        assert!(deserialized.description.is_none());
        assert!(deserialized.validation.is_none());
        assert!(deserialized.additional_properties.is_empty());
    }

    #[test]
    fn form_field_with_additional_properties() {
        // Test FormField with just additional properties
        let mut props = HashMap::new();
        props.insert("min".to_string(), 1.into());
        props.insert("max".to_string(), 100.into());
        props.insert("step".to_string(), 5.into());

        let field = FormField {
            title: None,
            format: None,
            required: false,
            default: None,
            description: None,
            validation: None,
            additional_properties: props,
        };

        let json = serde_json::to_string(&field).unwrap();

        // Check the JSON output
        assert!(json.contains(r#""min":1"#));
        assert!(json.contains(r#""max":100"#));
        assert!(json.contains(r#""step":5"#));

        // Test deserialization
        let deserialized: FormField = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.additional_properties.get("min").unwrap(),
            &serde_json::Value::Number(1.into())
        );
        assert_eq!(
            deserialized.additional_properties.get("max").unwrap(),
            &serde_json::Value::Number(100.into())
        );
        assert_eq!(
            deserialized.additional_properties.get("step").unwrap(),
            &serde_json::Value::Number(5.into())
        );
    }

    // Tests for FormSchema

    #[test]
    fn form_schema_serialization() {
        // Create form fields
        let name_field = FormField {
            title: Some("Full Name".to_string()),
            format: Some("text".to_string()),
            required: true,
            default: None,
            description: Some("Your legal name".to_string()),
            validation: None,
            additional_properties: HashMap::new(),
        };

        let email_field = FormField {
            title: Some("Email Address".to_string()),
            format: Some("email".to_string()),
            required: false,
            default: None,
            description: None,
            validation: None,
            additional_properties: HashMap::new(),
        };

        // Create form schema
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), name_field);
        properties.insert("email".to_string(), email_field);

        let mut additional_props = HashMap::new();
        additional_props.insert("title".to_string(), "User Registration Form".into());
        additional_props.insert("description".to_string(), "Enter your details below".into());

        let schema = FormSchema {
            properties,
            required: vec!["name".to_string()],
            additional_properties: additional_props,
        };

        let json = serde_json::to_string(&schema).unwrap();

        // Check the JSON output
        assert!(json.contains(r#""properties":{""#));
        assert!(json.contains(r#""name":{"#));
        assert!(json.contains(r#""email":{"#));
        assert!(json.contains(r#""required":["name"]"#));
        assert!(json.contains(r#""title":"User Registration Form""#));
        assert!(json.contains(r#""description":"Enter your details below""#));

        // Test deserialization
        let deserialized: FormSchema = serde_json::from_str(&json).unwrap();

        // Check properties
        assert_eq!(deserialized.properties.len(), 2);
        assert!(deserialized.properties.contains_key("name"));
        assert!(deserialized.properties.contains_key("email"));

        // Check a specific property
        let name_field = &deserialized.properties["name"];
        assert_eq!(name_field.title, Some("Full Name".to_string()));
        assert!(name_field.required);

        // Check required fields
        assert_eq!(deserialized.required.len(), 1);
        assert_eq!(deserialized.required[0], "name");

        // Check additional properties
        assert_eq!(
            deserialized.additional_properties.get("title").unwrap(),
            &serde_json::Value::String("User Registration Form".to_string())
        );
    }

    #[test]
    fn form_schema_minimal() {
        // Create a minimal form schema with no required fields
        let phone_field = FormField {
            title: Some("Phone".to_string()),
            format: None,
            required: false,
            default: None,
            description: None,
            validation: None,
            additional_properties: HashMap::new(),
        };

        let mut properties = HashMap::new();
        properties.insert("phone".to_string(), phone_field);

        let schema = FormSchema {
            properties,
            required: vec![],
            additional_properties: HashMap::new(),
        };

        let json = serde_json::to_string(&schema).unwrap();

        // Check the JSON output - should not include required since it's empty
        assert!(json.contains(r#""properties":{""#));
        assert!(json.contains(r#""phone":{"#));
        assert!(!json.contains(r#""required":"#));

        // Test deserialization
        let deserialized: FormSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.properties.len(), 1);
        assert!(deserialized.properties.contains_key("phone"));
        assert!(deserialized.required.is_empty());
        assert!(deserialized.additional_properties.is_empty());
    }
}
