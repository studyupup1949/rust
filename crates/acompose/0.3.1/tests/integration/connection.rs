// common is declared in main.rs
use crate::common::mocks::{MockSessionFactory, run_client};
use acompose::compositor::Compositor;
use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::schema::v1::{
    ContentBlock, ContentChunk, SessionNotification, SessionUpdate, TextContent,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

async fn recv_notification(
    rx: &mut tokio::sync::broadcast::Receiver<SessionNotification>,
) -> SessionNotification {
    timeout(Duration::from_secs(2), rx.recv())
        .await
        .unwrap()
        .unwrap()
}

fn test_compositor(factory: MockSessionFactory) -> Arc<Compositor> {
    Arc::new(
        Compositor::new(
            Arc::new(factory) as Arc<dyn acompose::agent::session_factory::SessionFactory>,
            None,
            None,
        )
        .unwrap(),
    )
}

#[tokio::test]
async fn test_initialize() {
    let compositor = test_compositor(MockSessionFactory::new());
    let client = run_client(compositor).await;
    let response = client.initialize().await.unwrap();
    assert_eq!(response.protocol_version, ProtocolVersion::V1);
}

#[tokio::test]
async fn test_not_initialized_error() {
    let compositor = test_compositor(MockSessionFactory::new());
    let client = run_client(compositor).await;
    let result = client.list_sessions().await;
    assert!(result.is_err(), "non-initialize request should fail");
}

#[tokio::test]
async fn test_list_sessions() {
    let factory = MockSessionFactory::new();
    let compositor = test_compositor(factory.clone());
    compositor
        .create_session("alpha", PathBuf::from("/tmp/a"), "", vec![], vec![])
        .await
        .unwrap();
    compositor
        .create_session("beta", PathBuf::from("/tmp/b"), "", vec![], vec![])
        .await
        .unwrap();

    let client = run_client(compositor).await;
    client.initialize().await.unwrap();
    let response = client.list_sessions().await.unwrap();
    assert_eq!(response.sessions.len(), 2);
}

#[tokio::test]
async fn test_new_session() {
    let factory = MockSessionFactory::new();
    let compositor = test_compositor(factory.clone());
    let client = run_client(compositor.clone()).await;
    client.initialize().await.unwrap();

    let response = client
        .new_session("test-s", &PathBuf::from("/tmp"), "", vec![], vec![])
        .await
        .unwrap();
    assert!(!response.session_id.to_string().is_empty());
    assert!(compositor.get_session("test-s").await.is_some());
    assert!(factory.agent("test-s").is_some());
}

#[tokio::test]
async fn test_load_session_replays_history() {
    let factory = MockSessionFactory::new();
    let compositor = test_compositor(factory.clone());
    compositor
        .create_session("s", PathBuf::from("/tmp"), "", vec![], vec![])
        .await
        .unwrap();

    let agent = factory.agent("s").unwrap();
    agent.set_response_text("agent reply");

    // First client loads the session and sends a prompt. The agent replies,
    // and the session handle accumulates both the user prompt and the reply.
    let first = run_client(compositor.clone()).await;
    first.initialize().await.unwrap();
    let mut first_rx = first.subscribe();
    first
        .load_session("s", &PathBuf::from("/tmp"), vec![])
        .await
        .unwrap();
    first.prompt("s", "user").await.unwrap();

    let n = recv_notification(&mut first_rx).await;
    assert!(matches!(n.update, SessionUpdate::AgentMessageChunk(_)));

    // A second client loading the same session should receive the replayed
    // history: the user prompt followed by the agent reply.
    let second = run_client(compositor).await;
    second.initialize().await.unwrap();
    let mut second_rx = second.subscribe();
    second
        .load_session("s", &PathBuf::from("/tmp"), vec![])
        .await
        .unwrap();

    let n1 = recv_notification(&mut second_rx).await;
    assert!(matches!(n1.update, SessionUpdate::UserMessageChunk(_)));
    let n2 = recv_notification(&mut second_rx).await;
    assert!(matches!(n2.update, SessionUpdate::AgentMessageChunk(_)));
}

#[tokio::test]
async fn test_prompt_suppressed_for_sender() {
    let factory = MockSessionFactory::new();
    let compositor = test_compositor(factory.clone());
    compositor
        .create_session("s", PathBuf::from("/tmp"), "", vec![], vec![])
        .await
        .unwrap();

    let sender = run_client(compositor.clone()).await;
    let observer = run_client(compositor.clone()).await;
    sender.initialize().await.unwrap();
    observer.initialize().await.unwrap();

    let mut sender_rx = sender.subscribe();
    let mut observer_rx = observer.subscribe();

    sender
        .load_session("s", &PathBuf::from("/tmp"), vec![])
        .await
        .unwrap();
    observer
        .load_session("s", &PathBuf::from("/tmp"), vec![])
        .await
        .unwrap();

    sender.prompt("s", "hello").await.unwrap();

    let n = recv_notification(&mut observer_rx).await;
    assert!(matches!(n.update, SessionUpdate::UserMessageChunk(_)));

    let result = timeout(Duration::from_millis(200), sender_rx.recv()).await;
    assert!(
        result.is_err(),
        "sender should not receive synthetic user_message_chunk"
    );
}

#[tokio::test]
async fn test_prompt() {
    let factory = MockSessionFactory::new();
    let compositor = test_compositor(factory.clone());
    compositor
        .create_session("s", PathBuf::from("/tmp"), "", vec![], vec![])
        .await
        .unwrap();

    let client = run_client(compositor).await;
    client.initialize().await.unwrap();
    client.prompt("s", "hello").await.unwrap();

    let agent = factory.agent("s").unwrap();
    let prompts = agent.recorded_prompts();
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0], "hello");
}

#[tokio::test]
async fn test_load_session_not_found() {
    let compositor = test_compositor(MockSessionFactory::new());
    let client = run_client(compositor).await;
    client.initialize().await.unwrap();
    let result = client
        .load_session("missing", &PathBuf::from("/tmp"), vec![])
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_forward_notifications() {
    let factory = MockSessionFactory::new();
    let compositor = test_compositor(factory.clone());
    compositor
        .create_session("s", PathBuf::from("/tmp"), "", vec![], vec![])
        .await
        .unwrap();

    let agent = factory.agent("s").unwrap();

    let client = run_client(compositor).await;
    client.initialize().await.unwrap();
    let mut rx = client.subscribe();
    client
        .load_session("s", &PathBuf::from("/tmp"), vec![])
        .await
        .unwrap();

    let live = SessionNotification::new(
        agent.session_id(),
        SessionUpdate::UserMessageChunk(ContentChunk::new(ContentBlock::Text(TextContent::new(
            "live",
        )))),
    );
    agent.send_live_notification(live);

    let n = recv_notification(&mut rx).await;
    assert!(matches!(n.update, SessionUpdate::UserMessageChunk(_)));
}

#[tokio::test]
async fn test_cancel_is_forwarded_to_agent() {
    let factory = MockSessionFactory::with_response_delay(Duration::from_mins(1));
    let compositor = test_compositor(factory.clone());
    compositor
        .create_session("s", PathBuf::from("/tmp"), "", vec![], vec![])
        .await
        .unwrap();

    let agent = factory.agent("s").unwrap();

    let sender = run_client(compositor.clone()).await;
    let canceler = run_client(compositor.clone()).await;
    sender.initialize().await.unwrap();
    canceler.initialize().await.unwrap();
    sender
        .load_session("s", &PathBuf::from("/tmp"), vec![])
        .await
        .unwrap();
    canceler
        .load_session("s", &PathBuf::from("/tmp"), vec![])
        .await
        .unwrap();

    let sender_for_prompt = sender.clone();
    let prompt_handle = tokio::spawn(async move {
        let _ = sender_for_prompt.prompt("s", "long task").await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    canceler.cancel("s").await.unwrap();

    tokio::time::sleep(Duration::from_millis(500)).await;
    let cancels = agent.recorded_cancels();
    assert_eq!(
        cancels.len(),
        1,
        "agent should receive exactly one cancel notification while prompt is in flight"
    );
    assert_eq!(cancels[0], agent.session_id());

    prompt_handle.abort();
}

#[tokio::test]
async fn test_cancel_on_same_connection_is_forwarded_before_prompt_finishes() {
    let factory = MockSessionFactory::with_response_delay(Duration::from_secs(2));
    let compositor = test_compositor(factory.clone());
    compositor
        .create_session("s", PathBuf::from("/tmp"), "", vec![], vec![])
        .await
        .unwrap();

    let agent = factory.agent("s").unwrap();

    let client = run_client(compositor).await;
    client.initialize().await.unwrap();
    client
        .load_session("s", &PathBuf::from("/tmp"), vec![])
        .await
        .unwrap();

    let client_for_prompt = client.clone();
    let prompt_handle = tokio::spawn(async move {
        let _ = client_for_prompt.prompt("s", "long task").await;
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    client.cancel("s").await.unwrap();

    tokio::time::sleep(Duration::from_millis(500)).await;
    let cancels = agent.recorded_cancels();
    assert_eq!(
        cancels.len(),
        1,
        "cancel should be forwarded on the same connection even while prompt is in flight"
    );
    assert_eq!(cancels[0], agent.session_id());

    prompt_handle.abort();
}

#[tokio::test]
async fn test_disconnect_stops_forwarding() {
    let factory = MockSessionFactory::new();
    let compositor = test_compositor(factory.clone());
    compositor
        .create_session("s", PathBuf::from("/tmp"), "", vec![], vec![])
        .await
        .unwrap();

    let agent = factory.agent("s").unwrap();

    let client = run_client(compositor).await;
    client.initialize().await.unwrap();
    let mut rx = client.subscribe();
    client
        .load_session("s", &PathBuf::from("/tmp"), vec![])
        .await
        .unwrap();

    let live = SessionNotification::new(
        agent.session_id(),
        SessionUpdate::UserMessageChunk(ContentChunk::new(ContentBlock::Text(TextContent::new(
            "live",
        )))),
    );
    agent.send_live_notification(live);
    let n = recv_notification(&mut rx).await;
    assert!(matches!(n.update, SessionUpdate::UserMessageChunk(_)));

    client.shutdown().await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let after = SessionNotification::new(
        agent.session_id(),
        SessionUpdate::UserMessageChunk(ContentChunk::new(ContentBlock::Text(TextContent::new(
            "after",
        )))),
    );
    agent.send_live_notification(after);
    let result = timeout(Duration::from_millis(200), rx.recv()).await;
    assert!(
        result.is_err() || matches!(result, Ok(Err(_))),
        "no further notifications should be forwarded after disconnect"
    );
}

#[tokio::test]
async fn test_two_clients_receive_same_live_update() {
    let factory = MockSessionFactory::new();
    let compositor = test_compositor(factory.clone());
    compositor
        .create_session("s", PathBuf::from("/tmp"), "", vec![], vec![])
        .await
        .unwrap();

    let agent = factory.agent("s").unwrap();

    let first = run_client(compositor.clone()).await;
    let second = run_client(compositor.clone()).await;
    first.initialize().await.unwrap();
    second.initialize().await.unwrap();

    let mut first_rx = first.subscribe();
    let mut second_rx = second.subscribe();
    first
        .load_session("s", &PathBuf::from("/tmp"), vec![])
        .await
        .unwrap();
    second
        .load_session("s", &PathBuf::from("/tmp"), vec![])
        .await
        .unwrap();

    let live = SessionNotification::new(
        agent.session_id(),
        SessionUpdate::AgentMessageChunk(ContentChunk::new(ContentBlock::Text(TextContent::new(
            "shared",
        )))),
    );
    agent.send_live_notification(live);

    let n1 = recv_notification(&mut first_rx).await;
    let n2 = recv_notification(&mut second_rx).await;
    assert!(matches!(n1.update, SessionUpdate::AgentMessageChunk(_)));
    assert!(matches!(n2.update, SessionUpdate::AgentMessageChunk(_)));
}

#[tokio::test]
async fn test_prompt_from_one_client_visible_to_other_as_user_message_chunk() {
    let factory = MockSessionFactory::new();
    let compositor = test_compositor(factory.clone());
    compositor
        .create_session("s", PathBuf::from("/tmp"), "", vec![], vec![])
        .await
        .unwrap();

    let sender = run_client(compositor.clone()).await;
    let observer = run_client(compositor.clone()).await;
    sender.initialize().await.unwrap();
    observer.initialize().await.unwrap();

    let mut sender_rx = sender.subscribe();
    let mut observer_rx = observer.subscribe();
    sender
        .load_session("s", &PathBuf::from("/tmp"), vec![])
        .await
        .unwrap();
    observer
        .load_session("s", &PathBuf::from("/tmp"), vec![])
        .await
        .unwrap();

    sender.prompt("s", "shared prompt").await.unwrap();

    let n = recv_notification(&mut observer_rx).await;
    assert!(matches!(n.update, SessionUpdate::UserMessageChunk(_)));

    let result = timeout(Duration::from_millis(200), sender_rx.recv()).await;
    assert!(
        result.is_err(),
        "sending client should not see its own prompt echoed back"
    );
}
