use std::collections::BTreeSet;
use std::pin::Pin;

use async_trait::async_trait;
use axum::http::HeaderMap;
use futures_core::Stream;

use crate::A2AError;
use crate::jsonrpc::PROTOCOL_VERSION;
use crate::types::{
    AgentCard, CancelTaskRequest, DeleteTaskPushNotificationConfigRequest,
    GetExtendedAgentCardRequest, GetTaskPushNotificationConfigRequest, GetTaskRequest,
    ListTaskPushNotificationConfigsRequest, ListTaskPushNotificationConfigsResponse,
    ListTasksRequest, ListTasksResponse, SendMessageRequest, SendMessageResponse, StreamResponse,
    SubscribeToTaskRequest, Task, TaskPushNotificationConfig,
};

/// Server-side stream of A2A `StreamResponse` values.
pub type A2AStream = Pin<Box<dyn Stream<Item = StreamResponse> + Send + 'static>>;

/// Core server trait for implementing an A2A agent.
///
/// The default capability helpers call `get_agent_card()` on each gated request.
/// Implementations that fetch the card from storage should cache it or override
/// the relevant operation methods.
#[async_trait]
pub trait A2AHandler: Send + Sync + 'static {
    /// Return the agent card served from discovery and capability endpoints.
    async fn get_agent_card(&self) -> Result<AgentCard, A2AError>;

    /// Process a unary `SendMessage` request.
    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, A2AError>;

    /// Stream responses for a submitted message.
    ///
    /// Message-only flows should emit exactly one `StreamResponse::Message`.
    /// Task-based flows should emit the initial task first, followed by status
    /// and artifact updates until the stream closes.
    async fn send_streaming_message(
        &self,
        _request: SendMessageRequest,
    ) -> Result<A2AStream, A2AError> {
        self.require_streaming_capability("SendStreamingMessage")
            .await?;
        Err(A2AError::UnsupportedOperation(
            "SendStreamingMessage".to_owned(),
        ))
    }

    /// Fetch a task by identifier.
    async fn get_task(&self, _request: GetTaskRequest) -> Result<Task, A2AError> {
        Err(A2AError::UnsupportedOperation("GetTask".to_owned()))
    }

    /// List tasks visible to the caller.
    async fn list_tasks(&self, _request: ListTasksRequest) -> Result<ListTasksResponse, A2AError> {
        Err(A2AError::UnsupportedOperation("ListTasks".to_owned()))
    }

    /// Attempt to cancel a task.
    async fn cancel_task(&self, _request: CancelTaskRequest) -> Result<Task, A2AError> {
        Err(A2AError::UnsupportedOperation("CancelTask".to_owned()))
    }

    /// Subscribe to updates for an existing task.
    ///
    /// Implementations must emit the current `StreamResponse::Task` first before
    /// any subsequent status or artifact updates.
    async fn subscribe_to_task(
        &self,
        _request: SubscribeToTaskRequest,
    ) -> Result<A2AStream, A2AError> {
        self.require_streaming_capability("SubscribeToTask").await?;
        Err(A2AError::UnsupportedOperation("SubscribeToTask".to_owned()))
    }

    /// Create or replace a push-notification configuration.
    async fn create_task_push_notification_config(
        &self,
        _request: TaskPushNotificationConfig,
    ) -> Result<TaskPushNotificationConfig, A2AError> {
        self.require_push_notifications_capability("CreateTaskPushNotificationConfig")
            .await?;
        Err(A2AError::UnsupportedOperation(
            "CreateTaskPushNotificationConfig".to_owned(),
        ))
    }

    /// Fetch a stored push-notification configuration.
    async fn get_task_push_notification_config(
        &self,
        _request: GetTaskPushNotificationConfigRequest,
    ) -> Result<TaskPushNotificationConfig, A2AError> {
        self.require_push_notifications_capability("GetTaskPushNotificationConfig")
            .await?;
        Err(A2AError::UnsupportedOperation(
            "GetTaskPushNotificationConfig".to_owned(),
        ))
    }

    /// List stored push-notification configurations.
    async fn list_task_push_notification_configs(
        &self,
        _request: ListTaskPushNotificationConfigsRequest,
    ) -> Result<ListTaskPushNotificationConfigsResponse, A2AError> {
        self.require_push_notifications_capability("ListTaskPushNotificationConfigs")
            .await?;
        Err(A2AError::UnsupportedOperation(
            "ListTaskPushNotificationConfigs".to_owned(),
        ))
    }

    /// Delete a stored push-notification configuration.
    async fn delete_task_push_notification_config(
        &self,
        _request: DeleteTaskPushNotificationConfigRequest,
    ) -> Result<(), A2AError> {
        self.require_push_notifications_capability("DeleteTaskPushNotificationConfig")
            .await?;
        Err(A2AError::UnsupportedOperation(
            "DeleteTaskPushNotificationConfig".to_owned(),
        ))
    }

    /// Fetch the extended agent card.
    async fn get_extended_agent_card(
        &self,
        _request: GetExtendedAgentCardRequest,
    ) -> Result<AgentCard, A2AError> {
        self.require_extended_agent_card_capability().await?;
        Err(A2AError::ExtendedAgentCardNotConfigured(
            "GetExtendedAgentCard".to_owned(),
        ))
    }

    /// Enforce the A2A streaming capability gate.
    ///
    /// Do not override unless you preserve the same protocol behavior.
    async fn require_streaming_capability(&self, operation: &str) -> Result<(), A2AError> {
        let card = self.get_agent_card().await?;
        if card.capabilities.streaming == Some(true) {
            return Ok(());
        }

        Err(A2AError::UnsupportedOperation(operation.to_owned()))
    }

    /// Enforce the A2A push-notifications capability gate.
    ///
    /// Do not override unless you preserve the same protocol behavior.
    async fn require_push_notifications_capability(&self, operation: &str) -> Result<(), A2AError> {
        let card = self.get_agent_card().await?;
        if card.capabilities.push_notifications == Some(true) {
            return Ok(());
        }

        Err(A2AError::PushNotificationNotSupported(operation.to_owned()))
    }

    /// Enforce the A2A extended-agent-card capability gate.
    ///
    /// Do not override unless you preserve the same protocol behavior.
    async fn require_extended_agent_card_capability(&self) -> Result<(), A2AError> {
        let card = self.get_agent_card().await?;
        if card.capabilities.extended_agent_card == Some(true) {
            return Ok(());
        }

        Err(A2AError::ExtendedAgentCardNotConfigured(
            "GetExtendedAgentCard".to_owned(),
        ))
    }

    /// Validate `A2A-Version` and `A2A-Extensions` request headers.
    async fn validate_protocol_headers(&self, headers: &HeaderMap) -> Result<(), A2AError> {
        let card = self.get_agent_card().await?;
        validate_supported_version(&card, headers)?;
        validate_required_extensions(&card, headers)
    }

    /// Enforce that the request version is supported by the advertised interfaces.
    async fn require_supported_version(&self, headers: &HeaderMap) -> Result<(), A2AError> {
        let card = self.get_agent_card().await?;
        validate_supported_version(&card, headers)
    }

    /// Enforce that all required agent extensions are acknowledged by the caller.
    async fn require_required_extensions(&self, headers: &HeaderMap) -> Result<(), A2AError> {
        let card = self.get_agent_card().await?;
        validate_required_extensions(&card, headers)
    }
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn validate_supported_version(card: &AgentCard, headers: &HeaderMap) -> Result<(), A2AError> {
    let requested_version = match header_value(headers, "A2A-Version") {
        Some(version) if version.trim().is_empty() => "0.3".to_owned(),
        Some(version) => version,
        None => PROTOCOL_VERSION.to_owned(),
    };
    let supported_versions = card
        .supported_interfaces
        .iter()
        .map(|interface| interface.protocol_version.as_str())
        .collect::<BTreeSet<_>>();

    if supported_versions.is_empty() || supported_versions.contains(requested_version.as_str()) {
        return Ok(());
    }

    Err(A2AError::VersionNotSupported(requested_version))
}

fn validate_required_extensions(card: &AgentCard, headers: &HeaderMap) -> Result<(), A2AError> {
    let required_extensions = card
        .capabilities
        .extensions
        .iter()
        .filter(|extension| extension.required)
        .map(|extension| extension.uri.as_str())
        .collect::<BTreeSet<_>>();

    if required_extensions.is_empty() {
        return Ok(());
    }

    let announced_extensions = header_value(headers, "A2A-Extensions")
        .into_iter()
        .flat_map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>();

    let missing = required_extensions
        .into_iter()
        .filter(|extension| !announced_extensions.contains(*extension))
        .collect::<Vec<_>>();

    if missing.is_empty() {
        return Ok(());
    }

    Err(A2AError::ExtensionSupportRequired(format!(
        "missing required extensions: {}",
        missing.join(", ")
    )))
}
