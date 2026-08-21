use std::sync::{Arc, Mutex};
use std::time::Duration;

use acai::{
    AgentCapabilities, AgentCard, AgentProvider, AgentSkill, Artifact, JsonRpcError,
    JsonRpcRequest, Message, MessageRole, Part, StreamingResponseContent, Task,
    TaskArtifactUpdateEvent, TaskSendParams, TaskState, TaskStatus, TaskStatusUpdateEvent,
    client::{Client, ClientConfig},
    server::{MethodRouter, Server, ServerConfig, make_typed_handler},
};
use futures::StreamExt;
use tokio::sync::mpsc;
use uuid::Uuid;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
// Define a type alias for our complex sender type to improve readability
type StreamSender = mpsc::Sender<std::result::Result<serde_json::Value, JsonRpcError>>;

// Stream handler for handling tasks/sendSubscribe
#[allow(clippy::type_complexity)]
async fn stream_handler(
    req: JsonRpcRequest<TaskSendParams>,
    // We use a channel to simulate the streaming responses
    tx: Arc<Mutex<Option<StreamSender>>>,
) -> std::result::Result<serde_json::Value, JsonRpcError> {
    let params = req.params();
    let task_id = params.id.clone();
    let message_text = params
        .message
        .parts
        .iter()
        .filter_map(|part| match part {
            Part::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<String>>()
        .join(" ");

    // Echo back what the user sent
    println!("Received streaming request with task ID: {}", task_id);
    println!("Message: {}", message_text);

    // Create a new channel for sending streaming updates
    let (new_tx, _rx) = mpsc::channel::<std::result::Result<serde_json::Value, JsonRpcError>>(10);

    // Store the sender for later use
    // SAFETY(claude): Mutex poisoning.
    let mut tx_guard = tx.lock().unwrap();
    *tx_guard = Some(new_tx);
    drop(tx_guard);

    // Initial "working" status
    let initial_status = TaskStatusUpdateEvent {
        id: task_id.clone(),
        status: TaskStatus {
            state: TaskState::Working,
            message: Some(Message {
                role: MessageRole::Agent,
                parts: vec![Part::Text {
                    text: "Processing your request...".to_string(),
                    metadata: None,
                }],
                metadata: None,
            }),
            timestamp: Some(chrono::Utc::now()),
        },
        final_status: false,
        metadata: None,
    };

    // Create a sender for the spawned task
    let tx_clone = tx.clone();

    // Spawn a task to send streaming updates
    tokio::spawn(async move {
        // Simulate processing time
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Send intermediate updates
        for i in 1..=3 {
            let update = TaskStatusUpdateEvent {
                id: task_id.clone(),
                status: TaskStatus {
                    state: TaskState::Working,
                    message: Some(Message {
                        role: MessageRole::Agent,
                        parts: vec![Part::Text {
                            text: format!("Progress update {}/3...", i),
                            metadata: None,
                        }],
                        metadata: None,
                    }),
                    timestamp: Some(chrono::Utc::now()),
                },
                final_status: false,
                metadata: None,
            };

            // Convert to JSON Value
            let json_value = match serde_json::to_value(update) {
                Ok(val) => val,
                Err(e) => {
                    eprintln!("Error serializing update: {}", e);
                    return;
                }
            };

            // Send the update
            // SAFETY(claude): Mutex poisoning.
            if let Some(sender) = tx_clone.lock().unwrap().as_ref() {
                if sender.try_send(Ok(json_value)).is_err() {
                    eprintln!("Failed to send update - channel full or closed");
                    return;
                }
            }

            // Wait between updates
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        // Send a sample artifact
        let artifact_update = TaskArtifactUpdateEvent {
            id: task_id.clone(),
            artifact: Artifact {
                name: Some("Generated Response".to_string()),
                description: Some("The completed response to your request".to_string()),
                parts: vec![Part::Text {
                    text: format!("Here's your response to: {}", message_text),
                    metadata: None,
                }],
                index: 0,
                append: None,
                last_chunk: Some(true),
                metadata: None,
            },
            metadata: None,
        };

        // Convert to JSON Value
        let json_value = match serde_json::to_value(artifact_update) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error serializing artifact: {}", e);
                return;
            }
        };

        // Send the artifact
        // SAFETY(claude): Mutex poisoning.
        if let Some(sender) = tx_clone.lock().unwrap().as_ref() {
            if sender.try_send(Ok(json_value)).is_err() {
                eprintln!("Failed to send artifact - channel full or closed");
                return;
            }
        }

        // Wait a bit before completing
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Send final "completed" status
        let final_status = TaskStatusUpdateEvent {
            id: task_id.clone(),
            status: TaskStatus {
                state: TaskState::Completed,
                message: Some(Message {
                    role: MessageRole::Agent,
                    parts: vec![Part::Text {
                        text: "Your request has been processed successfully.".to_string(),
                        metadata: None,
                    }],
                    metadata: None,
                }),
                timestamp: Some(chrono::Utc::now()),
            },
            final_status: true,
            metadata: None,
        };

        // Convert to JSON Value
        let json_value = match serde_json::to_value(final_status) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Error serializing final status: {}", e);
                return;
            }
        };

        // Send the final update
        // SAFETY(claude): Mutex poisoning.
        if let Some(sender) = tx_clone.lock().unwrap().as_ref() {
            if sender.try_send(Ok(json_value)).is_err() {
                eprintln!("Failed to send final status - channel full or closed");
            }
        }
    });

    // Return the initial status immediately
    serde_json::to_value(initial_status).map_err(JsonRpcError::serialization)
}

// Standard handler for tasks/send
async fn send_handler(
    req: JsonRpcRequest<TaskSendParams>,
) -> std::result::Result<Task, JsonRpcError> {
    let params = req.params();

    // Echo back what the user sent
    println!("Received standard request with task ID: {}", params.id);

    // Create a response message
    let response_message = Message {
        role: MessageRole::Agent,
        parts: vec![Part::Text {
            text: "This is a standard (non-streaming) response".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    // Create a completed task
    let task = Task {
        id: params.id.clone(),
        session_id: params.session_id.clone(),
        status: TaskStatus {
            state: TaskState::Completed,
            message: Some(response_message.clone()),
            timestamp: Some(chrono::Utc::now()),
        },
        artifacts: Some(vec![Artifact {
            name: Some("Response".to_string()),
            description: None,
            parts: vec![Part::Text {
                text: "Standard response content".to_string(),
                metadata: None,
            }],
            index: 0,
            append: None,
            last_chunk: None,
            metadata: None,
        }]),
        history: Some(vec![params.message.clone(), response_message]),
        metadata: None,
    };

    Ok(task)
}

// Start server function
async fn start_server() -> Result<()> {
    // Create a channel to hold the sender for streaming updates
    let tx = Arc::new(Mutex::new(None::<StreamSender>));

    // Create a router
    let mut router = MethodRouter::new();

    // Create an empty state for the standard send handler
    struct EmptyState;
    let empty_state = Arc::new(EmptyState);

    // Register handlers for tasks/send using make_typed_handler
    router.register(
        "tasks/send",
        make_typed_handler(
            empty_state.clone(),
            |_state: Arc<EmptyState>, params: TaskSendParams| async move {
                send_handler(JsonRpcRequest::new(
                    serde_json::json!(null),
                    "tasks/send",
                    params,
                ))
                .await
            },
        ),
    );

    // Create a state struct for the streaming handler
    struct StreamHandlerState {
        tx: Arc<Mutex<Option<StreamSender>>>,
    }

    // For the streaming handler, we need to capture the tx channel
    let stream_state = Arc::new(StreamHandlerState { tx: tx.clone() });

    // Register the streaming handler using make_typed_handler
    router.register(
        "tasks/sendSubscribe",
        make_typed_handler(
            stream_state,
            |state: Arc<StreamHandlerState>, params: TaskSendParams| async move {
                // Create a request to pass to the stream_handler
                let req =
                    JsonRpcRequest::new(serde_json::json!(null), "tasks/sendSubscribe", params);
                stream_handler(req, state.tx.clone()).await
            },
        ),
    );

    // Create server configuration
    let config = ServerConfig::new("127.0.0.1:3002")?;

    // Create an agent card
    let agent_card = AgentCard {
        name: "Streaming Echo Agent".to_string(),
        description: Some("A simple streaming agent that demonstrates SSE".to_string()),
        url: "http://127.0.0.1:3002".to_string(),
        provider: Some(AgentProvider {
            organization: "Acai Project".to_string(),
            url: Some("https://github.com/rescrv/acai".to_string()),
        }),
        version: "0.1.0".to_string(),
        documentation_url: Some("https://github.com/rescrv/acai".to_string()),
        capabilities: AgentCapabilities {
            streaming: true,
            push_notifications: false,
            state_transition_history: false,
        },
        authentication: None,
        default_input_modes: vec!["text".to_string()],
        default_output_modes: vec!["text".to_string()],
        skills: vec![AgentSkill {
            id: "stream".to_string(),
            name: "Streaming Echo".to_string(),
            description: Some("Echoes back messages with streaming updates".to_string()),
            tags: Some(vec!["echo".to_string(), "streaming".to_string()]),
            examples: Some(vec!["Stream this message".to_string()]),
            input_modes: Some(vec!["text".to_string()]),
            output_modes: Some(vec!["text".to_string()]),
        }],
    };

    // Create the server and add the agent card
    let server = Server::new(config, Arc::new(router)).with_agent_card(agent_card);

    println!("A2A streaming server listening on http://127.0.0.1:3002");
    println!("Agent card available at http://127.0.0.1:3002/.well-known/agent.json");

    server.serve().await.map_err(|e| e.into())
}

async fn run_streaming_client() -> Result<()> {
    println!("Starting streaming client...");

    // Connect to the server
    let config = ClientConfig::new("http://127.0.0.1:3002");
    let client = Client::new(config)?;

    // Create a unique task ID
    let task_id = format!("task_{}", Uuid::new_v4().simple());

    // Create a message
    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: "This is a streaming test message".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    // Create parameters
    let params = TaskSendParams {
        id: task_id.clone(),
        message,
        session_id: None,
        push_notification: None,
        history_length: None,
        metadata: None,
    };

    // Create the streaming request
    let request = params.into_send_subscribe_request(serde_json::json!("stream-request-id"));

    println!("Sending streaming request with task ID: {}", task_id);
    println!("Streaming events will appear below:");
    println!("-----------------------------------");

    // Stream the response and process each event
    let mut stream = client.stream(request).await?;
    let mut event_count = 0;

    while let Some(event) = stream.next().await {
        event_count += 1;

        match event {
            Ok(response) => {
                // Handle response based on what it contains
                if let Some(content) = &response.result {
                    match content {
                        StreamingResponseContent::StatusUpdate(status_event) => {
                            println!("\nSTATUS UPDATE (Event #{})", event_count);
                            println!("Task ID: {}", status_event.id);
                            println!("State: {:?}", status_event.status.state);

                            if let Some(timestamp) = &status_event.status.timestamp {
                                println!("Timestamp: {}", timestamp);
                            }

                            if let Some(msg) = &status_event.status.message {
                                if let Some(text_part) = msg.parts.iter().find_map(|p| match p {
                                    Part::Text { text, .. } => Some(text),
                                    _ => None,
                                }) {
                                    println!("Message: {}", text_part);
                                }
                            }

                            println!("Final: {}", status_event.final_status);

                            // If this is the final update, we're done
                            if status_event.final_status {
                                println!("\nReceived final update - stream complete");
                                break;
                            }
                        }
                        StreamingResponseContent::ArtifactUpdate(artifact_event) => {
                            println!("\nARTIFACT UPDATE (Event #{})", event_count);
                            println!("Task ID: {}", artifact_event.id);
                            println!(
                                "Artifact Name: {}",
                                artifact_event
                                    .artifact
                                    .name
                                    .as_ref()
                                    .map_or_else(String::new, |s| s.clone())
                            );
                            println!(
                                "Description: {}",
                                artifact_event
                                    .artifact
                                    .description
                                    .as_ref()
                                    .map_or_else(String::new, |s| s.clone())
                            );

                            for part in &artifact_event.artifact.parts {
                                match part {
                                    Part::Text { text, .. } => {
                                        println!("Content: {}", text);
                                    }
                                    Part::File { .. } => {
                                        println!("Content: [File content]");
                                    }
                                    Part::Data { data, .. } => {
                                        println!("Content: [Data: {:?}]", data);
                                    }
                                }
                            }
                        }
                    }
                } else if let Some(error) = &response.error {
                    // It's an error response
                    println!("\nERROR (Event #{})", event_count);
                    println!("Code: {}", error.code);
                    println!("Message: {}", error.message);
                    println!("Data: {:?}", error.data);
                    break;
                }
            }
            Err(e) => {
                println!("\nStream error: {:?}", e);
                break;
            }
        }
    }

    println!("-----------------------------------");
    println!(
        "Streaming session complete. Received {} events",
        event_count
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Check for client or server mode
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "client" {
        run_streaming_client().await
    } else {
        start_server().await
    }
}
