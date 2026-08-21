//! A2A Protocol client implementation
//!
//! This module provides a client for making requests to A2A Protocol servers.
//! It handles serialization, sending requests, and parsing responses.

// ReqwestClient is already internally wrapped in an Arc, so we don't need to use std::sync::Arc
use std::pin::Pin;
use std::time::Duration;

use futures::{Stream, StreamExt};
use reqwest::{Client as ReqwestClient, Error as ReqwestError};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::PushNotificationConfig;
use crate::types::{
    AgentCard, JsonRpcError, JsonRpcRequest, JsonRpcResponse, StreamingResponseContent,
    TaskIdParams, TaskPushNotificationConfig,
};

// Helper function to find a subsequence in a byte slice
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }

    'outer: for i in 0..=haystack.len() - needle.len() {
        for (j, &item) in needle.iter().enumerate() {
            if haystack[i + j] != item {
                continue 'outer;
            }
        }
        return Some(i);
    }

    None
}

/// Error type for client operations
#[derive(Debug)]
pub enum Error {
    /// HTTP error
    HttpError(ReqwestError),

    /// JSON-RPC error
    JsonRpcError(JsonRpcError),

    /// Method not found
    MethodNotFound(String),

    /// Invalid parameters
    InvalidParams(String),

    /// Missing result in response
    MissingResult,

    /// Address parsing error
    AddrParseError(std::net::AddrParseError),

    /// Serialization error
    SerializationError(serde_json::Error),

    /// Invalid streaming method error
    InvalidStreamingMethod(String),

    /// Content type not supported error
    ContentTypeNotSupported(String),

    /// Stream processing error
    StreamProcessingError(String),

    /// Task error
    TaskError(String),

    /// Internal error
    InternalError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<ReqwestError> for Error {
    fn from(err: ReqwestError) -> Self {
        Error::HttpError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(err)
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(err: std::net::AddrParseError) -> Self {
        Error::AddrParseError(err)
    }
}

/// Default timeout for client requests
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Configuration for the A2A client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// The URL of the A2A server
    pub server_url: String,

    /// Authentication token for the server
    pub auth_token: Option<String>,

    /// Timeout for requests
    pub timeout: Duration,
}

impl ClientConfig {
    /// Create a new client configuration with the given server URL
    pub fn new(server_url: &str) -> Self {
        Self {
            server_url: server_url.to_string(),
            auth_token: None,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Set a custom timeout and return an updated configuration
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set an authentication token and return an updated configuration
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }
}

/// A client for making requests to an A2A server
pub struct Client {
    /// The HTTP client
    client: ReqwestClient,

    /// Client configuration
    config: ClientConfig,
}

impl Client {
    /// Create a new client with the given configuration
    pub fn new(config: ClientConfig) -> Result<Self, Error> {
        let client = ReqwestClient::builder()
            .timeout(config.timeout)
            // Enforce HTTP/2 which is required by the A2A Protocol
            .http2_prior_knowledge()
            .build()
            .map_err(Error::HttpError)?;

        Ok(Self { client, config })
    }

    /// Configure push notifications for a task
    ///
    /// This is a helper method that creates and sends a request to configure
    /// push notifications for an existing task.
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use acai::client::{Client, ClientConfig};
    /// use acai::PushNotificationConfig;
    ///
    /// let client = Client::new(ClientConfig::new("http://example.com"))?;
    ///
    /// // Create a push notification configuration
    /// let config = PushNotificationConfig {
    ///     url: "https://example.com/webhook".to_string(),
    ///     token: Some("webhook-secret-token".to_string()),
    ///     authentication: None,
    /// };
    ///
    /// // Configure push notifications for a task
    /// let task_id = "task_123";
    /// let result = client.set_push_notification(task_id, config).await?;
    /// println!("Push notifications configured for task: {}", result.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_push_notification(
        &self,
        task_id: impl Into<String>,
        config: PushNotificationConfig,
    ) -> Result<TaskPushNotificationConfig, Error> {
        // Create the push notification config parameters
        let params = TaskPushNotificationConfig {
            id: task_id.into(),
            push_notification_config: config,
        };

        // Create the request
        let request = params.into_push_notification_set_request(serde_json::json!("request-1"));

        // Send the request
        self.send(request).await
    }

    /// Get push notification configuration for a task
    ///
    /// This is a helper method that creates and sends a request to retrieve
    /// the push notification configuration for an existing task.
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use acai::client::{Client, ClientConfig};
    ///
    /// let client = Client::new(ClientConfig::new("http://example.com"))?;
    ///
    /// // Get push notification configuration for a task
    /// let task_id = "task_123";
    /// let result = client.get_push_notification(task_id).await?;
    /// println!("Push notification URL: {}", result.push_notification_config.url);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_push_notification(
        &self,
        task_id: impl Into<String>,
    ) -> Result<TaskPushNotificationConfig, Error> {
        // Create the task ID parameters
        let params = TaskIdParams {
            id: task_id.into(),
            metadata: None,
        };

        // Create the request
        let request = params.into_push_notification_get_request(serde_json::json!("request-1"));

        // Send the request
        self.send(request).await
    }

    /// Send a request to the server and parse the response
    ///
    /// This method sends a JSON-RPC request to the server and parses the response.
    /// It handles authentication, serialization, error checking, and type conversion.
    ///
    /// # Arguments
    /// * `request` - A JSON-RPC request with parameters of type `P`
    ///
    /// # Returns
    /// * `Result<R, Error>` - The parsed response of type `R` or an error
    ///
    /// # Errors
    /// * `HttpError` - If there was an HTTP-level error
    /// * `JsonRpcError` - If the server returned a JSON-RPC error
    /// * `MissingResult` - If the response doesn't contain a result
    /// * Other serialization or parsing errors
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use acai::{
    ///     client::{Client, ClientConfig},
    ///     JsonRpcRequest, TaskQueryParams, Task
    /// };
    ///
    /// let client = Client::new(ClientConfig::new("http://example.com"))?;
    ///
    /// // Create a request to get a task
    /// let params = TaskQueryParams {
    ///     id: "task_123".to_string(),
    ///     history_length: None,
    ///     metadata: None,
    /// };
    /// let request = params.into_get_request(serde_json::json!("req-id"));
    ///
    /// // Send the request and get the task
    /// let task: Task = client.send(request).await?;
    /// println!("Task status: {:?}", task.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send<P, R>(&self, request: JsonRpcRequest<P>) -> Result<R, Error>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        // Clone the client configuration
        let server_url = self.config.server_url.clone();
        let auth_token = self.config.auth_token.clone();

        // Create a new request builder
        let mut builder = self.client.post(&server_url).json(&request);

        // Add authentication if provided
        if let Some(token) = auth_token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }

        // Send the request and check for HTTP errors
        let response = builder
            .send()
            .await?
            .error_for_status()
            .map_err(Error::HttpError)?;

        // Parse the response body
        let json_rpc_response: JsonRpcResponse<serde_json::Value> = response.json().await?;

        // Check for JSON-RPC errors
        if let Some(error) = json_rpc_response.get_error() {
            return Err(Error::JsonRpcError(error.clone()));
        }

        // Get the result or return an error
        let result = json_rpc_response
            .result()
            .ok_or_else(|| Error::MissingResult)?;

        // Parse the result into the expected type
        let typed_result = serde_json::from_value(result.clone())?;
        Ok(typed_result)
    }

    /// Fetch an agent card from a server
    ///
    /// This makes a GET request to the well-known location (/.well-known/agent.json)
    /// where servers advertise their capabilities.
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use acai::client::{Client, ClientConfig};
    ///
    /// let client = Client::new(ClientConfig::new("http://example.com"))?;
    /// let agent_card = client.fetch_agent_card().await?;
    /// println!("Agent name: {}", agent_card.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_agent_card(&self) -> Result<AgentCard, Error> {
        // Build the URL to the well-known agent card location
        let base_url = self.config.server_url.trim_end_matches('/');
        let agent_card_url = format!("{}/.well-known/agent.json", base_url);

        // Create a GET request
        let mut builder = self.client.get(&agent_card_url);

        // Add authentication if provided
        if let Some(token) = &self.config.auth_token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }

        // Send the request and check for HTTP errors
        let response = builder
            .send()
            .await?
            .error_for_status()
            .map_err(Error::HttpError)?;

        // Parse the agent card
        let agent_card = response
            .json::<AgentCard>()
            .await
            .map_err(Error::HttpError)?;

        Ok(agent_card)
    }

    /// Send a request to the server and stream the response events
    ///
    /// This is used for streaming endpoints like `tasks/sendSubscribe` and `tasks/resubscribe`
    /// which return a stream of Server-Sent Events (SSE).
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use acai::{
    ///     client::{Client, ClientConfig},
    ///     JsonRpcRequest, TaskSendParams, Message, MessageRole, Part, JsonRpcResponse,
    ///     StreamingResponseContent
    /// };
    /// use futures::StreamExt;
    ///
    /// let client = Client::new(ClientConfig::new("http://example.com"))?;
    ///
    /// // Create a message
    /// let message = Message {
    ///     role: MessageRole::User,
    ///     parts: vec![Part::Text {
    ///         text: "Generate some code".to_string(),
    ///         metadata: None,
    ///     }],
    ///     metadata: None,
    /// };
    ///
    /// // Create the parameters
    /// let params = TaskSendParams {
    ///     id: "task_123".to_string(),
    ///     message,
    ///     session_id: None,
    ///     push_notification: None,
    ///     history_length: None,
    ///     metadata: None,
    /// };
    ///
    /// // Create the streaming request
    /// let request = params.into_send_subscribe_request(serde_json::json!("stream-req"));
    ///
    /// // Stream the response and process each event
    /// let mut stream = client.stream(request).await?;
    /// while let Some(event) = stream.next().await {
    ///     match event {
    ///         Ok(response) => {
    ///             if let Some(content) = &response.result {
    ///                 match content {
    ///                     StreamingResponseContent::StatusUpdate(status_event) => {
    ///                         println!("Status: {:?}", status_event.status);
    ///                     }
    ///                     StreamingResponseContent::ArtifactUpdate(artifact_event) => {
    ///                         println!("Artifact: {:?}", artifact_event.artifact);
    ///                     }
    ///                 }
    ///             } else if let Some(error) = &response.error {
    ///                 println!("Error: {:?}", error);
    ///             }
    ///         }
    ///         Err(e) => {
    ///             println!("Stream error: {:?}", e);
    ///             break;
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stream<P>(
        &self,
        request: JsonRpcRequest<P>,
    ) -> Result<
        Pin<
            Box<dyn Stream<Item = Result<JsonRpcResponse<StreamingResponseContent>, Error>> + Send>,
        >,
        Error,
    >
    where
        P: Serialize,
    {
        // Create the URL
        let server_url = self.config.server_url.clone();
        let auth_token = self.config.auth_token.clone();

        // Verify this is a streaming request
        let method = request.method();
        if method != "tasks/sendSubscribe" && method != "tasks/resubscribe" {
            return Err(Error::InvalidStreamingMethod(method.to_string()));
        }

        // Create a new request builder
        let mut builder = self.client.post(&server_url).json(&request);

        // Add authentication if provided
        if let Some(token) = auth_token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }

        // Set to accept SSE
        builder = builder.header("Accept", "text/event-stream");

        // Send the request
        let response = builder
            .send()
            .await?
            .error_for_status()
            .map_err(Error::HttpError)?;

        // Check content type
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");

        if !content_type.contains("text/event-stream") {
            return Err(Error::ContentTypeNotSupported(content_type.to_string()));
        }

        // Create a stream that parses SSE events
        let stream = response
            .bytes_stream()
            .map(|result| result.map_err(Error::HttpError))
            .boxed();

        // Create a stream processor that converts bytes to SSE events and parses them as JSON-RPC
        let event_stream = futures::stream::unfold(
            (stream, Vec::new()),
            |(mut stream, mut buffer)| async move {
                while let Some(chunk_result) = stream.next().await {
                    match chunk_result {
                        Ok(chunk) => {
                            // Append the chunk to our buffer
                            buffer.extend_from_slice(&chunk);

                            // Look for the double newline sequence in bytes (\n\n)
                            let newline_seq = b"\n\n";
                            if let Some(pos) = find_subsequence(&buffer, newline_seq) {
                                // First check if the event starts with "data: "
                                let data_prefix = b"data: ";

                                // Create a temporary clone of the relevant part of the event data
                                let event_data = buffer[..pos].to_vec();

                                // Remove the processed data from the buffer (no longer borrowing)
                                buffer = buffer[(pos + 2)..].to_vec();

                                // Now check if the event data starts with "data: "
                                if event_data.len() > data_prefix.len()
                                    && &event_data[..data_prefix.len()] == data_prefix
                                {
                                    // Get the JSON content after "data: "
                                    let json_data = &event_data[data_prefix.len()..];

                                    // Parse the event as a streaming response
                                    match serde_json::from_slice::<
                                        JsonRpcResponse<StreamingResponseContent>,
                                    >(json_data)
                                    {
                                        Ok(response) => {
                                            return Some((Ok(response), (stream, buffer)));
                                        }
                                        Err(e) => {
                                            return Some((
                                                Err(Error::SerializationError(e)),
                                                (stream, buffer),
                                            ));
                                        }
                                    }
                                }
                            }

                            // If we didn't find a complete event, continue to the next chunk
                        }
                        Err(e) => {
                            return Some((Err(e), (stream, buffer)));
                        }
                    }
                }

                // End of stream
                None
            },
        )
        .boxed();

        Ok(event_stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentCapabilities, AgentProvider, Task};
    use crate::{
        JsonRpcRequest, Message, MessageRole, Part, PushNotificationConfig, TaskQueryParams,
        TaskSendParams, TaskState,
        server::{
            MethodRouter, RequestHandler, Server, ServerConfig, make_typed_handler,
            task_manager::TaskManager,
        },
    };
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio::sync::Mutex;
    use uuid::Uuid;

    // Helper struct to auto-abort server tasks on drop
    struct ServerGuard {
        handle: tokio::task::JoinHandle<()>,
    }

    impl ServerGuard {
        fn new(handle: tokio::task::JoinHandle<()>) -> Self {
            Self { handle }
        }
    }

    impl Drop for ServerGuard {
        fn drop(&mut self) {
            self.handle.abort();
        }
    }

    #[test]
    fn client_config_default_values() {
        let config = ClientConfig::new("http://localhost:8080");

        assert_eq!(config.server_url, "http://localhost:8080");
        assert_eq!(config.timeout, DEFAULT_TIMEOUT);
        assert!(config.auth_token.is_none());
    }

    #[test]
    fn client_config_with_methods() {
        let token = "test-token";
        let timeout = Duration::from_secs(60);

        let config = ClientConfig::new("https://api.example.com")
            .with_timeout(timeout)
            .with_auth_token(token);

        assert_eq!(config.server_url, "https://api.example.com");
        assert_eq!(config.timeout, timeout);
        assert_eq!(config.auth_token, Some(token.to_string()));
    }

    // A simple handler for tasks/send
    #[derive(Clone)]
    struct SimpleTaskHandler {
        task_manager: Arc<TaskManager>,
        // For SSE streaming tests, track subscribers to send events to
        subscribers: Arc<Mutex<Vec<uuid::Uuid>>>,
    }

    impl SimpleTaskHandler {
        fn new(task_manager: Arc<TaskManager>, subscribers: Arc<Mutex<Vec<uuid::Uuid>>>) -> Self {
            Self {
                task_manager,
                subscribers,
            }
        }
    }

    impl RequestHandler for SimpleTaskHandler {
        fn handle(
            &self,
            request: serde_json::Value,
        ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, crate::JsonRpcError>> + Send + '_>>
        {
            let task_manager = Arc::clone(&self.task_manager);
            let subscribers: Arc<Mutex<Vec<uuid::Uuid>>> = Arc::clone(&self.subscribers);

            Box::pin(async move {
                // Parse the request
                let request: JsonRpcRequest<TaskSendParams> =
                    serde_json::from_value(request).map_err(JsonRpcError::invalid_parameters)?;

                // Process the task
                task_manager
                    .upsert_task(&request.params)
                    .await
                    .map_err(JsonRpcError::internal_error)?;

                // Get the task to return
                let task_query = TaskQueryParams::from_id(request.params.id.clone());
                let task = task_manager
                    .get_task(&task_query)
                    .await
                    .map_err(JsonRpcError::internal_error)?;

                // If this is a streaming request, store the request ID for SSE events
                if request.method() == "tasks/sendSubscribe" {
                    // Parse the task id from the request
                    if let Some(id) = request.id() {
                        if let Some(id_str) = id.as_str() {
                            if let Ok(uuid) = uuid::Uuid::parse_str(id_str) {
                                let mut subs = subscribers.lock().await;
                                subs.push(uuid);
                            }
                        }
                    }
                }

                // Return the task
                serde_json::to_value(task).map_err(JsonRpcError::serialization)
            })
        }
    }

    // A simple handler for tasks/get and streaming
    struct TaskGetHandler {
        task_manager: Arc<TaskManager>,
    }

    impl TaskGetHandler {
        fn new(task_manager: Arc<TaskManager>) -> Self {
            Self { task_manager }
        }
    }

    impl RequestHandler for TaskGetHandler {
        fn handle(
            &self,
            request: serde_json::Value,
        ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, crate::JsonRpcError>> + Send + '_>>
        {
            let task_manager = Arc::clone(&self.task_manager);

            Box::pin(async move {
                // Parse the request
                let request: JsonRpcRequest<TaskQueryParams> =
                    serde_json::from_value(request).map_err(JsonRpcError::invalid_parameters)?;

                // Get the task
                let task = task_manager
                    .get_task(&request.params)
                    .await
                    .map_err(JsonRpcError::internal_error)?;

                // Return the task
                serde_json::to_value(task).map_err(JsonRpcError::serialization)
            })
        }
    }

    // Start a test server on a random port and ensure it's running - with subscribers for streaming
    async fn start_test_server_with_subscribers() -> (
        tokio::task::JoinHandle<()>,
        u16,
        Arc<TaskManager>,
        Arc<Mutex<Vec<uuid::Uuid>>>,
    ) {
        // Create a task manager
        let task_manager = Arc::new(TaskManager::new().unwrap());

        // Create a subscribers list for streaming tests
        let subscribers = Arc::new(Mutex::new(Vec::new()));

        // Create handlers
        let task_send_handler =
            SimpleTaskHandler::new(Arc::clone(&task_manager), Arc::clone(&subscribers));
        let task_get_handler = TaskGetHandler::new(Arc::clone(&task_manager));

        // Create push notification handlers
        let get_push_notification_handler = make_typed_handler(
            Arc::clone(&task_manager),
            crate::server::push_notification_handlers::get_push_notification,
        );
        let set_push_notification_handler = make_typed_handler(
            Arc::clone(&task_manager),
            crate::server::push_notification_handlers::set_push_notification,
        );

        // Create a method router and register handlers
        let mut router = MethodRouter::new();
        router.register("tasks/send", task_send_handler.clone());
        router.register("tasks/sendSubscribe", task_send_handler);
        router.register("tasks/get", task_get_handler);
        router.register("tasks/pushNotification/get", get_push_notification_handler);
        router.register("tasks/pushNotification/set", set_push_notification_handler);

        // Bind to a random port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener); // Close the listener to avoid address already in use errors

        // Create server
        let server_config = ServerConfig::new(&format!("127.0.0.1:{}", port)).unwrap();
        let server = Server::new(server_config, Arc::new(router));

        // Start the server in a separate task
        let server_handle = tokio::spawn(async move {
            println!("Starting A2A server on http://127.0.0.1:{}", port);
            server.serve().await.unwrap();
        });

        // Give the server a moment to start up
        tokio::time::sleep(Duration::from_millis(500)).await;

        (server_handle, port, task_manager, subscribers)
    }

    // Start a server specifically for testing agent card fetch
    async fn start_agent_card_test_server() -> (tokio::task::JoinHandle<()>, u16) {
        // Create a task manager (needed for server, but not used for this test)
        let _task_manager = Arc::new(TaskManager::new().unwrap());

        // Create a router
        let mut router = MethodRouter::new();

        // Add a special handler just for the agent card test
        // This handler will be accessible at /.well-known/agent.json
        struct WellKnownHandler {
            agent_card: crate::AgentCard,
        }

        impl RequestHandler for WellKnownHandler {
            fn handle(
                &self,
                _request: serde_json::Value,
            ) -> Pin<
                Box<
                    dyn Future<Output = Result<serde_json::Value, crate::JsonRpcError>> + Send + '_,
                >,
            > {
                let agent_card = self.agent_card.clone();
                Box::pin(async move {
                    // Return the agent card
                    serde_json::to_value(agent_card).map_err(JsonRpcError::serialization)
                })
            }
        }

        let agent_card = AgentCard {
            name: "Test Agent".to_string(),
            description: Some("A test agent for client integration tests".to_string()),
            url: "https://example.com".to_string(),
            provider: Some(AgentProvider {
                organization: "Test Org".to_string(),
                url: Some("https://example.com".to_string()),
            }),
            version: "1.0.0".to_string(),
            documentation_url: Some("https://example.com/docs".to_string()),
            capabilities: AgentCapabilities {
                streaming: true,
                push_notifications: true,
                state_transition_history: true,
            },
            authentication: None,
            default_input_modes: vec!["text".to_string()],
            default_output_modes: vec!["text".to_string()],
            skills: Vec::new(),
        };

        router.register(
            ".well-known/agent.json",
            WellKnownHandler {
                agent_card: agent_card.clone(),
            },
        );

        // Bind to a random port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener); // Close the listener to avoid address already in use errors

        // Create server
        let server_config = ServerConfig::new(&format!("127.0.0.1:{}", port)).unwrap();
        // Create the server with the agent card directly in addition to the router handler
        let server = Server::new(server_config, Arc::new(router)).with_agent_card(agent_card);

        // Start the server
        let server_handle = tokio::spawn(async move {
            println!("Starting agent card server on http://127.0.0.1:{}", port);
            server.serve().await.unwrap();
        });

        // Give the server a moment to start
        tokio::time::sleep(Duration::from_millis(500)).await;

        (server_handle, port)
    }

    #[tokio::test]
    async fn client_send() {
        // Start a test server
        let (server_handle, port, _task_manager, _subscribers) =
            start_test_server_with_subscribers().await;
        let _server_guard = ServerGuard::new(server_handle);

        // Create a client
        let client_config = ClientConfig::new(&format!("http://127.0.0.1:{}", port));
        let client = Client::new(client_config).unwrap();

        // Create a task
        let task_id = Uuid::new_v4().to_string();

        // Create a message
        let message = Message {
            role: MessageRole::User,
            parts: vec![Part::Text {
                text: "Test message".to_string(),
                metadata: None,
            }],
            metadata: None,
        };

        // Create a request
        let params = TaskSendParams {
            id: task_id.clone(),
            message,
            session_id: None,
            push_notification: None,
            history_length: None,
            metadata: None,
        };

        let request = params.into_send_request(serde_json::json!(task_id));

        // Send the request
        let response: Task = client.send(request).await.unwrap();

        // Verify the response
        assert_eq!(response.id, task_id);
        assert_eq!(response.status.state, TaskState::Submitted);

        // ServerGuard will clean up the server on drop
    }

    #[tokio::test]
    async fn client_push_notification() {
        // Create a webhook endpoint
        let webhook_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let webhook_port = webhook_listener.local_addr().unwrap().port();
        let webhook_url = format!("http://127.0.0.1:{}", webhook_port);

        // Start a webhook server that properly responds to validation requests
        let webhook_handle = tokio::spawn(async move {
            loop {
                if let Ok((stream, _)) = webhook_listener.accept().await {
                    let io = hyper_util::rt::TokioIo::new(stream);

                    tokio::spawn(async move {
                        // Create a service that echoes back validation tokens
                        let service = hyper::service::service_fn(|req| async move {
                            // Parse the query parameters to look for validationToken
                            if let Some(query) = req.uri().query() {
                                if let Some(token_part) = query
                                    .split('&')
                                    .find(|part| part.starts_with("validationToken="))
                                {
                                    if let Some(token) = token_part.strip_prefix("validationToken=")
                                    {
                                        // For validation requests, echo the token back
                                        return Ok::<_, hyper::Error>(
                                            hyper::Response::builder()
                                                .status(hyper::StatusCode::OK)
                                                .body(http_body_util::Full::new(
                                                    bytes::Bytes::from(token.to_string()),
                                                ))
                                                .unwrap(),
                                        );
                                    }
                                }
                            }

                            // For normal requests, just return OK
                            Ok::<_, hyper::Error>(
                                hyper::Response::builder()
                                    .status(hyper::StatusCode::OK)
                                    .body(http_body_util::Full::new(bytes::Bytes::from("OK")))
                                    .unwrap(),
                            )
                        });

                        let _ = hyper::server::conn::http1::Builder::new()
                            .serve_connection(io, service)
                            .await;
                    });
                }
            }
        });

        let _webhook_guard = ServerGuard::new(webhook_handle);

        // Start a test server on a random port
        let (server_handle, port, task_manager, _subscribers) =
            start_test_server_with_subscribers().await;

        let _server_guard = ServerGuard::new(server_handle);

        // Create a client
        let client_config = ClientConfig::new(&format!("http://127.0.0.1:{}", port));
        let client = Client::new(client_config).unwrap();

        // Create a task first so that it exists
        let task_id = Uuid::new_v4().to_string();
        let message = Message {
            role: MessageRole::User,
            parts: vec![Part::Text {
                text: "Test message".to_string(),
                metadata: None,
            }],
            metadata: None,
        };

        // Create a task through the task manager
        let params = TaskSendParams {
            id: task_id.clone(),
            message,
            session_id: None,
            push_notification: None,
            history_length: None,
            metadata: None,
        };

        // Insert the task manually
        task_manager.upsert_task(&params).await.unwrap();

        // Create a push notification config with our local webhook
        let push_config = PushNotificationConfig {
            url: webhook_url,
            token: Some("test-token".to_string()),
            authentication: None,
        };

        // Set the push notification
        let response = match client
            .set_push_notification(task_id.clone(), push_config.clone())
            .await
        {
            Ok(r) => r,
            Err(e) => {
                // Print detailed error information for debugging
                println!("Error setting push notification: {:?}", e);

                // If it's a serialization error, print more details
                if let Error::SerializationError(ser_err) = &e {
                    println!("Serialization error details: {}", ser_err);

                    // Create the JSON directly to see what we're dealing with
                    let params = crate::TaskPushNotificationConfig {
                        id: task_id.clone(),
                        push_notification_config: push_config.clone(),
                    };

                    match serde_json::to_string_pretty(&params) {
                        Ok(json) => println!("Original JSON we tried to send:\n{}", json),
                        Err(e) => println!("Error serializing params: {}", e),
                    }
                }

                panic!("Failed to set push notification: {:?}", e);
            }
        };

        // Verify the response
        assert_eq!(response.id, task_id);
        assert_eq!(response.push_notification_config.url, push_config.url);
        assert_eq!(response.push_notification_config.token, push_config.token);

        // Get the push notification
        let get_response = client.get_push_notification(task_id.clone()).await.unwrap();

        // Verify the response
        assert_eq!(get_response.id, task_id);
        assert_eq!(get_response.push_notification_config.url, push_config.url);
        assert_eq!(
            get_response.push_notification_config.token,
            push_config.token
        );

        // ServerGuard will clean up both servers on drop
    }

    #[tokio::test]
    async fn client_fetch_agent_card() {
        // Start a server on a random port just for the agent card
        let (server_handle, port) = start_agent_card_test_server().await;
        let _server_guard = ServerGuard::new(server_handle);

        // Create a client
        let client_config = ClientConfig::new(&format!("http://127.0.0.1:{}", port));
        let client = Client::new(client_config).unwrap();

        // Fetch the agent card
        let agent_card = client.fetch_agent_card().await.unwrap();

        // Verify the agent card
        assert_eq!(agent_card.name, "Test Agent");
        assert_eq!(
            agent_card.description,
            Some("A test agent for client integration tests".to_string())
        );
        assert_eq!(agent_card.version, "1.0.0");

        // ServerGuard will clean up the server on drop
    }

    // Stream test requires more complex setup, skipping for now
    #[tokio::test]
    async fn client_stream_basic() {
        // Start a test server on a random port
        let (server_handle, port, _task_manager, _subscribers) =
            start_test_server_with_subscribers().await;
        let _server_guard = ServerGuard::new(server_handle);

        // Create a client
        let client_config = ClientConfig::new(&format!("http://127.0.0.1:{}", port));
        let client = Client::new(client_config).unwrap();

        // Create a task
        let task_id = Uuid::new_v4().to_string();

        // Create a message
        let message = Message {
            role: MessageRole::User,
            parts: vec![Part::Text {
                text: "Test streaming message".to_string(),
                metadata: None,
            }],
            metadata: None,
        };

        // Create a streaming request
        let params = TaskSendParams {
            id: task_id.clone(),
            message,
            session_id: None,
            push_notification: None,
            history_length: None,
            metadata: None,
        };

        let request =
            params.into_send_subscribe_request(serde_json::json!(Uuid::new_v4().to_string()));

        // Test that we can create a stream (actual streaming events would require more setup)
        let _stream = client.stream(request).await.unwrap();

        // ServerGuard will clean up the server on drop
    }
}
