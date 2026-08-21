use serde_json::Value;
use std::sync::Arc;

// Import the ACAI types needed for the A2A protocol
use acai::{JsonRpcError, Message, MessageRole, Part, TaskQueryParams, TaskSendParams, TaskState};

// Import the agent-related components
use acai::server::task_manager::TaskManager;
use acai::server::{MethodRouter, Server, ServerConfig, make_typed_handler};

// Import the needed yammer types
// (note: we're assuming yammer is a dependency in Cargo.toml)
use yammer::{ChatMessage, ChatRequest};

struct OllamaAgent {
    task_manager: Arc<TaskManager>,
    model: String,
    ollama_host: String,
}

impl OllamaAgent {
    fn new(task_manager: Arc<TaskManager>, model: String, ollama_host: Option<String>) -> Self {
        Self {
            task_manager,
            model,
            ollama_host: yammer::ollama_host(ollama_host),
        }
    }

    async fn process_task(
        &self,
        task_id: &str,
        prompt: &str,
        history: Option<Vec<Message>>,
    ) -> Result<(), JsonRpcError> {
        // Log the task processing event
        println!(
            "Processing task: {}, prompt length: {}, history length: {}",
            task_id,
            prompt.len(),
            history.as_ref().map(|h| h.len()).unwrap_or(0)
        );

        // Update task status to in-progress using add_task_message
        let working_message = Message {
            role: MessageRole::Agent,
            parts: vec![Part::Text {
                text: "Processing your request with Ollama...".to_string(),
                metadata: None,
            }],
            metadata: Some({
                let mut metadata = std::collections::HashMap::new();
                metadata.insert(
                    "status".to_string(),
                    serde_json::json!("processing_with_ollama"),
                );
                metadata
            }),
        };

        self.task_manager
            .add_task_message(task_id, working_message, TaskState::Working)
            .await
            .map_err(|e| {
                println!("Task status update error: {}, task_id: {}", e, task_id);
                JsonRpcError::internal_error(e)
            })?;

        // Convert previous messages if they exist
        let mut chat_messages = Vec::new();
        if let Some(hist) = history.clone() {
            for msg in hist {
                // Only include text parts
                for part in msg.parts {
                    if let Part::Text { text, .. } = part {
                        let role = match msg.role {
                            MessageRole::User => "user",
                            MessageRole::Agent => "assistant",
                            MessageRole::System => "system",
                        };

                        chat_messages.push(ChatMessage {
                            role: role.to_string(),
                            content: text,
                            images: None,
                            tool_calls: None,
                        });
                    }
                }
            }
        }

        // Add the current prompt as a user message
        chat_messages.push(ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
            images: None,
            tool_calls: None,
        });

        // Create a chat request for Ollama - turn off streaming
        let request = ChatRequest {
            model: self.model.clone(),
            messages: chat_messages,
            stream: Some(false), // Disable streaming
            options: serde_json::json!({
                "temperature": 0.7,
                "num_predict": 512
            }),
            format: None,
            keep_alive: None,
            tools: None,
        };

        // Use the built-in make_request method to prepare the request
        // This method returns a RequestBuilder with the proper URL and body
        let req_builder = request.make_request(&self.ollama_host);

        // Send the request to Ollama and get the full response at once
        let response = req_builder.send().await.map_err(|e| {
            println!("Ollama request error: {}, task_id: {}", e, task_id);
            JsonRpcError::internal_error(format!("Failed to send request to Ollama: {}", e))
        })?;

        // Parse the response
        let response_json: Value = response.json().await.map_err(|e| {
            println!("Ollama response parse error: {}, task_id: {}", e, task_id);
            JsonRpcError::internal_error(format!("Failed to parse Ollama response: {}", e))
        })?;

        // Extract the content from the response
        let content = response_json
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str)
            .map(String::from)
            .unwrap_or_else(|| "No response from Ollama".to_string());

        // Create a response message with the content
        let response_message = Message {
            role: MessageRole::Agent,
            parts: vec![Part::Text {
                text: content.clone(),
                metadata: None,
            }],
            metadata: None,
        };

        // Create a task with the response
        let mut messages = Vec::new();
        if let Some(hist) = history {
            messages.extend(hist);
        }
        messages.push(response_message.clone());

        // We'll use the task history in the TaskSendParams instead of creating a separate Task object

        // Update the task with the completed status and response using add_task_message
        self.task_manager
            .add_task_message(task_id, response_message, TaskState::Completed)
            .await
            .map_err(|e| {
                println!("Task completion error: {}, task_id: {}", e, task_id);
                JsonRpcError::internal_error(e)
            })?;

        println!("Task processing complete: {}", task_id);

        Ok(())
    }
}

/// Handler function for tasks/send requests with Ollama processing
async fn handle_send_task(
    ollama_agent: Arc<OllamaAgent>,
    params: TaskSendParams,
) -> Result<acai::Task, JsonRpcError> {
    let task_manager = Arc::clone(&ollama_agent.task_manager);
    let task_id = &params.id;

    // Log the request
    println!("Request received: task_id={}", task_id);

    // Extract the prompt from the message parts
    let prompt = params
        .message
        .parts
        .iter()
        .find_map(|part| match part {
            Part::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .ok_or_else(|| JsonRpcError::invalid_parameters("No text prompt provided".to_string()))?;

    // Get existing task and history if available
    let mut task_history = None;
    let query_params = acai::TaskQueryParams::from_id(task_id.clone());

    if let Ok(existing_task) = task_manager.get_task(&query_params).await {
        task_history = existing_task.history.clone();
    }

    // Insert the task first
    match task_manager.upsert_task(&params).await {
        Ok(_) => {
            println!("Task upserted: {}", task_id);
        }
        Err(e) => {
            println!("Task upsert error: {}, task_id: {}", e, task_id);
            return Err(JsonRpcError::internal_error(e));
        }
    };

    // Get the updated task to return
    let task = match task_manager.get_task(&query_params).await {
        Ok(t) => t,
        Err(e) => {
            println!("Task get error after upsert: {}, task_id: {}", e, task_id);
            return Err(JsonRpcError::internal_error(e));
        }
    };

    // Process the task asynchronously
    let task_id_clone = task_id.clone();
    let ollama_agent_clone = ollama_agent.clone();
    let task_manager_clone = Arc::clone(&task_manager);

    tokio::spawn(async move {
        if let Err(e) = ollama_agent_clone
            .process_task(&task_id_clone, &prompt, task_history)
            .await
        {
            println!("Task processing error: {:?}, task_id: {}", e, task_id_clone);

            // Update task status to error using add_task_message
            let error_message = Message {
                role: MessageRole::Agent,
                parts: vec![Part::Text {
                    text: format!("Failed to process task: {:?}", e),
                    metadata: None,
                }],
                metadata: None,
            };

            if let Err(update_err) = task_manager_clone
                .add_task_message(&task_id_clone, error_message, TaskState::Failed)
                .await
            {
                println!(
                    "Task status update error: {}, task_id: {}",
                    update_err, task_id_clone
                );
            }
        }
    });

    // Log the response
    println!("Response sent: task_id={}", task_id);

    Ok(task)
}

// Add Clone implementation for OllamaAgent
impl Clone for OllamaAgent {
    fn clone(&self) -> Self {
        Self {
            task_manager: Arc::clone(&self.task_manager),
            model: self.model.clone(),
            ollama_host: self.ollama_host.clone(),
        }
    }
}

/// Handler function for tasks/get requests
async fn handle_task_get(
    task_manager: Arc<TaskManager>,
    params: TaskQueryParams,
) -> Result<acai::Task, JsonRpcError> {
    let task_id = &params.id;

    // Log the request
    println!("Get request received: task_id={}", task_id);

    // Get the task
    match task_manager.get_task(&params).await {
        Ok(task) => {
            println!("Task retrieved: {}", task_id);
            Ok(task)
        }
        Err(e) => {
            println!("Task get error: {}, task_id: {}", e, task_id);
            Err(JsonRpcError::internal_error(e))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Log startup
    println!("Starting Ollama agent server");

    // Print startup information
    println!(
        "Server startup: process={:?}",
        std::env::args().collect::<Vec<_>>()
    );

    // Create a task manager
    let task_manager = Arc::new(TaskManager::new()?);
    println!("Task manager created");

    // Create the Ollama agent
    let ollama_agent = Arc::new(OllamaAgent::new(
        Arc::clone(&task_manager),
        "gemma3:12b-it-qat".to_string(), // Using larger Gemma 3 model with instruction tuning
        None,                            // Use default Ollama host
    ));
    println!("Ollama agent created: model=gemma3:12b-it-qat");

    // Create a method router and register the handlers using make_typed_handler
    let mut router = MethodRouter::new();
    router.register(
        "tasks/send",
        make_typed_handler(Arc::clone(&ollama_agent), handle_send_task),
    );
    router.register(
        "tasks/get",
        make_typed_handler(Arc::clone(&task_manager), handle_task_get),
    );
    println!("Router created with methods: tasks/send, tasks/get");

    // Start the server (now supporting both HTTP/1.1 and HTTP/2 by default)
    let server_config = ServerConfig::new("127.0.0.1:3000")?;
    let server = Server::new(server_config, Arc::new(router));
    println!("Server starting on 127.0.0.1:3000");

    println!("Starting Ollama agent server on http://127.0.0.1:3000");
    println!("This server supports both tasks/send and tasks/get methods - non-streaming mode!");

    // Start server with structured logging of any errors
    match server.serve().await {
        Ok(_) => {
            println!("Server shutdown: clean");
            Ok(())
        }
        Err(e) => {
            println!("Server error: {:?}", e);
            Err(e.into())
        }
    }
}
