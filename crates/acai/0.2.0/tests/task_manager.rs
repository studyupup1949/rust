use std::collections::HashMap;
use std::sync::Arc;

use acai::server::task_manager::{
    CONTENT_TYPE_FORM_SCHEMA, CONTENT_TYPE_FORM_SUBMISSION, TaskManager,
};
use acai::{
    FormField, FormSchema, Message, MessageRole, Part, TaskIdParams, TaskSendParams, TaskState,
    TaskStatus,
};

#[tokio::test]
async fn task_manager_lifecycle() {
    // Create a task manager
    let task_manager = Arc::new(TaskManager::new().unwrap());

    // Generate a test task ID
    let task_id = "test_task_1".to_string();

    // Create a message
    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: "Hello task manager!".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    // Create send params (without push notification for now)
    let params = TaskSendParams {
        id: task_id.clone(),
        message: message.clone(),
        session_id: Some("test_session_1".to_string()),
        push_notification: None,
        history_length: None,
        metadata: None,
    };

    // Insert the task
    task_manager.upsert_task(&params).await.unwrap();

    // Get the task to check it
    let task_query_params = acai::TaskQueryParams::from_id(task_id.clone());
    let task = task_manager.get_task(&task_query_params).await.unwrap();

    // Verify the task was created correctly
    assert_eq!(task.id, task_id);
    assert_eq!(task.session_id, Some("test_session_1".to_string()));
    assert_eq!(task.status.state, TaskState::Submitted);

    // Get the task
    let task_query_params = acai::TaskQueryParams::from_id(task_id.clone());
    let retrieved_task = task_manager.get_task(&task_query_params).await.unwrap();

    // Verify retrieved task matches
    assert_eq!(retrieved_task.id, task_id);
    assert_eq!(
        retrieved_task.session_id,
        Some("test_session_1".to_string())
    );
    assert_eq!(retrieved_task.status.state, TaskState::Submitted);

    // Get the task history
    let history_params = acai::TaskQueryParams {
        id: task_id.clone(),
        history_length: Some(10),
        metadata: None,
    };
    let task_with_history = task_manager.get_task(&history_params).await.unwrap();

    // Verify history includes our message
    let history = task_with_history.history.unwrap();
    assert_eq!(history.len(), 1);
    assert!(matches!(history[0].role, MessageRole::User));
    assert!(
        matches!(&history[0].parts[0], Part::Text { text, .. } if text == "Hello task manager!")
    );

    // Update task status to Working
    let working_message = Message {
        role: MessageRole::Agent,
        parts: vec![Part::Text {
            text: "I'm working on it...".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    task_manager
        .add_task_message(&task_id, working_message.clone(), TaskState::Working)
        .await
        .unwrap();

    // Get the updated task
    let updated_task = task_manager.get_task(&task_query_params).await.unwrap();

    // Verify the task status was updated
    assert_eq!(updated_task.status.state, TaskState::Working);

    // Verify history was updated correctly
    let history_params = acai::TaskQueryParams {
        id: task_id.clone(),
        history_length: Some(10),
        metadata: None,
    };
    let task_with_updated_history = task_manager.get_task(&history_params).await.unwrap();

    // Verify history now has both messages
    let updated_history = task_with_updated_history.history.unwrap();
    assert_eq!(updated_history.len(), 2);
    assert!(matches!(updated_history[0].role, MessageRole::User));
    assert!(matches!(updated_history[1].role, MessageRole::Agent));

    // Complete the task
    let completion_message = Message {
        role: MessageRole::Agent,
        parts: vec![Part::Text {
            text: "Task completed!".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    task_manager
        .add_task_message(&task_id, completion_message, TaskState::Completed)
        .await
        .unwrap();

    // Get the completed task
    let completed_task = task_manager.get_task(&task_query_params).await.unwrap();

    // Verify the task was completed
    assert_eq!(completed_task.status.state, TaskState::Completed);

    // Test task cancellation - create a new task for this
    let cancel_task_id = "test_task_cancel".to_string();
    let cancel_params = TaskSendParams {
        id: cancel_task_id.clone(),
        message: message.clone(),
        session_id: Some("test_session_2".to_string()),
        push_notification: None,
        history_length: None,
        metadata: None,
    };

    // Insert the task to be canceled
    task_manager.upsert_task(&cancel_params).await.unwrap();

    // Cancel the task
    let cancel_params = TaskIdParams {
        id: cancel_task_id.clone(),
        metadata: None,
    };
    task_manager.cancel_task(&cancel_params).await.unwrap();

    // Get the task to verify it was canceled
    let task_query_params = acai::TaskQueryParams::from_id(cancel_task_id.clone());
    let canceled_task = task_manager.get_task(&task_query_params).await.unwrap();

    // Verify the task was canceled
    assert_eq!(canceled_task.id, cancel_task_id);
    assert_eq!(canceled_task.status.state, TaskState::Canceled);

    // We'll skip push notification testing in this test
    // Push notification testing requires a separate test with HTTP mocking

    // Test task not found
    let not_found_id = "nonexistent_task".to_string();
    let not_found_params = acai::TaskQueryParams {
        id: not_found_id,
        history_length: None,
        metadata: None,
    };

    let result = task_manager.get_task(&not_found_params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn task_update_transition() {
    // Create a task manager
    let task_manager = Arc::new(TaskManager::new().unwrap());

    // Generate a test task ID
    let task_id = "test_transition_task".to_string();

    // Create a message
    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: "Test task transition".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    // Create send params
    let params = TaskSendParams {
        id: task_id.clone(),
        message: message.clone(),
        session_id: None,
        push_notification: None,
        history_length: None,
        metadata: None,
    };

    // Insert the task
    task_manager.upsert_task(&params).await.unwrap();

    // Test updating task status
    let new_status = TaskStatus {
        state: TaskState::Working,
        message: None,
        timestamp: None,
    };

    task_manager
        .update_task_status(&task_id, new_status)
        .await
        .unwrap();

    // Get the task to verify it was updated
    let task_query_params = acai::TaskQueryParams::from_id(task_id.clone());
    let updated_task = task_manager.get_task(&task_query_params).await.unwrap();

    // Verify the task was updated
    assert_eq!(updated_task.id, task_id);
    assert_eq!(updated_task.status.state, TaskState::Working);

    // Update to completed
    let completed_status = TaskStatus {
        state: TaskState::Completed,
        message: None,
        timestamp: None,
    };

    task_manager
        .update_task_status(&task_id, completed_status)
        .await
        .unwrap();

    // Get the task to verify it was completed
    let task_query_params = acai::TaskQueryParams::from_id(task_id.clone());
    let completed_task = task_manager.get_task(&task_query_params).await.unwrap();

    // Verify the task was completed
    assert_eq!(completed_task.id, task_id);
    assert_eq!(completed_task.status.state, TaskState::Completed);

    // Test updating non-existent task
    let nonexistent_id = "nonexistent_task".to_string();
    let another_status = TaskStatus {
        state: TaskState::Working,
        message: None,
        timestamp: None,
    };
    let result = task_manager
        .update_task_status(&nonexistent_id, another_status)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn task_artifacts() {
    // Create a task manager
    let task_manager = Arc::new(TaskManager::new().unwrap());

    // Generate a test task ID
    let task_id = "test_artifacts_task".to_string();

    // Create a message
    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: "Generate some artifacts".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    // Create send params
    let params = TaskSendParams {
        id: task_id.clone(),
        message: message.clone(),
        session_id: None,
        push_notification: None,
        history_length: None,
        metadata: None,
    };

    // Insert the task
    task_manager.upsert_task(&params).await.unwrap();

    // Create an artifact
    let artifact = acai::Artifact {
        name: Some("Test Artifact".to_string()),
        description: Some("A test artifact".to_string()),
        parts: vec![Part::Text {
            text: "This is a test artifact".to_string(),
            metadata: None,
        }],
        index: 0,
        append: None,
        last_chunk: None,
        metadata: None,
    };

    // Add the artifact to the task
    task_manager
        .add_task_artifact(&task_id, artifact.clone())
        .await
        .unwrap();

    // Get the task to verify the artifact was added
    let task_query_params = acai::TaskQueryParams::from_id(task_id.clone());
    let updated_task = task_manager.get_task(&task_query_params).await.unwrap();

    // Verify the artifact was added
    let artifacts = updated_task.artifacts.unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].name, Some("Test Artifact".to_string()));
    assert_eq!(artifacts[0].index, 0);

    // Get the task with the artifact
    let task_query_params = acai::TaskQueryParams {
        id: task_id.clone(),
        history_length: None,
        metadata: None,
    };
    let retrieved_task = task_manager.get_task(&task_query_params).await.unwrap();

    // Verify the artifact is in the retrieved task
    let retrieved_artifacts = retrieved_task.artifacts.unwrap();
    assert_eq!(retrieved_artifacts.len(), 1);
    assert_eq!(
        retrieved_artifacts[0].description,
        Some("A test artifact".to_string())
    );

    // Test adding a second artifact
    let artifact2 = acai::Artifact {
        name: Some("Second Artifact".to_string()),
        description: Some("Another test artifact".to_string()),
        parts: vec![Part::Text {
            text: "This is the second artifact".to_string(),
            metadata: None,
        }],
        index: 1,
        append: None,
        last_chunk: None,
        metadata: None,
    };

    task_manager
        .add_task_artifact(&task_id, artifact2.clone())
        .await
        .unwrap();

    // Get the task to verify both artifacts are present
    let task_query_params = acai::TaskQueryParams::from_id(task_id.clone());
    let updated_task = task_manager.get_task(&task_query_params).await.unwrap();

    // Verify both artifacts are present
    let artifacts = updated_task.artifacts.unwrap();
    assert_eq!(artifacts.len(), 2);
    assert_eq!(artifacts[1].name, Some("Second Artifact".to_string()));
    assert_eq!(artifacts[1].index, 1);

    // Test appending to an artifact
    let append_artifact = acai::Artifact {
        name: Some("Second Artifact".to_string()), // Same name
        description: None,                         // No description needed for append
        parts: vec![Part::Text {
            text: " - with additional content".to_string(),
            metadata: None,
        }],
        index: 1,               // Same index
        append: Some(true),     // Append flag
        last_chunk: Some(true), // Last chunk flag
        metadata: None,
    };

    task_manager
        .add_task_artifact(&task_id, append_artifact)
        .await
        .unwrap();

    // Get the task to verify the artifact was appended
    let task_query_params = acai::TaskQueryParams::from_id(task_id.clone());
    let updated_task = task_manager.get_task(&task_query_params).await.unwrap();

    // Verify the artifact was appended to (or added)
    let artifacts = updated_task.artifacts.unwrap();
    assert!(artifacts.len() >= 2); // At least 2 artifacts

    // Look for the appended artifact by name
    let mut found_appended = false;
    for artifact in artifacts.iter() {
        if let Some(name) = &artifact.name {
            if name == "Second Artifact" {
                if let Part::Text { text, .. } = &artifact.parts[0] {
                    if text.contains("with additional content") {
                        found_appended = true;
                        break;
                    }
                }
            }
        }
    }
    assert!(
        found_appended,
        "Could not find artifact with appended content"
    );
}

#[tokio::test]
async fn task_form_handling() {
    // Create a task manager
    let task_manager = Arc::new(TaskManager::new().unwrap());

    // Generate a test task ID
    let task_id = "test_form_task".to_string();

    // Create a message
    let message = Message {
        role: MessageRole::User,
        parts: vec![Part::Text {
            text: "Start form workflow".to_string(),
            metadata: None,
        }],
        metadata: None,
    };

    // Create send params
    let params = TaskSendParams {
        id: task_id.clone(),
        message: message.clone(),
        session_id: None,
        push_notification: None,
        history_length: None,
        metadata: None,
    };

    // Insert the task
    task_manager.upsert_task(&params).await.unwrap();

    // Create a form schema
    let mut properties = HashMap::new();
    properties.insert(
        "name".to_string(),
        FormField {
            title: Some("Full Name".to_string()),
            format: Some("text".to_string()),
            required: true,
            default: None,
            description: Some("Please enter your full name".to_string()),
            validation: None,
            additional_properties: HashMap::new(),
        },
    );
    properties.insert(
        "email".to_string(),
        FormField {
            title: Some("Email Address".to_string()),
            format: Some("email".to_string()),
            required: true,
            default: None,
            description: None,
            validation: None,
            additional_properties: HashMap::new(),
        },
    );
    properties.insert(
        "age".to_string(),
        FormField {
            title: Some("Age".to_string()),
            format: Some("number".to_string()),
            required: false,
            default: None,
            description: None,
            validation: None,
            additional_properties: HashMap::new(),
        },
    );

    let form_schema = FormSchema {
        properties,
        required: vec!["name".to_string(), "email".to_string()],
        additional_properties: HashMap::new(),
    };

    // Request form input
    task_manager
        .request_form_input(
            &task_id,
            form_schema,
            Some("Please fill out your contact information".to_string()),
        )
        .await
        .unwrap();

    // Get the task to verify the status was updated
    let task_query_params = acai::TaskQueryParams::from_id(task_id.clone());
    let updated_task = task_manager.get_task(&task_query_params).await.unwrap();

    // Verify the task status was updated to InputRequired
    assert_eq!(updated_task.status.state, TaskState::InputRequired);

    // Verify there's a message in the history with a form schema part
    let has_form_schema = if let Some(history) = &updated_task.history {
        history.iter().any(|msg| {
            // Check all parts of the message
            msg.parts.iter().any(|part| {
                let Part::Data {
                    data,
                    metadata: Some(metadata),
                } = part
                else {
                    return false;
                };

                // First check content type if available
                if let Some(serde_json::Value::String(value)) = metadata.get("content_type") {
                    // Return true if it's a form schema with a form property, otherwise false
                    return value == CONTENT_TYPE_FORM_SCHEMA && data.contains_key("form");
                }

                // Fall back to checking for form structure without a content type
                data.contains_key("form")
            })
        })
    } else {
        false
    };

    assert!(has_form_schema, "Task history should contain a form schema");

    // Extract the form_id from the schema
    let mut form_id = None;
    if let Some(history) = &updated_task.history {
        'outer: for msg in history.iter() {
            for part in &msg.parts {
                if let Part::Data {
                    data,
                    metadata: Some(metadata),
                } = part
                {
                    if let Some(serde_json::Value::String(value)) = metadata.get("content_type") {
                        if value == CONTENT_TYPE_FORM_SCHEMA {
                            form_id = data
                                .get("form_id")
                                .and_then(|v| v.as_str())
                                .map(String::from);
                            if form_id.is_some() {
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
    }

    // Submit form data
    let mut form_data = HashMap::new();
    form_data.insert("name".to_string(), serde_json::json!("John Doe"));
    form_data.insert("email".to_string(), serde_json::json!("john@example.com"));
    form_data.insert("age".to_string(), serde_json::json!(30));

    // Include the form_id in the submission if it was found
    if let Some(id) = form_id {
        form_data.insert("form_id".to_string(), serde_json::json!(id));
    }

    task_manager
        .add_task_form_data(&task_id, form_data, TaskState::Completed)
        .await
        .unwrap();

    // Get the task to verify the status was updated
    let task_query_params = acai::TaskQueryParams::from_id(task_id.clone());
    let completed_task = task_manager.get_task(&task_query_params).await.unwrap();

    // Verify the task status was updated to Completed
    assert_eq!(completed_task.status.state, TaskState::Completed);

    // Verify there's a message in the history with form data
    let has_form_data = if let Some(history) = &completed_task.history {
        history.iter().any(|msg| {
            // Check all parts of the message
            msg.parts.iter().any(|part| {
                let Part::Data {
                    data,
                    metadata: Some(metadata),
                } = part
                else {
                    return false;
                };

                // First check content type if available
                if let Some(serde_json::Value::String(value)) = metadata.get("content_type") {
                    // Return true if it's a form submission with required fields, otherwise false
                    return value == CONTENT_TYPE_FORM_SUBMISSION
                        && data.contains_key("name")
                        && data.contains_key("email");
                }

                // Fall back to checking for required form fields without a content type
                data.contains_key("name") && data.contains_key("email")
            })
        })
    } else {
        false
    };

    assert!(
        has_form_data,
        "Task history should contain form submission data"
    );
}
