//! Tests for config path resolution and state persistence round-trips.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::common::mocks::MockSessionFactory;
use acompose::compositor::Compositor;
use acompose::compositor::state::{
    FileStateStore, MemoryStateStore, PromptJob, PromptStatus, SessionState, State, StateStore,
};
use acompose::config::{Config, CronJobConfig, MisfirePolicy};
use chrono::Utc;
use path_clean::PathClean;

fn temp_dir() -> PathBuf {
    let dir = PathBuf::from(format!(
        "/tmp/acompose_persist_test_{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

async fn build_compositor_with_store(
    store: Arc<dyn StateStore>,
) -> (Compositor, MockSessionFactory) {
    let factory = Arc::new(MockSessionFactory::new());
    let compositor = Compositor::new(
        Arc::clone(&factory) as Arc<dyn acompose::agent::session_factory::SessionFactory>,
        Some(store),
        None,
    )
    .unwrap();
    (compositor, (*factory).clone())
}

#[tokio::test]
async fn from_config_file_resolves_relative_cwd_and_state_path() {
    let dir = temp_dir();
    let config_path = dir.join("acompose.toml");
    let relative_cwd = PathBuf::from("./agents/moderator");
    let relative_state = PathBuf::from("./state/acompose.json");

    let toml = format!(
        r#"
acp_command = "kimi acp"
state_path = {:?}

[[session]]
name = "moderator"
cwd = {:?}
charter = "be nice"
"#,
        relative_state.to_string_lossy(),
        relative_cwd.to_string_lossy(),
    );
    tokio::fs::write(&config_path, toml).await.unwrap();

    let (comp, config, _state) = Compositor::from_config_file(&config_path, None, None, None)
        .await
        .expect("from_config_file should succeed");

    assert_eq!(
        config.sessions[0].cwd,
        dir.join("agents/moderator").clean(),
        "session cwd should be resolved against config directory"
    );
    assert_eq!(
        config.state_path,
        Some(dir.join("state/acompose.json").clean()),
        "state_path should be resolved against config directory"
    );

    // Ensure the compositor does not spawn sessions by default.
    let sessions = comp.list_sessions().await.unwrap();
    assert!(
        sessions.is_empty(),
        "from_config_file should not spawn sessions"
    );
}

#[tokio::test]
async fn from_config_file_uses_explicit_base_dir() {
    let config_dir = temp_dir();
    let base_dir = temp_dir().join("base");
    std::fs::create_dir_all(&base_dir).unwrap();
    let config_path = config_dir.join("acompose.toml");

    let toml = r#"
state_path = "./state.json"

[[session]]
name = "agent"
cwd = "./repo"
charter = ""
"#;
    tokio::fs::write(&config_path, toml).await.unwrap();

    let (_comp, config, _state) =
        Compositor::from_config_file(&config_path, Some(&base_dir), None, None)
            .await
            .unwrap();

    assert_eq!(config.sessions[0].cwd, base_dir.join("repo").clean());
    assert_eq!(config.state_path, Some(base_dir.join("state.json").clean()));
}

#[tokio::test]
async fn sent_prompt_is_persisted_as_pending_job() {
    let store = Arc::new(MemoryStateStore::new());
    let factory = Arc::new(MockSessionFactory::with_response_delay(
        Duration::from_millis(500),
    ));
    let comp = Compositor::new(
        Arc::clone(&factory) as Arc<dyn acompose::agent::session_factory::SessionFactory>,
        Some(Arc::clone(&store) as Arc<dyn StateStore>),
        None,
    )
    .unwrap();

    comp.create_session("test-session", PathBuf::from("/tmp"), "", vec![], vec![])
        .await
        .unwrap();

    // Send a prompt that the in-memory agent will not finish immediately.
    let prompt_id = comp
        .send_message_async("test-session", "hello", None, false)
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    let state = store.load().await.unwrap();
    let session = state
        .sessions
        .get("test-session")
        .expect("session should be persisted");
    let pending = session
        .jobs
        .iter()
        .find(|j| j.status == PromptStatus::Pending)
        .expect("sent prompt should be persisted as pending job");
    assert_eq!(pending.content, "hello");
    assert_eq!(pending.target, "test-session");
    assert_eq!(pending.result.as_ref().map(|r| r.text.clone()), None);

    let _ = prompt_id;
}

#[tokio::test]
async fn state_updates_after_adding_cron_job() {
    let store = Arc::new(MemoryStateStore::new());
    let (comp, _factory) =
        build_compositor_with_store(Arc::clone(&store) as Arc<dyn StateStore>).await;

    comp.create_session("test-session", PathBuf::from("/tmp"), "", vec![], vec![])
        .await
        .unwrap();

    comp.add_cron_job(
        "test-session",
        CronJobConfig {
            name: "daily".to_string(),
            schedule: Some("0 9 * * *".to_string()),
            prompt: "morning summary".to_string(),
            timezone: "UTC".to_string(),
            misfire_policy: MisfirePolicy::Skip,
            run_at: None,
        },
    )
    .await
    .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    let state = store.load().await.unwrap();
    let session = state
        .sessions
        .get("test-session")
        .expect("session should be persisted");
    assert_eq!(session.cwd, PathBuf::from("/tmp"));
    assert_eq!(session.cron_jobs.len(), 1);
    let job_state = session.cron_jobs.get("daily").unwrap();
    assert_eq!(job_state.config.prompt, "morning summary");
    assert!(job_state.next_run_at.is_some());
}

#[tokio::test]
async fn state_is_saved_to_file_store_and_round_trips() {
    let dir = temp_dir();
    let state_path = dir.join("state.json");
    let store: Arc<dyn StateStore> = Arc::new(FileStateStore::new(&state_path));

    {
        let (comp, _factory) = build_compositor_with_store(Arc::clone(&store)).await;
        comp.create_session("test-session", dir.clone(), "charter", vec![], vec![])
            .await
            .unwrap();

        comp.add_cron_job(
            "test-session",
            CronJobConfig {
                name: "once".to_string(),
                schedule: None,
                prompt: "backup".to_string(),
                timezone: "UTC".to_string(),
                misfire_policy: MisfirePolicy::Skip,
                run_at: Some((Utc::now() + chrono::Duration::hours(1)).to_rfc3339()),
            },
        )
        .await
        .unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    assert!(
        state_path.exists(),
        "state file should have been written to disk"
    );

    let loaded = store.load().await.unwrap();
    let session = loaded
        .sessions
        .get("test-session")
        .expect("session should be reloadable from file");
    assert_eq!(session.cwd, dir);
    assert_eq!(session.cron_jobs.len(), 1);
}

#[tokio::test]
async fn spawned_session_from_config_uses_absolute_cwd_in_persisted_state() {
    let dir = temp_dir();
    let subdir = dir.join("agents").join("moderator");
    std::fs::create_dir_all(&subdir).unwrap();
    let config_path = dir.join("acompose.toml");
    let state_path = dir.join("state.json");

    let toml = format!(
        r#"
state_path = {:?}

[[session]]
name = "moderator"
cwd = "./agents/moderator"
charter = ""
"#,
        state_path.file_name().unwrap().to_string_lossy()
    );
    tokio::fs::write(&config_path, toml).await.unwrap();

    let (comp, config, _state) = Compositor::from_config_file(&config_path, None, None, None)
        .await
        .unwrap();

    comp.spawn_sessions_from_config(&config, &State::default())
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    let store: Arc<dyn StateStore> = Arc::new(FileStateStore::new(&state_path));
    let state = store.load().await.unwrap();
    let session = state.sessions.get("moderator").unwrap();
    assert_eq!(
        session.cwd,
        subdir.clean(),
        "cwd in persisted state must be absolute"
    );
}

#[tokio::test]
async fn persisted_sessions_take_precedence_over_config() {
    let store = Arc::new(MemoryStateStore::new());
    let mut state = State::default();
    state.sessions.insert(
        "test-session".to_string(),
        SessionState {
            session_id: "persisted-sid".to_string(),
            cwd: PathBuf::from("/tmp/persisted"),
            charter: Some("persisted charter".to_string()),
            allowed_tool_kinds: vec![],
            mcp_servers: vec![],
            cron_jobs: HashMap::new(),
            jobs: vec![],
        },
    );
    store.save(&state).await.unwrap();

    let factory = Arc::new(MockSessionFactory::new());
    let comp = Compositor::new(
        Arc::clone(&factory) as Arc<dyn acompose::agent::session_factory::SessionFactory>,
        Some(Arc::clone(&store) as Arc<dyn StateStore>),
        None,
    )
    .unwrap();

    let mut config = Config::default();
    config.sessions.push(acompose::config::SessionConfig {
        name: "test-session".to_string(),
        cwd: PathBuf::from("/tmp/from-config"),
        charter: "config charter".to_string(),
        allowed_tool_kinds: vec![],
        mcp_servers: vec![],
        cron_jobs: vec![],
    });

    comp.spawn_sessions_from_config(&config, &state)
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    let info = comp
        .get_session("test-session")
        .await
        .expect("session should exist");
    assert_eq!(
        info.session_id, "persisted-sid",
        "session should be loaded from persisted state, not recreated from config"
    );

    let loaded = store.load().await.unwrap();
    let session = loaded.sessions.get("test-session").unwrap();
    assert_eq!(
        session.cwd,
        PathBuf::from("/tmp/persisted"),
        "persisted cwd should be preserved"
    );
}

#[tokio::test]
async fn persisted_session_resumes_pending_jobs() {
    let store = Arc::new(MemoryStateStore::new());
    let mut state = State::default();
    state.sessions.insert(
        "resume-session".to_string(),
        SessionState {
            session_id: "resume-sid".to_string(),
            cwd: PathBuf::from("/tmp"),
            charter: None,
            allowed_tool_kinds: vec![],
            mcp_servers: vec![],
            cron_jobs: HashMap::new(),
            jobs: vec![PromptJob {
                target: "resume-session".to_string(),
                content: "original prompt".to_string(),
                status: PromptStatus::Pending,
                send_result_to: None,
                cron_job_name: None,
                result: None,
                error: None,
                created_at: Utc::now(),
            }],
        },
    );
    store.save(&state).await.unwrap();

    let factory = Arc::new(MockSessionFactory::new());
    let comp = Compositor::new(
        Arc::clone(&factory) as Arc<dyn acompose::agent::session_factory::SessionFactory>,
        Some(Arc::clone(&store) as Arc<dyn StateStore>),
        None,
    )
    .unwrap();

    let config = Config::default();

    comp.spawn_sessions_from_config(&config, &state)
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(300)).await;

    let agent = factory
        .agent("resume-session")
        .expect("session should have been resumed");
    let prompts = agent.recorded_prompts();
    assert!(
        prompts
            .iter()
            .any(|p| p.contains("сессия была перезапущена")),
        "a continue prompt should be sent for the pending job, got: {:?}",
        prompts
    );
}
