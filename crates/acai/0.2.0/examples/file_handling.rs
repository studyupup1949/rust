use std::fs;
use std::path::Path;
use std::sync::Arc;

use acai::{
    AgentCapabilities, AgentCard, AgentSkill, FileContent, JsonRpcError, Message, MessageRole,
    Part, Task, TaskSendParams,
    client::{Client, ClientConfig},
    server::{MethodRouter, Server, ServerConfig, make_typed_handler},
};
use base64::{Engine, engine::general_purpose};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Utility function to load a file from disk and convert it to FileContent
///
/// This function reads a file from disk, base64 encodes its contents, and creates a
/// FileContent object that can be used in A2A protocol messages. It attempts to determine
/// the MIME type based on the file extension.
///
/// # Arguments
/// * `path` - The path to the file to load
///
/// # Returns
/// * `Result<FileContent>` - The file content object or an error
///
/// # Example
/// ```no_run
/// # use std::path::Path;
/// # use acai::FileContent;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let file_content = load_file(Path::new("example.txt"))?;
/// # Ok(())
/// # }
/// ```
fn load_file(path: &Path) -> Result<FileContent> {
    // Read the file
    let bytes = fs::read(path)?;

    // Encode the bytes as base64
    let base64_content = general_purpose::STANDARD.encode(bytes);

    // Get the filename if available
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    // Try to determine mime type based on extension
    // In a real application, you might want to use a more robust method like the mime_guess crate
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

/// Handler that displays file information
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
                let decoded_size = match general_purpose::STANDARD.decode(bytes) {
                    Ok(data) => data.len(),
                    Err(_) => 0,
                };
                description.push_str(&format!("- Size: {} bytes\n", decoded_size));

                // Show first few bytes of the file (for text files)
                if file.mime_type.as_deref().unwrap_or("").starts_with("text/") {
                    if let Ok(data) = general_purpose::STANDARD.decode(bytes) {
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

async fn start_server() -> Result<()> {
    // Create router with handler
    let mut router = MethodRouter::new();

    // Create state for the handler (empty unit type in this case)
    let state = Arc::new(());

    // Register the file info handler using make_typed_handler
    router.register("tasks/send", make_typed_handler(state, file_info_handler));

    // Create server configuration
    let config = ServerConfig::new("127.0.0.1:3003")?;

    // Create an agent card
    let agent_card = AgentCard {
        name: "File Info Agent".to_string(),
        description: Some("An agent that analyzes and displays file information".to_string()),
        url: "http://127.0.0.1:3003".to_string(),
        provider: None,
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
        skills: vec![AgentSkill {
            id: "file-info".to_string(),
            name: "File Information".to_string(),
            description: Some("Returns information about uploaded files".to_string()),
            tags: Some(vec!["files".to_string(), "analysis".to_string()]),
            examples: Some(vec!["Analyze this file".to_string()]),
            input_modes: Some(vec!["file".to_string()]),
            output_modes: Some(vec!["text".to_string()]),
        }],
    };

    // Create and start the server
    let server = Server::new(config, Arc::new(router)).with_agent_card(agent_card);

    println!("File Info Agent server listening on http://127.0.0.1:3003");
    println!("Agent card available at http://127.0.0.1:3003/.well-known/agent.json");

    server.serve().await.map_err(|e| e.into())
}

/// Creates a FileContent object with a URI reference instead of embedded bytes
///
/// This function demonstrates the URI-based approach to file handling, where the file content
/// is not embedded in the message but referenced by a URI. This is useful for large files
/// or when the file is already available at a public URL.
///
/// # Arguments
/// * `name` - The filename
/// * `mime_type` - The MIME type of the file (e.g., "image/jpeg")
/// * `uri` - The URI where the file can be accessed
///
/// # Returns
/// * `FileContent` - A FileContent object with the URI reference
fn create_uri_file_content(name: &str, mime_type: &str, uri: &str) -> FileContent {
    FileContent {
        name: Some(name.to_string()),
        mime_type: Some(mime_type.to_string()),
        bytes: None,
        uri: Some(uri.to_string()),
    }
}

async fn run_client(path: &Path) -> Result<()> {
    println!("Starting file handling client...");

    // Connect to the server
    let config = ClientConfig::new("http://127.0.0.1:3003");
    let client = Client::new(config)?;

    // First example: Send a file with base64-encoded bytes
    println!("\n--- Example 1: Sending file with base64-encoded bytes ---");

    // Load the file
    let file_content = match load_file(path) {
        Ok(content) => content,
        Err(e) => {
            println!("Error loading file: {}", e);
            return Err(e);
        }
    };

    println!(
        "File loaded: {}",
        file_content.name.as_deref().unwrap_or("Unnamed")
    );

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
    let task_id = format!("file_task_bytes_{}", chrono::Utc::now().timestamp());
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

    println!("Sending file with embedded bytes to agent...");

    // Send the request and process the response
    match client.send::<_, Task>(request).await {
        Ok(task) => {
            println!("\nResponse from File Info Agent:");
            if let Some(msg) = task.status.message {
                for part in msg.parts {
                    if let Part::Text { text, .. } = part {
                        println!("{}", text);
                    }
                }
            } else {
                println!("No response message received.");
            }
        }
        Err(e) => {
            println!("Error sending file: {}", e);
            return Err(Box::new(e));
        }
    }

    // Second example: Send a file with URI reference
    println!("\n--- Example 2: Sending file with URI reference ---");

    // Create a FileContent with a URI reference
    let uri_file = create_uri_file_content(
        "example-image.jpg",
        "image/jpeg",
        "https://example.com/images/example.jpg",
    );

    println!(
        "Created URI reference to file: {}",
        uri_file.name.as_deref().unwrap_or("Unnamed")
    );

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
    let task_id = format!("file_task_uri_{}", chrono::Utc::now().timestamp());
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

    println!("Sending file with URI reference to agent...");

    // Send the request and process the response
    match client.send::<_, Task>(request).await {
        Ok(task) => {
            println!("\nResponse from File Info Agent:");
            if let Some(msg) = task.status.message {
                for part in msg.parts {
                    if let Part::Text { text, .. } = part {
                        println!("{}", text);
                    }
                }
            } else {
                println!("No response message received.");
            }
        }
        Err(e) => {
            println!("Error sending file with URI: {}", e);
            return Err(Box::new(e));
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Check for client or server mode
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "client" {
        // Get the file path or use a default
        let path = if args.len() > 2 {
            Path::new(&args[2]).to_path_buf()
        } else {
            println!("No file path provided. Please specify a file path:");
            println!("  cargo run --example file_handling client <file_path>");
            return Ok(());
        };

        run_client(&path).await
    } else {
        start_server().await
    }
}
