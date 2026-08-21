#![cfg(feature = "client")]

use std::collections::BTreeMap;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::json;
use tokio::time::sleep;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

use a2a_rust::A2AError;
use a2a_rust::client::{A2AClient, AgentCardDiscovery, AgentCardDiscoveryConfig};
use a2a_rust::types::{
    AgentCapabilities, AgentCard, AgentInterface, Message, Part, Role, SendMessageRequest,
    SendMessageResponse, StreamResponse, SubscribeToTaskRequest, Task, TaskState, TaskStatus,
    TaskStatusUpdateEvent,
};

#[tokio::test]
async fn discovery_refetches_after_ttl_expiry() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/.well-known/agent-card.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(agent_card(
            vec![interface("/rpc", "JSONRPC")],
            capabilities(false, false),
        )))
        .mount(&server)
        .await;

    let discovery = AgentCardDiscovery::with_config(AgentCardDiscoveryConfig {
        ttl: Duration::from_millis(20),
    });

    discovery
        .discover(&server.uri())
        .await
        .expect("first discovery should succeed");
    discovery
        .discover(&server.uri())
        .await
        .expect("cached discovery should succeed");
    sleep(Duration::from_millis(30)).await;
    discovery
        .discover(&server.uri())
        .await
        .expect("discovery after ttl should refetch");

    let requests = server
        .received_requests()
        .await
        .expect("received requests should be available");
    let discovery_hits = requests
        .iter()
        .filter(|request| request.url.path() == "/.well-known/agent-card.json")
        .count();

    assert_eq!(discovery_hits, 2);
}

#[tokio::test]
async fn client_uses_first_supported_interface_from_agent_card() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/.well-known/agent-card.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(agent_card(
            vec![interface("/", "HTTP+JSON"), interface("/rpc", "JSONRPC")],
            capabilities(false, false),
        )))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/message:send"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(send_message_response("rest-first", None)),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let client = A2AClient::new(&server.uri()).expect("client should build");
    let response = client
        .send_message(user_message_request(None))
        .await
        .expect("rest-first transport should succeed");

    match response {
        SendMessageResponse::Message(message) => {
            assert_eq!(message.parts[0].text.as_deref(), Some("rest-first"));
        }
        SendMessageResponse::Task(_) => panic!("expected message response"),
    }

    let requests = server
        .received_requests()
        .await
        .expect("received requests should be available");
    assert!(
        requests
            .iter()
            .any(|request| request.url.path() == "/message:send"),
        "rest endpoint should have been called first"
    );
    assert!(
        !requests.iter().any(|request| request.url.path() == "/rpc"),
        "jsonrpc endpoint should not be used when HTTP+JSON is listed first"
    );
}

#[tokio::test]
async fn client_rejects_jsonrpc_response_with_mismatched_id() {
    let server = MockServer::start().await;
    mount_jsonrpc_discovery(&server).await;
    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(JsonRpcEnvelopeResponder {
            jsonrpc: "2.0",
            mismatched_id: true,
        })
        .mount(&server)
        .await;

    let client = A2AClient::new(&server.uri()).expect("client should build");
    let error = client
        .send_message(user_message_request(None))
        .await
        .expect_err("mismatched response ids should fail");

    match error {
        A2AError::InvalidAgentResponse(detail) => {
            assert!(detail.contains("response id did not match request id"));
        }
        other => panic!("expected invalid agent response, got {other}"),
    }
}

#[tokio::test]
async fn client_rejects_jsonrpc_response_with_invalid_version() {
    let server = MockServer::start().await;
    mount_jsonrpc_discovery(&server).await;
    Mock::given(method("POST"))
        .and(path("/rpc"))
        .respond_with(JsonRpcEnvelopeResponder {
            jsonrpc: "1.0",
            mismatched_id: false,
        })
        .mount(&server)
        .await;

    let client = A2AClient::new(&server.uri()).expect("client should build");
    let error = client
        .send_message(user_message_request(None))
        .await
        .expect_err("invalid jsonrpc versions should fail");

    match error {
        A2AError::InvalidAgentResponse(detail) => {
            assert!(detail.contains("jsonrpc must be \"2.0\""));
        }
        other => panic!("expected invalid agent response, got {other}"),
    }
}

#[tokio::test]
async fn client_parses_sse_streams_with_lf_frame_delimiters() {
    let server = MockServer::start().await;
    mount_http_json_discovery(&server, true).await;
    Mock::given(method("POST"))
        .and(path("/message:stream"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            sse_body(
                "\n\n",
                &[StreamResponse::Message(agent_message_response(
                    "lf-stream",
                    None,
                ))],
            ),
            "text/event-stream",
        ))
        .mount(&server)
        .await;

    let client = A2AClient::new(&server.uri()).expect("client should build");
    let items = client
        .send_streaming_message(user_message_request(None))
        .await
        .expect("stream should succeed")
        .collect::<Vec<_>>()
        .await;

    assert_eq!(items.len(), 1);
    match &items[0] {
        Ok(StreamResponse::Message(message)) => {
            assert_eq!(message.parts[0].text.as_deref(), Some("lf-stream"));
        }
        other => panic!("expected message stream response, got {other:?}"),
    }
}

#[tokio::test]
async fn client_parses_sse_streams_with_crlf_frame_delimiters() {
    let server = MockServer::start().await;
    mount_http_json_discovery(&server, true).await;
    Mock::given(method("GET"))
        .and(path("/tasks/task-1:subscribe"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            sse_body(
                "\r\n\r\n",
                &[
                    StreamResponse::Task(task("task-1")),
                    StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
                        task_id: "task-1".to_owned(),
                        context_id: "ctx-1".to_owned(),
                        status: TaskStatus {
                            state: TaskState::Completed,
                            message: None,
                            timestamp: Some("2026-03-13T08:31:00Z".to_owned()),
                        },
                        metadata: None,
                    }),
                ],
            ),
            "text/event-stream",
        ))
        .mount(&server)
        .await;

    let client = A2AClient::new(&server.uri()).expect("client should build");
    let items = client
        .subscribe_to_task(SubscribeToTaskRequest {
            id: "task-1".to_owned(),
            tenant: None,
        })
        .await
        .expect("subscribe stream should succeed")
        .collect::<Vec<_>>()
        .await;

    assert_eq!(items.len(), 2);
    match &items[0] {
        Ok(StreamResponse::Task(task)) => assert_eq!(task.id, "task-1"),
        other => panic!("expected task event first, got {other:?}"),
    }
    match &items[1] {
        Ok(StreamResponse::StatusUpdate(update)) => {
            assert_eq!(update.status.state, TaskState::Completed);
        }
        other => panic!("expected status update second, got {other:?}"),
    }
}

#[tokio::test]
async fn client_maps_http_problem_details_to_a2a_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/.well-known/agent-card.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(agent_card(
            vec![interface("/", "HTTP+JSON")],
            capabilities(false, false),
        )))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/message:send"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "type": "https://a2a-protocol.org/errors/extension-support-required",
            "title": "Extension support required",
            "status": 400,
            "detail": "missing required extensions: https://example.com/ext/required",
            "reason": "EXTENSION_SUPPORT_REQUIRED",
            "domain": "a2a-protocol.org",
        })))
        .mount(&server)
        .await;

    let client = A2AClient::new(&server.uri()).expect("client should build");
    let error = client
        .send_message(user_message_request(None))
        .await
        .expect_err("problem details should map to an A2A error");

    match error {
        A2AError::ExtensionSupportRequired(detail) => {
            assert!(detail.contains("missing required extensions"));
        }
        other => panic!("expected extension-support-required, got {other}"),
    }
}

#[tokio::test]
async fn client_rejects_agent_cards_without_a_supported_protocol_version() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/.well-known/agent-card.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(agent_card(
            vec![interface_with_version("/rpc", "JSONRPC", "0.9")],
            capabilities(false, false),
        )))
        .mount(&server)
        .await;

    let client = A2AClient::new(&server.uri()).expect("client should build");
    let error = client
        .send_message(user_message_request(None))
        .await
        .expect_err("unsupported interface versions should fail");

    match error {
        A2AError::VersionNotSupported(detail) => {
            assert!(detail.contains("1.0"));
            assert!(detail.contains("0.9"));
        }
        other => panic!("expected version-not-supported, got {other}"),
    }
}

async fn mount_jsonrpc_discovery(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/.well-known/agent-card.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(agent_card(
            vec![interface("/rpc", "JSONRPC")],
            capabilities(false, false),
        )))
        .mount(server)
        .await;
}

async fn mount_http_json_discovery(server: &MockServer, streaming: bool) {
    Mock::given(method("GET"))
        .and(path("/.well-known/agent-card.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(agent_card(
            vec![interface("/", "HTTP+JSON")],
            capabilities(streaming, false),
        )))
        .mount(server)
        .await;
}

fn agent_card(interfaces: Vec<AgentInterface>, capabilities: AgentCapabilities) -> AgentCard {
    AgentCard {
        name: "Wiremock Agent".to_owned(),
        description: "Client wiremock test agent".to_owned(),
        supported_interfaces: interfaces,
        provider: None,
        version: "0.1.0".to_owned(),
        documentation_url: None,
        capabilities,
        security_schemes: BTreeMap::new(),
        security_requirements: Vec::new(),
        default_input_modes: vec!["text/plain".to_owned()],
        default_output_modes: vec!["text/plain".to_owned()],
        skills: Vec::new(),
        signatures: Vec::new(),
        icon_url: None,
    }
}

fn interface(url: &str, protocol_binding: &str) -> AgentInterface {
    interface_with_version(url, protocol_binding, "1.0")
}

fn interface_with_version(
    url: &str,
    protocol_binding: &str,
    protocol_version: &str,
) -> AgentInterface {
    AgentInterface {
        url: url.to_owned(),
        protocol_binding: protocol_binding.to_owned(),
        tenant: None,
        protocol_version: protocol_version.to_owned(),
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

fn agent_message_response(text: &str, tenant: Option<String>) -> Message {
    Message {
        message_id: "msg-agent-1".to_owned(),
        context_id: Some("ctx-1".to_owned()),
        task_id: None,
        role: Role::Agent,
        parts: vec![Part {
            text: Some(text.to_owned()),
            raw: None,
            url: None,
            data: None,
            metadata: None,
            filename: None,
            media_type: None,
        }],
        metadata: tenant.map(|tenant| {
            let mut metadata = serde_json::Map::new();
            metadata.insert("tenant".to_owned(), serde_json::Value::String(tenant));
            metadata
        }),
        extensions: Vec::new(),
        reference_task_ids: Vec::new(),
    }
}

fn send_message_response(text: &str, tenant: Option<String>) -> SendMessageResponse {
    SendMessageResponse::Message(agent_message_response(text, tenant))
}

fn task(id: &str) -> Task {
    Task {
        id: id.to_owned(),
        context_id: Some("ctx-1".to_owned()),
        status: TaskStatus {
            state: TaskState::Working,
            message: None,
            timestamp: Some("2026-03-13T08:30:00Z".to_owned()),
        },
        artifacts: Vec::new(),
        history: Vec::new(),
        metadata: None,
    }
}

fn sse_body(delimiter: &str, items: &[StreamResponse]) -> String {
    items
        .iter()
        .map(|item| {
            let payload = serde_json::to_string(item).expect("stream item should serialize");
            format!("data: {payload}{delimiter}")
        })
        .collect::<String>()
}

struct JsonRpcEnvelopeResponder {
    jsonrpc: &'static str,
    mismatched_id: bool,
}

impl Respond for JsonRpcEnvelopeResponder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let request_body: serde_json::Value =
            serde_json::from_slice(&request.body).expect("request body should be valid json");
        let id = if self.mismatched_id {
            json!("wrong-id")
        } else {
            request_body
                .get("id")
                .cloned()
                .unwrap_or(serde_json::Value::Null)
        };

        ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": self.jsonrpc,
            "result": serde_json::to_value(send_message_response("rpc", None))
                .expect("response should serialize"),
            "id": id,
        }))
    }
}
