use std::sync::Arc;
use tokio::sync::mpsc;

use acai::{
    AgentCapabilities, AgentCard, AgentSkill, JsonRpcError, JsonRpcRequest, JsonRpcResponse,
    PushNotificationConfig, Task, TaskIdParams, TaskPushNotificationConfig, TaskSendParams, Value,
    client::{Client, ClientConfig, Error},
    server::push_notification_handlers,
    server::{MethodRouter, Server, ServerConfig, make_typed_handler, task_manager::TaskManager},
};

/// Handler function for tasks/send requests
async fn handle_send_task(
    task_manager: Arc<TaskManager>,
    params: TaskSendParams,
) -> std::result::Result<Task, JsonRpcError> {
    // Create/update the task
    task_manager.upsert_task(&params).await?;

    // Get the updated task to return
    let task_query = acai::TaskQueryParams::from_id(params.id.clone());
    Ok(task_manager.get_task(&task_query).await?)
}

// Our calculator service state (empty in this case)
struct CalculatorService;

// Add handler
async fn handle_add(
    _state: Arc<CalculatorService>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.len() != 2 {
        return Err(JsonRpcError::invalid_parameters(
            "add requires exactly 2 numbers",
        ));
    }

    let a = params[0]
        .as_i64()
        .ok_or_else(|| JsonRpcError::invalid_parameters("first parameter must be an integer"))?;
    let b = params[1]
        .as_i64()
        .ok_or_else(|| JsonRpcError::invalid_parameters("second parameter must be an integer"))?;

    let result = a + b;
    Ok(Value::from(result))
}

// Subtract handler
async fn handle_subtract(
    _state: Arc<CalculatorService>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.len() != 2 {
        return Err(JsonRpcError::invalid_parameters(
            "subtract requires exactly 2 numbers",
        ));
    }

    let a = params[0]
        .as_i64()
        .ok_or_else(|| JsonRpcError::invalid_parameters("first parameter must be an integer"))?;
    let b = params[1]
        .as_i64()
        .ok_or_else(|| JsonRpcError::invalid_parameters("second parameter must be an integer"))?;

    let result = a - b;
    Ok(Value::from(result))
}

// Multiply handler
async fn handle_multiply(
    _state: Arc<CalculatorService>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.len() != 2 {
        return Err(JsonRpcError::invalid_parameters(
            "multiply requires exactly 2 numbers",
        ));
    }

    let a = params[0]
        .as_i64()
        .ok_or_else(|| JsonRpcError::invalid_parameters("first parameter must be an integer"))?;
    let b = params[1]
        .as_i64()
        .ok_or_else(|| JsonRpcError::invalid_parameters("second parameter must be an integer"))?;

    let result = a * b;
    Ok(Value::from(result))
}

// Divide handler
async fn handle_divide(
    _state: Arc<CalculatorService>,
    params: Vec<Value>,
) -> Result<Value, JsonRpcError> {
    if params.len() != 2 {
        return Err(JsonRpcError::invalid_parameters(
            "divide requires exactly 2 numbers",
        ));
    }

    let a = params[0]
        .as_i64()
        .ok_or_else(|| JsonRpcError::invalid_parameters("first parameter must be an integer"))?;
    let b = params[1]
        .as_i64()
        .ok_or_else(|| JsonRpcError::invalid_parameters("second parameter must be an integer"))?;

    if b == 0 {
        return Err(JsonRpcError::invalid_parameters("division by zero"));
    }

    let result = a / b;
    Ok(Value::from(result))
}

async fn run_client(client: Client) -> Result<(), Error> {
    // Fetch the agent card to verify capabilities
    println!("Fetching agent card...");
    let agent_card = client.fetch_agent_card().await?;
    println!("Connected to agent: {}", agent_card.name);
    println!("Agent capabilities: {:?}", agent_card.capabilities);

    // Test addition
    let add_params = vec![Value::from(10), Value::from(5)];
    let add_req = JsonRpcRequest::new(serde_json::json!("add-req"), "add", add_params);
    let add_resp: JsonRpcResponse<Value> = client.send(add_req).await?;
    println!("10 + 5 = {:?}", add_resp.result());

    // Test subtraction
    let sub_params = vec![Value::from(10), Value::from(5)];
    let sub_req = JsonRpcRequest::new(serde_json::json!("sub-req"), "subtract", sub_params);
    let sub_resp: JsonRpcResponse<Value> = client.send(sub_req).await?;
    println!("10 - 5 = {:?}", sub_resp.result());

    // Test multiplication
    let mul_params = vec![Value::from(10), Value::from(5)];
    let mul_req = JsonRpcRequest::new(serde_json::json!("mul-req"), "multiply", mul_params);
    let mul_resp: JsonRpcResponse<Value> = client.send(mul_req).await?;
    println!("10 * 5 = {:?}", mul_resp.result());

    // Test division
    let div_params = vec![Value::from(10), Value::from(5)];
    let div_req = JsonRpcRequest::new(serde_json::json!("div-req"), "divide", div_params);
    let div_resp: JsonRpcResponse<Value> = client.send(div_req).await?;
    println!("10 / 5 = {:?}", div_resp.result());

    // Test error handling (division by zero)
    let div_zero_params = vec![Value::from(10), Value::from(0)];
    let div_zero_req =
        JsonRpcRequest::new(serde_json::json!("div-zero-req"), "divide", div_zero_params);
    match client.send::<_, JsonRpcResponse<Value>>(div_zero_req).await {
        Ok(resp) => println!("Unexpected success: {:?}", resp.result()),
        Err(err) => println!("Error caught (as expected): {}", err),
    }

    // Test push notification configuration
    println!("\nSetting push notification configuration...");
    let task_params = TaskSendParams {
        id: "task_123".to_string(),
        session_id: Some("session_abc".to_string()),
        message: acai::Message {
            role: acai::MessageRole::User,
            parts: vec![acai::Part::Text {
                text: "Calculate result".to_string(),
                metadata: None,
            }],
            metadata: None,
        },
        push_notification: Some(PushNotificationConfig {
            url: "https://example.com/webhook".to_string(),
            token: Some("secret-token".to_string()),
            authentication: None,
        }),
        metadata: None,
        history_length: None,
    };
    let task_req = task_params.into_send_request(serde_json::json!("task-1"));
    let _task_resp: JsonRpcResponse<acai::Task> = client.send(task_req).await?;
    println!("Task created with push notifications configured");

    // Get push notification configuration for the task
    println!("\nRetrieving push notification configuration...");
    let get_config_params = TaskIdParams {
        id: "task_123".to_string(),
        metadata: None,
    };
    let get_config_req = JsonRpcRequest::new(
        serde_json::json!("config-req"),
        "tasks/pushNotification/get",
        get_config_params,
    );
    match client
        .send::<_, JsonRpcResponse<TaskPushNotificationConfig>>(get_config_req)
        .await
    {
        Ok(resp) => {
            let config = resp.result().unwrap();
            println!(
                "Push notification URL: {}",
                config.push_notification_config.url
            );
            if let Some(token) = &config.push_notification_config.token {
                println!("Push notification token: {}", token);
            }
        }
        Err(err) => println!("Error getting push notification config: {}", err),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Port for our server
    let port = 3030;
    let address = format!("127.0.0.1:{}", port);

    // Create task manager for handling tasks and push notifications
    let task_manager = match TaskManager::new() {
        Ok(tm) => Arc::new(tm),
        Err(e) => {
            eprintln!("Failed to initialize task manager: {}", e);
            return Err(Error::StreamProcessingError(format!(
                "Task manager initialization error: {}",
                e
            )));
        }
    };

    // Create router with calculator handler for all methods
    let mut router = MethodRouter::new();

    // Create a shared calculator service state
    let calculator_service = Arc::new(CalculatorService);

    // Register handlers for different methods using make_typed_handler
    router
        .register(
            "add",
            make_typed_handler(calculator_service.clone(), handle_add),
        )
        .register(
            "subtract",
            make_typed_handler(calculator_service.clone(), handle_subtract),
        )
        .register(
            "multiply",
            make_typed_handler(calculator_service.clone(), handle_multiply),
        )
        .register(
            "divide",
            make_typed_handler(calculator_service.clone(), handle_divide),
        )
        // Add push notification handlers
        .register(
            "tasks/pushNotification/get",
            make_typed_handler(
                task_manager.clone(),
                push_notification_handlers::get_push_notification,
            ),
        )
        .register(
            "tasks/pushNotification/set",
            make_typed_handler(
                task_manager.clone(),
                push_notification_handlers::set_push_notification,
            ),
        )
        // Add a simple handler for tasks/send
        .register(
            "tasks/send",
            make_typed_handler(task_manager.clone(), handle_send_task),
        );

    // Create an agent card with calculator capabilities and push notification support
    let agent_card = AgentCard {
        name: "Calculator Agent".to_string(),
        description: Some("A calculator agent that supports push notifications".to_string()),
        url: format!("http://{}", address),
        provider: None,
        version: "1.0.0".to_string(),
        documentation_url: None,
        capabilities: AgentCapabilities {
            streaming: false,
            push_notifications: true,
            state_transition_history: true,
        },
        authentication: None,
        default_input_modes: vec!["numerical".to_string()],
        default_output_modes: vec!["numerical".to_string()],
        skills: vec![AgentSkill {
            id: "calculator".to_string(),
            name: "Calculator".to_string(),
            description: Some("Performs basic arithmetic operations".to_string()),
            tags: Some(vec!["math".to_string(), "arithmetic".to_string()]),
            examples: None,
            input_modes: None,
            output_modes: None,
        }],
    };

    // Create server with agent card
    let server_config = ServerConfig::new(&address)?;
    let server = Server::new(server_config, Arc::new(router)).with_agent_card(agent_card);

    // Create client
    let client_config = ClientConfig::new(&format!("http://{}", address));
    let client = Client::new(client_config)?;

    // Channel to signal when server is ready
    let (tx, mut rx) = mpsc::channel::<()>(1);

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        let tx = tx.clone();
        println!("A2A server listening on http://{}", address);

        // Signal that the server is ready
        let _ = tx.send(()).await;

        // Run the server
        if let Err(e) = server.serve().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Wait for server to be ready
    let _ = rx.recv().await;

    // Give the server a moment to fully start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Run client requests
    println!("Running client requests...");
    if let Err(e) = run_client(client).await {
        eprintln!("Client error: {}", e);
    }

    // Clean shutdown - in real app you'd wait for a signal
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Cancel the server task
    server_handle.abort();

    Ok(())
}
