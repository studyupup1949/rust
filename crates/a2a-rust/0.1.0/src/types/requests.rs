use serde::{Deserialize, Serialize};

use crate::A2AError;
use crate::types::JsonObject;

use super::message::Message;
use super::push::TaskPushNotificationConfig;
use super::task::TaskState;

/// Optional configuration for `SendMessage`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageConfiguration {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Output modes the caller can accept.
    pub accepted_output_modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional task push configuration to attach to the request.
    pub task_push_notification_config: Option<TaskPushNotificationConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Maximum history items requested in task responses.
    pub history_length: Option<i32>,
    #[serde(default, skip_serializing_if = "crate::types::is_false")]
    /// Whether the server should return immediately instead of waiting.
    pub return_immediately: bool,
}

/// Request payload for `SendMessage`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    /// Input message from the caller.
    pub message: Message,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional message handling configuration.
    pub configuration: Option<SendMessageConfiguration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional request metadata.
    pub metadata: Option<JsonObject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional tenant identifier.
    pub tenant: Option<String>,
}

impl SendMessageRequest {
    /// Validate nested message content.
    pub fn validate(&self) -> Result<(), A2AError> {
        self.message.validate()
    }
}

/// Request payload for `GetTask`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTaskRequest {
    /// Task identifier.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Maximum history items requested in the response.
    pub history_length: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional tenant identifier.
    pub tenant: Option<String>,
}

/// Request payload for `ListTasks`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTasksRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional tenant identifier.
    pub tenant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional context filter.
    pub context_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional task-state filter.
    pub status: Option<TaskState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Requested page size.
    pub page_size: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Opaque pagination token from a previous response.
    pub page_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Maximum history items requested per returned task.
    pub history_length: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Lower bound for task status timestamps.
    pub status_timestamp_after: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Whether artifacts should be included in results.
    pub include_artifacts: Option<bool>,
}

impl ListTasksRequest {
    /// Validate pagination bounds.
    pub fn validate(&self) -> Result<(), A2AError> {
        if let Some(page_size) = self.page_size
            && !(1..=100).contains(&page_size)
        {
            return Err(A2AError::InvalidRequest(
                "pageSize must be between 1 and 100".to_owned(),
            ));
        }

        Ok(())
    }
}

/// Request payload for `CancelTask`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelTaskRequest {
    /// Task identifier.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional tenant identifier.
    pub tenant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional request metadata.
    pub metadata: Option<JsonObject>,
}

/// Request payload for `GetTaskPushNotificationConfig`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTaskPushNotificationConfigRequest {
    /// Push configuration identifier.
    pub id: String,
    /// Owning task identifier.
    pub task_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional tenant identifier.
    pub tenant: Option<String>,
}

/// Request payload for `DeleteTaskPushNotificationConfig`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteTaskPushNotificationConfigRequest {
    /// Push configuration identifier.
    pub id: String,
    /// Owning task identifier.
    pub task_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional tenant identifier.
    pub tenant: Option<String>,
}

/// Request payload for `SubscribeToTask`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeToTaskRequest {
    /// Task identifier.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional tenant identifier.
    pub tenant: Option<String>,
}

/// Request payload for `ListTaskPushNotificationConfigs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTaskPushNotificationConfigsRequest {
    /// Owning task identifier.
    pub task_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Requested page size.
    pub page_size: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Opaque pagination token from a previous response.
    pub page_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional tenant identifier.
    pub tenant: Option<String>,
}

impl ListTaskPushNotificationConfigsRequest {
    /// Validate required identifiers.
    pub fn validate(&self) -> Result<(), A2AError> {
        if self.task_id.is_empty() {
            return Err(A2AError::InvalidRequest(
                "task_id must not be empty".to_owned(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ListTaskPushNotificationConfigsRequest, ListTasksRequest, SendMessageRequest};
    use crate::types::{Message, Part, Role};

    #[test]
    fn list_task_push_notification_configs_request_rejects_empty_task_id() {
        let request = ListTaskPushNotificationConfigsRequest {
            task_id: String::new(),
            page_size: None,
            page_token: None,
            tenant: None,
        };

        let error = request.validate().expect_err("request should be invalid");
        assert!(error.to_string().contains("task_id must not be empty"));
    }

    #[test]
    fn list_tasks_request_rejects_out_of_range_page_size() {
        let request = ListTasksRequest {
            tenant: None,
            context_id: None,
            status: None,
            page_size: Some(101),
            page_token: None,
            history_length: None,
            status_timestamp_after: None,
            include_artifacts: None,
        };

        let error = request.validate().expect_err("request should be invalid");
        assert!(
            error
                .to_string()
                .contains("pageSize must be between 1 and 100")
        );
    }

    #[test]
    fn send_message_request_rejects_empty_message_parts() {
        let request = SendMessageRequest {
            message: Message {
                message_id: "msg-1".to_owned(),
                context_id: None,
                task_id: None,
                role: Role::User,
                parts: Vec::new(),
                metadata: None,
                extensions: Vec::new(),
                reference_task_ids: Vec::new(),
            },
            configuration: None,
            metadata: None,
            tenant: None,
        };

        let error = request.validate().expect_err("request should be invalid");
        assert!(
            error
                .to_string()
                .contains("message must contain at least one part")
        );
    }

    #[test]
    fn send_message_request_validates_part_content() {
        let request = SendMessageRequest {
            message: Message {
                message_id: "msg-1".to_owned(),
                context_id: None,
                task_id: None,
                role: Role::User,
                parts: vec![Part {
                    text: Some("hello".to_owned()),
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
            },
            configuration: None,
            metadata: None,
            tenant: None,
        };

        let error = request.validate().expect_err("request should be invalid");
        assert!(
            error
                .to_string()
                .contains("part cannot contain more than one")
        );
    }
}

/// Request payload for `GetExtendedAgentCard`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetExtendedAgentCardRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional tenant identifier.
    pub tenant: Option<String>,
}
