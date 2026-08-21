#![cfg(feature = "server")]

use async_trait::async_trait;
use futures_util::stream;

use a2a_rust::A2AError;
use a2a_rust::server::{A2AHandler, A2AStream, router};
use a2a_rust::types::{
    AgentCapabilities, AgentCard, AgentInterface, GetTaskRequest, ListTasksRequest,
    ListTasksResponse, Message, Part, Role, SendMessageRequest, SendMessageResponse,
    StreamResponse, Task, TaskState, TaskStatus, TaskStatusUpdateEvent,
};
use axum::body::{Body, to_bytes};
use http::{Request, StatusCode};
use tower::util::ServiceExt;

#[derive(Clone)]
struct TestHandler;

#[derive(Clone)]
struct StreamingHandler;

#[derive(Clone)]
struct TenantEchoHandler;

#[derive(Clone)]
struct RequiredExtensionHandler;

fn tenant_metadata(tenant: Option<String>) -> Option<serde_json::Map<String, serde_json::Value>> {
    tenant.map(|tenant| {
        let mut metadata = serde_json::Map::new();
        metadata.insert("tenant".to_owned(), serde_json::Value::String(tenant));
        metadata
    })
}

#[async_trait]
impl A2AHandler for TestHandler {
    async fn get_agent_card(&self) -> Result<AgentCard, A2AError> {
        Ok(AgentCard {
            name: "Test Agent".to_owned(),
            description: "Integration test agent".to_owned(),
            supported_interfaces: vec![AgentInterface {
                url: "https://example.com/rpc".to_owned(),
                protocol_binding: "JSONRPC".to_owned(),
                tenant: None,
                protocol_version: "1.0".to_owned(),
            }],
            provider: None,
            version: "0.1.0".to_owned(),
            documentation_url: None,
            capabilities: AgentCapabilities::default(),
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
            message_id: "msg-2".to_owned(),
            context_id: request.message.context_id.clone(),
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
            metadata: None,
            extensions: Vec::new(),
            reference_task_ids: Vec::new(),
        }))
    }

    async fn get_task(&self, request: GetTaskRequest) -> Result<Task, A2AError> {
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
            metadata: None,
        })
    }

    async fn list_tasks(&self, _request: ListTasksRequest) -> Result<ListTasksResponse, A2AError> {
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
                metadata: None,
            }],
            next_page_token: String::new(),
            page_size: 1,
            total_size: 1,
        })
    }
}

#[async_trait]
impl A2AHandler for StreamingHandler {
    async fn get_agent_card(&self) -> Result<AgentCard, A2AError> {
        Ok(AgentCard {
            name: "Streaming Agent".to_owned(),
            description: "Streaming integration test agent".to_owned(),
            supported_interfaces: vec![AgentInterface {
                url: "https://example.com/rpc".to_owned(),
                protocol_binding: "JSONRPC".to_owned(),
                tenant: None,
                protocol_version: "1.0".to_owned(),
            }],
            provider: None,
            version: "0.1.0".to_owned(),
            documentation_url: None,
            capabilities: AgentCapabilities {
                streaming: Some(true),
                push_notifications: Some(false),
                extensions: Vec::new(),
                extended_agent_card: Some(false),
            },
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
        TestHandler.send_message(request).await
    }

    async fn send_streaming_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<A2AStream, A2AError> {
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
                metadata: None,
                extensions: Vec::new(),
                reference_task_ids: Vec::new(),
            },
        )])))
    }

    async fn subscribe_to_task(
        &self,
        request: a2a_rust::types::SubscribeToTaskRequest,
    ) -> Result<A2AStream, A2AError> {
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
                metadata: None,
            }),
            StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
                task_id: request.id,
                context_id: "ctx-1".to_owned(),
                status: TaskStatus {
                    state: TaskState::Completed,
                    message: None,
                    timestamp: Some("2026-03-12T12:01:00Z".to_owned()),
                },
                metadata: None,
            }),
        ])))
    }
}

#[async_trait]
impl A2AHandler for TenantEchoHandler {
    async fn get_agent_card(&self) -> Result<AgentCard, A2AError> {
        Ok(AgentCard {
            name: "Tenant Agent".to_owned(),
            description: "Tenant integration test agent".to_owned(),
            supported_interfaces: vec![AgentInterface {
                url: "https://example.com/rpc".to_owned(),
                protocol_binding: "JSONRPC".to_owned(),
                tenant: None,
                protocol_version: "1.0".to_owned(),
            }],
            provider: None,
            version: "0.1.0".to_owned(),
            documentation_url: None,
            capabilities: AgentCapabilities {
                streaming: Some(true),
                push_notifications: Some(true),
                extensions: Vec::new(),
                extended_agent_card: Some(true),
            },
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
            message_id: "tenant-msg-1".to_owned(),
            context_id: request.message.context_id,
            task_id: None,
            role: Role::Agent,
            parts: vec![Part {
                text: Some("tenant-pong".to_owned()),
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
        Ok(Box::pin(stream::iter(vec![StreamResponse::Message(
            Message {
                message_id: "tenant-stream-1".to_owned(),
                context_id: request.message.context_id,
                task_id: None,
                role: Role::Agent,
                parts: vec![Part {
                    text: Some("tenant-stream".to_owned()),
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
        Ok(Task {
            id: request.id,
            context_id: Some("ctx-tenant".to_owned()),
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
                id: "tenant-task-1".to_owned(),
                context_id: Some("ctx-tenant".to_owned()),
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

    async fn subscribe_to_task(
        &self,
        request: a2a_rust::types::SubscribeToTaskRequest,
    ) -> Result<A2AStream, A2AError> {
        Ok(Box::pin(stream::iter(vec![StreamResponse::Task(Task {
            id: request.id,
            context_id: Some("ctx-tenant".to_owned()),
            status: TaskStatus {
                state: TaskState::Working,
                message: None,
                timestamp: Some("2026-03-12T12:00:00Z".to_owned()),
            },
            artifacts: Vec::new(),
            history: Vec::new(),
            metadata: tenant_metadata(request.tenant),
        })])))
    }

    async fn list_task_push_notification_configs(
        &self,
        request: a2a_rust::types::ListTaskPushNotificationConfigsRequest,
    ) -> Result<a2a_rust::types::ListTaskPushNotificationConfigsResponse, A2AError> {
        Ok(a2a_rust::types::ListTaskPushNotificationConfigsResponse {
            configs: vec![a2a_rust::types::TaskPushNotificationConfig {
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

    async fn get_task_push_notification_config(
        &self,
        request: a2a_rust::types::GetTaskPushNotificationConfigRequest,
    ) -> Result<a2a_rust::types::TaskPushNotificationConfig, A2AError> {
        Ok(a2a_rust::types::TaskPushNotificationConfig {
            id: request.id,
            task_id: request.task_id,
            tenant: request.tenant,
            url: "https://example.com/push".to_owned(),
            token: None,
            authentication: None,
        })
    }

    async fn delete_task_push_notification_config(
        &self,
        _request: a2a_rust::types::DeleteTaskPushNotificationConfigRequest,
    ) -> Result<(), A2AError> {
        Ok(())
    }
}

#[async_trait]
impl A2AHandler for RequiredExtensionHandler {
    async fn get_agent_card(&self) -> Result<AgentCard, A2AError> {
        Ok(AgentCard {
            name: "Extension Agent".to_owned(),
            description: "Requires an extension".to_owned(),
            supported_interfaces: vec![AgentInterface {
                url: "https://example.com/rpc".to_owned(),
                protocol_binding: "JSONRPC".to_owned(),
                tenant: None,
                protocol_version: "1.0".to_owned(),
            }],
            provider: None,
            version: "0.1.0".to_owned(),
            documentation_url: None,
            capabilities: AgentCapabilities {
                streaming: Some(false),
                push_notifications: Some(false),
                extensions: vec![a2a_rust::types::AgentExtension {
                    uri: "https://example.com/extensions/required".to_owned(),
                    description: "Required extension".to_owned(),
                    required: true,
                    params: None,
                }],
                extended_agent_card: Some(false),
            },
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
        TestHandler.send_message(request).await
    }
}

#[tokio::test]
async fn well_known_endpoint_serves_agent_card() {
    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .uri("/.well-known/agent-card.json")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn rest_send_message_returns_protocol_shape() {
    let body = serde_json::json!({
        "message": {
            "messageId": "msg-1",
            "role": "ROLE_USER",
            "parts": [{"text": "ping"}]
        }
    });

    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/message:send")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["message"]["role"], "ROLE_AGENT");
    assert_eq!(json["message"]["parts"][0]["text"], "pong");
}

#[tokio::test]
async fn tenant_message_route_uses_path_tenant() {
    let body = serde_json::json!({
        "tenant": "wrong-tenant",
        "message": {
            "messageId": "msg-1",
            "role": "ROLE_USER",
            "parts": [{"text": "ping"}]
        }
    });

    let response = router(TenantEchoHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tenant-a/message:send")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["message"]["metadata"]["tenant"], "tenant-a");
}

#[tokio::test]
async fn rest_version_mismatch_returns_problem_details() {
    let body = serde_json::json!({
        "message": {
            "messageId": "msg-1",
            "role": "ROLE_USER",
            "parts": [{"text": "ping"}]
        }
    });

    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/message:send")
                .header("content-type", "application/json")
                .header("A2A-Version", "9.9")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.headers()["content-type"],
        "application/problem+json"
    );

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(
        json["type"],
        "https://a2a-protocol.org/errors/version-not-supported"
    );
    assert_eq!(json["reason"], "VERSION_NOT_SUPPORTED");
    assert_eq!(json["status"], 400);
}

#[tokio::test]
async fn rest_missing_required_extension_returns_problem_details() {
    let body = serde_json::json!({
        "message": {
            "messageId": "msg-1",
            "role": "ROLE_USER",
            "parts": [{"text": "ping"}]
        }
    });

    let response = router(RequiredExtensionHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/message:send")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.headers()["content-type"],
        "application/problem+json"
    );

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(
        json["type"],
        "https://a2a-protocol.org/errors/extension-support-required"
    );
    assert_eq!(json["reason"], "EXTENSION_SUPPORT_REQUIRED");
}

#[tokio::test]
async fn jsonrpc_send_message_dispatches_pascal_case_method() {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "req-1",
        "method": "SendMessage",
        "params": {
            "message": {
                "messageId": "msg-1",
                "role": "ROLE_USER",
                "parts": [{"text": "ping"}]
            }
        }
    });

    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["id"], "req-1");
    assert_eq!(json["result"]["message"]["parts"][0]["text"], "pong");
}

#[tokio::test]
async fn jsonrpc_missing_required_extension_returns_structured_error_info() {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "req-ext",
        "method": "SendMessage",
        "params": {
            "message": {
                "messageId": "msg-1",
                "role": "ROLE_USER",
                "parts": [{"text": "ping"}]
            }
        }
    });

    let response = router(RequiredExtensionHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["error"]["code"], -32008);
    assert_eq!(
        json["error"]["data"]["reason"],
        "EXTENSION_SUPPORT_REQUIRED"
    );
    assert_eq!(json["error"]["data"]["domain"], "a2a-protocol.org");
}

#[tokio::test]
async fn jsonrpc_unknown_method_returns_jsonrpc_error() {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "UnknownMethod",
        "params": {}
    });

    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["error"]["code"], -32601);
    assert_eq!(json["id"], 7);
}

#[tokio::test]
async fn jsonrpc_parse_error_returns_http_200() {
    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from("{"))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["error"]["code"], -32700);
    assert_eq!(json["id"], serde_json::Value::Null);
}

#[tokio::test]
async fn jsonrpc_invalid_version_returns_http_200() {
    let body = serde_json::json!({
        "jsonrpc": "1.0",
        "id": "req-invalid",
        "method": "ListTasks",
        "params": {}
    });

    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["error"]["code"], -32600);
    assert_eq!(json["id"], "req-invalid");
}

#[tokio::test]
async fn jsonrpc_list_tasks_allows_missing_params_object() {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "req-2",
        "method": "ListTasks"
    });

    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["id"], "req-2");
    assert_eq!(json["result"]["tasks"][0]["id"], "task-1");
}

#[tokio::test]
async fn tenant_list_tasks_route_uses_path_tenant() {
    let response = router(TenantEchoHandler)
        .oneshot(
            Request::builder()
                .uri("/tenant-b/tasks?tenant=wrong-tenant")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["tasks"][0]["metadata"]["tenant"], "tenant-b");
}

#[tokio::test]
async fn tenant_get_task_route_uses_path_tenant() {
    let response = router(TenantEchoHandler)
        .oneshot(
            Request::builder()
                .uri("/tenant-b/tasks/task-1?tenant=wrong-tenant")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["metadata"]["tenant"], "tenant-b");
}

#[tokio::test]
async fn non_tenant_list_tasks_rejects_query_tenant() {
    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .uri("/tasks?tenant=tenant-a")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["reason"], "INVALID_REQUEST");
    assert_eq!(json["status"], 400);
}

#[tokio::test]
async fn jsonrpc_get_extended_agent_card_allows_missing_params_object() {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "req-3",
        "method": "GetExtendedAgentCard"
    });

    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["id"], "req-3");
    assert_eq!(json["error"]["code"], -32007);
}

#[tokio::test]
async fn rest_get_extended_agent_card_returns_default_error() {
    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .uri("/extendedAgentCard")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["reason"], "EXTENDED_AGENT_CARD_NOT_CONFIGURED");
    assert_eq!(json["status"], 400);
}

#[tokio::test]
async fn get_cancel_path_returns_not_found() {
    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .uri("/tasks/task-1:cancel")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["reason"], "METHOD_NOT_FOUND");
    assert_eq!(json["status"], 404);
}

#[tokio::test]
async fn streaming_route_returns_unsupported_when_capability_is_disabled() {
    let body = serde_json::json!({
        "message": {
            "messageId": "msg-1",
            "role": "ROLE_USER",
            "parts": [{"text": "ping"}]
        }
    });

    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/message:stream")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["reason"], "UNSUPPORTED_OPERATION");
    assert_eq!(json["status"], 400);
}

#[tokio::test]
async fn subscribe_route_returns_unsupported_when_capability_is_disabled() {
    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .uri("/tasks/task-1:subscribe")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["reason"], "UNSUPPORTED_OPERATION");
    assert_eq!(json["status"], 400);
}

#[tokio::test]
async fn push_config_route_returns_not_supported_when_capability_is_disabled() {
    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .uri("/tasks/task-1/pushNotificationConfigs")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["reason"], "PUSH_NOTIFICATION_NOT_SUPPORTED");
    assert_eq!(json["status"], 400);
}

#[tokio::test]
async fn jsonrpc_push_config_returns_not_supported_when_capability_is_disabled() {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "req-4",
        "method": "ListTaskPushNotificationConfigs",
        "params": {
            "taskId": "task-1"
        }
    });

    let response = router(TestHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["error"]["code"], -32003);
}

#[tokio::test]
async fn jsonrpc_delete_push_config_returns_empty_object_result() {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "req-5",
        "method": "DeleteTaskPushNotificationConfig",
        "params": {
            "taskId": "task-1",
            "id": "cfg-1"
        }
    });

    let response = router(TenantEchoHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["id"], "req-5");
    assert_eq!(json["result"], serde_json::json!({}));
}

#[tokio::test]
async fn streaming_route_uses_sse_framing() {
    let body = serde_json::json!({
        "message": {
            "messageId": "msg-1",
            "contextId": "ctx-1",
            "role": "ROLE_USER",
            "parts": [{"text": "ping"}]
        }
    });

    let response = router(StreamingHandler)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/message:stream")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/event-stream");

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let body = String::from_utf8(bytes.to_vec()).expect("body should be utf8");
    assert!(body.starts_with("data: "));
    assert!(body.contains("\"message\":{\"messageId\":\"msg-stream-1\""));
    assert!(body.ends_with("\n\n"));
}

#[tokio::test]
async fn subscribe_route_streams_current_task_first() {
    let response = router(StreamingHandler)
        .oneshot(
            Request::builder()
                .uri("/tasks/task-1:subscribe")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/event-stream");

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let body = String::from_utf8(bytes.to_vec()).expect("body should be utf8");
    let frames = body
        .trim_end()
        .split("\n\n")
        .map(str::to_owned)
        .collect::<Vec<_>>();

    assert_eq!(frames.len(), 2);
    assert!(frames[0].contains("\"task\":{\"id\":\"task-1\""));
    assert!(frames[1].contains("\"statusUpdate\":{\"taskId\":\"task-1\""));
}

#[tokio::test]
async fn tenant_subscribe_route_uses_path_tenant() {
    let response = router(TenantEchoHandler)
        .oneshot(
            Request::builder()
                .uri("/tenant-c/tasks/task-1:subscribe?tenant=wrong-tenant")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let body = String::from_utf8(bytes.to_vec()).expect("body should be utf8");
    assert!(body.contains("\"metadata\":{\"tenant\":\"tenant-c\"}"));
}

#[tokio::test]
async fn non_tenant_subscribe_rejects_query_tenant() {
    let response = router(StreamingHandler)
        .oneshot(
            Request::builder()
                .uri("/tasks/task-1:subscribe?tenant=tenant-a")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["reason"], "INVALID_REQUEST");
    assert_eq!(json["status"], 400);
}

#[tokio::test]
async fn tenant_push_config_route_uses_path_tenant() {
    let response = router(TenantEchoHandler)
        .oneshot(
            Request::builder()
                .uri("/tenant-d/tasks/task-1/pushNotificationConfigs?tenant=wrong-tenant")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["configs"][0]["tenant"], "tenant-d");
}

#[tokio::test]
async fn tenant_get_push_config_route_uses_path_tenant() {
    let response = router(TenantEchoHandler)
        .oneshot(
            Request::builder()
                .uri("/tenant-d/tasks/task-1/pushNotificationConfigs/cfg-1?tenant=wrong-tenant")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("response should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json: serde_json::Value = serde_json::from_slice(&bytes).expect("body should deserialize");
    assert_eq!(json["tenant"], "tenant-d");
}
