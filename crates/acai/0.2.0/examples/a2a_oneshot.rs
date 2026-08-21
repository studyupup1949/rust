use std::env;
use std::time::Duration;
use uuid::Uuid;

use acai::{
    JsonRpcResponse, Message, MessageRole, Part, Task, TaskQueryParams, TaskSendParams, TaskState,
};
use reqwest::Client;

// Send a message and get the response, with fallback for servers without tasks/get
async fn oneshot_message(
    client: &Client,
    agent_url: &str,
    message: &str,
    poll: bool,
) -> Result<Task, String> {
    // Generate unique IDs
    let task_id = Uuid::new_v4().to_string();
    let session_id = Some(Uuid::new_v4().to_string());

    println!("Sending message to agent at {}...", agent_url);
    println!("Task ID: {}", task_id);

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
        id: task_id.clone(),
        message,
        session_id: session_id.clone(),
        push_notification: None,
        history_length: None,
        metadata: None,
    };

    // Create request
    let request = params.into_send_request(serde_json::json!(Uuid::new_v4().to_string()));

    // Send request
    let response = client
        .post(agent_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    // Parse the JSON response
    let response: JsonRpcResponse<Task> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response JSON: {}", e))?;

    // Check for errors
    if let Some(error) = response.get_error() {
        return Err(format!("Error: {}", error.message));
    }

    // Get the initial task response
    let initial_task = response
        .result()
        .cloned()
        .ok_or_else(|| "No task in response".to_string())?;

    // If polling is disabled, return initial task
    if !poll {
        return Ok(initial_task);
    }

    // If task is already completed, print the response and return it
    if matches!(
        initial_task.status.state,
        TaskState::Completed | TaskState::Failed | TaskState::Canceled
    ) {
        if let Some(status_msg) = &initial_task.status.message {
            if matches!(status_msg.role, MessageRole::Agent) {
                print_message(status_msg);
            }
        }
        return Ok(initial_task);
    }

    // Poll for updates until we have a complete response
    match poll_until_complete(client, agent_url, &task_id).await {
        Ok(final_task) => Ok(final_task),
        Err(e) => {
            // If polling failed (likely because server doesn't support tasks/get),
            // print the error and return the initial task instead
            eprintln!("Warning: {}", e);
            eprintln!("Falling back to initial task response");
            if let Some(status_msg) = &initial_task.status.message {
                if matches!(status_msg.role, MessageRole::Agent) {
                    print_message(status_msg);
                }
            }
            Ok(initial_task)
        }
    }
}

// Function to poll for updates
async fn poll_until_complete(
    client: &Client,
    agent_url: &str,
    task_id: &str,
) -> Result<Task, String> {
    println!("Waiting for response...");
    let mut attempts = 0;
    let max_attempts = 120; // 60 seconds with 500ms intervals
    let mut displayed_working_message = false;

    loop {
        if attempts >= max_attempts {
            return Err(
                "Maximum polling attempts reached. Task may still be processing.".to_string(),
            );
        }

        let task = get_task(client, agent_url, task_id).await?;

        // Show working status message once
        if matches!(task.status.state, TaskState::Working) && !displayed_working_message {
            if let Some(status_msg) = &task.status.message {
                if matches!(status_msg.role, MessageRole::Agent) {
                    print_message(status_msg);
                    displayed_working_message = true;
                }
            }
        }

        // Check if task is complete
        match task.status.state {
            TaskState::Completed | TaskState::Failed | TaskState::Canceled => {
                // Show the final response
                if let Some(status_msg) = &task.status.message {
                    if matches!(status_msg.role, MessageRole::Agent) {
                        print_message(status_msg);
                    }
                }
                return Ok(task);
            }
            _ => {
                // Sleep before polling again
                tokio::time::sleep(Duration::from_millis(500)).await;
                attempts += 1;
            }
        }
    }
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

// Helper to print messages
fn print_message(message: &Message) {
    println!("\nAgent response:");

    // Debug: check if parts are empty
    if message.parts.is_empty() {
        println!("[Warning: Message contains no parts]");
        return;
    }

    for part in &message.parts {
        match part {
            Part::Text { text, .. } => {
                if text.is_empty() {
                    println!("[Warning: Empty text content]");
                } else {
                    println!("{}", text);
                }
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

// Print the task details, including status and history
fn print_task(task: &Task) {
    println!("\nTask status: {:?}", task.status.state);

    // Print status message if available
    if let Some(status_msg) = &task.status.message {
        if !status_msg.parts.is_empty() {
            println!("\nStatus message:");
            for part in &status_msg.parts {
                if let Part::Text { text, .. } = part {
                    if !text.is_empty() {
                        println!("{}", text);
                    } else {
                        println!("[Warning: Empty text content in status message]");
                    }
                }
            }
        } else {
            println!("\n[Warning: Status message contains no parts]");
        }
    }

    // Print history if available
    if let Some(history) = &task.history {
        if !history.is_empty() {
            println!("\nConversation:");
            for msg in history {
                let role = match msg.role {
                    MessageRole::User => "User",
                    MessageRole::Agent => "Agent",
                    MessageRole::System => "System",
                };

                println!("\n{}: ", role);
                if msg.parts.is_empty() {
                    println!("[Warning: Message contains no parts]");
                    continue;
                }

                for part in &msg.parts {
                    match part {
                        Part::Text { text, .. } => {
                            if !text.is_empty() {
                                println!("{}", text);
                            } else {
                                println!("[Warning: Empty text content]");
                            }
                        }
                        Part::File { file, .. } => {
                            println!("[File: {}]", file.name.as_deref().unwrap_or("unnamed"));
                        }
                        Part::Data { data, .. } => {
                            println!("[Data: {:?}]", data);
                        }
                    }
                }
            }
        } else {
            println!("\n[Warning: History is empty]");
        }
    } else {
        println!("\n[Warning: Task has no history]");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();

    // Default values
    let mut agent_url = "http://localhost:3000".to_string();
    let mut message = String::new();
    let mut poll = true;

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--url" | "-u" => {
                if i + 1 < args.len() {
                    agent_url = args[i + 1].clone();
                    i += 2;
                } else {
                    println!("Error: Missing value for --url");
                    return Ok(());
                }
            }
            "--no-poll" | "-n" => {
                poll = false;
                i += 1;
            }
            "--help" | "-h" => {
                print_usage(&args[0]);
                return Ok(());
            }
            _ => {
                // Assume this is the message
                message = args[i].clone();
                i += 1;
            }
        }
    }

    // Check if we have a message
    if message.is_empty() {
        println!("Error: No message provided");
        print_usage(&args[0]);
        return Ok(());
    }

    // Initialize the HTTP client
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

    // Send the message and get response
    match oneshot_message(&client, &agent_url, &message, poll).await {
        Ok(task) => {
            if !poll {
                // Print the task details if we're not polling
                print_task(&task);
                println!("\nNOTE: Polling for updates was disabled.");
                println!("The server may still be processing your request.");
            } else {
                // For polling mode, we've already printed the response,
                // just show the final status
                println!("\nTask completed with status: {:?}", task.status.state);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    Ok(())
}

fn print_usage(program_name: &str) {
    println!("Usage: {} [OPTIONS] <message>", program_name);
    println!("OPTIONS:");
    println!("  -u, --url <url>     URL of the A2A agent (default: http://localhost:3000)");
    println!("  -n, --no-poll       Don't poll for updates, just get initial response");
    println!("  -h, --help          Print this help message");
    println!("\nEXAMPLES:");
    println!("  {} \"Tell me a joke\"", program_name);
    println!(
        "  {} --url http://example.com:8080 \"What's the weather?\"",
        program_name
    );
    println!("  {} --no-poll \"Quick question\"", program_name);
}
