use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use tokio::net::TcpListener;
use tokio::time::timeout;

use acai::server::task_manager::TaskManager;
use acai::server::{MethodRouter, Server, ServerConfig};
use acai::server::{make_typed_handler, push_notification_handlers};
use acai::{
    JsonRpcError, Message, MessageRole, Part, PushNotificationConfig, Task, TaskQueryParams,
    TaskSendParams, TaskState, TaskStatus,
};

// Struct to capture notifications received by the webhook server
struct NotificationCapture {
    // Store received notifications
    received: Arc<Mutex<Vec<String>>>,
    // Token received during validation
    validation_token: Arc<Mutex<Option<String>>>,
}

#[tokio::test]
async fn webhook_notification() {
    // Set up capturing of webhook notifications
    let notification_capture = Arc::new(NotificationCapture {
        received: Arc::new(Mutex::new(Vec::new())),
        validation_token: Arc::new(Mutex::new(None)),
    });

    // Start a webhook server that captures notifications
    let webhook_port = start_webhook_server(Arc::clone(&notification_capture)).await;
    let webhook_url = format!("http://127.0.0.1:{}", webhook_port);

    // Start the A2A server
    let task_manager = Arc::new(TaskManager::new().unwrap());
    let mut router = MethodRouter::new();

    // Add all the necessary handlers for the A2A protocol

    // First define a simple send task handler
    async fn handle_send_task(
        task_manager: Arc<TaskManager>,
        params: TaskSendParams,
    ) -> Result<Task, JsonRpcError> {
        // Create/update the task
        task_manager
            .upsert_task(&params)
            .await
            .map_err(JsonRpcError::internal_error)?;

        // Get the updated task to return
        let task_query = TaskQueryParams::from_id(params.id.clone());
        task_manager
            .get_task(&task_query)
            .await
            .map_err(JsonRpcError::internal_error)
    }

    // Add push notification handlers

    // Register all handlers
    router.register(
        "tasks/send",
        make_typed_handler(Arc::clone(&task_manager), handle_send_task),
    );
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

    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_port = listener.local_addr().unwrap().port();
    let server_config = ServerConfig::new(&format!("127.0.0.1:{}", server_port)).unwrap();
    let server = Server::new(server_config, Arc::new(router));

    // Start the server in the background
    let _server_handle = tokio::spawn(async move {
        server.serve().await.unwrap();
    });

    // Need to wait a moment for the server to start and bind to a port
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create a task with push notification configured
    let push_config = PushNotificationConfig {
        url: webhook_url.clone(),
        token: Some("test-token".to_string()),
        authentication: None,
    };

    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: "Test message".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    let task_id = "webhook_test_task".to_string();
    let params = TaskSendParams {
        id: task_id.clone(),
        message,
        session_id: None,
        push_notification: Some(push_config),
        history_length: None,
        metadata: None,
    };

    // Let's skip the HTTP API call and directly work with the task manager
    // Create a minimal task
    let _task = Task {
        id: task_id.clone(),
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

    // First insert the task
    task_manager.upsert_task(&params).await.unwrap();

    // Then update the status to trigger notification
    task_manager
        .update_task_status(
            &task_id,
            TaskStatus {
                state: TaskState::Completed,
                message: None,
                timestamp: None,
            },
        )
        .await
        .unwrap();

    // Wait for the notification with timeout
    let start_time = std::time::Instant::now();
    let timeout_duration = Duration::from_secs(5);
    let mut completed_notification_received = false;

    while start_time.elapsed() < timeout_duration {
        // Check if notification with completed state was received
        {
            // Scope the lock so it's dropped before the await
            let notifications = notification_capture.received.lock().unwrap();

            // Look through all notifications to find the one with completed state
            for notification in notifications.iter() {
                println!("Received notification: {}", notification);

                // Parse the notification as JSON
                let parsed: serde_json::Value = match serde_json::from_str(notification) {
                    Ok(val) => val,
                    Err(e) => {
                        println!("Error parsing notification JSON: {}", e);
                        continue;
                    }
                };

                // Check if this is for our task and fail if wrong ID is received
                if let Some(id) = parsed["id"].as_str() {
                    assert_eq!(id, task_id, "Received notification with incorrect task ID");
                } else {
                    panic!("Notification missing task ID field");
                }

                // Check the state
                if let Some(state) = parsed["status"]["state"].as_str() {
                    if state == "completed" {
                        completed_notification_received = true;

                        // Verify this is a final status update
                        assert!(
                            parsed["final"].as_bool().unwrap_or(false),
                            "Notification 'final' field should be true for completed state"
                        );

                        // Found what we're looking for
                        break;
                    }
                }
            }
        } // Lock is dropped here

        if completed_notification_received {
            break;
        }

        // Sleep a short time before checking again
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Final assertion to fail the test if no completed notification was received within timeout
    assert!(
        completed_notification_received,
        "No push notification with 'completed' state received within the {} second timeout",
        timeout_duration.as_secs()
    );

    // We don't need to cancel the server task since we're using an underscore variable
}

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
