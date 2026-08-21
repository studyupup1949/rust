//! Integration tests for the acompose MCP server using an in-memory transport.
//!
//! These tests exercise the public MCP surface (`send_message`) without
//! starting a real HTTP server or spawning real `acp_command` processes.

use std::path::PathBuf;
use std::sync::Arc;
// common is declared in main.rs
use crate::common::mocks::MockSessionFactory;
use acompose::compositor::Compositor;
use acompose::compositor::state::{MemoryStateStore, State};
use acompose::mcp_server::ComposeMcpServer;
use rmcp::model::CallToolRequestParams;
use rmcp::service::ServiceExt;

fn extract_text(result: &rmcp::model::CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|content| content.raw.as_text().map(|text| text.text.clone()))
        .collect()
}

async fn new_test_compositor() -> anyhow::Result<Arc<Compositor>> {
    let store = Arc::new(MemoryStateStore::new());
    let state = State::default();
    store.save(&state).await?;
    let factory = Arc::new(MockSessionFactory::new());

    let compositor = Arc::new(Compositor::new(
        factory,
        Some(Arc::clone(&store) as Arc<dyn acompose::compositor::state::StateStore>),
        None,
    )?);

    Ok(compositor)
}

async fn add_inmemory_session(
    comp: &Compositor,
    name: &str,
    _session_id: &str,
) -> anyhow::Result<()> {
    comp.create_session(name, PathBuf::from("/tmp"), "", None, vec![], vec![])
        .await
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!("failed to create test session: {}", e))
}

#[tokio::test]
async fn send_message_through_mcp() -> anyhow::Result<()> {
    let (server_transport, client_transport) = tokio::io::duplex(4096);

    let compositor = new_test_compositor().await?;
    add_inmemory_session(&compositor, "agent-2", "sid-agent-2").await?;

    let server = ComposeMcpServer::new(compositor);
    let server_handle = tokio::spawn(async move {
        let service = server.serve(server_transport).await?;
        service.waiting().await?;
        anyhow::Ok(())
    });

    let mut client = ().serve(client_transport).await?;

    let send_result = client
        .call_tool(
            CallToolRequestParams::new("send_message").with_arguments(
                serde_json::json!({
                    "target": "agent-2",
                    "content": "привет",
                    "need_result": false
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;
    let send_text = extract_text(&send_result);
    assert!(
        send_text.contains("Message queued"),
        "unexpected send result: {}",
        send_text
    );

    tokio::time::sleep(std::time::Duration::from_millis(700)).await;

    client.close().await?;
    server_handle.await??;

    Ok(())
}
