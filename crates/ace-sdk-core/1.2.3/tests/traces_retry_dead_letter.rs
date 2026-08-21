//! MED-2 (dead-letter): Structured dead-letter event on retry exhaustion.
//!
//! Tests prove that:
//!   (b) Exhaustion of all retries on persistent 503 emits a structured
//!       dead-letter event with attempts == 3 via the on_dead_letter callback.
//!   (f) Dead-letter carries required schema fields: event, attempts, status,
//!       timestamp — NOT a silent debug log.
//!
//! These tests require `AceClientOptions::on_dead_letter` which does not yet
//! exist — they will FAIL TO COMPILE until that field is added to client.rs.
//! That is the correct red-phase failure.

use ace_sdk_core::{AceClient, AceClientOptions, AceConfig, ExecutionResult, ExecutionTrace};
use std::sync::{Arc, Mutex};

fn make_trace() -> ExecutionTrace {
    ExecutionTrace {
        task: "test task for dead letter".to_string(),
        timestamp: "2026-06-11T00:00:00Z".to_string(),
        trajectory: vec![],
        result: ExecutionResult {
            success: true,
            output: "ok".to_string(),
            error: None,
            summary: None,
        },
        playbook_used: vec![],
        git: None,
        session_id: None,
        agent_id: None,
        agent_type: None,
        parent_agent_id: None,
        retrieval_id: None,
        applied_log_ids: None,
    }
}

/// When all 3 attempts fail with 503, the SDK must call on_dead_letter with
/// a structured event containing event == "ace_trace_dead_letter" and
/// attempts == 3.
#[tokio::test]
async fn should_emit_dead_letter_event_with_attempt_count_3_when_all_retries_exhausted() {
    let mut server = mockito::Server::new_async().await;

    for _ in 0..3 {
        server
            .mock("POST", "/traces")
            .with_status(503)
            .with_header("content-type", "text/plain")
            .with_body("Service Unavailable")
            .create_async()
            .await;
    }

    let dead_letters: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(vec![]));
    let dl_clone = dead_letters.clone();

    let config = AceConfig {
        server_url: server.url(),
        api_token: "ace_user_retrytest".to_string(),
        project_id: "prj_retry".to_string(),
        default_org_id: Some("org_retry".to_string()),
        ..Default::default()
    };
    // on_dead_letter is the new field — DOES NOT EXIST YET (causes compile error)
    let opts = AceClientOptions {
        on_dead_letter: Some(Box::new(move |ev: serde_json::Value| {
            dl_clone.lock().unwrap().push(ev);
        })),
        ..Default::default()
    };
    let client = AceClient::new(config, opts).expect("client");

    let _ = client.store_execution_trace(&make_trace()).await;

    let captured = dead_letters.lock().unwrap();
    assert_eq!(
        captured.len(),
        1,
        "Expected exactly 1 dead-letter event after retry exhaustion, got {}",
        captured.len()
    );
    let ev = &captured[0];
    assert_eq!(
        ev.get("event").and_then(|v| v.as_str()),
        Some("ace_trace_dead_letter"),
        "event field must be 'ace_trace_dead_letter', got: {:?}",
        ev.get("event")
    );
    let attempts = ev
        .get("attempts")
        .and_then(|v| v.as_u64())
        .expect("attempts field must be present as a number");
    assert_eq!(
        attempts, 3,
        "attempts must equal 3 (1 initial + 2 retries), got {}",
        attempts
    );
    assert!(
        ev.get("status").is_some(),
        "dead-letter must carry 'status' field"
    );
    assert!(
        ev.get("timestamp").is_some(),
        "dead-letter must carry 'timestamp' field"
    );
}

/// Dead-letter event must carry all required schema fields when exhausting
/// via 502 Bad Gateway.
#[tokio::test]
async fn should_produce_dead_letter_with_required_schema_fields_on_502_exhaustion() {
    let mut server = mockito::Server::new_async().await;

    for _ in 0..3 {
        server
            .mock("POST", "/traces")
            .with_status(502)
            .with_body("Bad Gateway")
            .create_async()
            .await;
    }

    let captured: Arc<Mutex<Option<serde_json::Value>>> = Arc::new(Mutex::new(None));
    let cap_clone = captured.clone();

    let config = AceConfig {
        server_url: server.url(),
        api_token: "ace_user_retrytest".to_string(),
        project_id: "prj_retry".to_string(),
        ..Default::default()
    };
    // on_dead_letter is the new field — DOES NOT EXIST YET (causes compile error)
    let opts = AceClientOptions {
        on_dead_letter: Some(Box::new(move |ev: serde_json::Value| {
            *cap_clone.lock().unwrap() = Some(ev);
        })),
        ..Default::default()
    };
    let client = AceClient::new(config, opts).expect("client");

    let _ = client.store_execution_trace(&make_trace()).await;

    let guard = captured.lock().unwrap();
    let ev = guard
        .as_ref()
        .expect("dead-letter callback must have been called");

    // Required schema fields
    assert_eq!(
        ev["event"].as_str(),
        Some("ace_trace_dead_letter"),
        "event field must equal 'ace_trace_dead_letter'"
    );
    assert!(
        ev["attempts"].as_u64().is_some(),
        "attempts field must be a number"
    );
    assert!(
        ev["timestamp"].as_str().is_some(),
        "timestamp field must be a string"
    );
    // status may be numeric (502) or null for network errors, but the key must exist
    assert!(
        ev.get("status").is_some(),
        "status key must be present in dead-letter"
    );
    // attempts must be 3 (all retries exhausted)
    assert_eq!(
        ev["attempts"].as_u64().unwrap(),
        3,
        "attempts must be 3 after exhausting max retries"
    );
}
