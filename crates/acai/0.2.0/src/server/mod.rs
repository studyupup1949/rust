//! A2A Protocol server implementation
//!
//! This module provides a server for handling A2A Protocol requests.
//! It includes routing, request handling, and response generation.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use http_body_util::{BodyExt, BodyStream, Full, combinators::BoxBody};
use hyper::service::Service;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tower::BoxError;

use crate::server::jwks::JwksManager;
use crate::server::jwks_handler::JwksHandler;
use crate::types::{AgentCard, JsonRpcError, JsonRpcRequest, JsonRpcResponse};

// Public modules for server components
pub mod handlers;
pub mod jwks;
pub mod jwks_handler;
pub mod push_notification;
pub mod push_notification_handlers;
pub mod streaming;
pub mod task_manager;

type ResponseFuture = Pin<
    Box<dyn Future<Output = Result<hyper::Response<BoxBody<Bytes, BoxError>>, BoxError>> + Send>,
>;

/// Server error types
#[derive(Debug)]
pub enum Error {
    /// HTTP server error
    HttpError(hyper::Error),

    /// JSON-RPC error
    JsonRpcError(JsonRpcError),

    /// Other errors
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Self {
        Error::HttpError(err)
    }
}

/// Handler for processing A2A requests
pub trait RequestHandler: Send + Sync + 'static {
    /// Handle a request and return a response
    fn handle(
        &self,
        request: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, JsonRpcError>> + Send + '_>>;
}

impl RequestHandler for () {
    fn handle(
        &self,
        _request: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, JsonRpcError>> + Send + '_>> {
        Box::pin(async move {
            let err = JsonRpcError::internal_error("nothing supported");
            serde_json::to_value(err).map_err(JsonRpcError::serialization)
        })
    }
}

/// A simple handler that delegates to method-specific handlers
#[derive(Default)]
pub struct MethodRouter {
    /// Handlers mapped by method name
    handlers: std::collections::HashMap<String, Arc<dyn RequestHandler>>,
}

impl MethodRouter {
    /// Create a new empty router
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }

    /// Register a handler for a specific method
    pub fn register<H>(&mut self, method: &str, handler: H) -> &mut Self
    where
        H: RequestHandler,
    {
        self.handlers.insert(method.to_string(), Arc::new(handler));
        self
    }
}

impl RequestHandler for MethodRouter {
    fn handle(
        &self,
        request: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, JsonRpcError>> + Send + '_>> {
        Box::pin(async move {
            let method = request.get("method").and_then(|m| m.as_str());

            let method = match method {
                Some(m) => m,
                None => {
                    return Err(JsonRpcError::invalid_request("Method field is required"));
                }
            };

            let handler = match self.handlers.get(method) {
                Some(h) => h,
                None => {
                    return Err(JsonRpcError::method_not_found(method));
                }
            };

            handler.handle(request).await
        })
    }
}

pub fn make_typed_handler<
    'a,
    S: Send + Sync + 'static + ?Sized,
    P: serde::de::DeserializeOwned + Send + 'static,
    R: serde::Serialize + 'static,
    Fut: Future<Output = Result<R, JsonRpcError>> + Send + 'static,
>(
    state: Arc<S>,
    handler: fn(Arc<S>, P) -> Fut,
) -> impl RequestHandler + 'a {
    struct TypedRequestHandler<
        S: Send + Sync + 'static + ?Sized,
        P: serde::de::DeserializeOwned + 'static,
        R: serde::Serialize + 'static,
        Fut: Future<Output = Result<R, JsonRpcError>> + 'static,
    > {
        state: Arc<S>,
        handler: fn(Arc<S>, P) -> Fut,
    }
    impl<
        S: Send + Sync + 'static + ?Sized,
        P: serde::de::DeserializeOwned + Send + 'static,
        R: serde::Serialize + 'static,
        Fut: Future<Output = Result<R, JsonRpcError>> + Send + 'static,
    > RequestHandler for TypedRequestHandler<S, P, R, Fut>
    {
        fn handle(
            &self,
            request: serde_json::Value,
        ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, JsonRpcError>> + Send + '_>>
        {
            let state = Arc::clone(&self.state);
            let handler = self.handler;
            Box::pin(async move {
                // Parse the request
                let json_rpc_request: JsonRpcRequest<P> = match serde_json::from_value(request) {
                    Ok(req) => req,
                    Err(e) => {
                        return Err(JsonRpcError::invalid_parameters(e));
                    }
                };

                // Call the handler
                let result = (handler)(Arc::clone(&state), json_rpc_request.params).await?;

                // Serialize the result
                let result_value = match serde_json::to_value(result) {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(JsonRpcError::serialization(e));
                    }
                };

                Ok(result_value)
            })
        }
    }
    TypedRequestHandler { state, handler }
}

/// Service that processes HTTP requests and routes them to the appropriate handler
#[derive(Clone)]
pub struct AgentService {
    /// The request handler
    handler: Arc<dyn RequestHandler>,
    /// Optional agent card for discovery
    agent_card: Option<Arc<AgentCard>>,
    /// Optional JWKS manager for authentication
    jwks_manager: Option<Arc<JwksManager>>,
}

impl AgentService {
    /// Create a new service with the given handler
    pub fn new(handler: Arc<dyn RequestHandler>) -> Self {
        Self {
            handler,
            agent_card: None,
            jwks_manager: None,
        }
    }

    /// Add an agent card to the service
    pub fn with_agent_card(mut self, agent_card: AgentCard) -> Self {
        self.agent_card = Some(Arc::new(agent_card));
        self
    }

    /// Add a JWKS manager to the service
    pub fn with_jwks_manager(mut self, jwks_manager: Arc<JwksManager>) -> Self {
        self.jwks_manager = Some(jwks_manager);
        self
    }
}

impl Service<Request<hyper::body::Incoming>> for AgentService {
    type Response = Response<BoxBody<Bytes, BoxError>>;
    type Error = BoxError;
    type Future = ResponseFuture;

    fn call(&self, req: Request<hyper::body::Incoming>) -> Self::Future {
        let handler = self.handler.clone();
        let agent_card = self.agent_card.clone();
        let jwks_manager = self.jwks_manager.clone();
        let path = req.uri().path().to_string();

        Box::pin(async move {
            // Check for agent card requests at the well-known location
            if path == "/.well-known/agent.json" || path == "/.well-known/agent" {
                if let Some(card) = agent_card {
                    match serde_json::to_string(&*card) {
                        Ok(json) => {
                            return Response::builder()
                                .status(StatusCode::OK)
                                .header("Content-Type", "application/json")
                                .body(full_body(json))
                                .map_err(|e| {
                                    BoxError::from(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        format!("Failed to build response: {}", e),
                                    ))
                                });
                        }
                        Err(e) => {
                            return Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(full_body(format!("Failed to serialize agent card: {}", e)))
                                .map_err(|e| {
                                    BoxError::from(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        format!("Failed to build error response: {}", e),
                                    ))
                                });
                        }
                    }
                } else {
                    return Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(full_body("Agent card not configured"))
                        .map_err(|e| {
                            BoxError::from(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Failed to build not found response: {}", e),
                            ))
                        });
                }
            }

            // Check for JWKS endpoint requests
            if path == "/.well-known/jwks.json" {
                if let Some(jwks) = jwks_manager {
                    let jwks_handler = JwksHandler::new(jwks);
                    return jwks_handler.handle_request(req).await;
                } else {
                    return Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(full_body("JWKS endpoint not configured"))
                        .map_err(|e| {
                            BoxError::from(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Failed to build not found response: {}", e),
                            ))
                        });
                }
            }

            // Handle regular API requests
            if req.method() != Method::POST {
                return Response::builder()
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .body(full_body("Method not allowed"))
                    .map_err(|e| {
                        BoxError::from(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to build method not allowed response: {}", e),
                        ))
                    });
            }

            // Parse the request body
            let body_bytes = req.collect().await?.to_bytes();
            let request: serde_json::Value = match serde_json::from_slice(&body_bytes) {
                Ok(req) => req,
                Err(e) => {
                    let error = JsonRpcError::parse_error(e);
                    return Ok(json_error_response(error, None));
                }
            };

            // Extract the request ID
            let id = request.get("id").cloned();

            // Check if this is a streaming request (tasks/sendSubscribe)
            let is_streaming = request
                .get("method")
                .and_then(|m| m.as_str())
                .map(|m| m == "tasks/sendSubscribe" || m == "tasks/resubscribe")
                .unwrap_or(false);

            // Handle the request
            if is_streaming {
                // For streaming requests, we use Server-Sent Events (SSE)
                match handler.handle(request).await {
                    Ok(result) => {
                        // Create the initial response
                        let initial_response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: id.clone(),
                            result: Some(result),
                            error: None,
                        };

                        // Log streaming response
                        indicio::clue!(crate::COLLECTOR, indicio::DEBUG, {
                            event: "streaming_response_serialization_start",
                        });

                        let json = match serde_json::to_string(&initial_response) {
                            Ok(j) => {
                                if j.len() < 1024 {
                                    indicio::clue!(crate::COLLECTOR, indicio::DEBUG, {
                                        event: "streaming_response_serialized",
                                        json_length: j.len(),
                                        json: &j,
                                    });
                                } else {
                                    indicio::clue!(crate::COLLECTOR, indicio::DEBUG, {
                                        event: "streaming_response_serialized",
                                        json_length: j.len(),
                                    });
                                }
                                j
                            }
                            Err(e) => {
                                indicio::clue!(crate::COLLECTOR, indicio::ERROR, {
                                    event: "streaming_response_serialization_error",
                                    error: e.to_string(),
                                });
                                let error = JsonRpcError::serialization(e);
                                return Ok(json_error_response(error, id));
                            }
                        };

                        // Create a broadcast channel for streaming updates
                        let (sender, _) = tokio::sync::broadcast::channel(100);

                        // Send the initial message
                        let _ = sender.send(json);

                        // Set up the SSE response using our streaming body
                        match sse_response(&sender) {
                            Ok(response) => {
                                // Store the sender in a task-specific location to allow updates
                                // This is where you would typically store the sender to update later
                                // For example, store it in a task manager with the task ID as the key

                                Ok(response)
                            }
                            Err(e) => {
                                indicio::clue!(crate::COLLECTOR, indicio::ERROR, {
                                    event: "streaming_response_build_error",
                                    error: e.to_string(),
                                });
                                Err(BoxError::from(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    format!("Failed to build SSE response: {}", e),
                                )))
                            }
                        }
                    }
                    Err(error) => {
                        indicio::clue!(crate::COLLECTOR, indicio::ERROR, {
                            event: "request_handler_error",
                            error_code: error.code,
                            error_message: &error.message,
                        });
                        Ok(json_error_response(error, id))
                    }
                }
            } else {
                // For non-streaming requests, use standard JSON-RPC
                match handler.handle(request).await {
                    Ok(result) => {
                        let response: JsonRpcResponse<serde_json::Value> = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: id.clone(),
                            result: Some(result),
                            error: None,
                        };

                        // Log the response before serializing
                        indicio::clue!(crate::COLLECTOR, indicio::DEBUG, {
                            event: "response_serialization_start",
                            response_type: "standard",
                        });

                        let json = match serde_json::to_string(&response) {
                            Ok(j) => {
                                if j.len() < 1024 {
                                    indicio::clue!(crate::COLLECTOR, indicio::DEBUG, {
                                        event: "response_serialized",
                                        json_length: j.len(),
                                        json: &j,
                                    });
                                } else {
                                    indicio::clue!(crate::COLLECTOR, indicio::DEBUG, {
                                        event: "response_serialized",
                                        json_length: j.len(),
                                    });
                                }
                                j
                            }
                            Err(e) => {
                                indicio::clue!(crate::COLLECTOR, indicio::ERROR, {
                                    event: "response_serialization_error",
                                    error: e.to_string(),
                                });
                                let error = JsonRpcError::serialization(e);
                                return Ok(json_error_response(error, id));
                            }
                        };

                        Response::builder()
                            .status(StatusCode::OK)
                            .header("Content-Type", "application/json")
                            .body(full_body(json))
                            .map_err(|e| {
                                indicio::clue!(crate::COLLECTOR, indicio::ERROR, {
                                    event: "response_build_error",
                                    error: e.to_string(),
                                });
                                BoxError::from(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    format!("Failed to build JSON response: {}", e),
                                ))
                            })
                    }
                    Err(error) => {
                        indicio::clue!(crate::COLLECTOR, indicio::ERROR, {
                            event: "request_handler_error",
                            error_code: error.code,
                            error_message: &error.message,
                        });
                        Ok(json_error_response(error, id))
                    }
                }
            }
        })
    }
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// The address to bind to
    pub addr: std::net::SocketAddr,
}

impl ServerConfig {
    /// Create a new server configuration with the given address string
    ///
    /// # Errors
    /// Returns an error if the address cannot be parsed as a socket address
    pub fn new(addr: &str) -> Result<Self, std::net::AddrParseError> {
        Ok(Self {
            addr: addr.parse()?,
        })
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        // This is a valid address format so it's safe to use unwrap in the default implementation
        Self {
            addr: "127.0.0.1:8080"
                .parse()
                .expect("Hardcoded valid address should never fail to parse"),
        }
    }
}

/// A2A Protocol server
pub struct Server {
    /// Server configuration
    config: ServerConfig,

    /// Request handler
    handler: Arc<dyn RequestHandler>,

    /// Optional agent card for discovery
    agent_card: Option<AgentCard>,

    /// Optional JWKS manager for authentication
    jwks_manager: Option<Arc<JwksManager>>,
}

impl Server {
    /// Create a new server with the given configuration and handler
    pub fn new(config: ServerConfig, handler: Arc<dyn RequestHandler>) -> Self {
        Self {
            config,
            handler,
            agent_card: None,
            jwks_manager: None,
        }
    }

    /// Add an agent card to the server
    pub fn with_agent_card(mut self, agent_card: AgentCard) -> Self {
        self.agent_card = Some(agent_card);
        self
    }

    /// Add JWKS support to the server with the provided key pair configurations
    pub fn with_jwks(
        mut self,
        key_configs: Vec<jwks::KeyPairConfig>,
    ) -> Result<Self, crate::server::jwks::JwksError> {
        let jwks_manager = JwksManager::new(key_configs)?;
        self.jwks_manager = Some(Arc::new(jwks_manager));
        Ok(self)
    }

    /// Add a specific JWKS manager to the server
    pub fn with_jwks_manager(mut self, jwks_manager: Arc<JwksManager>) -> Self {
        self.jwks_manager = Some(jwks_manager);
        self
    }

    /// Helper to create a new server with a string address
    ///
    /// # Errors
    /// Returns an error if the address cannot be parsed
    pub fn new_with_address(
        addr: &str,
        handler: Arc<dyn RequestHandler>,
    ) -> Result<Self, std::net::AddrParseError> {
        let config = ServerConfig::new(addr)?;
        Ok(Self::new(config, handler))
    }

    /// Start the server and run until stopped
    pub async fn serve(&self) -> Result<(), Error> {
        let addr = self.config.addr;
        indicio::clue!(crate::COLLECTOR, indicio::ALWAYS, {
            event: "server_starting",
            address: format!("http://{}", addr),
            protocols: ["HTTP/1.1", "HTTP/2"],
        });
        println!("Starting Agent server on http://{}", addr);

        // Create the service with the appropriate configuration
        let mut service = AgentService::new(Arc::clone(&self.handler));

        // Add agent card if available
        if let Some(card) = &self.agent_card {
            service = service.with_agent_card(card.clone());
        }

        // Add JWKS manager if available
        if let Some(jwks) = &self.jwks_manager {
            service = service.with_jwks_manager(Arc::clone(jwks));
        };

        // Create a TCP listener
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| Error::Other(format!("Failed to bind to {}: {}", addr, e)))?;

        // Accept connections
        loop {
            let (stream, remote_addr) = listener
                .accept()
                .await
                .map_err(|e| Error::Other(format!("Failed to accept connection: {}", e)))?;

            // Clone the service for this connection
            let service_clone = service.clone();

            // Spawn a task to handle the connection
            tokio::spawn(async move {
                indicio::clue!(crate::COLLECTOR, indicio::DEBUG, {
                    event: "connection_accepted",
                    remote_addr: format!("{}", remote_addr),
                });

                let io = TokioIo::new(stream);

                indicio::clue!(crate::COLLECTOR, indicio::DEBUG, {
                    event: "connection_handling",
                });

                // Create an HTTP server supporting both HTTP/1 and HTTP/2
                let server = hyper_util::server::conn::auto::Builder::new(
                    hyper_util::rt::TokioExecutor::new(),
                );

                // Start the connection
                let conn = server.serve_connection(io, service_clone);

                // Wait for the connection to complete
                match conn.await {
                    Ok(_) => {
                        indicio::clue!(crate::COLLECTOR, indicio::DEBUG, {
                            event: "connection_closed",
                            status: "graceful",
                        });
                    }
                    Err(err) => {
                        indicio::clue!(crate::COLLECTOR, indicio::ERROR, {
                            event: "connection_error",
                            remote_addr: remote_addr.to_string(),
                            error: err.to_string(),
                        });
                    }
                }
            });
        }
    }
}

/// Helper function to create a full body from a string or bytes
pub fn full_body<T: Into<Bytes>>(body: T) -> BoxBody<Bytes, BoxError> {
    Full::new(body.into())
        .map_err(|never| match never {})
        .boxed()
}

/// Helper function to create an SSE response from a broadcast channel
pub fn sse_response(
    sender: &tokio::sync::broadcast::Sender<String>,
) -> Result<hyper::Response<BoxBody<Bytes, BoxError>>, BoxError> {
    use crate::server::streaming::SseBody;

    let body = SseBody::from_sender(sender);

    // Convert the SseBody to a BoxBody
    let boxed_body = BodyStream::new(body).boxed();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(boxed_body)
        .map_err(|e| {
            BoxError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to build SSE response: {}", e),
            ))
        })
}

// Helper function to create a JSON-RPC error response
fn json_error_response(
    error: JsonRpcError,
    id: Option<serde_json::Value>,
) -> hyper::Response<BoxBody<Bytes, BoxError>> {
    indicio::clue!(crate::COLLECTOR, indicio::ERROR, {
        event: "json_rpc_error_response",
        error_code: error.code,
        error_message: &error.message,
        request_id: id.as_ref().map(|v| v.to_string()).unwrap_or_else(|| "none".to_string()),
    });

    let response: JsonRpcResponse<serde_json::Value> = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(error),
    };

    match serde_json::to_string(&response) {
        Ok(json) => {
            if json.len() < 1024 {
                indicio::clue!(crate::COLLECTOR, indicio::DEBUG, {
                    event: "error_response_serialized",
                    json_length: json.len(),
                    json: &json,
                });
            } else {
                indicio::clue!(crate::COLLECTOR, indicio::DEBUG, {
                    event: "error_response_serialized",
                    json_length: json.len(),
                });
            }

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(full_body(json))
                .unwrap_or_else(|e| {
                    indicio::clue!(crate::COLLECTOR, indicio::ERROR, {
                        event: "error_response_build_error",
                        error: e.to_string(),
                    });
                    // If we can't build the response, return a minimal valid JSON-RPC error
                    Response::new(full_body(
                        r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}}"#,
                    ))
                })
        }
        Err(e) => {
            indicio::clue!(crate::COLLECTOR, indicio::ERROR, {
                event: "error_response_serialization_error",
                error: e.to_string(),
            });

            // If we can't serialize the error, return a valid JSON-RPC error
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(full_body(r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal JSON-RPC error"}}"#))
                .unwrap_or_else(|e| {
                    indicio::clue!(crate::COLLECTOR, indicio::ERROR, {
                        event: "fallback_error_response_build_error",
                        error: e.to_string(),
                    });
                    // Absolute fallback - still a valid JSON-RPC error
                    Response::new(full_body(r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"}}"#))
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.addr.to_string(), "127.0.0.1:8080");
    }
}
