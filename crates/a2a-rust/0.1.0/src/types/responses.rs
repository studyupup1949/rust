use serde::{Deserialize, Serialize};

use crate::A2AError;
use crate::types::JsonObject;

use super::message::{Artifact, Message};
use super::push::TaskPushNotificationConfig;
use super::task::{Task, TaskStatus};

/// Streaming event emitted when a task status changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatusUpdateEvent {
    /// Task identifier.
    pub task_id: String,
    /// Context identifier shared with the task.
    pub context_id: String,
    /// Updated task status snapshot.
    pub status: TaskStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional event metadata.
    pub metadata: Option<JsonObject>,
}

/// Streaming event emitted when an artifact changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskArtifactUpdateEvent {
    /// Task identifier.
    pub task_id: String,
    /// Context identifier shared with the task.
    pub context_id: String,
    /// Artifact snapshot or chunk.
    pub artifact: Artifact,
    #[serde(default, skip_serializing_if = "crate::types::is_false")]
    /// Whether the artifact payload should append to prior chunks.
    pub append: bool,
    #[serde(default, skip_serializing_if = "crate::types::is_false")]
    /// Whether this is the last chunk for the artifact.
    pub last_chunk: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional event metadata.
    pub metadata: Option<JsonObject>,
}

fn validate_task(task: &Task) -> Result<(), A2AError> {
    for artifact in &task.artifacts {
        artifact.validate()?;
    }

    for message in &task.history {
        message.validate()?;
    }

    if let Some(message) = &task.status.message {
        message.validate()?;
    }

    Ok(())
}

/// Oneof-style result for `SendMessage`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SendMessageResponse {
    /// Response returned as a task.
    Task(Task),
    /// Response returned directly as a message.
    Message(Message),
}

impl SendMessageResponse {
    /// Validate nested task or message content.
    pub fn validate(&self) -> Result<(), A2AError> {
        match self {
            Self::Task(task) => validate_task(task),
            Self::Message(message) => message.validate(),
        }
    }
}

/// Oneof-style item emitted on streaming operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StreamResponse {
    /// Task snapshot event.
    Task(Task),
    /// Message event.
    Message(Message),
    /// Task status update event.
    StatusUpdate(TaskStatusUpdateEvent),
    /// Task artifact update event.
    ArtifactUpdate(TaskArtifactUpdateEvent),
}

impl StreamResponse {
    /// Validate nested event content.
    pub fn validate(&self) -> Result<(), A2AError> {
        match self {
            Self::Task(task) => validate_task(task),
            Self::Message(message) => message.validate(),
            Self::StatusUpdate(update) => {
                if let Some(message) = &update.status.message {
                    message.validate()?;
                }

                Ok(())
            }
            Self::ArtifactUpdate(update) => update.artifact.validate(),
        }
    }
}

/// Paginated response for `ListTasks`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTasksResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Returned task page.
    pub tasks: Vec<Task>,
    /// Opaque token for the next page, or an empty string when exhausted.
    pub next_page_token: String,
    /// Requested page size echoed in the response.
    pub page_size: i32,
    /// Total number of matching tasks.
    pub total_size: i32,
}

/// Paginated response for `ListTaskPushNotificationConfigs`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTaskPushNotificationConfigsResponse {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Returned push-notification configuration page.
    pub configs: Vec<TaskPushNotificationConfig>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    /// Opaque token for the next page, or an empty string when exhausted.
    pub next_page_token: String,
}

#[cfg(test)]
mod tests {
    use super::{
        ListTaskPushNotificationConfigsResponse, SendMessageResponse, StreamResponse,
        TaskArtifactUpdateEvent, TaskStatusUpdateEvent,
    };
    use crate::types::{Artifact, Message, Part, Role, Task, TaskState, TaskStatus};

    #[test]
    fn send_message_response_uses_proto_oneof_shape() {
        let response = SendMessageResponse::Message(Message {
            message_id: "msg-1".to_owned(),
            context_id: None,
            task_id: None,
            role: Role::Agent,
            parts: vec![Part {
                text: Some("done".to_owned()),
                raw: None,
                url: None,
                data: None,
                metadata: None,
                filename: None,
                media_type: None,
            }],
            metadata: None,
            extensions: Vec::new(),
            reference_task_ids: Vec::new(),
        });

        let json = serde_json::to_string(&response).expect("response should serialize");
        assert_eq!(
            json,
            r#"{"message":{"messageId":"msg-1","role":"ROLE_AGENT","parts":[{"text":"done"}]}}"#
        );
    }

    #[test]
    fn send_message_response_validate_rejects_invalid_part() {
        let response = SendMessageResponse::Message(Message {
            message_id: "msg-1".to_owned(),
            context_id: None,
            task_id: None,
            role: Role::Agent,
            parts: vec![Part {
                text: Some("done".to_owned()),
                raw: Some(vec![104, 105]),
                url: None,
                data: None,
                metadata: None,
                filename: None,
                media_type: None,
            }],
            metadata: None,
            extensions: Vec::new(),
            reference_task_ids: Vec::new(),
        });

        let error = response.validate().expect_err("response should be invalid");
        assert!(
            error
                .to_string()
                .contains("part cannot contain more than one")
        );
    }

    #[test]
    fn list_push_notification_response_uses_empty_string_for_no_next_page() {
        let response = ListTaskPushNotificationConfigsResponse {
            configs: Vec::new(),
            next_page_token: String::new(),
        };

        let json = serde_json::to_string(&response).expect("response should serialize");
        assert_eq!(json, "{}");
    }

    #[test]
    fn task_status_update_event_round_trip_serialization() {
        let event = TaskStatusUpdateEvent {
            task_id: "task-1".to_owned(),
            context_id: "ctx-1".to_owned(),
            status: TaskStatus {
                state: TaskState::Working,
                message: Some(Message {
                    message_id: "msg-1".to_owned(),
                    context_id: Some("ctx-1".to_owned()),
                    task_id: Some("task-1".to_owned()),
                    role: Role::Agent,
                    parts: vec![Part {
                        text: Some("still working".to_owned()),
                        raw: None,
                        url: None,
                        data: None,
                        metadata: None,
                        filename: None,
                        media_type: None,
                    }],
                    metadata: None,
                    extensions: Vec::new(),
                    reference_task_ids: Vec::new(),
                }),
                timestamp: Some("2026-03-12T12:00:00Z".to_owned()),
            },
            metadata: None,
        };

        let json = serde_json::to_string(&event).expect("event should serialize");
        let round_trip: TaskStatusUpdateEvent =
            serde_json::from_str(&json).expect("event should deserialize");

        assert_eq!(round_trip.task_id, "task-1");
        assert_eq!(round_trip.status.state, TaskState::Working);
    }

    #[test]
    fn task_artifact_update_event_round_trip_serialization() {
        let event = TaskArtifactUpdateEvent {
            task_id: "task-1".to_owned(),
            context_id: "ctx-1".to_owned(),
            artifact: Artifact {
                artifact_id: "artifact-1".to_owned(),
                name: Some("result".to_owned()),
                description: None,
                parts: vec![Part {
                    text: Some("partial".to_owned()),
                    raw: None,
                    url: None,
                    data: None,
                    metadata: None,
                    filename: None,
                    media_type: None,
                }],
                metadata: None,
                extensions: Vec::new(),
            },
            append: true,
            last_chunk: false,
            metadata: None,
        };

        let json = serde_json::to_string(&event).expect("event should serialize");
        let round_trip: TaskArtifactUpdateEvent =
            serde_json::from_str(&json).expect("event should deserialize");

        assert!(round_trip.append);
        assert!(!round_trip.last_chunk);
        assert_eq!(round_trip.artifact.artifact_id, "artifact-1");
    }

    #[test]
    fn stream_response_round_trip_serialization() {
        let response = StreamResponse::Task(Task {
            id: "task-1".to_owned(),
            context_id: Some("ctx-1".to_owned()),
            status: TaskStatus {
                state: TaskState::Submitted,
                message: None,
                timestamp: Some("2026-03-12T12:00:00Z".to_owned()),
            },
            artifacts: Vec::new(),
            history: Vec::new(),
            metadata: None,
        });

        let json = serde_json::to_string(&response).expect("response should serialize");
        let round_trip: StreamResponse =
            serde_json::from_str(&json).expect("response should deserialize");

        match round_trip {
            StreamResponse::Task(task) => assert_eq!(task.id, "task-1"),
            _ => panic!("expected task stream response"),
        }
    }
}
