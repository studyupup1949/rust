use acai::{
    AgentCard, Message, MessageRole, Part, PushNotificationConfig, StreamingResponseContent, Task,
    TaskIdParams, TaskPushNotificationConfig, TaskQueryParams, TaskSendParams, TaskState,
    client::{Client, ClientConfig, Error as ClientError},
};
use futures::StreamExt;
use std::time::Duration;
use uuid::Uuid;

/// Create a basic client for the A2A protocol
async fn create_client(
    server_url: &str,
    auth_token: Option<&str>,
    timeout: Option<Duration>,
) -> Result<Client, ClientError> {
    // Create base configuration
    let mut config = ClientConfig::new(server_url);

    // Add authentication if provided
    if let Some(token) = auth_token {
        config = config.with_auth_token(token);
    }

    // Set custom timeout if provided
    if let Some(timeout) = timeout {
        config = config.with_timeout(timeout);
    }

    // Create the client
    Client::new(config)
}

/// Create and send a basic text message to an agent
async fn send_text_message(
    client: &Client,
    task_id: &str,
    text: &str,
) -> Result<Task, ClientError> {
    // Create a message with text content
    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: text.to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    // Create task parameters
    let params = TaskSendParams {
        id: task_id.to_string(),
        message,
        session_id: None,
        push_notification: None,
        history_length: None,
        metadata: None,
    };

    // Create the request
    let request = params.into_send_request(serde_json::json!("request-id"));

    // Send the request
    client.send(request).await
}

/// Get information about a task
async fn get_task(
    client: &Client,
    task_id: &str,
    history_length: Option<i32>,
) -> Result<Task, ClientError> {
    // Create task query parameters
    let params = TaskQueryParams {
        id: task_id.to_string(),
        history_length,
        metadata: None,
    };

    // Create the request
    let request = params.into_get_request(serde_json::json!("get-task-request"));

    // Send the request
    client.send(request).await
}

/// Cancel a task
async fn cancel_task(client: &Client, task_id: &str) -> Result<Task, ClientError> {
    // Create task ID parameters
    let params = TaskIdParams {
        id: task_id.to_string(),
        metadata: None,
    };

    // Create the request
    let request = params.into_cancel_request(serde_json::json!("cancel-task-request"));

    // Send the request
    client.send(request).await
}

/// Configure push notifications for a task
async fn set_push_notification(
    client: &Client,
    task_id: &str,
    webhook_url: &str,
    webhook_token: Option<&str>,
) -> Result<TaskPushNotificationConfig, ClientError> {
    // Create a push notification configuration
    let config = PushNotificationConfig {
        url: webhook_url.to_string(),
        token: webhook_token.map(|t| t.to_string()),
        authentication: None,
    };

    // Create and send the request
    client.set_push_notification(task_id, config).await
}

/// Get push notification configuration for a task
async fn get_push_notification(
    client: &Client,
    task_id: &str,
) -> Result<TaskPushNotificationConfig, ClientError> {
    // Create and send the request
    client.get_push_notification(task_id).await
}

/// Send a message to an agent and receive streaming responses
async fn stream_task_responses(
    client: &Client,
    task_id: &str,
    text: &str,
) -> Result<(), ClientError> {
    // Create a message with text content
    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: text.to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    // Create task parameters
    let params = TaskSendParams {
        id: task_id.to_string(),
        message,
        session_id: None,
        push_notification: None,
        history_length: None,
        metadata: None,
    };

    // Create a streaming request with a unique ID
    let request = params.into_send_subscribe_request(serde_json::json!(Uuid::new_v4().to_string()));

    // Send the request and get a stream of responses
    let mut stream = client.stream(request).await?;

    println!("Streaming responses for task {}:", task_id);

    // Process each event in the stream
    while let Some(event) = stream.next().await {
        match event {
            Ok(response) => {
                // Handle response based on what it contains
                if let Some(content) = &response.result {
                    match content {
                        StreamingResponseContent::StatusUpdate(status_event) => {
                            println!(
                                "Status: {:?} - Final: {}",
                                status_event.status.state, status_event.final_status
                            );

                            // If the status includes a message, print it
                            if let Some(message) = &status_event.status.message {
                                for part in &message.parts {
                                    if let Part::Text { text, .. } = part {
                                        println!("Message: {}", text);
                                    }
                                }
                            }

                            // If this is the final status and it's completed, exit the loop
                            if status_event.final_status
                                && status_event.status.state == TaskState::Completed
                            {
                                println!("Task completed successfully");
                                break;
                            }

                            // If task failed or was canceled, exit the loop
                            if status_event.status.state == TaskState::Failed
                                || status_event.status.state == TaskState::Canceled
                            {
                                println!("Task ended with state: {:?}", status_event.status.state);
                                break;
                            }
                        }
                        StreamingResponseContent::ArtifactUpdate(artifact_event) => {
                            println!(
                                "Artifact: {} (index: {})",
                                artifact_event
                                    .artifact
                                    .name
                                    .as_ref()
                                    .map_or("Unnamed", |s| s.as_str()),
                                artifact_event.artifact.index
                            );

                            // Print the artifact contents if it's text
                            for part in &artifact_event.artifact.parts {
                                if let Part::Text { text, .. } = part {
                                    println!("Content: {}", text);
                                }
                            }

                            // Check if this is the last chunk of the artifact
                            if let Some(true) = artifact_event.artifact.last_chunk {
                                println!("This was the final chunk of the artifact");
                            }
                        }
                    }
                } else if let Some(error) = &response.error {
                    // It's an error response
                    println!("Error: Code {} - {}", error.code, error.message);
                    break;
                }
            }
            Err(e) => {
                println!("Stream error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Fetch agent capabilities from the well-known endpoint
async fn fetch_agent_card(client: &Client) -> Result<AgentCard, ClientError> {
    client.fetch_agent_card().await
}

/// Display agent capabilities in a human-readable format
fn display_agent_capabilities(card: &AgentCard) {
    println!("Agent: {}", card.name);
    if let Some(desc) = &card.description {
        println!("Description: {}", desc);
    }

    println!("Version: {}", card.version);

    if let Some(provider) = &card.provider {
        println!(
            "Provider: {} ({})",
            provider.organization,
            provider.url.as_deref().unwrap_or("No URL")
        );
    }

    println!("Capabilities:");
    println!("  Streaming: {}", card.capabilities.streaming);
    println!(
        "  Push Notifications: {}",
        card.capabilities.push_notifications
    );
    println!(
        "  State Transition History: {}",
        card.capabilities.state_transition_history
    );

    println!("Input Modes: {}", card.default_input_modes.join(", "));
    println!("Output Modes: {}", card.default_output_modes.join(", "));

    if !card.skills.is_empty() {
        println!("Skills:");
        for skill in &card.skills {
            println!("  {} ({})", skill.name, skill.id);
            if let Some(desc) = &skill.description {
                println!("    Description: {}", desc);
            }
            if let Some(tags) = &skill.tags {
                println!("    Tags: {}", tags.join(", "));
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration
    let server_url = "http://127.0.0.1:3000";
    let auth_token = None; // Optional auth token
    let custom_timeout = Some(Duration::from_secs(60));

    println!("Creating A2A client for {}", server_url);
    let client = create_client(server_url, auth_token, custom_timeout).await?;

    // Generate a unique task ID
    let task_id = Uuid::new_v4().to_string();
    println!("Using task ID: {}", task_id);

    // Try to fetch the agent card
    match fetch_agent_card(&client).await {
        Ok(card) => {
            println!("\n--- Agent Card ---");
            display_agent_capabilities(&card);
        }
        Err(e) => {
            println!("Failed to fetch agent card: {}", e);
        }
    }

    // Sending a basic message
    println!("\n--- Sending a message ---");
    match send_text_message(&client, &task_id, "Hello, this is a test message").await {
        Ok(task) => {
            println!("Message sent successfully!");
            println!("Task state: {:?}", task.status.state);
        }
        Err(e) => {
            println!("Failed to send message: {}", e);
        }
    }

    // Getting task information
    println!("\n--- Getting task information ---");
    match get_task(&client, &task_id, Some(5)).await {
        Ok(task) => {
            println!("Task state: {:?}", task.status.state);
            if let Some(history) = task.history {
                println!("Message history: {} messages", history.len());
            }
        }
        Err(e) => {
            println!("Failed to get task: {}", e);
        }
    }

    // Setting up push notifications
    println!("\n--- Setting up push notifications ---");
    match set_push_notification(
        &client,
        &task_id,
        "https://example.com/webhook",
        Some("webhook-secret-token"),
    )
    .await
    {
        Ok(config) => {
            println!(
                "Push notifications configured for URL: {}",
                config.push_notification_config.url
            );
        }
        Err(e) => {
            println!("Failed to set push notification: {}", e);
        }
    }

    // Getting push notification configuration
    println!("\n--- Getting push notification configuration ---");
    match get_push_notification(&client, &task_id).await {
        Ok(config) => {
            println!(
                "Push notification URL: {}",
                config.push_notification_config.url
            );
        }
        Err(e) => {
            println!("Failed to get push notification config: {}", e);
        }
    }

    // Streaming task responses (using a new task ID)
    let streaming_task_id = Uuid::new_v4().to_string();
    println!(
        "\n--- Streaming task responses (task ID: {}) ---",
        streaming_task_id
    );

    if let Err(e) = stream_task_responses(
        &client,
        &streaming_task_id,
        "Generate a short poem about AI agents communicating with each other",
    )
    .await
    {
        println!("Failed to stream task responses: {}", e);
    }

    // Canceling a task
    println!("\n--- Canceling a task ---");
    match cancel_task(&client, &task_id).await {
        Ok(task) => {
            println!("Task canceled successfully");
            println!("Task state: {:?}", task.status.state);
        }
        Err(e) => {
            println!("Failed to cancel task: {}", e);
        }
    }

    println!("\nAll A2A Protocol operations completed.");
    Ok(())
}
