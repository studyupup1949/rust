use acai::client::{Client, ClientConfig};
use acai::{PushNotificationConfig, TaskSendParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client to connect to an A2A server
    let client = Client::new(ClientConfig::new("http://localhost:8080"))?;

    // Create a task
    let task_id = "push_notification_example_task".to_string();

    // Set up a push notification configuration
    let push_notification_config = PushNotificationConfig {
        url: "https://example.com/webhook".to_string(),
        token: Some("webhook-secret-token".to_string()),
        authentication: None,
    };

    // Method 1: Configure push notifications when creating a task
    // ----------------------------------------------------------

    println!("Method 1: Setting push notifications when creating a task");

    // Create a message for the task
    let message = acai::Message {
        role: acai::MessageRole::User,
        parts: vec![acai::Part::Text {
            text: "Example task message".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    // Create the task parameters with push notification config included
    let params = TaskSendParams {
        id: task_id.clone(),
        message,
        session_id: None,
        push_notification: Some(push_notification_config.clone()),
        history_length: None,
        metadata: None,
    };

    // Create a simple task first without using the helper method
    let request = params.into_send_request(serde_json::json!("send-request-id"));

    match client.send::<_, acai::Task>(request).await {
        Ok(task) => {
            println!(
                "Created task with ID: {} and push notifications configured",
                task.id
            );
        }
        Err(e) => {
            eprintln!("Error creating task: {}", e);
        }
    }

    // Method 2: Set push notifications for an existing task
    // ----------------------------------------------------

    println!("\nMethod 2: Setting push notifications for an existing task");

    // Using the helper method to set up push notifications

    // Send the request to the server using the helper method
    match client
        .set_push_notification(task_id.clone(), push_notification_config.clone())
        .await
    {
        Ok(config) => {
            println!(
                "Push notifications configured for task {}: {}",
                config.id, config.push_notification_config.url
            );

            // Now we can also retrieve the configuration to verify it using the helper method
            match client.get_push_notification(task_id.clone()).await {
                Ok(config) => {
                    println!(
                        "Retrieved push notification config for task {}: {}",
                        config.id, config.push_notification_config.url
                    );

                    if let Some(token) = &config.push_notification_config.token {
                        println!("  With token: {}", token);
                    }
                }
                Err(e) => {
                    eprintln!("Error retrieving push notification config: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error setting push notification config: {}", e);
        }
    }

    // For a complete example, add this to library functionality
    println!("\nThis example demonstrates how to set up push notifications in two ways:");
    println!(
        "1. When creating a task, by including a PushNotificationConfig in the TaskSendParams"
    );
    println!("2. After a task is created, using the tasks/pushNotification/set endpoint");
    println!("\nTo use this functionality in a real application:");
    println!("1. Set up a webhook endpoint that can receive and process the notifications");
    println!("2. Configure the webhook URL and optional token in the PushNotificationConfig");
    println!("3. Handle incoming notifications at your webhook endpoint");

    Ok(())
}
