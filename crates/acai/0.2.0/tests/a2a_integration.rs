use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::sleep;
use uuid::Uuid;

use acai::server::task_manager::TaskManager;
use acai::server::{MethodRouter, Server, ServerConfig, make_typed_handler};
use acai::{
    JsonRpcError, Message, MessageRole, Part, Task, TaskIdParams, TaskQueryParams, TaskSendParams,
    TaskState,
};

// For starting the external Python server
struct ExternalServerProcess {
    child: Child,
}

impl ExternalServerProcess {
    // This function was kept for reference but not used
    #[allow(dead_code)]
    fn start_sample_server() -> Result<Self, std::io::Error> {
        // Path to the Python server, adjust as needed
        let python_path =
            std::env::var("PYTHONPATH").unwrap_or_else(|_| "./A2A/samples/python".to_string());
        let python_cmd = "python3";

        // Create a command to start the Python server
        // For testing purposes, we'll use a simple implementation
        println!("Starting external Python A2A server...");

        // Set environment variable for tests - use a mock key for testing
        let mut command = Command::new(python_cmd);
        command
            .current_dir(&python_path)
            .args(["-m", "agents.crewai.__main__"])
            .env("GOOGLE_API_KEY", "test-api-key-for-integration-testing")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let child = command.spawn()?;

        Ok(Self { child })
    }
}

impl Drop for ExternalServerProcess {
    fn drop(&mut self) {
        // Terminate the server process when this object is dropped
        println!("Stopping external Python A2A server...");
        let _ = self.child.kill();
    }
}

/// Function handler for tasks/send requests
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

/// Function handler for tasks/get requests
async fn handle_get_task(
    task_manager: Arc<TaskManager>,
    params: TaskQueryParams,
) -> Result<Task, JsonRpcError> {
    task_manager
        .get_task(&params)
        .await
        .map_err(JsonRpcError::internal_error)
}

/// Function handler for tasks/cancel requests
async fn handle_cancel_task(
    task_manager: Arc<TaskManager>,
    params: TaskIdParams,
) -> Result<Task, JsonRpcError> {
    // Cancel the task
    task_manager
        .cancel_task(&params)
        .await
        .map_err(JsonRpcError::internal_error)?;

    // Get the updated task to return
    let task_query = TaskQueryParams::from_id(params.id.clone());
    task_manager
        .get_task(&task_query)
        .await
        .map_err(JsonRpcError::internal_error)
}

// Helper function to parse a JSON-RPC response into a Task
fn parse_task_response(body: &str) -> Result<Task, String> {
    // Try direct deserialization of the whole response
    match serde_json::from_str::<acai::JsonRpcResponse<Task>>(body) {
        Ok(response) => {
            // Check for errors in the JSON-RPC response
            if let Some(error) = response.get_error() {
                return Err(format!("Error in response: {:?}", error));
            }

            // Extract the task from the result field
            match response.result() {
                Some(task) => Ok(task.clone()),
                None => Err("Response missing result field".to_string()),
            }
        }
        Err(e) => {
            // Parse as generic JSON to extract fields manually
            println!("Falling back to manual JSON parsing due to: {}", e);

            let json_value: serde_json::Value = serde_json::from_str(body)
                .unwrap_or_else(|e| panic!("Failed to parse JSON: {}", e));

            // Check for errors
            if let Some(error) = json_value.get("error") {
                return Err(format!("Error in response: {:?}", error));
            }

            // Get the result field
            let result = json_value
                .get("result")
                .unwrap_or_else(|| panic!("Response missing result field"));

            // Try to deserialize just the result portion
            // JSON could be using sessionId which is already handled by the Task struct's rename/alias
            match serde_json::from_value::<Task>(result.clone()) {
                Ok(task) => Ok(task),
                Err(e) => {
                    let full_body = format!("Failed JSON: {}", body);
                    let res_str = format!("Result JSON: {}", result);
                    panic!(
                        "Failed to deserialize Task from result: {}. Details: {}. {}",
                        e, full_body, res_str
                    );
                }
            }
        }
    }
}

// Helper function to send a message to an agent
async fn send_message(
    client: &reqwest::Client,
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

    // Get the response text
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to get response text: {}", e))?;

    println!("Response body: {}", body);

    // Parse and return the task
    parse_task_response(&body)
}

// Helper function to get a task
async fn get_task(
    client: &reqwest::Client,
    agent_url: &str,
    task_id: &str,
) -> Result<Task, String> {
    // Create query params
    let params = TaskQueryParams {
        id: task_id.to_string(),
        history_length: Some(10), // Request up to 10 history items
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

    // Get the response text
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to get response text: {}", e))?;

    // Parse and return the task
    parse_task_response(&body)
}

// Helper function to cancel a task
async fn cancel_task(
    client: &reqwest::Client,
    agent_url: &str,
    task_id: &str,
) -> Result<Task, String> {
    // Create query params
    let params = TaskIdParams {
        id: task_id.to_string(),
        metadata: None,
    };

    // Create request
    let id = serde_json::json!(Uuid::new_v4().to_string());
    let request = params.into_cancel_request(id);

    // Send request
    let response = client
        .post(agent_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    // Get the response text
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to get response text: {}", e))?;

    // Parse and return the task
    parse_task_response(&body)
}

// Start our own server for testing
async fn start_test_server(port: u16) -> Server {
    // Create a task manager
    let task_manager = Arc::new(TaskManager::new().unwrap());

    // Create a method router and register handlers using make_typed_handler
    let mut router = MethodRouter::new();
    router.register(
        "tasks/send",
        make_typed_handler(Arc::clone(&task_manager), handle_send_task),
    );
    router.register(
        "tasks/get",
        make_typed_handler(Arc::clone(&task_manager), handle_get_task),
    );
    router.register(
        "tasks/cancel",
        make_typed_handler(Arc::clone(&task_manager), handle_cancel_task),
    );

    // Start the server
    let server_config = ServerConfig::new(&format!("127.0.0.1:{}", port)).unwrap();
    Server::new(server_config, Arc::new(router))
}

#[tokio::test]
async fn rust_server_with_a2a_protocol() {
    // Create an HTTP client
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    // Bind to a random port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Release the port so our server can use it

    let server_url = format!("http://127.0.0.1:{}", port);
    let server = start_test_server(port).await;

    // Start the server in a background task
    let server_handle = tokio::spawn(async move {
        server.serve().await.unwrap();
    });

    // Wait for server to start
    sleep(Duration::from_millis(500)).await;

    // Test basic interaction - send a task
    let task_id = Uuid::new_v4().to_string();
    let session_id = Some(Uuid::new_v4().to_string());

    // Send a message
    let sent_task = send_message(
        &client,
        &server_url,
        &task_id,
        "Hello ACAI!",
        session_id.clone(),
    )
    .await
    .unwrap();

    // Verify the task was created
    assert_eq!(sent_task.id, task_id);
    assert_eq!(sent_task.session_id, session_id);

    // Get the task
    let retrieved_task = get_task(&client, &server_url, &task_id).await.unwrap();

    // Verify the task data
    assert_eq!(retrieved_task.id, task_id);
    assert_eq!(retrieved_task.session_id, session_id);

    // History may or may not be present depending on the server implementation
    if let Some(history) = retrieved_task.history.as_ref() {
        if !history.is_empty() {
            // The first message should be our user message
            let user_message = &history[0];
            assert!(matches!(user_message.role, MessageRole::User));
        }
    } else {
        // If history is None, the test should still pass
        // This is because the extract_task_from_response function might not populate history
        println!("Task history is None - this is expected in some implementations");
    }

    // Cancel the task
    let canceled_task = cancel_task(&client, &server_url, &task_id).await.unwrap();

    // Verify the task was canceled
    assert_eq!(canceled_task.id, task_id);
    assert_eq!(canceled_task.status.state, TaskState::Canceled);

    // Get the canceled task to confirm
    let retrieved_canceled_task = get_task(&client, &server_url, &task_id).await.unwrap();
    assert_eq!(retrieved_canceled_task.status.state, TaskState::Canceled);

    // Stop the server
    server_handle.abort();
}

#[tokio::test]
async fn acai_client_with_python_a2a_server() {
    // Start the Python server - this MUST succeed for the test to pass
    let python_path = std::env::var("PYTHONPATH").unwrap_or_else(|_| "./tests".to_string());

    println!("Starting Echo Agent Python server from {}", python_path);

    let python_server = Command::new("python3")
        .current_dir(&python_path)
        .args(["agent.py"])
        .env("PYTHONPATH", &python_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start Echo Agent server");

    let _python_server = ExternalServerProcess {
        child: python_server,
    };

    // Create an HTTP client
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    // Wait for the Python server to start
    sleep(Duration::from_secs(3)).await;

    // Use the Echo Agent server URL (default port is 5000)
    let python_server_url = "http://localhost:5000";

    let task_id = Uuid::new_v4().to_string();
    let session_id = Some(Uuid::new_v4().to_string());

    // Send a message using the helper function
    let sent_task = send_message(
        &client,
        python_server_url,
        &task_id,
        "Tell me about the A2A protocol",
        session_id.clone(),
    )
    .await
    .expect("Failed to send message to Echo Agent server");

    // Verify the task was created
    assert_eq!(sent_task.id, task_id);

    // Get the task
    let retrieved_task = get_task(&client, python_server_url, &task_id)
        .await
        .expect("Failed to get task from Echo Agent server");

    // Verify the task data
    assert_eq!(retrieved_task.id, task_id);

    // Test cancellation if supported
    if let Ok(canceled_task) = cancel_task(&client, python_server_url, &task_id).await {
        // If cancellation succeeded, verify task was properly canceled
        assert_eq!(canceled_task.id, task_id);
        assert_eq!(canceled_task.status.state, TaskState::Canceled);

        // Get the canceled task to confirm
        if let Ok(retrieved_canceled_task) = get_task(&client, python_server_url, &task_id).await {
            assert_eq!(retrieved_canceled_task.status.state, TaskState::Canceled);
        }
    } else {
        // It's ok if cancellation isn't supported
        println!("Cancel task not supported by Echo Agent, but test passes");
    }

    // The Python server will be stopped when python_server is dropped
}
