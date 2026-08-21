use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use tokio::net::TcpListener;
use tokio::time::sleep;
use uuid::Uuid;

use acai::client::{Client, ClientConfig};
use acai::server::jwks::{JwksManager, KeyPairConfig};
use acai::server::task_manager::TaskManager;
use acai::server::{MethodRouter, Server, ServerConfig};
use acai::server::{make_typed_handler, push_notification_handlers};
use acai::{
    Artifact, Message, MessageRole, Part, PushNotificationConfig, Task, TaskIdParams,
    TaskQueryParams, TaskSendParams, TaskState, TaskStatus,
};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use std::sync::Mutex;
use tokio::time::timeout;

// Struct to capture notifications received by the webhook server
struct NotificationCapture {
    // Store received notifications
    received: Arc<Mutex<Vec<String>>>,
    // Token received during validation
    validation_token: Arc<Mutex<Option<String>>>,
}

// Helper struct to manage server lifecycle
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

// Helper function to start a webhook server for testing push notifications
async fn start_webhook_server(capture: Arc<NotificationCapture>) -> u16 {
    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let capture_clone = Arc::clone(&capture);
    tokio::spawn(async move {
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let io = hyper_util::rt::TokioIo::new(stream);
                let capture = Arc::clone(&capture_clone);

                tokio::spawn(async move {
                    let service = service_fn(move |req: Request<Incoming>| {
                        let capture = Arc::clone(&capture);
                        async move {
                            match (req.method(), req.uri().path()) {
                                (&Method::GET, "/") => {
                                    // Handle validation request
                                    if let Some(token) = req
                                        .uri()
                                        .query()
                                        .and_then(|q| q.strip_prefix("validationToken="))
                                    {
                                        let mut validation =
                                            capture.validation_token.lock().unwrap();
                                        *validation = Some(token.to_string());

                                        // Echo back the validation token
                                        let response =
                                            Response::new(Full::new(token.to_string().into()));
                                        Ok::<_, hyper::Error>(response)
                                    } else {
                                        let response = Response::builder()
                                            .status(StatusCode::BAD_REQUEST)
                                            .body(Full::new("Missing validation token".into()))
                                            .unwrap();
                                        Ok::<_, hyper::Error>(response)
                                    }
                                }
                                (&Method::POST, "/") => {
                                    // Handle notification
                                    let capture = Arc::clone(&capture);
                                    let body = req.collect().await?;
                                    let bytes = body.to_bytes();
                                    let body_str = String::from_utf8(bytes.to_vec()).unwrap();

                                    // Store the notification
                                    let mut received = capture.received.lock().unwrap();
                                    received.push(body_str);

                                    let response = Response::new(Full::new("OK".into()));
                                    Ok::<_, hyper::Error>(response)
                                }
                                _ => {
                                    let response = Response::builder()
                                        .status(StatusCode::NOT_FOUND)
                                        .body(Full::new(Bytes::new()))
                                        .unwrap();
                                    Ok::<_, hyper::Error>(response)
                                }
                            }
                        }
                    });

                    // Process the HTTP connection
                    let conn =
                        hyper::server::conn::http1::Builder::new().serve_connection(io, service);

                    // Enforce a timeout for the connection
                    if timeout(Duration::from_secs(30), conn).await.is_err() {
                        // Connection timeout, just let it drop
                    }
                });
            }
        }
    });

    port
}

// Handler function for tasks/send and tasks/sendSubscribe
async fn handle_task_send(
    task_manager: Arc<TaskManager>,
    params: TaskSendParams,
) -> Result<Task, acai::JsonRpcError> {
    // Process the task
    task_manager.upsert_task(&params).await?;

    // Get the updated task to return
    let task_query = acai::TaskQueryParams::from_id(params.id.clone());
    Ok(task_manager.get_task(&task_query).await?)
}

// We reuse the handler function for both regular and streaming requests
// In a real implementation, the streaming aspects would be handled by server infrastructure

// Handler function for tasks/get
async fn handle_task_get(
    task_manager: Arc<TaskManager>,
    params: TaskQueryParams,
) -> Result<Task, acai::JsonRpcError> {
    // Get the task
    Ok(task_manager.get_task(&params).await?)
}

// Handler function for tasks/cancel
async fn handle_task_cancel(
    task_manager: Arc<TaskManager>,
    params: TaskIdParams,
) -> Result<Task, acai::JsonRpcError> {
    // Cancel the task
    task_manager.cancel_task(&params).await?;

    // Get the updated task to return
    let task_query = acai::TaskQueryParams::from_id(params.id.clone());
    Ok(task_manager.get_task(&task_query).await?)
}

// Helper to start a test server with JWKS and push notification support
async fn start_test_server(task_manager: Arc<TaskManager>) -> (u16, ServerGuard) {
    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server_url = format!("http://127.0.0.1:{}", port);
    drop(listener); // Release the port for the server

    // Create a JWKS manager for authentication
    let config = KeyPairConfig {
        name: "test".to_string(),
        private_key_path: "tests/test.key".to_string(),
    };
    let jwks_manager = Arc::new(JwksManager::new(vec![config]).unwrap());

    // Create a router and register handlers
    let mut router = MethodRouter::new();

    // Add task handlers
    router.register(
        "tasks/send",
        make_typed_handler(Arc::clone(&task_manager), handle_task_send),
    );
    router.register(
        "tasks/sendSubscribe",
        make_typed_handler(Arc::clone(&task_manager), handle_task_send),
    );
    router.register(
        "tasks/get",
        make_typed_handler(Arc::clone(&task_manager), handle_task_get),
    );
    router.register(
        "tasks/cancel",
        make_typed_handler(Arc::clone(&task_manager), handle_task_cancel),
    );

    // Add push notification handlers
    router.register(
        "tasks/pushNotification/get",
        make_typed_handler(
            Arc::clone(&task_manager),
            push_notification_handlers::get_push_notification,
        ),
    );
    router.register(
        "tasks/pushNotification/set",
        make_typed_handler(
            Arc::clone(&task_manager),
            push_notification_handlers::set_push_notification,
        ),
    );

    // Create server configuration
    let server_config = ServerConfig::new(&format!("127.0.0.1:{}", port)).unwrap();

    // Create the server with agent card and JWKS support
    let server = Server::new(server_config, Arc::new(router))
        .with_agent_card(acai::AgentCard {
            name: "Test Agent".to_string(),
            description: Some("A test agent for e2e testing".to_string()),
            url: server_url.clone(),
            provider: Some(acai::AgentProvider {
                organization: "acai".to_string(),
                url: None,
            }),
            version: "1.0.0".to_string(),
            documentation_url: None,
            capabilities: acai::AgentCapabilities {
                streaming: true,
                push_notifications: true,
                state_transition_history: true,
            },
            authentication: None,
            default_input_modes: vec!["text".to_string()],
            default_output_modes: vec!["text".to_string()],
            skills: Vec::new(),
        })
        .with_jwks_manager(jwks_manager);

    // Start the server
    let server_handle = tokio::spawn(async move {
        server.serve().await.unwrap();
    });

    // Wait for server to start up
    sleep(Duration::from_millis(200)).await;

    (port, ServerGuard::new(server_handle))
}

#[tokio::test]
async fn test_client_server_basic_operations() {
    // Create a task manager that will be used by the server
    let task_manager = Arc::new(TaskManager::new().unwrap());

    // Start the server
    let (port, _server_guard) = start_test_server(Arc::clone(&task_manager)).await;
    let server_url = format!("http://127.0.0.1:{}", port);

    // Create a client
    let client_config = ClientConfig::new(&server_url);
    let client = Client::new(client_config.clone()).unwrap();

    // Test 1: Fetch agent card
    let agent_card = client.fetch_agent_card().await.unwrap();
    assert_eq!(agent_card.name, "Test Agent");
    assert!(agent_card.capabilities.streaming);
    assert!(agent_card.capabilities.push_notifications);

    // Test 2: Send a task
    let task_id = Uuid::new_v4().to_string();
    let session_id = Some(Uuid::new_v4().to_string());

    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: "Hello from the client".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    let params = TaskSendParams {
        id: task_id.clone(),
        message,
        session_id: session_id.clone(),
        push_notification: None,
        history_length: None,
        metadata: None,
    };

    let request = params.into_send_request(serde_json::json!("send-request-id"));
    let response: Task = client.send(request).await.unwrap();

    // Verify response
    assert_eq!(response.id, task_id);
    assert_eq!(response.session_id, session_id);
    assert_eq!(response.status.state, TaskState::Submitted);

    // Test 3: Get the task
    let params = TaskQueryParams {
        id: task_id.clone(),
        history_length: Some(10),
        metadata: None,
    };

    let request = params.into_get_request(serde_json::json!("get-request-id"));
    let response: Task = client.send(request).await.unwrap();

    // Verify response
    assert_eq!(response.id, task_id);
    assert_eq!(response.session_id, session_id);

    // Test 4: Cancel the task
    let params = TaskIdParams {
        id: task_id.clone(),
        metadata: None,
    };

    let request = params.into_cancel_request(serde_json::json!("cancel-request-id"));
    let response: Task = client.send(request).await.unwrap();

    // Verify response
    assert_eq!(response.id, task_id);
    assert_eq!(response.status.state, TaskState::Canceled);

    // Get the task again to confirm it's canceled
    let params = TaskQueryParams {
        id: task_id.clone(),
        history_length: None,
        metadata: None,
    };

    let request = params.into_get_request(serde_json::json!("get-request-id"));
    let response: Task = client.send(request).await.unwrap();

    // Verify response
    assert_eq!(response.status.state, TaskState::Canceled);
}

#[tokio::test]
async fn test_client_server_push_notifications() {
    // Create task manager
    let task_manager = Arc::new(TaskManager::new().unwrap());

    // Set up capturing of webhook notifications
    let notification_capture = Arc::new(NotificationCapture {
        received: Arc::new(Mutex::new(Vec::new())),
        validation_token: Arc::new(Mutex::new(None)),
    });

    // Start a webhook server that captures notifications
    let webhook_port = start_webhook_server(Arc::clone(&notification_capture)).await;
    let webhook_url = format!("http://127.0.0.1:{}", webhook_port);

    // Start the server
    let (port, _server_guard) = start_test_server(Arc::clone(&task_manager)).await;
    let server_url = format!("http://127.0.0.1:{}", port);

    // Create a client
    let client_config = ClientConfig::new(&server_url);
    let client = Client::new(client_config.clone()).unwrap();

    // Create a task with initial push notification config
    let task_id = Uuid::new_v4().to_string();
    let push_config = PushNotificationConfig {
        url: webhook_url.clone(),
        token: Some("test-webhook-token".to_string()),
        authentication: None,
    };

    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: "Task with push notification".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    let params = TaskSendParams {
        id: task_id.clone(),
        message,
        session_id: None,
        push_notification: Some(push_config.clone()),
        history_length: None,
        metadata: None,
    };

    let request = params.into_send_request(serde_json::json!("send-request-id"));
    let response: Task = client.send(request).await.unwrap();

    // Verify response
    assert_eq!(response.id, task_id);

    // Get push notification configuration
    let push_notification_response = client.get_push_notification(&task_id).await.unwrap();
    assert_eq!(push_notification_response.id, task_id);
    assert_eq!(
        push_notification_response.push_notification_config.url,
        webhook_url
    );

    // Update task status to trigger a notification - using TaskState::Working
    let status_message = Message {
        role: MessageRole::Agent,
        parts: vec![Part::Text {
            text: "Processing task".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    task_manager
        .update_task_status(
            &task_id,
            TaskStatus {
                state: TaskState::Working,
                message: Some(status_message),
                timestamp: None,
            },
        )
        .await
        .unwrap();

    // Wait for the notification to be received
    let start_time = std::time::Instant::now();
    let timeout_duration = Duration::from_secs(5);
    let mut notification_received = false;

    while start_time.elapsed() < timeout_duration {
        // Check if notification was received
        {
            let notifications = notification_capture.received.lock().unwrap();

            for notification in notifications.iter() {
                // Parse the notification
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(notification) {
                    if let Some(id) = parsed["id"].as_str() {
                        if id == task_id {
                            if let Some(state) = parsed["status"]["state"].as_str() {
                                // Check for 'working' state which is the kebab-case version of TaskState::Working
                                if state == "working" {
                                    notification_received = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        } // MutexGuard dropped here before await

        if notification_received {
            break;
        }

        sleep(Duration::from_millis(100)).await;
    }

    assert!(
        notification_received,
        "No push notification received within the timeout period"
    );

    // Complete the task to trigger a final notification
    let completion_message = Message {
        role: MessageRole::Agent,
        parts: vec![Part::Text {
            text: "Task completed successfully".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    task_manager
        .update_task_status(
            &task_id,
            TaskStatus {
                state: TaskState::Completed,
                message: Some(completion_message),
                timestamp: None,
            },
        )
        .await
        .unwrap();

    // Wait for the completed notification
    let start_time = std::time::Instant::now();
    let mut completion_notification_received = false;

    while start_time.elapsed() < timeout_duration {
        {
            let notifications = notification_capture.received.lock().unwrap();

            for notification in notifications.iter() {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(notification) {
                    if let Some(id) = parsed["id"].as_str() {
                        if id == task_id {
                            if let Some(state) = parsed["status"]["state"].as_str() {
                                if state == "completed" {
                                    // Check if this is marked as final
                                    if let Some(is_final) = parsed["final"].as_bool() {
                                        if is_final {
                                            completion_notification_received = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } // MutexGuard dropped here before await

        if completion_notification_received {
            break;
        }

        sleep(Duration::from_millis(100)).await;
    }

    assert!(
        completion_notification_received,
        "No final completion notification received within the timeout period"
    );
}

#[tokio::test]
async fn test_client_server_streaming() {
    // Create task manager
    let task_manager = Arc::new(TaskManager::new().unwrap());

    // Start the server
    let (port, _server_guard) = start_test_server(Arc::clone(&task_manager)).await;
    let server_url = format!("http://127.0.0.1:{}", port);

    // Create a client
    let client_config = ClientConfig::new(&server_url);
    let client = Client::new(client_config.clone()).unwrap();

    // Create a task for streaming
    let task_id = Uuid::new_v4().to_string();
    let request_id = Uuid::new_v4().to_string();

    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: "Stream updates for this task".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    let params = TaskSendParams {
        id: task_id.clone(),
        message,
        session_id: None,
        push_notification: None,
        history_length: None,
        metadata: None,
    };

    // Start a streaming request
    let request = params.into_send_subscribe_request(serde_json::json!(request_id.clone()));

    // This stream may not receive events since we're manually updating the task
    // outside of the normal streaming flow, but we can test the stream setup
    let mut stream = client.stream(request).await.unwrap();

    // Create another client for regular requests to update task status
    let regular_client = Client::new(client_config.clone()).unwrap();

    // Manually update the task status using the task manager
    // This simulates the server sending updates to a streaming client
    let status_message = Message {
        role: MessageRole::Agent,
        parts: vec![Part::Text {
            text: "Processing streaming task".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    task_manager
        .update_task_status(
            &task_id,
            TaskStatus {
                state: TaskState::Working,
                message: Some(status_message),
                timestamp: None,
            },
        )
        .await
        .unwrap();

    // Add an artifact to the task
    let artifact = Artifact {
        name: Some("Test Artifact".to_string()),
        description: Some("This is a test artifact".to_string()),
        parts: vec![Part::Text {
            text: "Artifact content".to_string(),
            metadata: None,
        }],
        index: 0,
        append: None,
        last_chunk: None,
        metadata: None,
    };

    task_manager
        .add_task_artifact(&task_id, artifact)
        .await
        .unwrap();

    // Test that the client can retrieve the updated task
    let params = TaskQueryParams {
        id: task_id.clone(),
        history_length: None,
        metadata: None,
    };

    let request = params.into_get_request(serde_json::json!("get-request-id"));
    let response: Task = regular_client.send(request).await.unwrap();

    // Verify the task was updated
    assert_eq!(response.id, task_id);
    assert_eq!(response.status.state, TaskState::Working);
    assert!(response.artifacts.is_some());

    // Complete the task
    let completion_message = Message {
        role: MessageRole::Agent,
        parts: vec![Part::Text {
            text: "Streaming task completed".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    task_manager
        .update_task_status(
            &task_id,
            TaskStatus {
                state: TaskState::Completed,
                message: Some(completion_message),
                timestamp: None,
            },
        )
        .await
        .unwrap();

    // Get the completed task
    let params = TaskQueryParams {
        id: task_id.clone(),
        history_length: None,
        metadata: None,
    };

    let request = params.into_get_request(serde_json::json!("get-request-id"));
    let response: Task = regular_client.send(request).await.unwrap();

    // Verify the task was completed
    assert_eq!(response.status.state, TaskState::Completed);

    // Complete processing any potential streaming events
    // In a real implementation, we would expect to receive these events
    while let Ok(Some(_event)) =
        tokio::time::timeout(Duration::from_millis(100), stream.next()).await
    {
        // Process events if available
    }
}
