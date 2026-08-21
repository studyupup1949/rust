use std::collections::HashMap;
use std::io::Read;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use chrono::Utc;
use futures::stream::Stream;
use pin_project::pin_project;
use serde_json;
use tokio::sync::{RwLock, broadcast};

use crate::server::push_notification::{
    PushNotificationError, PushNotificationSender, PushNotificationSupport,
};
use crate::types::{
    Artifact, JsonRpcError, JsonRpcResponse, PushNotificationConfig, StreamingResponseContent,
    Task, TaskArtifactUpdateEvent, TaskIdParams, TaskPushNotificationConfig, TaskQueryParams,
    TaskSendParams, TaskState, TaskStatus, TaskStatusUpdateEvent,
};

// Constants for content types
pub const CONTENT_TYPE_FORM_SCHEMA: &str = "form-schema";
pub const CONTENT_TYPE_FORM_SUBMISSION: &str = "form-submission";

/// Task manager error
#[derive(Debug)]
pub enum FormError {
    InvalidField(String),
    MissingRequiredField(String),
    InvalidFormat(String),
    ValidationFailed { field: String, reason: String },
}

impl std::fmt::Display for FormError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidField(field) => write!(f, "Invalid form field: {}", field),
            Self::MissingRequiredField(field) => write!(f, "Missing required field: {}", field),
            Self::InvalidFormat(field) => write!(f, "Invalid format for field: {}", field),
            Self::ValidationFailed { field, reason } => {
                write!(f, "Validation failed for field {}: {}", field, reason)
            }
        }
    }
}

impl std::error::Error for FormError {}

#[derive(Debug)]
pub enum TaskManagerError {
    TaskNotFound(String),
    PushNotificationError(PushNotificationError),
    SerializationError(serde_json::Error),
    FormError(FormError),
    Other(String),
}

impl std::fmt::Display for TaskManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            Self::PushNotificationError(e) => write!(f, "Push notification error: {}", e),
            Self::SerializationError(e) => write!(f, "Serialization error: {}", e),
            Self::FormError(e) => write!(f, "Form error: {}", e),
            Self::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for TaskManagerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PushNotificationError(e) => Some(e),
            Self::SerializationError(e) => Some(e),
            Self::FormError(e) => Some(e),
            Self::TaskNotFound(_) | Self::Other(_) => None,
        }
    }
}

impl From<PushNotificationError> for TaskManagerError {
    fn from(error: PushNotificationError) -> Self {
        Self::PushNotificationError(error)
    }
}

impl From<serde_json::Error> for TaskManagerError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerializationError(error)
    }
}

impl From<FormError> for TaskManagerError {
    fn from(error: FormError) -> Self {
        Self::FormError(error)
    }
}

// Add a From implementation for TaskManagerError -> JsonRpcError
impl From<TaskManagerError> for JsonRpcError {
    fn from(error: TaskManagerError) -> Self {
        match error {
            TaskManagerError::TaskNotFound(id) => JsonRpcError::task_not_found(id),
            TaskManagerError::PushNotificationError(e) => JsonRpcError::push_notification(e),
            TaskManagerError::SerializationError(e) => JsonRpcError::serialization(e),
            TaskManagerError::FormError(e) => JsonRpcError::invalid_parameters(e),
            TaskManagerError::Other(msg) => JsonRpcError::internal_error(msg),
        }
    }
}

/// Custom stream implementation for task updates
#[pin_project]
pub struct TaskUpdateStream {
    #[pin]
    receiver: broadcast::Receiver<JsonRpcResponse<StreamingResponseContent>>,
    initial_status: Option<JsonRpcResponse<StreamingResponseContent>>,
    completed: bool,
}

impl TaskUpdateStream {
    fn new(
        receiver: broadcast::Receiver<JsonRpcResponse<StreamingResponseContent>>,
        initial_status: JsonRpcResponse<StreamingResponseContent>,
    ) -> Self {
        Self {
            receiver,
            initial_status: Some(initial_status),
            completed: false,
        }
    }
}

impl Stream for TaskUpdateStream {
    type Item = JsonRpcResponse<StreamingResponseContent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        // First yield the initial status if it exists
        if let Some(initial) = this.initial_status.take() {
            return Poll::Ready(Some(initial));
        }

        // If we're already completed, return None
        if *this.completed {
            return Poll::Ready(None);
        }

        // Poll the broadcast receiver using recv method
        let mut receiver_pin = this.receiver;

        // Create a future for the next message and poll it
        let mut recv_fut = Box::pin(async move { receiver_pin.recv().await });
        match Pin::new(&mut recv_fut).poll(cx) {
            Poll::Ready(Ok(update)) => {
                // Check if this is a final status update
                // Check if this is a status update with final_status = true
                if let Some(StreamingResponseContent::StatusUpdate(status_event)) = &update.result {
                    if status_event.final_status {
                        *this.completed = true;
                    }
                }
                Poll::Ready(Some(update))
            }
            Poll::Ready(Err(_)) => {
                // Channel closed
                *this.completed = true;
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// TaskManager for managing task lifecycle
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    push_notification_configs: Arc<RwLock<HashMap<String, PushNotificationConfig>>>,
    push_notification_sender: Arc<PushNotificationSender>,
    task_update_senders:
        Arc<RwLock<HashMap<String, broadcast::Sender<JsonRpcResponse<StreamingResponseContent>>>>>,
}

impl TaskManager {
    /// Create a new task manager
    ///
    /// # Errors
    /// Returns an error if unable to initialize the push notification sender
    /// with a secure random key from /dev/urandom
    pub fn new() -> Result<Self, std::io::Error> {
        // Read 32 bytes from /dev/urandom for a secure random key
        let mut key_bytes = [0u8; 32];
        std::fs::File::open("/dev/urandom")?.read_exact(&mut key_bytes)?;

        // Create a secure push notification sender
        let push_notification_sender = PushNotificationSender::new(&key_bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        Ok(Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            push_notification_configs: Arc::new(RwLock::new(HashMap::new())),
            push_notification_sender: Arc::new(push_notification_sender),
            task_update_senders: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Helper to generate task status update events
    async fn generate_status_update_event(
        &self,
        task: &Task,
        final_status: bool,
    ) -> TaskStatusUpdateEvent {
        TaskStatusUpdateEvent {
            id: task.id.clone(),
            status: task.status.clone(),
            final_status,
            metadata: task.metadata.clone(),
        }
    }

    /// Helper to generate artifact update events
    async fn generate_artifact_update_event(
        &self,
        task_id: &str,
        artifact: &Artifact,
    ) -> TaskArtifactUpdateEvent {
        TaskArtifactUpdateEvent {
            id: task_id.to_string(),
            artifact: artifact.clone(),
            metadata: None,
        }
    }

    /// Helper to send updates to all subscribers
    async fn broadcast_status_update(
        &self,
        task: &Task,
        final_status: bool,
    ) -> Result<(), TaskManagerError> {
        let event = self.generate_status_update_event(task, final_status).await;

        // Convert to the response format
        let response = JsonRpcResponse::status_update(None, event.clone());

        // Send to all subscribers
        if let Some(sender) = self.task_update_senders.read().await.get(&task.id) {
            let _ = sender.send(response);
        }

        // Send push notification if configured
        self.send_task_status_notification(task).await?;

        Ok(())
    }

    /// Helper to broadcast artifact updates
    async fn broadcast_artifact_update(
        &self,
        task_id: &str,
        artifact: &Artifact,
    ) -> Result<(), TaskManagerError> {
        let event = self.generate_artifact_update_event(task_id, artifact).await;

        // Convert to the response format
        let response = JsonRpcResponse::artifact_update(None, event);

        // Send to all subscribers
        if let Some(sender) = self.task_update_senders.read().await.get(task_id) {
            let _ = sender.send(response);
        }

        // Get the task to send push notification
        let task = {
            let tasks = self.tasks.read().await;
            tasks.get(task_id).cloned()
        };

        // Send push notification if task exists and is configured
        if let Some(task) = task {
            self.send_task_status_notification(&task).await?;
        }

        Ok(())
    }

    /// Helper to get or create a broadcast channel for task updates
    async fn get_or_create_update_sender(
        &self,
        task_id: &str,
    ) -> broadcast::Sender<JsonRpcResponse<StreamingResponseContent>> {
        let mut senders = self.task_update_senders.write().await;

        if let Some(sender) = senders.get(task_id) {
            sender.clone()
        } else {
            // Create a new channel with a decent buffer size
            let (sender, _) = broadcast::channel(100);
            senders.insert(task_id.to_string(), sender.clone());
            sender
        }
    }

    /// Get a task by ID
    pub async fn get_task(&self, params: &TaskQueryParams) -> Result<Task, TaskManagerError> {
        let tasks = self.tasks.read().await;

        let task = tasks
            .get(&params.id)
            .cloned()
            .ok_or_else(|| TaskManagerError::TaskNotFound(params.id.clone()))?;

        // Apply history length if specified
        let mut result = task;

        if let Some(length) = params.history_length {
            if length >= 0 {
                let history_len = result.history.as_ref().map(|h| h.len()).unwrap_or(0);
                let start = if history_len as i32 > length {
                    history_len - length as usize
                } else {
                    0
                };

                if let Some(history) = result.history.as_mut() {
                    *history = history.drain(start..).collect();
                }
            }
        }

        Ok(result)
    }

    /// Create or update a task
    pub async fn upsert_task(&self, params: &TaskSendParams) -> Result<(), TaskManagerError> {
        let mut tasks = self.tasks.write().await;

        let task = if let Some(existing_task) = tasks.get_mut(&params.id) {
            // Update existing task
            if let Some(history) = existing_task.history.as_mut() {
                history.push(params.message.clone());
            } else {
                existing_task.history = Some(vec![params.message.clone()]);
            }

            // Update status to working if it was submitted
            if existing_task.status.state == TaskState::Submitted {
                existing_task.status = TaskStatus {
                    state: TaskState::Working,
                    message: None,
                    timestamp: Some(Utc::now()),
                };
            }

            existing_task.clone()
        } else {
            // Create new task
            let task = Task {
                id: params.id.clone(),
                session_id: params.session_id.clone(),
                status: TaskStatus {
                    state: TaskState::Submitted,
                    message: None,
                    timestamp: Some(Utc::now()),
                },
                artifacts: None,
                history: Some(vec![params.message.clone()]),
                metadata: params.metadata.clone(),
            };

            tasks.insert(params.id.clone(), task.clone());
            task
        };

        // Store push notification config if provided
        if let Some(ref push_config) = params.push_notification {
            if self
                .push_notification_sender
                .verify_push_notification_url(&push_config.url)
                .await?
            {
                let mut configs = self.push_notification_configs.write().await;
                configs.insert(params.id.clone(), push_config.clone());
            }
        }

        // Drop the lock before broadcasting
        drop(tasks);

        // Broadcast update
        self.broadcast_status_update(&task, false).await?;

        Ok(())
    }

    /// Update task status
    pub async fn update_task_status(
        &self,
        id: &str,
        status: TaskStatus,
    ) -> Result<(), TaskManagerError> {
        let mut tasks = self.tasks.write().await;

        let task = tasks
            .get_mut(id)
            .ok_or_else(|| TaskManagerError::TaskNotFound(id.to_string()))?;

        // Update status
        task.status = status;

        // Add message to history if provided
        if let Some(ref message) = task.status.message {
            if let Some(history) = task.history.as_mut() {
                history.push(message.clone());
            } else {
                task.history = Some(vec![message.clone()]);
            }
        }

        let is_final = matches!(
            task.status.state,
            TaskState::Completed | TaskState::Failed | TaskState::Canceled
        );

        let result = task.clone();

        // Drop the lock before broadcasting
        drop(tasks);

        // Broadcast update
        self.broadcast_status_update(&result, is_final).await?;

        Ok(())
    }

    /// Add an artifact to a task
    pub async fn add_task_artifact(
        &self,
        id: &str,
        artifact: Artifact,
    ) -> Result<(), TaskManagerError> {
        let mut tasks = self.tasks.write().await;

        let task = tasks
            .get_mut(id)
            .ok_or_else(|| TaskManagerError::TaskNotFound(id.to_string()))?;

        // Add artifact
        if let Some(artifacts) = task.artifacts.as_mut() {
            artifacts.push(artifact.clone());
        } else {
            task.artifacts = Some(vec![artifact.clone()]);
        }

        // Drop the lock before broadcasting
        drop(tasks);

        // Broadcast artifact update
        self.broadcast_artifact_update(id, &artifact).await?;

        Ok(())
    }

    /// Add a message to a task and update its status
    pub async fn add_task_message(
        &self,
        id: &str,
        message: crate::types::Message,
        state: TaskState,
    ) -> Result<(), TaskManagerError> {
        let mut tasks = self.tasks.write().await;

        let task = tasks
            .get_mut(id)
            .ok_or_else(|| TaskManagerError::TaskNotFound(id.to_string()))?;

        // Update the task's status
        task.status = TaskStatus {
            state,
            message: Some(message.clone()),
            timestamp: Some(Utc::now()),
        };

        // Add message to history if it doesn't already exist
        if let Some(history) = task.history.as_mut() {
            history.push(message);
        } else {
            task.history = Some(vec![message]);
        }

        // Copy task for broadcasting
        let task_copy = task.clone();

        // Drop the lock before broadcasting
        drop(tasks);

        // Broadcast status update, marking as final if it's a terminal state
        let is_final = matches!(
            state,
            TaskState::Completed | TaskState::Failed | TaskState::Canceled
        );

        self.broadcast_status_update(&task_copy, is_final).await?;

        Ok(())
    }

    /// Add form data to a task and update its status
    ///
    /// This method is a specialized version of `add_task_message` for handling form
    /// submissions. It creates a message with a Data part containing the form data.
    pub async fn add_task_form_data(
        &self,
        id: &str,
        form_data: HashMap<String, serde_json::Value>,
        state: TaskState,
    ) -> Result<(), TaskManagerError> {
        // Create a message with the form data
        use crate::types::{Message, MessageRole, Part};

        // Validate the form data (basic validation example)
        let task = self
            .tasks
            .read()
            .await
            .get(id)
            .cloned()
            .ok_or_else(|| TaskManagerError::TaskNotFound(id.to_string()))?;

        // Extract the form_id from the submission if provided
        let submitted_form_id = form_data
            .get("form_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Helper function to validate form data against a schema
        fn validate_form_data(
            form: &serde_json::Value,
            form_data: &HashMap<String, serde_json::Value>,
        ) -> Result<(), FormError> {
            // Get properties from the form schema
            let Some(properties) = form.get("properties").and_then(|p| p.as_object()) else {
                return Ok(());
            };

            // Extract required fields
            let required_fields =
                if let Some(serde_json::Value::Array(fields)) = form.get("required") {
                    fields
                        .iter()
                        .filter_map(|v| {
                            if let serde_json::Value::String(s) = v {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<String>>()
                } else {
                    Vec::new()
                };

            // Check required fields
            for field in &required_fields {
                if !form_data.contains_key(field)
                    || form_data.get(field).is_none()
                    || form_data.get(field).is_some_and(|v| v.is_null())
                {
                    return Err(FormError::MissingRequiredField(field.clone()));
                }
            }

            // Validate field formats if specified
            for (field_name, field_def) in properties {
                // Skip if the field is not in the submitted data
                let Some(field_value) = form_data.get(field_name) else {
                    continue;
                };

                // Get the field format if specified
                if let Some(format) = field_def.get("format").and_then(|f| f.as_str()) {
                    match format {
                        "email" => {
                            // Basic email validation - must contain @ and at least one dot after @
                            if let Some(email) = field_value.as_str() {
                                if !email.contains('@')
                                    || !email.split('@').nth(1).is_some_and(|s| s.contains('.'))
                                {
                                    return Err(FormError::InvalidFormat(field_name.clone()));
                                }
                            }
                        }
                        "number" => {
                            // Check that the value is a number
                            if !field_value.is_number() {
                                return Err(FormError::InvalidFormat(field_name.clone()));
                            }
                        }
                        "url" => {
                            // Basic URL validation - must start with http or https
                            if let Some(url) = field_value.as_str() {
                                if !url.starts_with("http://") && !url.starts_with("https://") {
                                    return Err(FormError::InvalidFormat(field_name.clone()));
                                }
                            }
                        }
                        // Add additional format validations as needed
                        _ => {}
                    }
                }
            }

            Ok(())
        }

        // Find the matching form schema in task history
        let mut found_matching_schema = false;
        let mut form_validated = false;

        if let Some(history) = &task.history {
            for msg in history.iter() {
                // Skip if not from agent (forms are requested by agents)
                if msg.role != MessageRole::Agent {
                    continue;
                }

                // Iterate through all parts of the message
                for part in &msg.parts {
                    let Part::Data {
                        data,
                        metadata: Some(metadata),
                    } = part
                    else {
                        continue;
                    };

                    let Some(serde_json::Value::String(content_type)) =
                        metadata.get("content_type")
                    else {
                        continue;
                    };

                    // Only process form schema content types
                    if content_type != CONTENT_TYPE_FORM_SCHEMA {
                        continue;
                    }

                    let Some(form) = data.get("form") else {
                        continue;
                    };

                    // Get the form's ID (required for all forms)
                    let schema_form_id = match data
                        .get("form_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                    {
                        Some(id) => id,
                        None => {
                            // Skip schemas without form_id
                            continue;
                        }
                    };

                    // Get the submission ID (required for all submissions)
                    let submission_id = match &submitted_form_id {
                        Some(id) => id,
                        None => {
                            // No form ID provided in submission, can't match
                            continue;
                        }
                    };

                    // Check if this form matches the submission
                    if submission_id != &schema_form_id {
                        // Not a match, continue searching
                        continue;
                    }

                    // We found a matching schema
                    found_matching_schema = true;

                    // Validate the form submission against the schema
                    if let Err(e) = validate_form_data(form, &form_data) {
                        return Err(e.into());
                    }

                    // Form validated successfully
                    form_validated = true;

                    // We found the exact form we're looking for, stop searching
                    break;
                }

                // Stop looking if we found and validated a matching form
                if found_matching_schema && form_validated {
                    break;
                }
            }
        }

        // Form ID is required for all submissions
        if submitted_form_id.is_none() {
            return Err(TaskManagerError::FormError(FormError::ValidationFailed {
                field: "form_id".to_string(),
                reason: "Form ID is required in all form submissions".to_string(),
            }));
        }

        // Must find a matching form schema
        if !found_matching_schema {
            return Err(TaskManagerError::FormError(FormError::ValidationFailed {
                field: "form_id".to_string(),
                reason: format!(
                    "No matching form schema found for ID: {}",
                    submitted_form_id.unwrap()
                ),
            }));
        }

        // Validation must succeed
        if !form_validated {
            return Err(TaskManagerError::FormError(FormError::ValidationFailed {
                field: "form".to_string(),
                reason: "Form validation failed".to_string(),
            }));
        }

        // Create message parts
        let mut parts = Vec::new();

        // Add a summary text part
        parts.push(Part::Text {
            text: format!("Form submitted with {} fields", form_data.len()),
            metadata: None,
        });

        // Create metadata for form submission
        let mut metadata = HashMap::new();
        metadata.insert(
            "content_type".to_string(),
            serde_json::Value::String(CONTENT_TYPE_FORM_SUBMISSION.to_string()),
        );

        // Add form_id to metadata if it was provided
        if let Some(form_id) = &submitted_form_id {
            metadata.insert(
                "form_id".to_string(),
                serde_json::Value::String(form_id.clone()),
            );
        }

        // Add the form data part
        parts.push(Part::Data {
            data: form_data,
            metadata: Some(metadata),
        });

        let message = Message {
            role: MessageRole::User,
            parts,
            metadata: None,
        };

        // Use the existing add_task_message method
        self.add_task_message(id, message, state).await
    }

    /// Request form input from the user
    ///
    /// This is a specialized method to update a task status to InputRequired
    /// with a form schema that the user needs to fill out.
    pub async fn request_form_input(
        &self,
        id: &str,
        form_schema: crate::types::FormSchema,
        instructions: Option<String>,
    ) -> Result<(), TaskManagerError> {
        use crate::types::{Message, MessageRole, Part};
        use uuid::Uuid;

        // Generate a unique form ID
        let form_id = Uuid::new_v4().to_string();

        // Serialize the form schema to a Value
        let form_schema_value =
            serde_json::to_value(&form_schema).map_err(TaskManagerError::SerializationError)?;

        // Create form data structure
        let mut data = HashMap::new();
        data.insert("form".to_string(), form_schema_value);
        data.insert("form_id".to_string(), serde_json::json!(form_id));

        // Create message parts
        let mut parts = Vec::new();

        // Add a text part with instructions if provided and also add to data
        if let Some(instr) = instructions {
            // Add to form data
            data.insert(
                "instructions".to_string(),
                serde_json::Value::String(instr.clone()),
            );

            // Add as text part
            parts.push(Part::Text {
                text: instr,
                metadata: None,
            });
        }

        // Create metadata with content type and form ID
        let mut metadata = HashMap::new();
        metadata.insert(
            "content_type".to_string(),
            serde_json::Value::String(CONTENT_TYPE_FORM_SCHEMA.to_string()),
        );
        metadata.insert("form_id".to_string(), serde_json::json!(form_id));

        // Add the form schema data part
        parts.push(Part::Data {
            data,
            metadata: Some(metadata),
        });

        // Create a message with the form schema
        let message = Message {
            role: MessageRole::Agent,
            parts,
            metadata: None,
        };

        // Update task with this message and set state to InputRequired
        self.add_task_message(id, message, TaskState::InputRequired)
            .await
    }

    /// Cancel a task
    pub async fn cancel_task(&self, params: &TaskIdParams) -> Result<(), TaskManagerError> {
        let mut tasks = self.tasks.write().await;

        let task = tasks
            .get_mut(&params.id)
            .ok_or_else(|| TaskManagerError::TaskNotFound(params.id.clone()))?;

        // Only allow cancellation if task is in a cancellable state
        if matches!(
            task.status.state,
            TaskState::Submitted | TaskState::Working | TaskState::InputRequired
        ) {
            task.status = TaskStatus {
                state: TaskState::Canceled,
                message: None,
                timestamp: Some(Utc::now()),
            };

            let task_copy = task.clone();

            // Drop the lock before broadcasting
            drop(tasks);

            // Broadcast update
            self.broadcast_status_update(&task_copy, true).await?;

            Ok(())
        } else {
            Err(TaskManagerError::Other(format!(
                "Task {} cannot be canceled in state {:?}",
                params.id, task.status.state
            )))
        }
    }

    /// Subscribe to task updates
    pub async fn subscribe_to_task(&self, id: &str) -> Result<TaskUpdateStream, TaskManagerError> {
        // Check that task exists
        let tasks = self.tasks.read().await;
        if !tasks.contains_key(id) {
            return Err(TaskManagerError::TaskNotFound(id.to_string()));
        }

        // Get the current task for initial update
        let task = tasks.get(id).unwrap().clone();

        // Drop the lock
        drop(tasks);

        // Get broadcast sender
        let sender = self.get_or_create_update_sender(id).await;

        // Create receiver
        let receiver = sender.subscribe();

        // Create the initial status update event
        let initial_event = self.generate_status_update_event(&task, false).await;

        // Initial status response
        let initial_response = JsonRpcResponse::status_update(None, initial_event);

        // Create the stream with the initial status
        Ok(TaskUpdateStream::new(receiver, initial_response))
    }

    /// Set push notification configuration for a task
    ///
    /// This is the handler for the JSON-RPC API endpoint. It internally uses
    /// the `set_push_notification_config` method from the `PushNotificationSupport` trait.
    pub async fn set_push_notification(
        &self,
        config: &TaskPushNotificationConfig,
    ) -> Result<TaskPushNotificationConfig, TaskManagerError> {
        // Verify the task exists
        let tasks = self.tasks.read().await;
        if !tasks.contains_key(&config.id) {
            return Err(TaskManagerError::TaskNotFound(config.id.clone()));
        }
        drop(tasks);

        // Use the trait implementation
        self.set_push_notification_config(&config.id, config.push_notification_config.clone())
            .await?;

        Ok(config.clone())
    }

    /// Get push notification configuration for a task
    ///
    /// This is the handler for the JSON-RPC API endpoint. It internally uses
    /// the `get_push_notification_config` method from the `PushNotificationSupport` trait.
    pub async fn get_push_notification(
        &self,
        params: &TaskIdParams,
    ) -> Result<TaskPushNotificationConfig, TaskManagerError> {
        // Check that task exists
        let tasks = self.tasks.read().await;
        if !tasks.contains_key(&params.id) {
            return Err(TaskManagerError::TaskNotFound(params.id.clone()));
        }
        drop(tasks);

        // Use the trait implementation
        let config = self
            .get_push_notification_config(&params.id)
            .await?
            .ok_or_else(|| {
                TaskManagerError::Other(format!(
                    "No push notification configuration found for task {}",
                    params.id
                ))
            })?;

        Ok(TaskPushNotificationConfig {
            id: params.id.clone(),
            push_notification_config: config,
        })
    }
}

// Implement PushNotificationSupport for TaskManager
#[async_trait::async_trait]
impl PushNotificationSupport for TaskManager {
    async fn set_push_notification_config(
        &self,
        task_id: &str,
        config: PushNotificationConfig,
    ) -> Result<(), PushNotificationError> {
        // Verify URL before storing
        if !self
            .push_notification_sender
            .verify_push_notification_url(&config.url)
            .await?
        {
            return Err(PushNotificationError::Other(format!(
                "Failed to validate push notification URL: {}",
                config.url
            )));
        }

        // Store the config
        let mut configs = self.push_notification_configs.write().await;
        configs.insert(task_id.to_string(), config);

        Ok(())
    }

    async fn get_push_notification_config(
        &self,
        task_id: &str,
    ) -> Result<Option<PushNotificationConfig>, PushNotificationError> {
        let configs = self.push_notification_configs.read().await;
        Ok(configs.get(task_id).cloned())
    }

    async fn has_push_notification_config(
        &self,
        task_id: &str,
    ) -> Result<bool, PushNotificationError> {
        let configs = self.push_notification_configs.read().await;
        Ok(configs.contains_key(task_id))
    }

    async fn send_task_status_notification(
        &self,
        task: &Task,
    ) -> Result<(), PushNotificationError> {
        if let Ok(true) = self.has_push_notification_config(&task.id).await {
            if let Ok(Some(config)) = self.get_push_notification_config(&task.id).await {
                self.push_notification_sender
                    .send_push_notification(&config, task)
                    .await?;
            }
        }

        Ok(())
    }
}
