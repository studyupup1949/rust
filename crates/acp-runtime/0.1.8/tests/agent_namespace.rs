use acp_runtime::agent::AcpAgent;
use acp_runtime::messages::{DeliveryMode, DeliveryState, MessageClass};
use acp_runtime::options::AcpAgentOptions;
use httpmock::prelude::*;
use serde_json::{Map, Value};

#[test]
fn send_with_namespace_publishes_namespaced_envelope() {
    let server = MockServer::start();
    let inbox_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/acp/inbox")
            .body_includes("\"namespace\":\"namespace.demo\"");
        then.status(200)
            .header("Content-Type", "application/json")
            .body("{\"ok\":true}");
    });

    let temp = tempfile::tempdir().expect("tempdir");
    let mut sender = AcpAgent::load_or_create(
        "agent:sender.namespace@localhost:9930",
        Some(AcpAgentOptions {
            storage_dir: temp.path().join("sender"),
            endpoint: Some("http://localhost:9930/acp/inbox".to_string()),
            allow_insecure_http: true,
            discovery_scheme: "http".to_string(),
            ..AcpAgentOptions::default()
        }),
    )
    .expect("sender agent");
    let recipient = AcpAgent::load_or_create(
        "agent:recipient.namespace@localhost:9931",
        Some(AcpAgentOptions {
            storage_dir: temp.path().join("recipient"),
            endpoint: Some(server.url("/acp/inbox")),
            allow_insecure_http: true,
            discovery_scheme: "http".to_string(),
            ..AcpAgentOptions::default()
        }),
    )
    .expect("recipient agent");
    sender
        .register_identity_document(recipient.identity_document.clone())
        .expect("register recipient identity");

    let mut payload = Map::new();
    payload.insert(
        "type".to_string(),
        Value::String("namespace-test".to_string()),
    );

    let result = sender
        .send_with_namespace(
            vec![recipient.agent_id().to_string()],
            payload,
            Some("ctx:namespace".to_string()),
            MessageClass::Send,
            300,
            Some("namespace.demo".to_string()),
            None,
            None,
            Some(DeliveryMode::Direct),
        )
        .expect("send should succeed");

    inbox_mock.assert();
    assert_eq!(
        Some(&DeliveryState::Delivered),
        result.outcomes.first().map(|outcome| &outcome.state)
    );
}
