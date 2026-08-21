//! End-to-end test for the restart-and-continue flow.
//!
//! Simulates a crash while a user prompt is active, then starts a fresh
//! compositor from the persisted state and verifies that the session receives
//! a "continue" message.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// common is declared in main.rs

use crate::common::mocks::MockSessionFactory;
use acompose::compositor::Compositor;
use acompose::compositor::state::{MemoryStateStore, PromptJob, PromptStatus, SessionState, State};
use chrono::Utc;

fn build_state(
    session_id: &str,
    session_name: &str,
    jobs: Vec<(&str, &str, PromptStatus)>,
) -> State {
    let mut state = State::default();
    state.sessions.insert(
        session_name.to_string(),
        SessionState {
            session_id: session_id.to_string(),
            cwd: std::path::PathBuf::from("/tmp"),
            charter: Some("test charter".to_string()),
            allowed_tool_kinds: vec![],
            mcp_servers: vec![],
            cron_jobs: HashMap::new(),
            jobs: jobs
                .into_iter()
                .enumerate()
                .map(|(idx, (_id, content, status))| PromptJob {
                    target: session_name.to_string(),
                    content: content.to_string(),
                    status,
                    send_result_to: None,
                    cron_job_name: None,
                    result: None,
                    error: None,
                    created_at: Utc::now()
                        + chrono::Duration::milliseconds(
                            i64::try_from(idx).expect("job index fits in i64"),
                        ),
                })
                .collect(),
        },
    );
    state
}

async fn build_compositor_with_state(
    state: State,
) -> (Compositor, Arc<MemoryStateStore>, MockSessionFactory) {
    let store = Arc::new(MemoryStateStore::new());
    store.save(&state).await.unwrap();
    let factory = MockSessionFactory::new();

    let compositor = Compositor::new(
        Arc::new(factory.clone()) as Arc<dyn acompose::agent::session_factory::SessionFactory>,
        Some(Arc::clone(&store) as Arc<dyn acompose::compositor::state::StateStore>),
        None,
    )
    .unwrap();

    for (name, session_state) in state.sessions.clone() {
        let _ = compositor.load_session(&name, &session_state).await;
    }

    (compositor, store, factory)
}

#[tokio::test]
async fn sends_continue_message_after_restart() {
    let session_id = "session-restart-1";
    let session_name = "restart-session";
    let state = build_state(
        session_id,
        session_name,
        vec![("7", "original user prompt", PromptStatus::Pending)],
    );
    let (_compositor, store, factory) = build_compositor_with_state(state).await;

    // Yield so the background persistence actor can process the queued
    // state-save messages before we assert on the store contents.
    tokio::task::yield_now().await;

    let persisted = store
        .load()
        .await
        .expect("state should load from memory store");
    let session = persisted.sessions.get(session_name).unwrap();
    assert_eq!(session.jobs.len(), 1);
    assert_eq!(session.jobs[0].status, PromptStatus::Pending);

    tokio::time::sleep(Duration::from_millis(500)).await;

    let agent = factory.agent(session_name).expect("agent should exist");
    let prompts = agent.recorded_prompts();
    assert!(
        prompts
            .iter()
            .any(|p| p == "сессия была перезапущена, продолжай"),
        "agent should receive continue message after restart, got: {:?}",
        prompts
    );
}

#[tokio::test]
async fn preserves_all_queued_jobs_after_restart() {
    let session_id = "session-restart-queued";
    let session_name = "restart-session-queued";
    let state = build_state(
        session_id,
        session_name,
        vec![
            ("1", "queued one", PromptStatus::Queued),
            ("2", "queued two", PromptStatus::Queued),
            ("3", "queued three", PromptStatus::Queued),
        ],
    );
    let (_compositor, _store, factory) = build_compositor_with_state(state).await;

    tokio::time::sleep(Duration::from_millis(800)).await;

    let agent = factory.agent(session_name).expect("agent should exist");
    let prompts = agent.recorded_prompts();
    assert!(
        prompts.iter().any(|p| p == "queued one"),
        "got: {:?}",
        prompts
    );
    assert!(
        prompts.iter().any(|p| p == "queued two"),
        "got: {:?}",
        prompts
    );
    assert!(
        prompts.iter().any(|p| p == "queued three"),
        "got: {:?}",
        prompts
    );
}

#[tokio::test]
async fn converts_extra_pending_jobs_to_queued_after_restart() {
    let session_id = "session-restart-pending";
    let session_name = "restart-session-pending";
    let state = build_state(
        session_id,
        session_name,
        vec![
            ("1", "pending one", PromptStatus::Pending),
            ("2", "pending two", PromptStatus::Pending),
            ("3", "pending three", PromptStatus::Pending),
        ],
    );
    let (_compositor, store, factory) = build_compositor_with_state(state).await;

    // Yield so the background persistence actor can process the queued
    // state-save messages before we assert on the store contents.
    tokio::task::yield_now().await;

    let persisted = store
        .load()
        .await
        .expect("state should load from memory store");
    let session = persisted.sessions.get(session_name).unwrap();
    let pending_count = session
        .jobs
        .iter()
        .filter(|j| j.status == PromptStatus::Pending)
        .count();
    let queued_count = session
        .jobs
        .iter()
        .filter(|j| j.status == PromptStatus::Queued)
        .count();
    assert_eq!(pending_count, 1, "only one pending job should remain");
    assert_eq!(queued_count, 2, "extra pending jobs should become queued");

    tokio::time::sleep(Duration::from_millis(800)).await;

    let agent = factory.agent(session_name).expect("agent should exist");
    let prompts = agent.recorded_prompts();
    assert!(
        prompts
            .iter()
            .any(|p| p == "сессия была перезапущена, продолжай"),
        "earliest pending job should receive continue message, got: {:?}",
        prompts
    );
    assert!(
        prompts.iter().any(|p| p == "pending two"),
        "got: {:?}",
        prompts
    );
    assert!(
        prompts.iter().any(|p| p == "pending three"),
        "got: {:?}",
        prompts
    );
}
