use std::io::{self, Write};
use std::time::Duration;
use uuid::Uuid;

use acai::{
    JsonRpcResponse, Message, MessageRole, Part, Task, TaskQueryParams, TaskSendParams, TaskState,
};
use reqwest::{Client, Url};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

// Command handlers
async fn send_message(
    client: &Client,
    agent_url: &str,
    task_id: &str,
    message: &str,
    session_id: Option<String>,
) -> Result<Task, String> {
    // Create message
    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: message.to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    // Create send params
    let params = TaskSendParams {
        id: task_id.to_string(),
        message,
        session_id,
        push_notification: None,
        history_length: None,
        metadata: None,
    };

    // Create request
    let id = serde_json::json!(Uuid::new_v4().to_string());
    let request = params.into_send_request(id);

    // Send request
    let response = client
        .post(agent_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    // Parse the JSON response directly
    let response: JsonRpcResponse<Task> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response JSON: {}", e))?;

    // Check for errors
    if let Some(error) = response.get_error() {
        return Err(format!("Error: {}", error.message));
    }

    // Return the task
    response
        .result()
        .cloned()
        .ok_or_else(|| "No task in response".to_string())
}

async fn get_task(client: &Client, agent_url: &str, task_id: &str) -> Result<Task, String> {
    // Create query params
    let params = TaskQueryParams {
        id: task_id.to_string(),
        history_length: None,
        metadata: None,
    };

    // Create request
    let id = serde_json::json!(Uuid::new_v4().to_string());
    let request = params.into_get_request(id);

    // Send request
    let response = client
        .post(agent_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    // Parse the JSON response directly
    let response: JsonRpcResponse<Task> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response JSON: {}", e))?;

    // Check for errors
    if let Some(error) = response.get_error() {
        return Err(format!("Error: {}", error.message));
    }

    // Return the task
    response
        .result()
        .cloned()
        .ok_or_else(|| "No task in response".to_string())
}

// Function to poll for updates
async fn poll_until_complete(
    client: &Client,
    agent_url: &str,
    task_id: &str,
) -> Result<Task, String> {
    let mut last_history_len = 0;

    loop {
        let task = get_task(client, agent_url, task_id).await?;

        // Print any new messages
        if let Some(history) = &task.history {
            if history.len() > last_history_len {
                for msg in history.iter().skip(last_history_len) {
                    // Only print agent messages (user messages were input by the user)
                    // Compare the role as a string to avoid PartialEq requirement
                    if matches!(msg.role, MessageRole::Agent) {
                        print_message(msg);
                    }
                }
                last_history_len = history.len();
            }
        }

        // Check if task is complete
        match task.status.state {
            TaskState::Completed | TaskState::Failed | TaskState::Canceled => {
                return Ok(task);
            }
            _ => {
                // Sleep before polling again
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

// Helper to print messages
fn print_message(message: &Message) {
    print!("\nAgent: ");
    io::stdout().flush().unwrap();

    for part in &message.parts {
        match part {
            Part::Text { text, .. } => {
                println!("{}", text);
            }
            Part::File { file, .. } => {
                println!("[File: {}]", file.name.as_deref().unwrap_or("unnamed"));
            }
            Part::Data { data, .. } => {
                println!("[Data: {:?}]", data);
            }
        }
    }
    println!();
}

// Display help message
fn print_help() {
    println!("\nA2A Chat - Commands:");
    println!("  /help              - Show this help message");
    println!("  /quit              - Exit the program");
    println!("  /connect <url>     - Connect to an A2A agent at the specified URL");
    println!("  /new               - Start a new conversation (creates a new task)");
    println!("  /history           - Show conversation history");
    println!("  /status            - Show current task status");
    println!();
    println!("Just type your message and press Enter to send it to the agent.");
    println!();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the HTTP client
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    // Initialize rustyline for better command-line input
    let mut rl = DefaultEditor::new()?;

    // Default to localhost:3000 if available
    let mut agent_url: Option<String> = Some("http://localhost:3000".to_string());
    let mut current_task_id: Option<String> = Some(Uuid::new_v4().to_string());
    let mut session_id: Option<String> = Some(Uuid::new_v4().to_string());

    println!("A2A Chat Client");
    println!("Connected to http://localhost:3000");
    println!(
        "New conversation started with task ID: {}",
        current_task_id.as_ref().unwrap()
    );
    println!("Type /help for available commands");

    loop {
        // We always have an agent URL now, but keeping the logic for clarity
        let prompt = "You: ";

        let readline = rl.readline(prompt);

        match readline {
            Ok(line) => {
                rl.add_history_entry(line.clone())?;
                let input = line.trim();

                if input.starts_with('/') {
                    // Command handling
                    let parts: Vec<&str> = input.splitn(2, ' ').collect();
                    let command = parts[0];
                    let args = parts.get(1).map(|s| s.trim());

                    match command {
                        "/help" => {
                            print_help();
                        }
                        "/quit" => {
                            println!("Goodbye!");
                            break;
                        }
                        "/connect" => {
                            if let Some(url) = args {
                                // Validate URL
                                match Url::parse(url) {
                                    Ok(_) => {
                                        agent_url = Some(url.to_string());
                                        println!("Connected to {}", url);

                                        // Create a new session ID for this connection
                                        session_id = Some(Uuid::new_v4().to_string());

                                        // Start with a new task
                                        current_task_id = Some(Uuid::new_v4().to_string());
                                        println!(
                                            "New conversation started with task ID: {}",
                                            current_task_id.as_ref().unwrap()
                                        );
                                    }
                                    Err(_) => {
                                        println!("Invalid URL format. Please provide a valid URL.");
                                    }
                                }
                            } else {
                                println!("Please provide a URL. Usage: /connect <url>");
                            }
                        }
                        "/new" => {
                            if agent_url.is_some() {
                                current_task_id = Some(Uuid::new_v4().to_string());
                                println!(
                                    "New conversation started with task ID: {}",
                                    current_task_id.as_ref().unwrap()
                                );
                            } else {
                                println!("Not connected to any agent. Use /connect <url> first.");
                            }
                        }
                        "/history" => {
                            if let (Some(url), Some(task_id)) = (&agent_url, &current_task_id) {
                                match get_task(&client, url, task_id).await {
                                    Ok(task) => {
                                        if let Some(history) = task.history {
                                            println!("\nConversation history:");
                                            for (i, msg) in history.iter().enumerate() {
                                                let role = match msg.role {
                                                    MessageRole::User => "You",
                                                    MessageRole::Agent => "Agent",
                                                    MessageRole::System => "System",
                                                };

                                                print!("\n[{}] {}: ", i + 1, role);

                                                for part in &msg.parts {
                                                    if let Part::Text { text, .. } = part {
                                                        println!("{}", text);
                                                    }
                                                }
                                            }
                                            println!();
                                        } else {
                                            println!("No conversation history yet.");
                                        }
                                    }
                                    Err(e) => {
                                        println!("Error retrieving history: {}", e);
                                    }
                                }
                            } else {
                                println!(
                                    "Not in an active conversation. Use /connect and /new first."
                                );
                            }
                        }
                        "/status" => {
                            if let (Some(url), Some(task_id)) = (&agent_url, &current_task_id) {
                                match get_task(&client, url, task_id).await {
                                    Ok(task) => {
                                        println!("\nTask status: {:?}", task.status.state);
                                        if let Some(msg) = &task.status.message {
                                            print_message(msg);
                                        }
                                        if let Some(timestamp) = &task.status.timestamp {
                                            println!("Last updated: {}", timestamp);
                                        }
                                        println!();
                                    }
                                    Err(e) => {
                                        println!("Error retrieving status: {}", e);
                                    }
                                }
                            } else {
                                println!(
                                    "Not in an active conversation. Use /connect and /new first."
                                );
                            }
                        }
                        _ => {
                            println!("Unknown command. Type /help for available commands.");
                        }
                    }
                } else if !input.is_empty() {
                    // Regular message - send to agent
                    if let (Some(url), Some(task_id)) = (&agent_url, &current_task_id) {
                        print!("Sending message... ");
                        io::stdout().flush().unwrap();

                        match send_message(&client, url, task_id, input, session_id.clone()).await {
                            Ok(_) => {
                                println!("sent.");

                                // Poll for response
                                print!("Waiting for response... ");
                                io::stdout().flush().unwrap();

                                match poll_until_complete(&client, url, task_id).await {
                                    Ok(_) => { /* Responses are printed in poll_until_complete */ }
                                    Err(e) => {
                                        println!("\nError: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("\nError sending message: {}", e);
                            }
                        }
                    } else {
                        println!("Not connected to any agent. Use /connect <url> first.");
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl-C pressed. Type /quit to exit.");
            }
            Err(ReadlineError::Eof) => {
                println!("Ctrl-D pressed. Exiting.");
                break;
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }

    Ok(())
}
