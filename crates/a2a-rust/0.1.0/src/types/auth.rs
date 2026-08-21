use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::A2AError;

use super::{JsonObject, Message, Task, TaskState, TaskStatus};

/// Conventional metadata payload used when a task enters `TASK_STATE_AUTH_REQUIRED`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequiredMetadata {
    /// Authorization URL the user should visit.
    pub auth_url: String,
    /// Authentication scheme, such as `oauth2` or `apiKey`.
    pub auth_scheme: String,
    /// Scopes requested by the agent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
    /// Human-readable explanation for the authorization request.
    pub description: String,
}

impl AuthRequiredMetadata {
    /// Parse the convention from a metadata object.
    pub fn from_metadata(metadata: &JsonObject) -> Result<Self, A2AError> {
        serde_json::from_value(Value::Object(metadata.clone())).map_err(A2AError::from)
    }

    /// Convert the convention into a message metadata object.
    pub fn into_metadata(self) -> Result<JsonObject, A2AError> {
        match serde_json::to_value(self)? {
            Value::Object(object) => Ok(object),
            _ => Err(A2AError::Internal(
                "auth-required metadata did not serialize to an object".to_owned(),
            )),
        }
    }
}

impl Message {
    /// Parse `TASK_STATE_AUTH_REQUIRED` metadata from this message, if present.
    ///
    /// This helper is intended for messages already known to participate in the
    /// auth-required flow. If `metadata` exists but does not match the
    /// `AuthRequiredMetadata` schema, this returns `Err` rather than `Ok(None)`.
    pub fn auth_required_metadata(&self) -> Result<Option<AuthRequiredMetadata>, A2AError> {
        self.metadata
            .as_ref()
            .map(AuthRequiredMetadata::from_metadata)
            .transpose()
    }

    /// Replace this message's metadata with the auth-required convention payload.
    pub fn set_auth_required_metadata(
        &mut self,
        metadata: AuthRequiredMetadata,
    ) -> Result<(), A2AError> {
        self.metadata = Some(metadata.into_metadata()?);
        Ok(())
    }
}

impl TaskStatus {
    /// Parse auth-required metadata from the current status message when present.
    ///
    /// If the nested status message carries unrelated metadata, this returns
    /// the underlying parse error instead of `Ok(None)`.
    pub fn auth_required_metadata(&self) -> Result<Option<AuthRequiredMetadata>, A2AError> {
        self.message
            .as_ref()
            .map(Message::auth_required_metadata)
            .transpose()
            .map(|metadata| metadata.flatten())
    }

    /// Validate that `TASK_STATE_AUTH_REQUIRED` carries the expected metadata convention.
    pub fn validate_auth_required_metadata(&self) -> Result<(), A2AError> {
        if self.state != TaskState::AuthRequired {
            return Ok(());
        }

        let Some(message) = &self.message else {
            return Err(A2AError::InvalidRequest(
                "TASK_STATE_AUTH_REQUIRED requires a status message carrying auth metadata"
                    .to_owned(),
            ));
        };

        if message.auth_required_metadata()?.is_none() {
            return Err(A2AError::InvalidRequest(
                "TASK_STATE_AUTH_REQUIRED status message metadata must include authUrl, authScheme, scopes, and description"
                    .to_owned(),
            ));
        }

        Ok(())
    }
}

impl Task {
    /// Return auth-required metadata from the current status message or last history item.
    ///
    /// This returns `Ok(None)` when the task is not in `TASK_STATE_AUTH_REQUIRED`.
    /// When the task is in that state, unrelated metadata on the candidate
    /// message is treated as an error so callers can distinguish malformed
    /// auth-required payloads from the absence of auth metadata.
    pub fn auth_required_metadata(&self) -> Result<Option<AuthRequiredMetadata>, A2AError> {
        if self.status.state != TaskState::AuthRequired {
            return Ok(None);
        }

        if let Some(metadata) = self.status.auth_required_metadata()? {
            return Ok(Some(metadata));
        }

        self.history
            .last()
            .map(Message::auth_required_metadata)
            .transpose()
            .map(|metadata| metadata.flatten())
    }

    /// Validate the repository's `TASK_STATE_AUTH_REQUIRED` metadata convention.
    pub fn validate_auth_required_convention(&self) -> Result<(), A2AError> {
        if self.status.state != TaskState::AuthRequired {
            return Ok(());
        }

        if self.auth_required_metadata()?.is_none() {
            return Err(A2AError::InvalidRequest(
                "TASK_STATE_AUTH_REQUIRED requires auth metadata on the status message or last task message"
                    .to_owned(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AuthRequiredMetadata;
    use crate::types::{Message, Part, Role, Task, TaskState, TaskStatus};

    #[test]
    fn auth_required_metadata_round_trips_through_message_metadata() {
        let mut message = Message {
            message_id: "msg-auth-1".to_owned(),
            context_id: Some("ctx-1".to_owned()),
            task_id: Some("task-1".to_owned()),
            role: Role::Agent,
            parts: vec![Part {
                text: Some("Please authorize access.".to_owned()),
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
        };

        message
            .set_auth_required_metadata(AuthRequiredMetadata {
                auth_url: "https://example.com/oauth/authorize".to_owned(),
                auth_scheme: "oauth2".to_owned(),
                scopes: vec!["calendar.read".to_owned()],
                description: "Grant calendar access".to_owned(),
            })
            .expect("metadata should set");

        let metadata = message
            .auth_required_metadata()
            .expect("metadata should parse")
            .expect("metadata should exist");

        assert_eq!(metadata.auth_scheme, "oauth2");
        assert_eq!(metadata.scopes, vec!["calendar.read"]);
    }

    #[test]
    fn task_validates_auth_required_convention() {
        let mut message = Message {
            message_id: "msg-auth-1".to_owned(),
            context_id: Some("ctx-1".to_owned()),
            task_id: Some("task-1".to_owned()),
            role: Role::Agent,
            parts: vec![Part {
                text: Some("Authorize to continue.".to_owned()),
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
        };
        message
            .set_auth_required_metadata(AuthRequiredMetadata {
                auth_url: "https://example.com/oauth/authorize".to_owned(),
                auth_scheme: "oauth2".to_owned(),
                scopes: vec!["drive.readonly".to_owned()],
                description: "Grant drive access".to_owned(),
            })
            .expect("metadata should set");

        let task = Task {
            id: "task-1".to_owned(),
            context_id: Some("ctx-1".to_owned()),
            status: TaskStatus {
                state: TaskState::AuthRequired,
                message: Some(message),
                timestamp: Some("2026-03-13T12:00:00Z".to_owned()),
            },
            artifacts: Vec::new(),
            history: Vec::new(),
            metadata: None,
        };

        task.validate_auth_required_convention()
            .expect("convention should validate");
    }

    #[test]
    fn task_rejects_auth_required_without_metadata() {
        let task = Task {
            id: "task-1".to_owned(),
            context_id: Some("ctx-1".to_owned()),
            status: TaskStatus {
                state: TaskState::AuthRequired,
                message: Some(Message {
                    message_id: "msg-auth-1".to_owned(),
                    context_id: Some("ctx-1".to_owned()),
                    task_id: Some("task-1".to_owned()),
                    role: Role::Agent,
                    parts: vec![Part {
                        text: Some("Authorize to continue.".to_owned()),
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
                timestamp: Some("2026-03-13T12:00:00Z".to_owned()),
            },
            artifacts: Vec::new(),
            history: Vec::new(),
            metadata: None,
        };

        let error = task
            .validate_auth_required_convention()
            .expect_err("convention should fail");
        assert!(error.to_string().contains("TASK_STATE_AUTH_REQUIRED"));
    }
}
