use acai::{
    Message, MessageRole, Part, PushNotificationConfig, Task, TaskSendParams,
    client::{Client, ClientConfig},
};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde_json::Value;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;

/// Simple webhook server using Hyper to receive A2A push notifications
struct WebhookServer {
    /// Channel to send received notifications
    sender: mpsc::Sender<Value>,
    /// Server address
    addr: SocketAddr,
    /// Secret token to validate webhook requests
    token: String,
}

// Helper function to create a response with string body
fn static_response(status: StatusCode, body: &str) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .body(Full::new(Bytes::from(body.to_string())))
        .unwrap()
}

impl WebhookServer {
    /// Create a new webhook server
    fn new(port: u16, token: &str) -> (Self, mpsc::Receiver<Value>) {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let (sender, receiver) = mpsc::channel(100);

        (
            Self {
                sender,
                addr,
                token: token.to_string(),
            },
            receiver,
        )
    }

    /// Start the webhook server
    async fn start(self) {
        // Create a shared sender and token
        let sender = Arc::new(Mutex::new(self.sender));
        let token = Arc::new(self.token);

        // Create the TCP listener
        let listener = TcpListener::bind(self.addr).await.unwrap();
        println!("Starting webhook server on {}", self.addr);

        // Accept connections and process requests
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let io = TokioIo::new(stream);

                    // Clone the sender and token for the service
                    let sender = sender.clone();
                    let token = token.clone();

                    // Process the connection
                    tokio::task::spawn(async move {
                        // Create a service function to handle requests
                        let service = service_fn(move |req: Request<hyper::body::Incoming>| {
                            let sender = sender.clone();
                            let token = token.clone();

                            async move {
                                match (req.method(), req.uri().path()) {
                                    (&Method::GET, "/health") => {
                                        // Simple health check endpoint
                                        Ok::<_, Infallible>(static_response(StatusCode::OK, "OK"))
                                    }
                                    (&Method::POST, "/webhook") => {
                                        // Webhook endpoint

                                        // Check the webhook token using query string parameter
                                        if !token.is_empty() {
                                            // Parse query string parameters
                                            let query = req.uri().query().unwrap_or("");
                                            let params: Vec<(&str, &str)> = query
                                                .split('&')
                                                .filter_map(|p| {
                                                    let parts: Vec<&str> = p.split('=').collect();
                                                    if parts.len() == 2 {
                                                        Some((parts[0], parts[1]))
                                                    } else {
                                                        None
                                                    }
                                                })
                                                .collect();

                                            // Find the validationToken parameter
                                            let validation_token = params
                                                .iter()
                                                .find(|(k, _)| *k == "validationToken")
                                                .map(|(_, v)| *v)
                                                .unwrap_or("");

                                            // If tokens don't match, return unauthorized
                                            if validation_token != *token {
                                                return Ok(static_response(
                                                    StatusCode::UNAUTHORIZED,
                                                    "Unauthorized: Invalid validation token",
                                                ));
                                            }
                                        }

                                        // Read the request body
                                        let body = req.into_body();

                                        // Collect the body data
                                        let body_bytes = match body.collect().await {
                                            Ok(collected) => collected.to_bytes(),
                                            Err(_) => {
                                                return Ok(static_response(
                                                    StatusCode::BAD_REQUEST,
                                                    "Bad request body",
                                                ));
                                            }
                                        };

                                        // Parse the JSON payload
                                        let payload: Value =
                                            match serde_json::from_slice(&body_bytes) {
                                                Ok(json) => json,
                                                Err(_) => {
                                                    return Ok(static_response(
                                                        StatusCode::BAD_REQUEST,
                                                        "Invalid JSON payload",
                                                    ));
                                                }
                                            };

                                        // Send the notification payload to the channel
                                        let sender_guard = sender.lock().await;
                                        if let Err(e) = sender_guard.send(payload.clone()).await {
                                            eprintln!("Failed to send notification: {}", e);
                                        } else {
                                            println!("Received webhook notification");
                                        }

                                        Ok(static_response(StatusCode::OK, "OK"))
                                    }
                                    _ => {
                                        // Return 404 Not Found for other paths
                                        Ok(static_response(StatusCode::NOT_FOUND, "Not Found"))
                                    }
                                }
                            }
                        });

                        // Process HTTP1 connection
                        if let Err(err) = http1::Builder::new().serve_connection(io, service).await
                        {
                            eprintln!("Error serving connection: {:?}", err);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Connection error: {}", e);
                }
            }
        }
    }
}

/// Create an A2A client with default configuration
fn create_client(server_url: &str) -> Result<Client> {
    let config = ClientConfig::new(server_url);
    Ok(Client::new(config)?)
}

/// Send a message to the agent with push notification configuration
async fn send_task_with_push_notification(
    client: &Client,
    task_id: &str,
    message_text: &str,
    webhook_url: &str,
    webhook_token: &str,
) -> Result<Task> {
    // Create push notification config
    let push_config = PushNotificationConfig {
        url: webhook_url.to_string(),
        token: Some(webhook_token.to_string()),
        authentication: None,
    };

    // Create a message
    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: message_text.to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    // Create task parameters with push notification config
    let params = TaskSendParams {
        id: task_id.to_string(),
        message,
        session_id: None,
        push_notification: Some(push_config),
        history_length: None,
        metadata: None,
    };

    // Create the request
    let request = params.into_send_request(serde_json::json!("task-id"));

    // Send the request
    Ok(client.send(request).await?)
}

/// Configure push notifications for an existing task
async fn set_push_notification_for_task(
    client: &Client,
    task_id: &str,
    webhook_url: &str,
    webhook_token: &str,
) -> Result<acai::TaskPushNotificationConfig> {
    // Create push notification config
    let push_config = PushNotificationConfig {
        url: webhook_url.to_string(),
        token: Some(webhook_token.to_string()),
        authentication: None,
    };

    // Set the push notification for the task
    Ok(client.set_push_notification(task_id, push_config).await?)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Configuration
    let server_url = "http://127.0.0.1:3000";
    let webhook_port = 8090;
    let webhook_token = "secret-webhook-token";
    // Base URL for the webhook server - we'll add the token separately
    let webhook_url = format!("http://127.0.0.1:{}/webhook", webhook_port);

    // Create and start webhook server
    let (webhook_server, mut notification_receiver) =
        WebhookServer::new(webhook_port, webhook_token);

    // Start the webhook server in a background task
    tokio::spawn(async move {
        webhook_server.start().await;
    });

    // Give the webhook server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    println!("Creating A2A client for {}", server_url);
    let client = create_client(server_url)?;

    // Example 1: Send a task with push notification configuration
    let task_id_1 = Uuid::new_v4().to_string();
    println!(
        "\n--- Sending a task with push notification config (task ID: {}) ---",
        task_id_1
    );

    let webhook_url_with_token = format!("{}?validationToken={}", webhook_url, webhook_token);
    match send_task_with_push_notification(
        &client,
        &task_id_1,
        "Please analyze this text and notify me when complete",
        &webhook_url_with_token,
        webhook_token,
    )
    .await
    {
        Ok(task) => {
            println!("Task sent successfully!");
            println!("Task state: {:?}", task.status.state);
        }
        Err(e) => {
            println!("Failed to send task: {}", e);
        }
    }

    // Example 2: Set up push notification for an existing task
    let task_id_2 = Uuid::new_v4().to_string();
    println!(
        "\n--- Setting up push notification for a task (task ID: {}) ---",
        task_id_2
    );

    // First create a task without push notification
    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: "Another test task".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    let params = TaskSendParams {
        id: task_id_2.to_string(),
        message,
        session_id: None,
        push_notification: None, // No push notification initially
        history_length: None,
        metadata: None,
    };

    let request = params.into_send_request(serde_json::json!("task-id"));

    match client.send::<_, Task>(request).await {
        Ok(task) => {
            println!("Task created successfully!");
            println!("Task state: {:?}", task.status.state);

            // Now add push notification to the existing task
            let webhook_url_with_token =
                format!("{}?validationToken={}", webhook_url, webhook_token);
            match set_push_notification_for_task(
                &client,
                &task_id_2,
                &webhook_url_with_token,
                webhook_token,
            )
            .await
            {
                Ok(config) => {
                    println!(
                        "Push notification configured with URL: {}",
                        config.push_notification_config.url
                    );
                }
                Err(e) => {
                    println!("Failed to configure push notification: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create task: {}", e);
        }
    }

    // Wait for and process notifications
    println!("\n--- Waiting for push notifications (press Ctrl+C to exit) ---");
    println!("Note: You may need to trigger state changes on the server for the tasks");

    // Simple loop to receive and display notifications
    while let Ok(notification) = tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        notification_receiver.recv(),
    )
    .await
    {
        match notification {
            Some(payload) => {
                println!("\nReceived push notification:");
                println!("{}", serde_json::to_string_pretty(&payload).unwrap());

                // Extract and display task ID and state if available
                if let Some(task_id) = payload.get("id").and_then(|v| v.as_str()) {
                    println!("Task ID: {}", task_id);

                    if let Some(status) = payload.get("status") {
                        if let Some(state) = status.get("state").and_then(|v| v.as_str()) {
                            println!("Task state: {}", state);
                        }
                    }
                }
            }
            None => {
                println!("Notification channel closed");
                break;
            }
        }
    }

    println!("\nPush notification example completed (timeout or channel closed)");
    Ok(())
}
