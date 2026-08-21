use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use acai::server::push_notification_handlers;
use acai::server::task_manager::TaskManager;
use acai::server::{MethodRouter, Server, ServerConfig};
use acai::{
    AgentCapabilities, AgentCard, AgentSkill, JsonRpcError, Message, MessageRole, Part,
    PushNotificationConfig, Task, TaskIdParams, TaskPushNotificationConfig, TaskQueryParams,
    TaskSendParams, TaskStatus,
};

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Method, StatusCode};
use tokio::net::TcpListener;

// Struct to capture notifications received by the webhook server
struct NotificationCapture {
    // Store received notifications
    received: Arc<Mutex<Vec<String>>>,
    // Token received during validation
    validation_token: Arc<Mutex<Option<String>>>,
}

/// Handler function for sending tasks
async fn handle_send_task(
    task_manager: Arc<TaskManager>,
    params: TaskSendParams,
) -> Result<Task, JsonRpcError> {
    // Create/update the task
    task_manager.upsert_task(&params).await?;

    // Get the updated task to return
    let task_query = TaskQueryParams::from_id(params.id.clone());
    Ok(task_manager.get_task(&task_query).await?)
}

/// Handler function for getting tasks
async fn handle_get_task(
    task_manager: Arc<TaskManager>,
    params: TaskQueryParams,
) -> Result<Task, JsonRpcError> {
    Ok(task_manager.get_task(&params).await?)
}

/// Handler function for canceling tasks
async fn handle_cancel_task(
    task_manager: Arc<TaskManager>,
    params: TaskIdParams,
) -> Result<Task, JsonRpcError> {
    // Cancel the task
    task_manager.cancel_task(&params).await?;

    // Get the updated task to return
    let task_query = TaskQueryParams::from_id(params.id.clone());
    Ok(task_manager.get_task(&task_query).await?)
}

/// Parameters for updating task status
#[derive(serde::Deserialize)]
struct UpdateTaskStatusParams {
    id: String,
    status: TaskStatus,
}

/// Handler function for updating task status
async fn handle_update_task_status(
    task_manager: Arc<TaskManager>,
    params: UpdateTaskStatusParams,
) -> Result<Task, JsonRpcError> {
    // Update the task status
    task_manager
        .update_task_status(&params.id, params.status)
        .await?;

    // Get the updated task to return
    let task_query = TaskQueryParams::from_id(params.id.clone());
    Ok(task_manager.get_task(&task_query).await?)
}

/// Starts a webhook server that captures push notifications
async fn start_webhook_server(capture: Arc<NotificationCapture>) -> u16 {
    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    println!("Webhook server listening on port {}", port);

    let capture_clone = Arc::clone(&capture);
    tokio::spawn(async move {
        loop {
            if let Ok((stream, _)) = listener.accept().await {
                let io = hyper_util::rt::TokioIo::new(stream);
                let capture = Arc::clone(&capture_clone);

                tokio::spawn(async move {
                    let service = service_fn(move |req: hyper::Request<Incoming>| {
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
                                        println!("Received validation token: {}", token);

                                        let mut validation =
                                            capture.validation_token.lock().unwrap();
                                        *validation = Some(token.to_string());

                                        // Echo back the validation token
                                        let response = hyper::Response::new(Full::new(
                                            token.to_string().into(),
                                        ));
                                        Ok::<_, hyper::Error>(response)
                                    } else {
                                        println!("Missing validation token in request");
                                        let response = hyper::Response::builder()
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

                                    println!("Received notification: {}", body_str);

                                    // Store the notification
                                    let mut received = capture.received.lock().unwrap();
                                    received.push(body_str);

                                    let response = hyper::Response::new(Full::new("OK".into()));
                                    Ok::<_, hyper::Error>(response)
                                }
                                _ => {
                                    println!(
                                        "Received request for unknown path: {}",
                                        req.uri().path()
                                    );
                                    let response = hyper::Response::builder()
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
                    if tokio::time::timeout(Duration::from_secs(30), conn)
                        .await
                        .is_err()
                    {
                        println!("Connection timeout");
                    }
                });
            }
        }
    });

    port
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting push notification example...");

    // Set up capturing of webhook notifications
    let notification_capture = Arc::new(NotificationCapture {
        received: Arc::new(Mutex::new(Vec::new())),
        validation_token: Arc::new(Mutex::new(None)),
    });

    // Start a webhook server that captures notifications
    let webhook_port = start_webhook_server(Arc::clone(&notification_capture)).await;
    let webhook_url = format!("http://127.0.0.1:{}", webhook_port);

    println!("Webhook URL: {}", webhook_url);

    // Start the A2A server
    let server_port = 8080;
    let server_url = format!("http://127.0.0.1:{}", server_port);

    println!("Starting A2A server on {}", server_url);

    // Create the task manager
    let task_manager = Arc::new(TaskManager::new()?);

    // Create the router
    let mut router = MethodRouter::new();

    // Register handlers
    router
        .register(
            "tasks/send",
            acai::server::make_typed_handler(task_manager.clone(), handle_send_task),
        )
        .register(
            "tasks/get",
            acai::server::make_typed_handler(task_manager.clone(), handle_get_task),
        )
        .register(
            "tasks/cancel",
            acai::server::make_typed_handler(task_manager.clone(), handle_cancel_task),
        )
        .register(
            "tasks/updateStatus",
            acai::server::make_typed_handler(task_manager.clone(), handle_update_task_status),
        )
        .register(
            "tasks/pushNotification/get",
            acai::server::make_typed_handler(
                task_manager.clone(),
                push_notification_handlers::get_push_notification,
            ),
        )
        .register(
            "tasks/pushNotification/set",
            acai::server::make_typed_handler(
                task_manager.clone(),
                push_notification_handlers::set_push_notification,
            ),
        );

    // Create an agent card with push notification capabilities
    let agent_card = AgentCard {
        name: "Push Notification Example Agent".to_string(),
        description: Some("An example agent that demonstrates push notifications".to_string()),
        url: server_url.clone(),
        provider: None,
        version: "1.0.0".to_string(),
        documentation_url: None,
        capabilities: AgentCapabilities {
            streaming: true,
            push_notifications: true,
            state_transition_history: true,
        },
        authentication: None,
        default_input_modes: vec!["text".to_string()],
        default_output_modes: vec!["text".to_string()],
        skills: vec![AgentSkill {
            id: "push_notification_example".to_string(),
            name: "Push Notification Example".to_string(),
            description: Some("Demonstrates push notification capabilities".to_string()),
            tags: None,
            examples: None,
            input_modes: None,
            output_modes: None,
        }],
    };

    // Create the server config
    let config = ServerConfig::new(&format!("127.0.0.1:{}", server_port))?;

    // Create the server with the router and agent card
    let server = Server::new(config, Arc::new(router)).with_agent_card(agent_card);

    // Start the server in a background task
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.serve().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Wait for the server to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Create a client to demonstrate the push notification functionality
    let client = reqwest::Client::new();

    // EXAMPLE 1: Create a task with push notification configured in TaskSendParams
    println!("\n--- Example 1: Create a task with push notification configured ---");

    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: "This is a test task with push notifications configured on creation".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    let push_config = PushNotificationConfig {
        url: webhook_url.clone(),
        token: Some("example-token-1".to_string()),
        authentication: None,
    };

    let task_id_1 = "task_with_push_1".to_string();
    let params = TaskSendParams {
        id: task_id_1.clone(),
        message,
        session_id: None,
        push_notification: Some(push_config),
        history_length: None,
        metadata: Some(HashMap::from([(
            "example".to_string(),
            serde_json::json!("Example 1"),
        )])),
    };

    let request = params.into_send_request(serde_json::json!("create-task-1"));

    println!("Sending task creation request for task {}", task_id_1);

    let response = client
        .post(format!("{}/api", server_url))
        .json(&request)
        .send()
        .await?;

    let _task_response: serde_json::Value = response.json().await?;
    println!("Task created: {}", task_id_1);

    // Update the task status to trigger a notification
    let status_update = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "update-status-1",
        "method": "tasks/updateStatus",
        "params": {
            "id": task_id_1,
            "status": {
                "state": "working",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        }
    });

    println!("Updating task {} status to 'working'", task_id_1);

    let _response = client
        .post(format!("{}/api", server_url))
        .json(&status_update)
        .send()
        .await?;

    // Wait a moment for the notification to be processed
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Check if we received a notification
    {
        let notifications = notification_capture.received.lock().unwrap();
        if !notifications.is_empty() {
            println!("Received notification for task_with_push_1:");
            for notification in notifications.iter() {
                let parsed: serde_json::Value = serde_json::from_str(notification).unwrap();
                if let Some(id) = parsed["id"].as_str() {
                    if id == task_id_1 {
                        println!("✓ Notification received with correct task ID");
                        if let Some(state) = parsed["status"]["state"].as_str() {
                            println!("✓ Task state in notification: {}", state);
                        }
                    }
                }
            }
        } else {
            println!("No notifications received for task_with_push_1");
        }
    }

    // EXAMPLE 2: Create a task first, then set up push notification
    println!("\n--- Example 2: Create a task and set push notification later ---");

    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: "This is a test task with push notifications configured after creation"
                .to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    let task_id_2 = "task_with_push_2".to_string();
    let params = TaskSendParams {
        id: task_id_2.clone(),
        message,
        session_id: None,
        push_notification: None, // No push notification initially
        history_length: None,
        metadata: Some(HashMap::from([(
            "example".to_string(),
            serde_json::json!("Example 2"),
        )])),
    };

    let request = params.into_send_request(serde_json::json!("create-task-2"));

    println!("Sending task creation request for task {}", task_id_2);

    let response = client
        .post(format!("{}/api", server_url))
        .json(&request)
        .send()
        .await?;

    let _task_response: serde_json::Value = response.json().await?;
    println!("Task created: {}", task_id_2);

    // Now set up push notification for this task
    let push_config = PushNotificationConfig {
        url: webhook_url.clone(),
        token: Some("example-token-2".to_string()),
        authentication: None,
    };

    let push_notification_config = TaskPushNotificationConfig {
        id: task_id_2.clone(),
        push_notification_config: push_config,
    };

    let set_push_request = push_notification_config
        .into_push_notification_set_request(serde_json::json!("set-push-request"));

    println!("Setting up push notification for task {}", task_id_2);

    let response = client
        .post(format!("{}/api", server_url))
        .json(&set_push_request)
        .send()
        .await?;

    let push_response: serde_json::Value = response.json().await?;

    if push_response["result"].is_object() {
        println!("✓ Push notification configured successfully");
    } else if let Some(error) = push_response["error"].as_object() {
        println!("Error setting push notification: {:?}", error);
    }

    // Update the task status to completed (triggers notification)
    let status_update = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "update-status-2",
        "method": "tasks/updateStatus",
        "params": {
            "id": task_id_2,
            "status": {
                "state": "completed",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        }
    });

    println!("Updating task {} status to 'completed'", task_id_2);

    let _response = client
        .post(format!("{}/api", server_url))
        .json(&status_update)
        .send()
        .await?;

    // Wait a moment for the notification to be processed
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Check if we received both notifications
    {
        let notifications = notification_capture.received.lock().unwrap();
        println!("\n--- Received Notifications Summary ---");
        println!("Total notifications received: {}", notifications.len());

        for (i, notification) in notifications.iter().enumerate() {
            let parsed: serde_json::Value = serde_json::from_str(notification).unwrap();
            println!("Notification {}:", i + 1);

            if let Some(id) = parsed["id"].as_str() {
                println!("  Task ID: {}", id);
            }

            if let Some(state) = parsed["status"]["state"].as_str() {
                println!("  State: {}", state);
            }

            if let Some(final_status) = parsed["final"].as_bool() {
                println!("  Final status: {}", final_status);
            }
        }
    }

    // EXAMPLE 3: Use the get push notification endpoint
    println!("\n--- Example 3: Get push notification configuration ---");

    let get_params = TaskIdParams {
        id: task_id_2.clone(),
        metadata: None,
    };

    let get_push_request =
        get_params.into_push_notification_get_request(serde_json::json!("get-push-request"));

    println!("Getting push notification config for task {}", task_id_2);

    let response = client
        .post(format!("{}/api", server_url))
        .json(&get_push_request)
        .send()
        .await?;

    let get_response: serde_json::Value = response.json().await?;

    if let Some(result) = get_response["result"].as_object() {
        if let Some(config) = result["pushNotificationConfig"].as_object() {
            println!("✓ Retrieved push notification config:");
            println!("  URL: {}", config["url"].as_str().unwrap_or(""));
            println!("  Token: {}", config["token"].as_str().unwrap_or(""));
        }
    } else if let Some(error) = get_response["error"].as_object() {
        println!("Error getting push notification: {:?}", error);
    }

    // Clean up and finish
    server_handle.abort();
    println!("\nPush notification example completed successfully!");

    Ok(())
}
