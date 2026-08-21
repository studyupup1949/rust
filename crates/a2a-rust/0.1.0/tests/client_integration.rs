#![cfg(all(feature = "client", feature = "server"))]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{StreamExt, stream};
use tokio::task::JoinHandle;

use a2a_rust::A2AError;
use a2a_rust::client::{A2AClient, A2AClientConfig, AgentCardDiscovery, AgentCardDiscoveryConfig};
use a2a_rust::server::{A2AHandler, A2AStream, router};
use a2a_rust::types::{
    AgentCapabilities, AgentCard, AgentInterface, CancelTaskRequest,
    DeleteTaskPushNotificationConfigRequest, GetExtendedAgentCardRequest,
    GetTaskPushNotificationConfigRequest, GetTaskRequest, ListTaskPushNotificationConfigsRequest,
    ListTaskPushNotificationConfigsResponse, ListTasksRequest, ListTasksResponse, Message, Part,
    Role, SendMessageRequest, SendMessageResponse, StreamResponse, SubscribeToTaskRequest, Task,
    TaskPushNotificationConfig, TaskState, TaskStatus, TaskStatusUpdateEvent,
};

#[derive(Clone)]
struct ClientTestHandler {
    card_hits: Arc<AtomicUsize>,
    interfaces: Vec<AgentInterface>,
    capabilities: AgentCapabilities,
}

#[async_trait]
impl A2AHandler for ClientTestHandler {
    async fn get_agent_card(&self) -> Result<AgentCard, A2AError> {
        self.card_hits.fetch_add(1, Ordering::SeqCst);

        Ok(AgentCard {
            name: "Client Test Agent".to_owned(),
            description: "Client integration test agent".to_owned(),
            supported_interfaces: self.interfaces.clone(),
            provider: None,
            version: "0.1.0".to_owned(),
            documentation_url: None,
            capabilities: self.capabilities.clone(),
            security_schemes: Default::default(),
            security_requirements: Vec::new(),
            default_input_modes: vec!["text/plain".to_owned()],
            default_output_modes: vec!["text/plain".to_owned()],
            skills: Vec::new(),
            signatures: Vec::new(),
            icon_url: None,
        })
    }

    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, A2AError> {
        Ok(SendMessageResponse::Message(Message {
            message_id: "msg-client-1".to_owned(),
            context_id: request.message.context_id,
            task_id: None,
            role: Role::Agent,
            parts: vec![Part {
                text: Some("pong".to_owned()),
                raw: None,
                url: None,
                data: None,
                metadata: None,
                filename: None,
                media_type: None,
            }],
            metadata: tenant_metadata(request.tenant),
            extensions: Vec::new(),
            reference_task_ids: Vec::new(),
        }))
    }

    async fn send_streaming_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<A2AStream, A2AError> {
        self.require_streaming_capability("SendStreamingMessage")
            .await?;

        Ok(Box::pin(stream::iter(vec![StreamResponse::Message(
            Message {
                message_id: "msg-stream-1".to_owned(),
                context_id: request.message.context_id,
                task_id: None,
                role: Role::Agent,
                parts: vec![Part {
                    text: Some("stream-pong".to_owned()),
                    raw: None,
                    url: None,
                    data: None,
                    metadata: None,
                    filename: None,
                    media_type: None,
                }],
                metadata: tenant_metadata(request.tenant),
                extensions: Vec::new(),
                reference_task_ids: Vec::new(),
            },
        )])))
    }

    async fn get_task(&self, request: GetTaskRequest) -> Result<Task, A2AError> {
        if request.id == "missing" {
            return Err(A2AError::TaskNotFound(request.id));
        }

        Ok(Task {
            id: request.id,
            context_id: Some("ctx-1".to_owned()),
            status: TaskStatus {
                state: TaskState::Working,
                message: None,
                timestamp: Some("2026-03-12T12:00:00Z".to_owned()),
            },
            artifacts: Vec::new(),
            history: Vec::new(),
            metadata: tenant_metadata(request.tenant),
        })
    }

    async fn list_tasks(&self, request: ListTasksRequest) -> Result<ListTasksResponse, A2AError> {
        Ok(ListTasksResponse {
            tasks: vec![Task {
                id: "task-1".to_owned(),
                context_id: Some("ctx-1".to_owned()),
                status: TaskStatus {
                    state: TaskState::Submitted,
                    message: None,
                    timestamp: Some("2026-03-12T12:00:00Z".to_owned()),
                },
                artifacts: Vec::new(),
                history: Vec::new(),
                metadata: tenant_metadata(request.tenant),
            }],
            next_page_token: String::new(),
            page_size: 1,
            total_size: 1,
        })
    }

    async fn cancel_task(&self, request: CancelTaskRequest) -> Result<Task, A2AError> {
        Ok(Task {
            id: request.id,
            context_id: Some("ctx-1".to_owned()),
            status: TaskStatus {
                state: TaskState::Canceled,
                message: None,
                timestamp: Some("2026-03-12T12:00:00Z".to_owned()),
            },
            artifacts: Vec::new(),
            history: Vec::new(),
            metadata: tenant_metadata(request.tenant),
        })
    }

    async fn subscribe_to_task(
        &self,
        request: SubscribeToTaskRequest,
    ) -> Result<A2AStream, A2AError> {
        self.require_streaming_capability("SubscribeToTask").await?;

        Ok(Box::pin(stream::iter(vec![
            StreamResponse::Task(Task {
                id: request.id.clone(),
                context_id: Some("ctx-1".to_owned()),
                status: TaskStatus {
                    state: TaskState::Working,
                    message: None,
                    timestamp: Some("2026-03-12T12:00:00Z".to_owned()),
                },
                artifacts: Vec::new(),
                history: Vec::new(),
                metadata: tenant_metadata(request.tenant.clone()),
            }),
            StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
                task_id: request.id,
                context_id: "ctx-1".to_owned(),
                status: TaskStatus {
                    state: TaskState::Completed,
                    message: None,
                    timestamp: Some("2026-03-12T12:01:00Z".to_owned()),
                },
                metadata: tenant_metadata(request.tenant),
            }),
        ])))
    }

    async fn create_task_push_notification_config(
        &self,
        request: TaskPushNotificationConfig,
    ) -> Result<TaskPushNotificationConfig, A2AError> {
        Ok(request)
    }

    async fn get_task_push_notification_config(
        &self,
        request: GetTaskPushNotificationConfigRequest,
    ) -> Result<TaskPushNotificationConfig, A2AError> {
        Ok(TaskPushNotificationConfig {
            id: request.id.clone(),
            task_id: request.task_id,
            tenant: request.tenant,
            url: "https://example.com/push".to_owned(),
            token: Some("secret".to_owned()),
            authentication: None,
        })
    }

    async fn list_task_push_notification_configs(
        &self,
        request: ListTaskPushNotificationConfigsRequest,
    ) -> Result<ListTaskPushNotificationConfigsResponse, A2AError> {
        Ok(ListTaskPushNotificationConfigsResponse {
            configs: vec![TaskPushNotificationConfig {
                id: "cfg-1".to_owned(),
                task_id: request.task_id,
                tenant: request.tenant,
                url: "https://example.com/push".to_owned(),
                token: None,
                authentication: None,
            }],
            next_page_token: String::new(),
        })
    }

    async fn delete_task_push_notification_config(
        &self,
        _request: DeleteTaskPushNotificationConfigRequest,
    ) -> Result<(), A2AError> {
        Ok(())
    }

    async fn get_extended_agent_card(
        &self,
        _request: GetExtendedAgentCardRequest,
    ) -> Result<AgentCard, A2AError> {
        self.get_agent_card().await
    }
}

struct TestServer {
    base_url: String,
    handle: JoinHandle<()>,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

#[tokio::test]
async fn discovery_caches_agent_card_until_refresh() {
    let card_hits = Arc::new(AtomicUsize::new(0));
    let server = spawn_server(ClientTestHandler {
        card_hits: Arc::clone(&card_hits),
        interfaces: vec![interface("/rpc", "JSONRPC")],
        capabilities: capabilities(false, true),
    })
    .await;
    let discovery = AgentCardDiscovery::with_config(AgentCardDiscoveryConfig {
        ttl: Duration::from_secs(60),
    });

    let first = discovery
        .discover(&server.base_url)
        .await
        .expect("discovery should succeed");
    let second = discovery
        .discover(&server.base_url)
        .await
        .expect("cached discovery should succeed");
    let refreshed = discovery
        .refresh(&server.base_url)
        .await
        .expect("refresh should succeed");

    assert_eq!(first.name, "Client Test Agent");
    assert_eq!(second.name, "Client Test Agent");
    assert_eq!(refreshed.name, "Client Test Agent");
    assert_eq!(card_hits.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn client_respects_server_interface_order() {
    let server = spawn_server(ClientTestHandler {
        card_hits: Arc::new(AtomicUsize::new(0)),
        interfaces: vec![
            interface("/", "HTTP+JSON"),
            interface("/bad-rpc", "JSONRPC"),
        ],
        capabilities: capabilities(false, true),
    })
    .await;
    let client = A2AClient::new(&server.base_url).expect("client should build");

    let response = client
        .send_message(user_message_request(None))
        .await
        .expect("first supported interface should succeed");

    match response {
        SendMessageResponse::Message(message) => {
            assert_eq!(message.message_id, "msg-client-1");
            assert_eq!(message.parts[0].text.as_deref(), Some("pong"));
        }
        SendMessageResponse::Task(_) => panic!("expected message response"),
    }
}

#[tokio::test]
async fn client_falls_back_to_http_json_and_uses_tenant_paths() {
    let server = spawn_server(ClientTestHandler {
        card_hits: Arc::new(AtomicUsize::new(0)),
        interfaces: vec![interface("/", "HTTP+JSON")],
        capabilities: capabilities(false, true),
    })
    .await;
    let client = A2AClient::new(&server.base_url).expect("client should build");

    let response = client
        .send_message(user_message_request(Some("tenant-a".to_owned())))
        .await
        .expect("rest fallback should succeed");
    let task = client
        .get_task(GetTaskRequest {
            id: "task-1".to_owned(),
            history_length: None,
            tenant: Some("tenant-a".to_owned()),
        })
        .await
        .expect("tenant task fetch should succeed");

    match response {
        SendMessageResponse::Message(message) => {
            assert_eq!(
                message
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("tenant"))
                    .and_then(|v| v.as_str()),
                Some("tenant-a")
            );
        }
        SendMessageResponse::Task(_) => panic!("expected message response"),
    }
    assert_eq!(
        task.metadata
            .as_ref()
            .and_then(|m| m.get("tenant"))
            .and_then(|v| v.as_str()),
        Some("tenant-a")
    );
}

#[tokio::test]
async fn client_maps_jsonrpc_a2a_errors() {
    let server = spawn_server(ClientTestHandler {
        card_hits: Arc::new(AtomicUsize::new(0)),
        interfaces: vec![interface("/rpc", "JSONRPC")],
        capabilities: capabilities(false, true),
    })
    .await;
    let client = A2AClient::new(&server.base_url).expect("client should build");

    let error = client
        .get_task(GetTaskRequest {
            id: "missing".to_owned(),
            history_length: None,
            tenant: None,
        })
        .await
        .expect_err("missing task should fail");

    match error {
        A2AError::TaskNotFound(task_id) => assert_eq!(task_id, "missing"),
        other => panic!("expected task not found error, got {other}"),
    }
}

#[tokio::test]
async fn client_supports_unary_rest_and_jsonrpc_operations() {
    let server = spawn_server(ClientTestHandler {
        card_hits: Arc::new(AtomicUsize::new(0)),
        interfaces: vec![interface("/rpc", "JSONRPC")],
        capabilities: capabilities(false, true),
    })
    .await;
    let client = A2AClient::with_config(
        &server.base_url,
        A2AClientConfig {
            discovery_ttl: Duration::from_secs(60),
            extensions: vec!["streaming".to_owned()],
        },
    )
    .expect("client should build");

    let list = client
        .list_tasks(ListTasksRequest::default())
        .await
        .expect("list should succeed");
    let canceled = client
        .cancel_task(CancelTaskRequest {
            id: "task-1".to_owned(),
            tenant: None,
            metadata: None,
        })
        .await
        .expect("cancel should succeed");
    let card = client
        .get_extended_agent_card(GetExtendedAgentCardRequest { tenant: None })
        .await
        .expect("extended card should succeed");

    assert_eq!(list.tasks.len(), 1);
    assert_eq!(canceled.status.state, TaskState::Canceled);
    assert_eq!(card.name, "Client Test Agent");
}

#[tokio::test]
async fn client_streams_send_message_over_http_json_sse() {
    let server = spawn_server(ClientTestHandler {
        card_hits: Arc::new(AtomicUsize::new(0)),
        interfaces: vec![interface("/", "HTTP+JSON")],
        capabilities: capabilities(true, true),
    })
    .await;
    let client = A2AClient::new(&server.base_url).expect("client should build");

    let mut stream = client
        .send_streaming_message(user_message_request(Some("tenant-a".to_owned())))
        .await
        .expect("streaming request should succeed");
    let first = stream
        .next()
        .await
        .expect("stream should yield an event")
        .expect("event should deserialize");

    match first {
        StreamResponse::Message(message) => {
            assert_eq!(message.parts[0].text.as_deref(), Some("stream-pong"));
            assert_eq!(
                message
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("tenant"))
                    .and_then(|v| v.as_str()),
                Some("tenant-a")
            );
        }
        other => panic!("expected message stream response, got {other:?}"),
    }
}

#[tokio::test]
async fn client_streams_subscribe_to_task_over_http_json_sse() {
    let server = spawn_server(ClientTestHandler {
        card_hits: Arc::new(AtomicUsize::new(0)),
        interfaces: vec![interface("/", "HTTP+JSON")],
        capabilities: capabilities(true, true),
    })
    .await;
    let client = A2AClient::new(&server.base_url).expect("client should build");

    let stream = client
        .subscribe_to_task(SubscribeToTaskRequest {
            id: "task-1".to_owned(),
            tenant: Some("tenant-a".to_owned()),
        })
        .await
        .expect("subscribe request should succeed");
    let events = stream
        .collect::<Vec<Result<StreamResponse, A2AError>>>()
        .await;

    assert_eq!(events.len(), 2);
    match &events[0] {
        Ok(StreamResponse::Task(task)) => {
            assert_eq!(task.id, "task-1");
            assert_eq!(
                task.metadata
                    .as_ref()
                    .and_then(|m| m.get("tenant"))
                    .and_then(|v| v.as_str()),
                Some("tenant-a")
            );
        }
        other => panic!("expected task as first event, got {other:?}"),
    }
    match &events[1] {
        Ok(StreamResponse::StatusUpdate(update)) => {
            assert_eq!(update.task_id, "task-1");
            assert_eq!(update.status.state, TaskState::Completed);
        }
        other => panic!("expected status update as second event, got {other:?}"),
    }
}

#[tokio::test]
async fn client_supports_push_notification_config_operations() {
    let server = spawn_server(ClientTestHandler {
        card_hits: Arc::new(AtomicUsize::new(0)),
        interfaces: vec![interface("/rpc", "JSONRPC")],
        capabilities: capabilities(false, true),
    })
    .await;
    let client = A2AClient::new(&server.base_url).expect("client should build");

    let created = client
        .create_task_push_notification_config(TaskPushNotificationConfig {
            task_id: "task-1".to_owned(),
            id: "cfg-1".to_owned(),
            tenant: Some("tenant-a".to_owned()),
            url: "https://example.com/push".to_owned(),
            token: Some("secret".to_owned()),
            authentication: None,
        })
        .await
        .expect("create should succeed");
    let fetched = client
        .get_task_push_notification_config(GetTaskPushNotificationConfigRequest {
            id: "cfg-1".to_owned(),
            task_id: "task-1".to_owned(),
            tenant: Some("tenant-a".to_owned()),
        })
        .await
        .expect("get should succeed");
    let listed = client
        .list_task_push_notification_configs(ListTaskPushNotificationConfigsRequest {
            task_id: "task-1".to_owned(),
            page_size: Some(10),
            page_token: None,
            tenant: Some("tenant-a".to_owned()),
        })
        .await
        .expect("list should succeed");
    client
        .delete_task_push_notification_config(DeleteTaskPushNotificationConfigRequest {
            id: "cfg-1".to_owned(),
            task_id: "task-1".to_owned(),
            tenant: Some("tenant-a".to_owned()),
        })
        .await
        .expect("delete should succeed");

    assert_eq!(created.id, "cfg-1");
    assert_eq!(created.tenant.as_deref(), Some("tenant-a"));
    assert_eq!(fetched.url, "https://example.com/push");
    assert_eq!(listed.configs.len(), 1);
}

fn interface(url: &str, protocol_binding: &str) -> AgentInterface {
    AgentInterface {
        url: url.to_owned(),
        protocol_binding: protocol_binding.to_owned(),
        tenant: None,
        protocol_version: "1.0".to_owned(),
    }
}

fn capabilities(streaming: bool, push_notifications: bool) -> AgentCapabilities {
    AgentCapabilities {
        streaming: Some(streaming),
        push_notifications: Some(push_notifications),
        extensions: Vec::new(),
        extended_agent_card: Some(true),
    }
}

fn tenant_metadata(tenant: Option<String>) -> Option<serde_json::Map<String, serde_json::Value>> {
    tenant.map(|tenant| {
        let mut metadata = serde_json::Map::new();
        metadata.insert("tenant".to_owned(), serde_json::Value::String(tenant));
        metadata
    })
}

fn user_message_request(tenant: Option<String>) -> SendMessageRequest {
    SendMessageRequest {
        message: Message {
            message_id: "msg-1".to_owned(),
            context_id: Some("ctx-1".to_owned()),
            task_id: None,
            role: Role::User,
            parts: vec![Part {
                text: Some("ping".to_owned()),
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
        },
        configuration: None,
        metadata: None,
        tenant,
    }
}

async fn spawn_server<H>(handler: H) -> TestServer
where
    H: A2AHandler,
{
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let address = listener.local_addr().expect("listener should have address");
    let handle = tokio::spawn(async move {
        axum::serve(listener, router(handler))
            .await
            .expect("server should run");
    });

    TestServer {
        base_url: format!("http://{}", address),
        handle,
    }
}
