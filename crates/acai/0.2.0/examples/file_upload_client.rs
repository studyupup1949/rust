use acai::{
    FileContent, Message, MessageRole, Part, Task, TaskSendParams,
    client::{Client, ClientConfig, Error as ClientError},
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use std::fs;
use uuid::Uuid;

/// Create a basic A2A client
fn create_client(server_url: &str) -> Result<Client, ClientError> {
    let config = ClientConfig::new(server_url);
    Client::new(config)
}

/// Read a file from disk and encode it as base64
fn read_file_as_base64(file_path: &str) -> Result<String, std::io::Error> {
    let bytes = fs::read(file_path)?;
    Ok(BASE64.encode(bytes))
}

/// Send a file to an agent
async fn send_file_to_agent(
    client: &Client,
    task_id: &str,
    file_path: &str,
    file_name: &str,
    mime_type: &str,
    message_text: &str,
) -> Result<Task, ClientError> {
    // Read and encode the file
    let file_content = match read_file_as_base64(file_path) {
        Ok(content) => content,
        Err(e) => {
            return Err(ClientError::StreamProcessingError(format!(
                "Failed to read file: {}",
                e
            )));
        }
    };

    // Create a file content object
    let file = FileContent {
        name: Some(file_name.to_string()),
        mime_type: Some(mime_type.to_string()),
        bytes: Some(file_content),
        uri: None,
    };

    // Create a message with both text and file parts
    let message = Message {
        role: MessageRole::User,
        parts: vec![
            Part::Text {
                text: message_text.to_string(),
                metadata: None,
            },
            Part::File {
                file,
                metadata: None,
            },
        ],
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
    let request = params.into_send_request(serde_json::json!("upload-file"));

    // Send the request
    client.send(request).await
}

/// Send a file reference (URI) to an agent
async fn send_file_uri_to_agent(
    client: &Client,
    task_id: &str,
    file_uri: &str,
    file_name: &str,
    mime_type: &str,
    message_text: &str,
) -> Result<Task, ClientError> {
    // Create a file content object with URI
    let file = FileContent {
        name: Some(file_name.to_string()),
        mime_type: Some(mime_type.to_string()),
        bytes: None,
        uri: Some(file_uri.to_string()),
    };

    // Create a message with both text and file parts
    let message = Message {
        role: MessageRole::User,
        parts: vec![
            Part::Text {
                text: message_text.to_string(),
                metadata: None,
            },
            Part::File {
                file,
                metadata: None,
            },
        ],
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
    let request = params.into_send_request(serde_json::json!("upload-file"));

    // Send the request
    client.send(request).await
}

/// Send structured data to an agent
async fn send_structured_data_to_agent(
    client: &Client,
    task_id: &str,
    message_text: &str,
) -> Result<Task, ClientError> {
    use serde_json::json;
    use std::collections::HashMap;

    // Create a data object (example: JSON data for a chart)
    let mut data = HashMap::new();
    data.insert("type".to_string(), json!("chart"));
    data.insert("values".to_string(), json!([10, 20, 15, 30, 25]));
    data.insert("labels".to_string(), json!(["A", "B", "C", "D", "E"]));

    // Create metadata for the data part
    let mut metadata = HashMap::new();
    metadata.insert("format".to_string(), json!("bar-chart"));

    // Create a message with both text and data parts
    let message = Message {
        role: MessageRole::User,
        parts: vec![
            Part::Text {
                text: message_text.to_string(),
                metadata: None,
            },
            Part::Data {
                data,
                metadata: Some(metadata),
            },
        ],
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
    let request = params.into_send_request(serde_json::json!("upload-file"));

    // Send the request
    client.send(request).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration
    let server_url = "http://127.0.0.1:3000";

    println!("Creating A2A client for {}", server_url);
    let client = create_client(server_url)?;

    // Example 1: Send a file from disk
    let task_id_1 = Uuid::new_v4().to_string();
    println!(
        "\n--- Sending a file to the agent (task ID: {}) ---",
        task_id_1
    );

    // Replace with actual file path on your system
    let file_path = "./examples/sample.txt";
    let file_name = "sample.txt";
    let mime_type = "text/plain";
    let message_text = "Please analyze this text file";

    match send_file_to_agent(
        &client,
        &task_id_1,
        file_path,
        file_name,
        mime_type,
        message_text,
    )
    .await
    {
        Ok(task) => {
            println!("File sent successfully!");
            println!("Task state: {:?}", task.status.state);
        }
        Err(e) => {
            println!("Failed to send file: {}", e);
        }
    }

    // Example 2: Send a file reference (URI)
    let task_id_2 = Uuid::new_v4().to_string();
    println!(
        "\n--- Sending a file URI to the agent (task ID: {}) ---",
        task_id_2
    );

    let file_uri = "https://example.com/documents/sample.pdf";
    let file_name = "sample.pdf";
    let mime_type = "application/pdf";
    let message_text = "Please analyze this PDF document";

    match send_file_uri_to_agent(
        &client,
        &task_id_2,
        file_uri,
        file_name,
        mime_type,
        message_text,
    )
    .await
    {
        Ok(task) => {
            println!("File URI sent successfully!");
            println!("Task state: {:?}", task.status.state);
        }
        Err(e) => {
            println!("Failed to send file URI: {}", e);
        }
    }

    // Example 3: Send structured data
    let task_id_3 = Uuid::new_v4().to_string();
    println!(
        "\n--- Sending structured data to the agent (task ID: {}) ---",
        task_id_3
    );

    let message_text = "Please create a visualization based on this data";

    match send_structured_data_to_agent(&client, &task_id_3, message_text).await {
        Ok(task) => {
            println!("Structured data sent successfully!");
            println!("Task state: {:?}", task.status.state);
        }
        Err(e) => {
            println!("Failed to send structured data: {}", e);
        }
    }

    println!("\nAll file and data upload operations completed.");
    Ok(())
}
