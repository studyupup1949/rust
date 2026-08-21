use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use futures_core::Stream;
use futures_util::stream;
use reqwest::Url;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::A2AError;
use crate::error::ProblemDetails;
use crate::jsonrpc::{
    CONTENT_TYPE_NOT_SUPPORTED, EXTENDED_AGENT_CARD_NOT_CONFIGURED, EXTENSION_SUPPORT_REQUIRED,
    INTERNAL_ERROR, INVALID_AGENT_RESPONSE, INVALID_PARAMS, INVALID_REQUEST, JSONRPC_VERSION,
    JsonRpcError, JsonRpcId, JsonRpcRequest, JsonRpcResponse, METHOD_CANCEL_TASK,
    METHOD_CREATE_TASK_PUSH_NOTIFICATION_CONFIG, METHOD_DELETE_TASK_PUSH_NOTIFICATION_CONFIG,
    METHOD_GET_EXTENDED_AGENT_CARD, METHOD_GET_TASK, METHOD_GET_TASK_PUSH_NOTIFICATION_CONFIG,
    METHOD_LIST_TASK_PUSH_NOTIFICATION_CONFIGS, METHOD_LIST_TASKS, METHOD_NOT_FOUND,
    METHOD_SEND_MESSAGE, PARSE_ERROR, PROTOCOL_VERSION, PUSH_NOTIFICATION_NOT_SUPPORTED,
    TASK_NOT_CANCELABLE, TASK_NOT_FOUND, UNSUPPORTED_OPERATION, VERSION_NOT_SUPPORTED,
};
use crate::types::{
    AgentCard, AgentInterface, CancelTaskRequest, DeleteTaskPushNotificationConfigRequest,
    GetExtendedAgentCardRequest, GetTaskPushNotificationConfigRequest, GetTaskRequest,
    ListTaskPushNotificationConfigsRequest, ListTaskPushNotificationConfigsResponse,
    ListTasksRequest, ListTasksResponse, SendMessageRequest, SendMessageResponse, StreamResponse,
    SubscribeToTaskRequest, Task, TaskPushNotificationConfig,
};

use super::discovery::{
    AgentCardDiscovery, AgentCardDiscoveryConfig, ensure_trailing_slash, normalize_base_url,
    resolve_interface_url,
};

/// Configuration for [`A2AClient`].
#[derive(Debug, Clone)]
pub struct A2AClientConfig {
    /// Discovery cache time-to-live.
    pub discovery_ttl: Duration,
    /// Extension URIs sent as `A2A-Extensions: uri1,uri2`.
    pub extensions: Vec<String>,
}

impl Default for A2AClientConfig {
    fn default() -> Self {
        Self {
            discovery_ttl: Duration::from_secs(300),
            extensions: Vec::new(),
        }
    }
}

#[derive(Debug)]
enum TransportEndpoint {
    JsonRpc(Url),
    HttpJson(Url),
}

/// Stream of validated SSE items returned by streaming client operations.
pub type A2AClientStream =
    Pin<Box<dyn Stream<Item = Result<StreamResponse, A2AError>> + Send + 'static>>;

/// HTTP client for discovery, unary calls, and SSE streams against a remote agent.
#[derive(Debug)]
pub struct A2AClient {
    base_url: Url,
    client: reqwest::Client,
    discovery: AgentCardDiscovery,
    config: A2AClientConfig,
    request_ids: Arc<AtomicU64>,
}

impl A2AClient {
    /// Create a client with default configuration and a default `reqwest` client.
    pub fn new(base_url: &str) -> Result<Self, A2AError> {
        Self::with_config(base_url, A2AClientConfig::default())
    }

    /// Create a client with explicit SDK configuration.
    pub fn with_config(base_url: &str, config: A2AClientConfig) -> Result<Self, A2AError> {
        Self::with_http_client(base_url, reqwest::Client::new(), config)
    }

    /// Create a client with an explicit `reqwest` client and SDK configuration.
    pub fn with_http_client(
        base_url: &str,
        client: reqwest::Client,
        config: A2AClientConfig,
    ) -> Result<Self, A2AError> {
        let base_url = normalize_base_url(base_url)?;
        let discovery = AgentCardDiscovery::with_http_client(
            client.clone(),
            AgentCardDiscoveryConfig {
                ttl: config.discovery_ttl,
            },
        );

        Ok(Self {
            base_url,
            client,
            discovery,
            config,
            request_ids: Arc::new(AtomicU64::new(1)),
        })
    }

    /// Discover the remote agent card, using the discovery cache when fresh.
    pub async fn discover_agent_card(&self) -> Result<AgentCard, A2AError> {
        self.discovery.discover(self.base_url.as_ref()).await
    }

    /// Refresh the remote agent card and replace any cached copy.
    pub async fn refresh_agent_card(&self) -> Result<AgentCard, A2AError> {
        self.discovery.refresh(self.base_url.as_ref()).await
    }

    /// Invoke `SendMessage` over the server's preferred unary transport.
    pub async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, A2AError> {
        request.validate()?;

        let response: SendMessageResponse = match self.transport().await? {
            TransportEndpoint::JsonRpc(url) => {
                self.jsonrpc_call(&url, METHOD_SEND_MESSAGE, &request)
                    .await?
            }
            TransportEndpoint::HttpJson(base_url) => {
                let url = rest_url(&base_url, request.tenant.as_deref(), &["message:send"])?;
                self.read_json_response(
                    self.apply_protocol_headers(self.client.post(url))
                        .json(&request)
                        .send()
                        .await?,
                )
                .await?
            }
        };

        response.validate()?;
        Ok(response)
    }

    /// Invoke `SendStreamingMessage` over HTTP+JSON SSE.
    pub async fn send_streaming_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<A2AClientStream, A2AError> {
        request.validate()?;

        let base_url = self.http_json_transport().await?;
        let url = rest_url(&base_url, request.tenant.as_deref(), &["message:stream"])?;
        let response = self
            .apply_protocol_headers(
                self.client
                    .post(url)
                    .header(reqwest::header::ACCEPT, "text/event-stream"),
            )
            .json(&request)
            .send()
            .await?;

        self.read_sse_response(response).await
    }

    /// Fetch a task by identifier.
    pub async fn get_task(&self, request: GetTaskRequest) -> Result<Task, A2AError> {
        match self.transport().await? {
            TransportEndpoint::JsonRpc(url) => {
                self.jsonrpc_call(&url, METHOD_GET_TASK, &request).await
            }
            TransportEndpoint::HttpJson(base_url) => {
                let url = rest_url(
                    &base_url,
                    request.tenant.as_deref(),
                    &["tasks", &request.id],
                )?;
                self.read_json_response(
                    self.apply_protocol_headers(self.client.get(url))
                        .query(&GetTaskQuery {
                            history_length: request.history_length,
                        })
                        .send()
                        .await?,
                )
                .await
            }
        }
    }

    /// List tasks using the server's preferred unary transport.
    pub async fn list_tasks(
        &self,
        request: ListTasksRequest,
    ) -> Result<ListTasksResponse, A2AError> {
        request.validate()?;

        match self.transport().await? {
            TransportEndpoint::JsonRpc(url) => {
                self.jsonrpc_call(&url, METHOD_LIST_TASKS, &request).await
            }
            TransportEndpoint::HttpJson(base_url) => {
                let url = rest_url(&base_url, request.tenant.as_deref(), &["tasks"])?;
                self.read_json_response(
                    self.apply_protocol_headers(self.client.get(url))
                        .query(&ListTasksQuery {
                            context_id: request.context_id,
                            status: request.status,
                            page_size: request.page_size,
                            page_token: request.page_token,
                            history_length: request.history_length,
                            status_timestamp_after: request.status_timestamp_after,
                            include_artifacts: request.include_artifacts,
                        })
                        .send()
                        .await?,
                )
                .await
            }
        }
    }

    /// Request cancellation of a task.
    pub async fn cancel_task(&self, request: CancelTaskRequest) -> Result<Task, A2AError> {
        match self.transport().await? {
            TransportEndpoint::JsonRpc(url) => {
                self.jsonrpc_call(&url, METHOD_CANCEL_TASK, &request).await
            }
            TransportEndpoint::HttpJson(base_url) => {
                let cancel_segment = format!("{}:cancel", request.id);
                let url = rest_url(
                    &base_url,
                    request.tenant.as_deref(),
                    &["tasks", &cancel_segment],
                )?;
                let builder = self.apply_protocol_headers(self.client.post(url));
                let builder = if let Some(metadata) = &request.metadata {
                    builder.json(&CancelTaskBody {
                        metadata: Some(metadata.clone()),
                    })
                } else {
                    builder
                };
                self.read_json_response(builder.send().await?).await
            }
        }
    }

    /// Fetch the extended agent card when the remote agent advertises it.
    pub async fn get_extended_agent_card(
        &self,
        request: GetExtendedAgentCardRequest,
    ) -> Result<AgentCard, A2AError> {
        match self.transport().await? {
            TransportEndpoint::JsonRpc(url) => {
                self.jsonrpc_call(&url, METHOD_GET_EXTENDED_AGENT_CARD, &request)
                    .await
            }
            TransportEndpoint::HttpJson(base_url) => {
                let url = rest_url(&base_url, request.tenant.as_deref(), &["extendedAgentCard"])?;
                self.read_json_response(
                    self.apply_protocol_headers(self.client.get(url))
                        .send()
                        .await?,
                )
                .await
            }
        }
    }

    /// Create or replace a push-notification configuration for a task.
    pub async fn create_task_push_notification_config(
        &self,
        request: TaskPushNotificationConfig,
    ) -> Result<TaskPushNotificationConfig, A2AError> {
        match self.transport().await? {
            TransportEndpoint::JsonRpc(url) => {
                self.jsonrpc_call(&url, METHOD_CREATE_TASK_PUSH_NOTIFICATION_CONFIG, &request)
                    .await
            }
            TransportEndpoint::HttpJson(base_url) => {
                let url = rest_url(
                    &base_url,
                    request.tenant.as_deref(),
                    &["tasks", &request.task_id, "pushNotificationConfigs"],
                )?;
                self.read_json_response(
                    self.apply_protocol_headers(self.client.post(url))
                        .json(&request)
                        .send()
                        .await?,
                )
                .await
            }
        }
    }

    /// Fetch a single push-notification configuration by identifier.
    pub async fn get_task_push_notification_config(
        &self,
        request: GetTaskPushNotificationConfigRequest,
    ) -> Result<TaskPushNotificationConfig, A2AError> {
        match self.transport().await? {
            TransportEndpoint::JsonRpc(url) => {
                self.jsonrpc_call(&url, METHOD_GET_TASK_PUSH_NOTIFICATION_CONFIG, &request)
                    .await
            }
            TransportEndpoint::HttpJson(base_url) => {
                let url = rest_url(
                    &base_url,
                    request.tenant.as_deref(),
                    &[
                        "tasks",
                        &request.task_id,
                        "pushNotificationConfigs",
                        &request.id,
                    ],
                )?;
                self.read_json_response(
                    self.apply_protocol_headers(self.client.get(url))
                        .send()
                        .await?,
                )
                .await
            }
        }
    }

    /// List push-notification configurations for a task.
    pub async fn list_task_push_notification_configs(
        &self,
        request: ListTaskPushNotificationConfigsRequest,
    ) -> Result<ListTaskPushNotificationConfigsResponse, A2AError> {
        request.validate()?;

        match self.transport().await? {
            TransportEndpoint::JsonRpc(url) => {
                self.jsonrpc_call(&url, METHOD_LIST_TASK_PUSH_NOTIFICATION_CONFIGS, &request)
                    .await
            }
            TransportEndpoint::HttpJson(base_url) => {
                let url = rest_url(
                    &base_url,
                    request.tenant.as_deref(),
                    &["tasks", &request.task_id, "pushNotificationConfigs"],
                )?;
                self.read_json_response(
                    self.apply_protocol_headers(self.client.get(url))
                        .query(&ListTaskPushNotificationConfigsQuery {
                            page_size: request.page_size,
                            page_token: request.page_token,
                        })
                        .send()
                        .await?,
                )
                .await
            }
        }
    }

    /// Delete a push-notification configuration by identifier.
    pub async fn delete_task_push_notification_config(
        &self,
        request: DeleteTaskPushNotificationConfigRequest,
    ) -> Result<(), A2AError> {
        match self.transport().await? {
            TransportEndpoint::JsonRpc(url) => self
                .jsonrpc_call::<_, serde_json::Value>(
                    &url,
                    METHOD_DELETE_TASK_PUSH_NOTIFICATION_CONFIG,
                    &request,
                )
                .await
                .map(|_| ()),
            TransportEndpoint::HttpJson(base_url) => {
                let url = rest_url(
                    &base_url,
                    request.tenant.as_deref(),
                    &[
                        "tasks",
                        &request.task_id,
                        "pushNotificationConfigs",
                        &request.id,
                    ],
                )?;
                self.read_json_response::<serde_json::Value>(
                    self.apply_protocol_headers(self.client.delete(url))
                        .send()
                        .await?,
                )
                .await
                .map(|_| ())
            }
        }
    }

    /// Subscribe to task updates over HTTP+JSON SSE.
    pub async fn subscribe_to_task(
        &self,
        request: SubscribeToTaskRequest,
    ) -> Result<A2AClientStream, A2AError> {
        let base_url = self.http_json_transport().await?;
        let subscribe_segment = format!("{}:subscribe", request.id);
        let url = rest_url(
            &base_url,
            request.tenant.as_deref(),
            &["tasks", &subscribe_segment],
        )?;
        let response = self
            .apply_protocol_headers(
                self.client
                    .get(url)
                    .header(reqwest::header::ACCEPT, "text/event-stream"),
            )
            .send()
            .await?;

        self.read_sse_response(response).await
    }

    async fn transport(&self) -> Result<TransportEndpoint, A2AError> {
        let card = self.discover_agent_card().await?;
        select_transport(&self.base_url, &card.supported_interfaces)
    }

    async fn http_json_transport(&self) -> Result<Url, A2AError> {
        let card = self.discover_agent_card().await?;
        select_http_json_transport(&self.base_url, &card.supported_interfaces)
    }

    async fn jsonrpc_call<P, R>(&self, url: &Url, method: &str, params: &P) -> Result<R, A2AError>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let id = JsonRpcId::String(format!(
            "req-{}",
            self.request_ids.fetch_add(1, Ordering::Relaxed)
        ));
        let request = JsonRpcRequest {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            method: method.to_owned(),
            params: Some(serde_json::to_value(params)?),
            id: id.clone(),
        };

        let response = self
            .apply_protocol_headers(self.client.post(url.clone()))
            .json(&request)
            .send()
            .await?;
        let bytes = response.bytes().await?;
        let envelope: JsonRpcResponse = serde_json::from_slice(&bytes)
            .map_err(|error| A2AError::InvalidAgentResponse(error.to_string()))?;

        if envelope.jsonrpc != JSONRPC_VERSION {
            return Err(A2AError::InvalidAgentResponse(
                "jsonrpc must be \"2.0\"".to_owned(),
            ));
        }

        if envelope.id != id {
            return Err(A2AError::InvalidAgentResponse(
                "response id did not match request id".to_owned(),
            ));
        }

        match (envelope.result, envelope.error) {
            (Some(result), None) => serde_json::from_value(result)
                .map_err(|error| A2AError::InvalidAgentResponse(error.to_string())),
            (None, Some(error)) => Err(map_jsonrpc_error(error)),
            _ => Err(A2AError::InvalidAgentResponse(
                "response must contain exactly one of result or error".to_owned(),
            )),
        }
    }

    async fn read_json_response<T>(&self, response: reqwest::Response) -> Result<T, A2AError>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let bytes = response.bytes().await?;

        if status.is_success() {
            return serde_json::from_slice(&bytes)
                .map_err(|error| A2AError::InvalidAgentResponse(error.to_string()));
        }

        if let Ok(problem) = serde_json::from_slice::<ProblemDetails>(&bytes) {
            return Err(problem.to_a2a_error());
        }

        if let Ok(error) = serde_json::from_slice::<LegacyRestErrorEnvelope>(&bytes) {
            return Err(map_jsonrpc_error(error.error));
        }

        Err(A2AError::InvalidAgentResponse(format!(
            "unexpected HTTP status {}",
            status
        )))
    }

    async fn read_sse_response(
        &self,
        response: reqwest::Response,
    ) -> Result<A2AClientStream, A2AError> {
        let status = response.status();
        if !status.is_success() {
            let bytes = response.bytes().await?;
            if let Ok(problem) = serde_json::from_slice::<ProblemDetails>(&bytes) {
                return Err(problem.to_a2a_error());
            }

            if let Ok(error) = serde_json::from_slice::<LegacyRestErrorEnvelope>(&bytes) {
                return Err(map_jsonrpc_error(error.error));
            }

            return Err(A2AError::InvalidAgentResponse(format!(
                "unexpected HTTP status {}",
                status
            )));
        }

        Ok(Box::pin(sse_stream(response)))
    }

    fn apply_protocol_headers(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut builder = builder.header("A2A-Version", PROTOCOL_VERSION);
        if !self.config.extensions.is_empty() {
            builder = builder.header("A2A-Extensions", self.config.extensions.join(","));
        }

        builder
    }
}

fn select_transport(
    base_url: &Url,
    interfaces: &[AgentInterface],
) -> Result<TransportEndpoint, A2AError> {
    let mut advertised_versions = Vec::new();

    for interface in interfaces {
        if !interface
            .protocol_version
            .eq_ignore_ascii_case(PROTOCOL_VERSION)
        {
            advertised_versions.push(interface.protocol_version.clone());
            continue;
        }

        if interface.protocol_binding.eq_ignore_ascii_case("JSONRPC") {
            return resolve_interface_url(base_url, &interface.url).map(TransportEndpoint::JsonRpc);
        }

        if interface.protocol_binding.eq_ignore_ascii_case("HTTP+JSON") {
            return resolve_interface_url(base_url, &interface.url)
                .map(ensure_trailing_slash)
                .map(TransportEndpoint::HttpJson);
        }
    }

    if !advertised_versions.is_empty() {
        advertised_versions.sort();
        advertised_versions.dedup();
        return Err(A2AError::VersionNotSupported(format!(
            "client supports A2A-Version {}, agent advertised {}",
            PROTOCOL_VERSION,
            advertised_versions.join(", ")
        )));
    }

    Err(A2AError::InvalidAgentResponse(
        "agent card does not advertise a supported interface".to_owned(),
    ))
}

fn select_http_json_transport(
    base_url: &Url,
    interfaces: &[AgentInterface],
) -> Result<Url, A2AError> {
    let mut advertised_versions = Vec::new();

    for interface in interfaces {
        if !interface.protocol_binding.eq_ignore_ascii_case("HTTP+JSON") {
            continue;
        }

        if !interface
            .protocol_version
            .eq_ignore_ascii_case(PROTOCOL_VERSION)
        {
            advertised_versions.push(interface.protocol_version.clone());
            continue;
        }

        return resolve_interface_url(base_url, &interface.url).map(ensure_trailing_slash);
    }

    if !advertised_versions.is_empty() {
        advertised_versions.sort();
        advertised_versions.dedup();
        return Err(A2AError::VersionNotSupported(format!(
            "client supports A2A-Version {}, agent advertised HTTP+JSON {}",
            PROTOCOL_VERSION,
            advertised_versions.join(", ")
        )));
    }

    Err(A2AError::InvalidAgentResponse(
        "agent card does not advertise an HTTP+JSON interface".to_owned(),
    ))
}

fn rest_url(base_url: &Url, tenant: Option<&str>, segments: &[&str]) -> Result<Url, A2AError> {
    let mut url = ensure_trailing_slash(base_url.clone());
    {
        let mut path_segments = url
            .path_segments_mut()
            .map_err(|_| A2AError::InvalidRequest("base URL cannot be a base".to_owned()))?;
        path_segments.pop_if_empty();
        if let Some(tenant) = tenant {
            path_segments.push(tenant);
        }
        for segment in segments {
            path_segments.push(segment);
        }
    }

    Ok(url)
}

fn map_jsonrpc_error(error: JsonRpcError) -> A2AError {
    if let Some(info) = error.first_error_info() {
        return A2AError::from_error_info(error.code, &error.message, Some(&info));
    }

    let detail = error
        .data
        .as_ref()
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&error.message)
        .to_owned();

    match error.code {
        TASK_NOT_FOUND => A2AError::TaskNotFound(detail),
        TASK_NOT_CANCELABLE => A2AError::TaskNotCancelable(detail),
        PUSH_NOTIFICATION_NOT_SUPPORTED => A2AError::PushNotificationNotSupported(detail),
        UNSUPPORTED_OPERATION => A2AError::UnsupportedOperation(detail),
        CONTENT_TYPE_NOT_SUPPORTED => A2AError::ContentTypeNotSupported(detail),
        INVALID_AGENT_RESPONSE => A2AError::InvalidAgentResponse(detail),
        EXTENDED_AGENT_CARD_NOT_CONFIGURED => A2AError::ExtendedAgentCardNotConfigured(detail),
        EXTENSION_SUPPORT_REQUIRED => A2AError::ExtensionSupportRequired(detail),
        VERSION_NOT_SUPPORTED => A2AError::VersionNotSupported(detail),
        PARSE_ERROR => A2AError::ParseError(detail),
        INVALID_REQUEST => A2AError::InvalidRequest(detail),
        METHOD_NOT_FOUND => A2AError::MethodNotFound(detail),
        INVALID_PARAMS => A2AError::InvalidParams(detail),
        INTERNAL_ERROR => A2AError::Internal(detail),
        code => A2AError::Internal(format!("jsonrpc error {}: {}", code, error.message)),
    }
}

fn sse_stream(
    response: reqwest::Response,
) -> impl Stream<Item = Result<StreamResponse, A2AError>> + Send {
    stream::try_unfold(
        SseState {
            response,
            buffer: Vec::new(),
        },
        |mut state| async move {
            loop {
                if let Some(frame) = take_sse_frame(&mut state.buffer, false)?
                    && let Some(item) = parse_sse_frame(frame)?
                {
                    item.validate()?;
                    return Ok(Some((item, state)));
                }

                match state.response.chunk().await? {
                    Some(chunk) => state.buffer.extend_from_slice(&chunk),
                    None => match take_sse_frame(&mut state.buffer, true)? {
                        Some(frame) => {
                            if let Some(item) = parse_sse_frame(frame)? {
                                item.validate()?;
                                return Ok(Some((item, state)));
                            }
                        }
                        None => return Ok(None),
                    },
                }
            }
        },
    )
}

#[derive(Debug)]
struct SseState {
    response: reqwest::Response,
    buffer: Vec<u8>,
}

fn take_sse_frame(buffer: &mut Vec<u8>, eof: bool) -> Result<Option<Vec<u8>>, A2AError> {
    if let Some((index, delimiter_len)) = sse_frame_boundary(buffer) {
        let frame = buffer[..index].to_vec();
        buffer.drain(..index + delimiter_len);
        return Ok(Some(frame));
    }

    if eof && !buffer.is_empty() {
        return Ok(Some(std::mem::take(buffer)));
    }

    Ok(None)
}

fn sse_frame_boundary(buffer: &[u8]) -> Option<(usize, usize)> {
    for index in 0..buffer.len().saturating_sub(1) {
        if buffer[index] == b'\n' && buffer[index + 1] == b'\n' {
            return Some((index, 2));
        }

        if index + 3 < buffer.len() && &buffer[index..index + 4] == b"\r\n\r\n" {
            return Some((index, 4));
        }
    }

    None
}

fn parse_sse_frame(frame: Vec<u8>) -> Result<Option<StreamResponse>, A2AError> {
    let text = String::from_utf8(frame)
        .map_err(|error| A2AError::InvalidAgentResponse(error.to_string()))?;
    let mut data_lines = Vec::new();

    for line in text.lines() {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.is_empty() || line.starts_with(':') {
            continue;
        }

        if let Some(data) = line.strip_prefix("data:") {
            let data = data.strip_prefix(' ').unwrap_or(data);
            data_lines.push(data.to_owned());
        }
    }

    if data_lines.is_empty() {
        return Ok(None);
    }

    serde_json::from_str::<StreamResponse>(&data_lines.join("\n"))
        .map(Some)
        .map_err(|error| A2AError::InvalidAgentResponse(error.to_string()))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GetTaskQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    history_length: Option<i32>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ListTasksQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    context_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<crate::types::TaskState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    history_length: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_timestamp_after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_artifacts: Option<bool>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ListTaskPushNotificationConfigsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    page_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_token: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CancelTaskBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<crate::types::JsonObject>,
}

#[derive(serde::Deserialize)]
struct LegacyRestErrorEnvelope {
    error: JsonRpcError,
}

#[cfg(test)]
mod tests {
    use super::map_jsonrpc_error;
    use crate::A2AError;
    use crate::jsonrpc::JsonRpcError;
    use crate::jsonrpc::{
        CONTENT_TYPE_NOT_SUPPORTED, EXTENDED_AGENT_CARD_NOT_CONFIGURED, EXTENSION_SUPPORT_REQUIRED,
        INTERNAL_ERROR, INVALID_AGENT_RESPONSE, INVALID_PARAMS, INVALID_REQUEST, METHOD_NOT_FOUND,
        PARSE_ERROR, PUSH_NOTIFICATION_NOT_SUPPORTED, TASK_NOT_CANCELABLE, TASK_NOT_FOUND,
        UNSUPPORTED_OPERATION, VERSION_NOT_SUPPORTED,
    };

    #[test]
    fn map_jsonrpc_error_covers_all_protocol_codes() {
        let cases = [
            (TASK_NOT_FOUND, "task missing"),
            (TASK_NOT_CANCELABLE, "task busy"),
            (PUSH_NOTIFICATION_NOT_SUPPORTED, "push unsupported"),
            (UNSUPPORTED_OPERATION, "operation unsupported"),
            (CONTENT_TYPE_NOT_SUPPORTED, "content type unsupported"),
            (INVALID_AGENT_RESPONSE, "invalid agent response"),
            (
                EXTENDED_AGENT_CARD_NOT_CONFIGURED,
                "extended agent card missing",
            ),
            (EXTENSION_SUPPORT_REQUIRED, "extension required"),
            (VERSION_NOT_SUPPORTED, "version unsupported"),
            (PARSE_ERROR, "parse error"),
            (INVALID_REQUEST, "invalid request"),
            (METHOD_NOT_FOUND, "missing method"),
            (INVALID_PARAMS, "invalid params"),
            (INTERNAL_ERROR, "internal error"),
        ];

        for (code, detail) in cases {
            let mapped = map_jsonrpc_error(JsonRpcError {
                code,
                message: format!("message for {code}"),
                data: Some(serde_json::Value::String(detail.to_owned())),
            });

            match code {
                TASK_NOT_FOUND => {
                    assert!(matches!(mapped, A2AError::TaskNotFound(value) if value == detail));
                }
                TASK_NOT_CANCELABLE => {
                    assert!(
                        matches!(mapped, A2AError::TaskNotCancelable(value) if value == detail)
                    );
                }
                PUSH_NOTIFICATION_NOT_SUPPORTED => {
                    assert!(
                        matches!(mapped, A2AError::PushNotificationNotSupported(value) if value == detail)
                    );
                }
                UNSUPPORTED_OPERATION => {
                    assert!(
                        matches!(mapped, A2AError::UnsupportedOperation(value) if value == detail)
                    );
                }
                CONTENT_TYPE_NOT_SUPPORTED => {
                    assert!(
                        matches!(mapped, A2AError::ContentTypeNotSupported(value) if value == detail)
                    );
                }
                INVALID_AGENT_RESPONSE => {
                    assert!(
                        matches!(mapped, A2AError::InvalidAgentResponse(value) if value == detail)
                    );
                }
                EXTENDED_AGENT_CARD_NOT_CONFIGURED => {
                    assert!(
                        matches!(mapped, A2AError::ExtendedAgentCardNotConfigured(value) if value == detail)
                    );
                }
                EXTENSION_SUPPORT_REQUIRED => {
                    assert!(
                        matches!(mapped, A2AError::ExtensionSupportRequired(value) if value == detail)
                    );
                }
                VERSION_NOT_SUPPORTED => {
                    assert!(
                        matches!(mapped, A2AError::VersionNotSupported(value) if value == detail)
                    );
                }
                PARSE_ERROR => {
                    assert!(matches!(mapped, A2AError::ParseError(value) if value == detail));
                }
                INVALID_REQUEST => {
                    assert!(matches!(mapped, A2AError::InvalidRequest(value) if value == detail));
                }
                METHOD_NOT_FOUND => {
                    assert!(matches!(mapped, A2AError::MethodNotFound(value) if value == detail));
                }
                INVALID_PARAMS => {
                    assert!(matches!(mapped, A2AError::InvalidParams(value) if value == detail));
                }
                INTERNAL_ERROR => {
                    assert!(matches!(mapped, A2AError::Internal(value) if value == detail));
                }
                _ => unreachable!("all cases should be covered"),
            }
        }
    }

    #[test]
    fn map_jsonrpc_error_prefers_structured_error_info() {
        let mapped = map_jsonrpc_error(JsonRpcError {
            code: TASK_NOT_FOUND,
            message: "fallback message".to_owned(),
            data: Some(serde_json::json!({
                "@type": "type.googleapis.com/google.rpc.ErrorInfo",
                "reason": "TASK_NOT_FOUND",
                "domain": "a2a-protocol.org",
                "metadata": {
                    "taskId": "task-123"
                }
            })),
        });

        assert!(matches!(mapped, A2AError::TaskNotFound(value) if value == "task-123"));
    }
}
