//! Integration tests for the compositor using in-memory mock sessions.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use acompose::compositor::Compositor;
use acompose::compositor::state::{CronJobState, MemoryStateStore, SessionState, State};
use acompose::config::{Config, CronJobConfig, MisfirePolicy, SessionConfig};
use acompose::cron::{next_run_utc, parse_cron, parse_timezone};
use chrono::Utc;

// common is declared in main.rs
use crate::common::mocks::MockSessionFactory;

async fn build_compositor_with_state(
    state: State,
) -> (Compositor, Arc<MemoryStateStore>, MockSessionFactory) {
    let store = Arc::new(MemoryStateStore::new());
    store.save(&state).await.unwrap();
    let factory = Arc::new(MockSessionFactory::new());

    let compositor = Compositor::new(
        Arc::clone(&factory) as Arc<dyn acompose::agent::session_factory::SessionFactory>,
        Some(Arc::clone(&store) as Arc<dyn acompose::compositor::state::StateStore>),
        None,
    )
    .unwrap();

    for (name, session_state) in state.sessions.clone() {
        let _ = compositor.load_session(&name, &session_state).await;
    }

    (compositor, store, (*factory).clone())
}

async fn setup_test_session(comp: &Compositor, name: &str) -> String {
    let (info, _charter_prompt_id) = comp
        .create_session(name, PathBuf::from("/tmp"), "", vec![], vec![])
        .await
        .expect("create_session should succeed");
    info.session_id
}

fn base_session_state(session_id: &str) -> SessionState {
    SessionState {
        session_id: session_id.to_string(),
        cwd: PathBuf::from("/tmp"),
        charter: Some("test charter".to_string()),
        allowed_tool_kinds: vec![],
        mcp_servers: vec![],
        cron_jobs: HashMap::new(),
        jobs: vec![],
    }
}

#[tokio::test]
async fn message_result_is_forwarded_with_text_and_sender_name() {
    let (comp, _store, factory) = build_compositor_with_state(State::default()).await;
    setup_test_session(&comp, "alice").await;
    setup_test_session(&comp, "bob").await;

    factory
        .agent("bob")
        .expect("bob agent should exist")
        .set_response_text("hello back");

    comp.send_message_async("bob", "hi", Some("alice"), true)
        .await
        .expect("send_message_async should return prompt id");

    // Wait for bob to process the prompt and for the result to be forwarded to alice.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let bob_agent = factory.agent("bob").expect("bob agent should exist");
    assert!(
        bob_agent
            .recorded_prompts()
            .iter()
            .any(|p| p == "Message from agent 'alice':\n\nhi"),
        "bob should receive the original message, got: {:?}",
        bob_agent.recorded_prompts()
    );

    let alice_agent = factory.agent("alice").expect("alice agent should exist");
    assert!(
        alice_agent
            .recorded_prompts()
            .iter()
            .any(|p| p == "Message from agent 'bob':\n\nhello back"),
        "alice should receive the forwarded result with sender name and text, got: {:?}",
        alice_agent.recorded_prompts()
    );
}

#[tokio::test]
async fn async_prompt_is_queued_and_processed() {
    let (comp, _store, factory) = build_compositor_with_state(State::default()).await;
    setup_test_session(&comp, "test-session").await;

    let prompt_id = comp
        .send_message_async("test-session", "hello", None, false)
        .await
        .expect("send_message_async should return prompt id");

    // Wait for the in-memory agent to finish.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let agent = factory.agent("test-session").expect("agent should exist");
    assert!(
        agent.recorded_prompts().iter().any(|p| p == "hello"),
        "agent should receive prompt, got: {:?}",
        agent.recorded_prompts()
    );

    let info = comp
        .get_session("test-session")
        .await
        .expect("session should exist");
    assert!(
        info.current_prompt.is_none(),
        "prompt should be removed after finalization"
    );

    let _ = prompt_id;
}

#[tokio::test]
async fn user_prompt_is_queued_and_processed() {
    let (comp, _store, factory) = build_compositor_with_state(State::default()).await;
    setup_test_session(&comp, "test-session").await;

    let prompt_id = comp
        .send_message_async("test-session", "hello user", None, false)
        .await
        .expect("send_message_async should return prompt id");

    // Wait briefly so the prompt can be queued.
    tokio::time::sleep(Duration::from_millis(50)).await;

    let agent = factory.agent("test-session").expect("agent should exist");
    assert!(
        agent.recorded_prompts().iter().any(|p| p == "hello user"),
        "agent should receive prompt, got: {:?}",
        agent.recorded_prompts()
    );

    let _ = prompt_id;
}

#[tokio::test]
async fn add_cron_job_persists_and_lists() {
    let (comp, _store, _factory) = build_compositor_with_state(State::default()).await;
    setup_test_session(&comp, "test-session").await;

    let job = CronJobConfig {
        name: "daily".to_string(),
        schedule: Some("0 9 * * *".to_string()),
        prompt: "morning summary".to_string(),
        timezone: "UTC".to_string(),
        misfire_policy: MisfirePolicy::Skip,
        run_at: None,
    };

    comp.add_cron_job("test-session", job.clone())
        .await
        .expect("add_cron_job should succeed");

    let jobs = comp
        .list_cron_jobs("test-session")
        .await
        .expect("list should succeed");
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].config.name, "daily");
    assert_eq!(jobs[0].config.schedule, Some("0 9 * * *".to_string()));
    assert!(jobs[0].next_run_at.is_some());
    assert!(jobs[0].next_run_at.unwrap() > Utc::now());

    // Remove the cron job so it does not outlive the test.
    comp.remove_cron_job("test-session", "daily")
        .await
        .expect("remove_cron_job should succeed");
}

#[tokio::test]
async fn remove_cron_job_deletes_and_stops() {
    let (comp, _store, _factory) = build_compositor_with_state(State::default()).await;
    setup_test_session(&comp, "test-session").await;

    let job = CronJobConfig {
        name: "daily".to_string(),
        schedule: Some("0 9 * * *".to_string()),
        prompt: "morning summary".to_string(),
        timezone: "UTC".to_string(),
        misfire_policy: MisfirePolicy::Skip,
        run_at: None,
    };

    comp.add_cron_job("test-session", job.clone())
        .await
        .expect("add_cron_job should succeed");
    comp.remove_cron_job("test-session", "daily")
        .await
        .expect("remove_cron_job should succeed");

    let jobs = comp
        .list_cron_jobs("test-session")
        .await
        .expect("list should succeed");
    assert!(jobs.is_empty());
}

#[tokio::test]
async fn add_one_time_cron_job_persists_and_lists() {
    let (comp, _store, _factory) = build_compositor_with_state(State::default()).await;
    setup_test_session(&comp, "test-session").await;

    let run_at = (Utc::now() + chrono::Duration::hours(2)).to_rfc3339();
    let job = CronJobConfig {
        name: "snapshot".to_string(),
        schedule: None,
        prompt: "backup".to_string(),
        timezone: "UTC".to_string(),
        misfire_policy: MisfirePolicy::Skip,
        run_at: Some(run_at.clone()),
    };

    let info = comp
        .add_cron_job("test-session", job.clone())
        .await
        .expect("add_cron_job should succeed");
    assert!(info.config.schedule.is_none());
    assert_eq!(info.config.run_at, Some(run_at.clone()));
    assert!(info.next_run_at.is_some());
    assert!(info.description.contains("One-time"));

    let jobs = comp
        .list_cron_jobs("test-session")
        .await
        .expect("list should succeed");
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].config.name, "snapshot");
    assert!(jobs[0].config.schedule.is_none());
    assert_eq!(jobs[0].config.run_at, Some(run_at));
    assert!(jobs[0].description.contains("One-time"));

    comp.remove_cron_job("test-session", "snapshot")
        .await
        .expect("remove_cron_job should succeed");
}

#[tokio::test]
async fn one_time_cron_job_rejects_run_at_in_past() {
    let (comp, _store, _factory) = build_compositor_with_state(State::default()).await;
    setup_test_session(&comp, "test-session").await;

    let run_at = (Utc::now() - chrono::Duration::seconds(1)).to_rfc3339();
    let job = CronJobConfig {
        name: "past-shot".to_string(),
        schedule: None,
        prompt: "run immediately".to_string(),
        timezone: "UTC".to_string(),
        misfire_policy: MisfirePolicy::Skip,
        run_at: Some(run_at),
    };

    let result = comp.add_cron_job("test-session", job).await;
    assert!(
        result.is_err(),
        "add_cron_job should reject run_at in the past"
    );
}

#[tokio::test]
async fn one_time_cron_job_removes_itself_after_firing() {
    let run_at = Utc::now() - chrono::Duration::seconds(1);
    let mut state = State::default();
    let mut session = base_session_state("sid1");
    session.cron_jobs.insert(
        "past-shot".to_string(),
        CronJobState {
            config: CronJobConfig {
                name: "past-shot".to_string(),
                schedule: None,
                prompt: "run immediately".to_string(),
                timezone: "UTC".to_string(),
                misfire_policy: MisfirePolicy::Skip,
                run_at: Some(run_at.to_rfc3339()),
            },
            last_run_at: None,
            next_run_at: Some(run_at),
        },
    );
    state.sessions.insert("test-session".to_string(), session);

    let (comp, _store, _factory) = build_compositor_with_state(state).await;

    comp.add_cron_job(
        "test-session",
        CronJobConfig {
            name: "past-shot".to_string(),
            schedule: None,
            prompt: "run immediately".to_string(),
            timezone: "UTC".to_string(),
            misfire_policy: MisfirePolicy::Skip,
            run_at: Some(run_at.to_rfc3339()),
        },
    )
    .await
    .expect("add_cron_job should accept persisted one-shot job with run_at in past");

    tokio::time::sleep(Duration::from_millis(500)).await;

    let jobs = comp
        .list_cron_jobs("test-session")
        .await
        .expect("list should succeed");
    assert!(
        jobs.iter().all(|j| j.config.name != "past-shot"),
        "one-shot job should be removed after firing"
    );
}

#[tokio::test]
async fn stale_one_time_cron_job_is_removed_on_startup() {
    let run_at = Utc::now() - chrono::Duration::hours(1);
    let mut state = State::default();
    let mut session = base_session_state("sid1");
    session.cron_jobs.insert(
        "stale-shot".to_string(),
        CronJobState {
            config: CronJobConfig {
                name: "stale-shot".to_string(),
                schedule: None,
                prompt: "run immediately".to_string(),
                timezone: "UTC".to_string(),
                misfire_policy: MisfirePolicy::Skip,
                run_at: Some(run_at.to_rfc3339()),
            },
            last_run_at: Some(run_at),
            next_run_at: Some(run_at),
        },
    );
    state.sessions.insert("test-session".to_string(), session);

    let (comp, _store, _factory) = build_compositor_with_state(state).await;

    let result = comp
        .add_cron_job(
            "test-session",
            CronJobConfig {
                name: "stale-shot".to_string(),
                schedule: None,
                prompt: "run immediately".to_string(),
                timezone: "UTC".to_string(),
                misfire_policy: MisfirePolicy::Skip,
                run_at: Some(run_at.to_rfc3339()),
            },
        )
        .await;
    assert!(
        result.is_err(),
        "add_cron_job should report a stale one-shot job"
    );

    let jobs = comp
        .list_cron_jobs("test-session")
        .await
        .expect("list should succeed");
    assert!(
        jobs.iter().all(|j| j.config.name != "stale-shot"),
        "stale one-shot job should be removed from state"
    );
}

#[tokio::test]
async fn misfire_skip_computes_future_run() {
    let cron = parse_cron("* * * * *").expect("valid cron");
    let tz = parse_timezone("UTC").expect("valid timezone");
    let yesterday = Utc::now() - chrono::Duration::days(1);
    let next_missed = next_run_utc(&cron, tz, yesterday).expect("next occurrence exists");

    let mut state = State::default();
    let mut session = base_session_state("sid1");
    session.cron_jobs.insert(
        "daily".to_string(),
        CronJobState {
            config: CronJobConfig {
                name: "daily".to_string(),
                schedule: Some("* * * * *".to_string()),
                prompt: "morning summary".to_string(),
                timezone: "UTC".to_string(),
                misfire_policy: MisfirePolicy::Skip,
                run_at: None,
            },
            last_run_at: Some(yesterday),
            next_run_at: Some(next_missed),
        },
    );
    state.sessions.insert("test-session".to_string(), session);

    let (comp, _store, _factory) = build_compositor_with_state(state).await;

    let job = CronJobConfig {
        name: "daily".to_string(),
        schedule: Some("* * * * *".to_string()),
        prompt: "morning summary".to_string(),
        timezone: "UTC".to_string(),
        misfire_policy: MisfirePolicy::Skip,
        run_at: None,
    };

    comp.add_cron_job("test-session", job)
        .await
        .expect("add_cron_job should succeed");

    let jobs = comp
        .list_cron_jobs("test-session")
        .await
        .expect("list should succeed");
    let job = jobs
        .iter()
        .find(|j| j.config.name == "daily")
        .expect("daily job exists");
    let next = job.next_run_at.expect("next_run_at is set");
    // The missed run should have been skipped, so the next run is in the future.
    assert!(next > Utc::now());

    comp.remove_cron_job("test-session", "daily")
        .await
        .expect("remove_cron_job should succeed");
}

#[tokio::test]
async fn cron_prompt_is_sent_on_fire() {
    let cron = parse_cron("* * * * *").expect("valid cron");
    let tz = parse_timezone("UTC").expect("valid timezone");
    let yesterday = Utc::now() - chrono::Duration::days(1);
    let next_missed = next_run_utc(&cron, tz, yesterday).expect("next occurrence exists");

    let mut state = State::default();
    let mut session = base_session_state("sid1");
    session.cron_jobs.insert(
        "daily".to_string(),
        CronJobState {
            config: CronJobConfig {
                name: "daily".to_string(),
                schedule: Some("* * * * *".to_string()),
                prompt: "morning summary".to_string(),
                timezone: "UTC".to_string(),
                misfire_policy: MisfirePolicy::FireOnce,
                run_at: None,
            },
            last_run_at: Some(yesterday),
            next_run_at: Some(next_missed),
        },
    );
    state.sessions.insert("test-session".to_string(), session);

    let (comp, _store, _factory) = build_compositor_with_state(state).await;

    let job = CronJobConfig {
        name: "daily".to_string(),
        schedule: Some("* * * * *".to_string()),
        prompt: "morning summary".to_string(),
        timezone: "UTC".to_string(),
        misfire_policy: MisfirePolicy::FireOnce,
        run_at: None,
    };

    comp.add_cron_job("test-session", job)
        .await
        .expect("add_cron_job should succeed");

    // Wait for the worker to fire and persist the run timestamp.
    tokio::time::sleep(Duration::from_millis(500)).await;

    let jobs = comp
        .list_cron_jobs("test-session")
        .await
        .expect("list should succeed");
    let job = jobs
        .iter()
        .find(|j| j.config.name == "daily")
        .expect("daily job exists");
    assert!(
        job.last_run_at.is_some(),
        "cron job should have fired and persisted last_run_at"
    );
    assert!(
        job.last_run_at.unwrap() > yesterday,
        "last_run_at should be after the seeded last_run"
    );

    comp.remove_cron_job("test-session", "daily")
        .await
        .expect("remove_cron_job should succeed");
}

#[tokio::test]
async fn recreate_session_appends_extra_charter_and_sends_prompt() {
    let (comp, _store, factory) = build_compositor_with_state(State::default()).await;

    comp.create_session(
        "recreate-session",
        PathBuf::from("/tmp"),
        "base charter",
        vec![],
        vec![],
    )
    .await
    .expect("create_session should succeed");

    // Wait for the initial charter prompt to be processed.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let (info, charter_prompt_id) = comp
        .recreate_session("recreate-session", Some("extra instructions"))
        .await
        .expect("recreate_session should succeed");

    assert!(
        charter_prompt_id.is_some(),
        "recreate_session should return a charter prompt id when extra_charter is provided"
    );

    // Wait for the recreated session's charter prompt to be processed.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let agent = factory
        .agent("recreate-session")
        .expect("agent should exist");
    let prompts = agent.recorded_prompts();
    assert!(
        prompts
            .iter()
            .any(|p| p == "base charter\n\nextra instructions"),
        "agent should receive combined charter after recreate, got: {:?}",
        prompts
    );

    let _ = info;
}

#[tokio::test]
async fn recreate_session_without_extra_charter_resends_base_charter() {
    let (comp, _store, factory) = build_compositor_with_state(State::default()).await;

    comp.create_session(
        "recreate-session-no-extra",
        PathBuf::from("/tmp"),
        "base charter",
        vec![],
        vec![],
    )
    .await
    .expect("create_session should succeed");

    // Wait for the initial charter prompt to be processed.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let (info, charter_prompt_id) = comp
        .recreate_session("recreate-session-no-extra", None)
        .await
        .expect("recreate_session should succeed");

    assert!(
        charter_prompt_id.is_some(),
        "recreate_session should return a charter prompt id"
    );

    // Wait for the recreated session's charter prompt to be processed.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let agent = factory
        .agent("recreate-session-no-extra")
        .expect("agent should exist");
    let prompts = agent.recorded_prompts();
    assert_eq!(
        prompts,
        vec!["base charter"],
        "after recreate the agent should see only the resent base charter, got: {:?}",
        prompts
    );

    let _ = info;
}

#[tokio::test]
async fn recreate_session_does_not_accumulate_extra_charter() {
    let (comp, _store, factory) = build_compositor_with_state(State::default()).await;

    comp.create_session(
        "recreate-session-twice",
        PathBuf::from("/tmp"),
        "base charter",
        vec![],
        vec![],
    )
    .await
    .expect("create_session should succeed");

    // Wait for the initial charter prompt to be processed.
    tokio::time::sleep(Duration::from_millis(200)).await;

    comp.recreate_session("recreate-session-twice", Some("extra one"))
        .await
        .expect("first recreate_session should succeed");
    tokio::time::sleep(Duration::from_millis(200)).await;

    comp.recreate_session("recreate-session-twice", Some("extra two"))
        .await
        .expect("second recreate_session should succeed");
    tokio::time::sleep(Duration::from_millis(200)).await;

    let agent = factory
        .agent("recreate-session-twice")
        .expect("agent should exist");
    let prompts = agent.recorded_prompts();

    assert!(
        prompts.iter().any(|p| p == "base charter\n\nextra two"),
        "agent should receive the latest combined charter, got: {:?}",
        prompts
    );
    assert!(
        !prompts
            .iter()
            .any(|p| p.contains("extra one") && p.contains("extra two")),
        "extra charters should not accumulate across recreates, got: {:?}",
        prompts
    );
}

#[tokio::test]
async fn config_cron_jobs_are_scheduled_on_startup() {
    let (comp, _store, _factory) = build_compositor_with_state(State::default()).await;

    let mut config = Config::default();
    config.sessions.push(SessionConfig {
        name: "cron-session".to_string(),
        cwd: PathBuf::from("/tmp"),
        charter: String::new(),
        allowed_tool_kinds: vec![],
        mcp_servers: vec![],
        cron_jobs: vec![CronJobConfig {
            name: "config-daily".to_string(),
            schedule: Some("0 9 * * *".to_string()),
            prompt: "config morning summary".to_string(),
            timezone: "UTC".to_string(),
            misfire_policy: MisfirePolicy::Skip,
            run_at: None,
        }],
    });

    comp.spawn_sessions_from_config(&config, &State::default())
        .await
        .expect("spawn_sessions_from_config should succeed");

    tokio::time::sleep(Duration::from_millis(100)).await;

    let jobs = comp
        .list_cron_jobs("cron-session")
        .await
        .expect("list should succeed");
    assert_eq!(
        jobs.len(),
        1,
        "config cron jobs should be scheduled on startup, got: {:?}",
        jobs
    );
    assert_eq!(jobs[0].config.name, "config-daily");

    comp.remove_cron_job("cron-session", "config-daily")
        .await
        .expect("remove_cron_job should succeed");
}
