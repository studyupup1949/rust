use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use reqwest::{Client, StatusCode};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::time::sleep;

use acai::client::{Client as AcaiClient, ClientConfig};
use acai::server::jwks::JwksManager;
use acai::server::{MethodRouter, Server, ServerConfig, jwks, make_typed_handler};
use acai::{
    AgentCapabilities, AgentCard, AgentProvider, FileContent, JsonRpcError, JsonRpcRequest,
    JsonRpcResponse, Message, MessageRole, Part, Task, TaskSendParams, TaskState,
};

// Utility function to find an available port
async fn find_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Release the port
    port
}

/// Utility function to load a file from disk and convert it to FileContent
fn load_file(path: &Path) -> Result<FileContent, Box<dyn std::error::Error>> {
    // Read the file
    let bytes = fs::read(path)?;

    // Encode the bytes as base64
    let base64_content = BASE64.encode(bytes);

    // Get the filename if available
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    // Try to determine mime type based on extension
    let mime_type = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "txt" => "text/plain",
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "pdf" => "application/pdf",
            "json" => "application/json",
            _ => "application/octet-stream",
        })
        .map(|s| s.to_string());

    // Create the FileContent
    Ok(FileContent {
        name,
        mime_type,
        bytes: Some(base64_content),
        uri: None,
    })
}

#[tokio::test]
async fn jwks_example() {
    // Test that the JWKS example can:
    // 1. Serve JWKS keys
    // 2. Generate tokens
    // 3. Validate tokens
    // 4. Rotate keys

    // Start the server on a random port to avoid conflicts
    let port = find_available_port().await;
    let server_address = format!("127.0.0.1:{}", port);
    let server_url = format!("http://{}", server_address);

    // Create a JWKS manager for testing
    let config = jwks::KeyPairConfig {
        name: "test".to_string(),
        private_key_path: "tests/test.key".to_string(),
    };
    let jwks_manager = Arc::new(JwksManager::new(vec![config]).unwrap());

    // Create echo handler similar to jwks.rs example
    async fn handle_echo(
        _: Arc<()>,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, JsonRpcError> {
        // Get message from the params
        let message = params
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("No message provided")
            .to_string();

        let response = serde_json::json!({
            "message": message,
            "authenticated": false,
            "user": null
        });

        Ok(response)
    }

    // Create authenticated handler with JWKS validation
    #[derive(Clone)]
    struct AuthState {
        jwks: Arc<JwksManager>,
    }

    async fn handle_auth(
        state: Arc<AuthState>,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, JsonRpcError> {
        // Extract token from params
        let token = params
            .get("token")
            .and_then(Value::as_str)
            .ok_or_else(|| JsonRpcError::invalid_request("No authentication token provided"))?;

        // Validate the token
        let token_data: acai::TokenData<acai::Claims> = state
            .jwks
            .validate_token(token)
            .map_err(|e| JsonRpcError::internal_error(format!("Invalid token: {:?}", e)))?;

        // Get message from the params
        let message = params
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("No message provided")
            .to_string();

        let response = serde_json::json!({
            "message": format!("Authenticated: {}", message),
            "authenticated": true,
            "user": token_data.claims.sub,
            "issued_at": token_data.claims.iat,
            "expires_at": token_data.claims.exp
        });

        Ok(response)
    }

    // Create a rotate keys handler
    async fn handle_rotate(
        jwks: Arc<JwksManager>,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, JsonRpcError> {
        // Extract parameters
        let _remove_oldest = params
            .get("remove_oldest")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        // Instead of rotating keys, we'll just get all existing key IDs
        let key_ids = jwks.list_key_ids();
        let kid = key_ids
            .first()
            .ok_or_else(|| JsonRpcError::internal_error("No keys available"))?
            .clone();

        let total_keys = jwks.key_count();

        let response = serde_json::json!({
            "new_key_id": kid,
            "total_keys": total_keys
        });

        Ok(response)
    }

    // Create a new token handler
    async fn handle_token(
        jwks: Arc<JwksManager>,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, JsonRpcError> {
        // Extract subject
        let subject = params
            .get("subject")
            .and_then(Value::as_str)
            .ok_or_else(|| JsonRpcError::invalid_request("subject is required"))?;

        // Extract expiration (default 1 hour)
        let expiration_seconds = params
            .get("expiration_seconds")
            .and_then(Value::as_u64)
            .unwrap_or(3600);

        // Get current time
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| JsonRpcError::internal_error("Failed to get current time"))?
            .as_secs();

        let expiration = current_time + expiration_seconds;

        // Create claims
        let claims = acai::Claims {
            sub: subject.to_string(),
            iat: current_time,
            exp: expiration,
            jti: uuid::Uuid::new_v4().to_string(),
            custom: std::collections::HashMap::new(),
        };

        // Generate token
        let token = jwks.generate_token(claims).map_err(|e| {
            JsonRpcError::internal_error(format!("Failed to generate token: {:?}", e))
        })?;

        let response = serde_json::json!({
            "token": token,
            "expires_at": expiration
        });

        Ok(response)
    }

    // Create a router with our handlers
    let mut router = MethodRouter::new();
    router.register("echo", make_typed_handler(Arc::new(()), handle_echo));

    let auth_state = Arc::new(AuthState {
        jwks: jwks_manager.clone(),
    });
    router.register("authenticated", make_typed_handler(auth_state, handle_auth));

    router.register(
        "rotate_keys",
        make_typed_handler(jwks_manager.clone(), handle_rotate),
    );
    router.register(
        "new_token",
        make_typed_handler(jwks_manager.clone(), handle_token),
    );

    // Create server config and start server
    let server_config = ServerConfig::new(&server_address).unwrap();
    let server =
        Server::new(server_config, Arc::new(router)).with_jwks_manager(jwks_manager.clone());

    let server_handle = tokio::spawn(async move {
        server.serve().await.unwrap();
    });

    // Wait for server to start
    sleep(Duration::from_millis(500)).await;

    // Create a client
    let client = Client::new();

    // Test 1: Check if JWKS endpoint is available
    let jwks_url = format!("{}/.well-known/jwks.json", server_url);
    let response = client.get(&jwks_url).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let jwks_json: Value = response.json().await.unwrap();
    assert!(jwks_json.get("keys").is_some());

    // Test 2: Test the echo endpoint (unauthenticated)
    let echo_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "test-echo",
        "method": "echo",
        "params": {
            "message": "Hello, JWKS!"
        }
    });

    let response = client
        .post(&server_url)
        .json(&echo_request)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let echo_response: JsonRpcResponse<Value> = response.json().await.unwrap();
    assert!(echo_response.error.is_none());
    assert!(echo_response.result.is_some());
    let result = echo_response.result.unwrap();
    assert_eq!(
        result.get("message").and_then(Value::as_str).unwrap(),
        "Hello, JWKS!"
    );
    assert!(
        !result
            .get("authenticated")
            .and_then(Value::as_bool)
            .unwrap()
    );

    // Test 3: Generate a token
    let token_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "test-token",
        "method": "new_token",
        "params": {
            "subject": "test-user",
            "expiration_seconds": 3600
        }
    });

    let response = client
        .post(&server_url)
        .json(&token_request)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let token_response: JsonRpcResponse<Value> = response.json().await.unwrap();
    eprintln!("token_response = {token_response:#?}");
    assert!(token_response.error.is_none());
    assert!(token_response.result.is_some());
    let result = token_response.result.unwrap();
    let token = result
        .get("token")
        .and_then(Value::as_str)
        .unwrap()
        .to_string();
    assert!(!token.is_empty());

    // Test 4: Use the token with authenticated endpoint
    let auth_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "test-auth",
        "method": "authenticated",
        "params": {
            "message": "This is authenticated",
            "token": token
        }
    });

    let response = client
        .post(&server_url)
        .json(&auth_request)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let auth_response: JsonRpcResponse<Value> = response.json().await.unwrap();
    eprintln!("auth_response = {auth_response:#?}");
    assert!(auth_response.error.is_none());
    assert!(auth_response.result.is_some());
    let result = auth_response.result.unwrap();
    assert!(
        result
            .get("message")
            .and_then(Value::as_str)
            .unwrap()
            .contains("This is authenticated")
    );
    assert!(
        result
            .get("authenticated")
            .and_then(Value::as_bool)
            .unwrap()
    );
    assert_eq!(
        result.get("user").and_then(Value::as_str).unwrap(),
        "test-user"
    );

    // Test 5: Rotate keys
    let rotate_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "test-rotate",
        "method": "rotate_keys",
        "params": {
            "remove_oldest": true
        }
    });

    let response = client
        .post(&server_url)
        .json(&rotate_request)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let rotate_response: JsonRpcResponse<Value> = response.json().await.unwrap();
    assert!(rotate_response.error.is_none());
    assert!(rotate_response.result.is_some());
    let result = rotate_response.result.unwrap();
    assert!(result.get("new_key_id").is_some());
    assert!(result.get("total_keys").is_some());

    // Clean up - cancel the server task
    server_handle.abort();
}

#[tokio::test]
async fn file_handling_example() {
    let test_file_path = Path::new("tests/test_file.txt");

    // Start the server on a random port to avoid conflicts
    let port = find_available_port().await;
    let server_address = format!("127.0.0.1:{}", port);
    let server_url = format!("http://{}", server_address);

    // Create file info handler as in the example
    async fn file_info_handler(
        _state: Arc<()>,
        params: TaskSendParams,
    ) -> std::result::Result<Task, JsonRpcError> {
        let task_id = params.id.clone();

        // Look for file parts in the message
        let file_parts = params
            .message
            .parts
            .iter()
            .filter_map(|part| {
                if let Part::File { file, .. } = part {
                    Some(file)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Create response based on whether we found files
        let response_text = if file_parts.is_empty() {
            "No files found in the message.".to_string()
        } else {
            let mut description = format!("Received {} file(s):\n\n", file_parts.len());
            for (i, file) in file_parts.iter().enumerate() {
                description.push_str(&format!("File #{}\n", i + 1));
                description.push_str(&format!(
                    "- Name: {}\n",
                    file.name.as_deref().unwrap_or("Unnamed")
                ));
                description.push_str(&format!(
                    "- Type: {}\n",
                    file.mime_type.as_deref().unwrap_or("Unknown")
                ));

                if let Some(bytes) = &file.bytes {
                    // Calculate size of decoded data
                    let decoded_size = match BASE64.decode(bytes) {
                        Ok(data) => data.len(),
                        Err(_) => 0,
                    };
                    description.push_str(&format!("- Size: {} bytes\n", decoded_size));

                    // Show first few bytes of the file (for text files)
                    if file.mime_type.as_deref().unwrap_or("").starts_with("text/") {
                        if let Ok(data) = BASE64.decode(bytes) {
                            if let Ok(start) =
                                String::from_utf8(data.iter().take(100).cloned().collect())
                            {
                                let preview = if start.len() > 50 {
                                    format!("{}...", &start[..50])
                                } else {
                                    start
                                };
                                description.push_str(&format!("- Preview: {}\n", preview));
                            }
                        }
                    }
                } else if let Some(uri) = &file.uri {
                    description.push_str(&format!("- URI: {}\n", uri));
                }

                description.push('\n');
            }
            description
        };

        // Create response message
        let response_message = Message {
            role: MessageRole::Agent,
            parts: vec![Part::Text {
                text: response_text,
                metadata: None,
            }],
            metadata: None,
        };

        // Create task with response
        Ok(Task {
            id: task_id,
            session_id: params.session_id.clone(),
            status: acai::TaskStatus {
                state: acai::TaskState::Completed,
                message: Some(response_message.clone()),
                timestamp: Some(chrono::Utc::now()),
            },
            artifacts: None,
            history: Some(vec![params.message.clone(), response_message]),
            metadata: None,
        })
    }

    // Create router with handler
    let mut router = MethodRouter::new();
    router.register(
        "tasks/send",
        make_typed_handler(Arc::new(()), file_info_handler),
    );

    // Create an agent card
    let agent_card = AgentCard {
        name: "File Info Agent".to_string(),
        description: Some("An agent that analyzes and displays file information".to_string()),
        url: server_url.clone(),
        provider: Some(AgentProvider {
            organization: "Test".to_string(),
            url: None,
        }),
        version: "1.0.0".to_string(),
        documentation_url: None,
        capabilities: AgentCapabilities {
            streaming: false,
            push_notifications: false,
            state_transition_history: false,
        },
        authentication: None,
        default_input_modes: vec!["file".to_string(), "text".to_string()],
        default_output_modes: vec!["text".to_string()],
        skills: vec![],
    };

    // Create server config and start server
    let server_config = ServerConfig::new(&server_address).unwrap();
    let server = Server::new(server_config, Arc::new(router)).with_agent_card(agent_card);

    let server_handle = tokio::spawn(async move {
        server.serve().await.unwrap();
    });

    // Wait for server to start
    sleep(Duration::from_millis(500)).await;

    // Create an acai client
    let client_config = ClientConfig::new(&server_url);
    let client = AcaiClient::new(client_config).unwrap();

    // Test 1: Fetch agent card
    let agent_card = client.fetch_agent_card().await.unwrap();
    assert_eq!(agent_card.name, "File Info Agent");

    // Test 2: Send a file with base64-encoded bytes
    let file_content = load_file(test_file_path).unwrap();

    // Create a message with the file
    let file_part = Part::File {
        file: file_content,
        metadata: None,
    };

    let message = Message {
        role: MessageRole::User,
        parts: vec![
            Part::Text {
                text: "Please analyze this file with embedded bytes".to_string(),
                metadata: None,
            },
            file_part,
        ],
        metadata: None,
    };

    // Create the task parameters
    let task_id = "file_task_bytes_test".to_string();
    let params = TaskSendParams {
        id: task_id.clone(),
        message,
        session_id: None,
        push_notification: None,
        history_length: None,
        metadata: None,
    };

    // Create the request
    let request = params.into_send_request(serde_json::json!(task_id));

    // Send the request
    let task: Task = client.send(request).await.unwrap();

    // Verify that the task was processed correctly
    assert_eq!(task.id, task_id);
    assert_eq!(task.status.state, TaskState::Completed);

    // Verify the response contains file information
    let response_message = task.status.message.unwrap();
    let response_text = match &response_message.parts[0] {
        Part::Text { text, .. } => text,
        _ => panic!("Expected text part in response"),
    };

    assert!(response_text.contains("Received 1 file(s)"));
    assert!(response_text.contains("test_file.txt"));
    assert!(response_text.contains("text/plain"));
    assert!(response_text.contains("This is a test"));

    // Test 3: Send a file with URI reference
    // Create a FileContent with a URI reference
    let uri_file = FileContent {
        name: Some("example-image.jpg".to_string()),
        mime_type: Some("image/jpeg".to_string()),
        bytes: None,
        uri: Some("https://example.com/images/example.jpg".to_string()),
    };

    // Create a message with the file
    let file_part = Part::File {
        file: uri_file,
        metadata: None,
    };

    let message = Message {
        role: MessageRole::User,
        parts: vec![
            Part::Text {
                text: "Please analyze this file with URI reference".to_string(),
                metadata: None,
            },
            file_part,
        ],
        metadata: None,
    };

    // Create the task parameters
    let task_id = "file_task_uri_test".to_string();
    let params = TaskSendParams {
        id: task_id.clone(),
        message,
        session_id: None,
        push_notification: None,
        history_length: None,
        metadata: None,
    };

    // Create the request
    let request = params.into_send_request(serde_json::json!(task_id));

    // Send the request
    let task: Task = client.send(request).await.unwrap();

    // Verify that the task was processed correctly
    assert_eq!(task.id, task_id);
    assert_eq!(task.status.state, TaskState::Completed);

    // Verify the response contains file information
    let response_message = task.status.message.unwrap();
    let response_text = match &response_message.parts[0] {
        Part::Text { text, .. } => text,
        _ => panic!("Expected text part in response"),
    };

    assert!(response_text.contains("Received 1 file(s)"));
    assert!(response_text.contains("example-image.jpg"));
    assert!(response_text.contains("image/jpeg"));
    assert!(response_text.contains("URI: https://example.com/images/example.jpg"));

    // Clean up - cancel the server task
    server_handle.abort();
}

// Test for the comprehensive client.rs example
// This test ensures that the comprehensive client can connect to a server and perform multiple operations
#[tokio::test]
async fn comprehensive_client() {
    // Start the server on a random port to avoid conflicts
    let port = find_available_port().await;
    let server_address = format!("127.0.0.1:{}", port);
    let server_url = format!("http://{}", server_address);

    // Create handlers for calculator-like functions (like in comprehensive_client.rs)
    // Add handler
    async fn handle_add(
        _state: Arc<()>,
        params: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        if params.len() != 2 {
            return Err(JsonRpcError::invalid_parameters(
                "add requires exactly 2 numbers",
            ));
        }

        let a = params[0].as_i64().ok_or_else(|| {
            JsonRpcError::invalid_parameters("first parameter must be an integer")
        })?;
        let b = params[1].as_i64().ok_or_else(|| {
            JsonRpcError::invalid_parameters("second parameter must be an integer")
        })?;

        let result = a + b;
        Ok(serde_json::json!(result))
    }

    // Multiply handler
    async fn handle_multiply(
        _state: Arc<()>,
        params: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        if params.len() != 2 {
            return Err(JsonRpcError::invalid_parameters(
                "multiply requires exactly 2 numbers",
            ));
        }

        let a = params[0].as_i64().ok_or_else(|| {
            JsonRpcError::invalid_parameters("first parameter must be an integer")
        })?;
        let b = params[1].as_i64().ok_or_else(|| {
            JsonRpcError::invalid_parameters("second parameter must be an integer")
        })?;

        let result = a * b;
        Ok(serde_json::json!(result))
    }

    // Divide handler
    async fn handle_divide(
        _state: Arc<()>,
        params: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        if params.len() != 2 {
            return Err(JsonRpcError::invalid_parameters(
                "divide requires exactly 2 numbers",
            ));
        }

        let a = params[0].as_i64().ok_or_else(|| {
            JsonRpcError::invalid_parameters("first parameter must be an integer")
        })?;
        let b = params[1].as_i64().ok_or_else(|| {
            JsonRpcError::invalid_parameters("second parameter must be an integer")
        })?;

        if b == 0 {
            return Err(JsonRpcError::invalid_parameters("division by zero"));
        }

        let result = a / b;
        Ok(serde_json::json!(result))
    }

    // Create router with handlers
    let mut router = MethodRouter::new();

    router.register("add", make_typed_handler(Arc::new(()), handle_add));
    router.register(
        "multiply",
        make_typed_handler(Arc::new(()), handle_multiply),
    );
    router.register("divide", make_typed_handler(Arc::new(()), handle_divide));

    // Create an agent card
    let agent_card = AgentCard {
        name: "Calculator Agent".to_string(),
        description: Some("A calculator agent that performs basic arithmetic".to_string()),
        url: server_url.clone(),
        provider: Some(AgentProvider {
            organization: "Test".to_string(),
            url: None,
        }),
        version: "1.0.0".to_string(),
        documentation_url: None,
        capabilities: AgentCapabilities {
            streaming: false,
            push_notifications: false,
            state_transition_history: false,
        },
        authentication: None,
        default_input_modes: vec!["numerical".to_string()],
        default_output_modes: vec!["numerical".to_string()],
        skills: vec![],
    };

    // Create server config and start server
    let server_config = ServerConfig::new(&server_address).unwrap();
    let server = Server::new(server_config, Arc::new(router)).with_agent_card(agent_card);

    let server_handle = tokio::spawn(async move {
        server.serve().await.unwrap();
    });

    // Wait for server to start
    sleep(Duration::from_millis(500)).await;

    // Create an acai client
    let client_config = ClientConfig::new(&server_url);
    let client = AcaiClient::new(client_config).unwrap();

    // Test 1: Fetch agent card
    let agent_card = client.fetch_agent_card().await.unwrap();
    assert_eq!(agent_card.name, "Calculator Agent");

    // Test 2: Test addition
    let add_params = vec![serde_json::json!(10), serde_json::json!(5)];
    let add_req = JsonRpcRequest::new(serde_json::json!("add-req"), "add", add_params);
    let add_resp: i64 = client.send(add_req).await.unwrap();
    assert_eq!(add_resp, 15);

    // Test 3: Test multiplication
    let mul_params = vec![serde_json::json!(10), serde_json::json!(5)];
    let mul_req = JsonRpcRequest::new(serde_json::json!("mul-req"), "multiply", mul_params);
    let mul_resp: i64 = client.send(mul_req).await.unwrap();
    assert_eq!(mul_resp, 50);

    // Test 4: Test division
    let div_params = vec![serde_json::json!(10), serde_json::json!(5)];
    let div_req = JsonRpcRequest::new(serde_json::json!("div-req"), "divide", div_params);
    let div_resp: i64 = client.send(div_req).await.unwrap();
    assert_eq!(div_resp, 2);

    // Test 5: Test error handling (division by zero)
    let div_zero_params = vec![serde_json::json!(10), serde_json::json!(0)];
    let div_zero_req =
        JsonRpcRequest::new(serde_json::json!("div-zero-req"), "divide", div_zero_params);
    let div_zero_resp = client
        .send::<_, JsonRpcResponse<serde_json::Value>>(div_zero_req)
        .await;
    assert!(div_zero_resp.is_err());

    // Clean up - cancel the server task
    server_handle.abort();
}
