//! MED-2: Bounded retry on POST /traces (retry-count and 4xx-no-retry tests)
//!
//! Tests prove that:
//!   (a) store_execution_trace retries exactly 2 times (3 total attempts)
//!       on HTTP 503, then succeeds on the final attempt.
//!   (c) store_execution_trace does NOT retry on 4xx errors (400, 401,
//!       429/QuotaExceeded, 422).
//!   (d) store_execution_trace retries on HTTP 502 and 504.
//!   (e) store_execution_trace_stream retries the underlying POST on 503.
//!
//! ALL tests in this file are expected to FAIL until the retry logic is
//! implemented in client.rs.

use ace_sdk_core::{
    AceClient, AceClientOptions, AceConfig, AceError, ExecutionResult, ExecutionTrace,
};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn make_trace() -> ExecutionTrace {
    ExecutionTrace {
        task: "test task for retry".to_string(),
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

fn make_client(server_url: &str) -> AceClient {
    let config = AceConfig {
        server_url: server_url.to_string(),
        api_token: "ace_user_retrytest".to_string(),
        project_id: "prj_retry".to_string(),
        default_org_id: Some("org_retry".to_string()),
        ..Default::default()
    };
    AceClient::new(config, AceClientOptions::default()).expect("client")
}

const LEARN_RESP: &str = r#"{
    "stored": true,
    "task": "test task for retry",
    "timestamp": "2026-06-11T00:00:00Z",
    "analysis_performed": true
}"#;

// ─── (a) Retry fires exactly twice on 503; succeeds on third attempt ─────────

/// The SDK must retry POST /traces on a transient 503.
/// mockito serves 503 twice then 200 on the 3rd call.
/// mock assertions confirm exactly 2 + 1 hits.
#[tokio::test]
async fn should_attempt_three_times_when_first_two_calls_return_503() {
    let mut server = mockito::Server::new_async().await;

    // Mockito serves queued responses in order for the same path+method
    let mock_503a = server
        .mock("POST", "/traces")
        .with_status(503)
        .with_header("content-type", "text/plain")
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let mock_503b = server
        .mock("POST", "/traces")
        .with_status(503)
        .with_header("content-type", "text/plain")
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let mock_ok = server
        .mock("POST", "/traces")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(LEARN_RESP)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let result = client.store_execution_trace(&make_trace()).await;

    assert!(
        result.is_ok(),
        "Expected success after retry, got: {:?}",
        result.err()
    );
    assert!(result.unwrap().stored, "stored should be true");

    mock_503a.assert_async().await;
    mock_503b.assert_async().await;
    mock_ok.assert_async().await;
}

/// When first call returns 503 and second succeeds, only 2 total attempts.
#[tokio::test]
async fn should_succeed_on_second_attempt_when_first_call_returns_503() {
    let mut server = mockito::Server::new_async().await;

    server
        .mock("POST", "/traces")
        .with_status(503)
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let mock_ok = server
        .mock("POST", "/traces")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(LEARN_RESP)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let result = client.store_execution_trace(&make_trace()).await;

    assert!(
        result.is_ok(),
        "Expected success on 2nd attempt, got: {:?}",
        result.err()
    );
    mock_ok.assert_async().await;
}

// ─── (c) No retry on 4xx ─────────────────────────────────────────────────────

/// 400 Bad Request is a client error — must NOT be retried.
/// The mock has .expect(1) so a retry would cause assertion failure.
#[tokio::test]
async fn should_not_retry_when_response_is_400_bad_request() {
    let mut server = mockito::Server::new_async().await;

    let mock_400 = server
        .mock("POST", "/traces")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"bad payload","code":"BAD_REQUEST"}"#)
        .expect(1)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .store_execution_trace(&make_trace())
        .await
        .expect_err("should error on 400");

    match err {
        AceError::Http { status, .. } => assert_eq!(status, 400),
        other => panic!("Expected Http(400), got {:?}", other),
    }
    mock_400.assert_async().await;
}

/// 422 Unprocessable is a validation error — must NOT be retried.
#[tokio::test]
async fn should_not_retry_when_response_is_422_validation_error() {
    let mut server = mockito::Server::new_async().await;

    let mock_422 = server
        .mock("POST", "/traces")
        .with_status(422)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"validation failed","code":"VALIDATION"}"#)
        .expect(1)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .store_execution_trace(&make_trace())
        .await
        .expect_err("should error on 422");

    match err {
        AceError::Http { status, .. } => assert_eq!(status, 422),
        other => panic!("Expected Http(422), got {:?}", other),
    }
    mock_422.assert_async().await;
}

/// 429 QuotaExceeded must return graceful Ok(quota_exceeded:true)
/// and must NOT be retried.
#[tokio::test]
async fn should_not_retry_and_return_graceful_ok_when_response_is_429_quota_exceeded() {
    let mut server = mockito::Server::new_async().await;

    let mock_429 = server
        .mock("POST", "/traces")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":"quota_exceeded","code":"TRACES_LIMIT","resource":"traces","current":100,"limit":100,"upgrade_url":"https://ace.example.com/upgrade","message":"Quota exceeded"}"#)
        .expect(1)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let result = client.store_execution_trace(&make_trace()).await;

    assert!(
        result.is_ok(),
        "QuotaExceeded should return graceful Ok, got: {:?}",
        result.err()
    );
    let resp = result.unwrap();
    assert_eq!(
        resp.quota_exceeded,
        Some(true),
        "quota_exceeded must be Some(true)"
    );
    assert!(!resp.stored, "stored must be false when quota exceeded");

    mock_429.assert_async().await;
}

/// 401 Unauthorized — must NOT be retried.
#[tokio::test]
async fn should_not_retry_when_response_is_401_unauthorized() {
    let mut server = mockito::Server::new_async().await;

    let mock_401 = server
        .mock("POST", "/traces")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"Unauthorized"}"#)
        .expect(1)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .store_execution_trace(&make_trace())
        .await
        .expect_err("should error on 401");

    assert!(err.is_auth_error());
    mock_401.assert_async().await;
}

// ─── (d) Retry on 502 and 504 ────────────────────────────────────────────────

/// 502 Bad Gateway is a gateway-layer transient error — must be retried.
#[tokio::test]
async fn should_retry_when_response_is_502_bad_gateway_then_succeed() {
    let mut server = mockito::Server::new_async().await;

    server
        .mock("POST", "/traces")
        .with_status(502)
        .with_body("Bad Gateway")
        .create_async()
        .await;

    let mock_ok = server
        .mock("POST", "/traces")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(LEARN_RESP)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let result = client.store_execution_trace(&make_trace()).await;

    assert!(
        result.is_ok(),
        "Expected success after 502 retry, got: {:?}",
        result.err()
    );
    mock_ok.assert_async().await;
}

/// 504 Gateway Timeout is a gateway-layer transient error — must be retried.
#[tokio::test]
async fn should_retry_when_response_is_504_gateway_timeout_then_succeed() {
    let mut server = mockito::Server::new_async().await;

    server
        .mock("POST", "/traces")
        .with_status(504)
        .with_body("Gateway Timeout")
        .create_async()
        .await;

    let mock_ok = server
        .mock("POST", "/traces")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(LEARN_RESP)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let result = client.store_execution_trace(&make_trace()).await;

    assert!(
        result.is_ok(),
        "Expected success after 504 retry, got: {:?}",
        result.err()
    );
    mock_ok.assert_async().await;
}

/// All 3 attempts fail with 503 — SDK returns an error after exhaustion.
/// (Dead-letter emission is tested separately in traces_retry_dead_letter.rs)
#[tokio::test]
async fn should_return_error_after_all_three_attempts_fail_with_503() {
    let mut server = mockito::Server::new_async().await;

    for _ in 0..3 {
        server
            .mock("POST", "/traces")
            .with_status(503)
            .with_body("Service Unavailable")
            .create_async()
            .await;
    }

    let client = make_client(&server.url());
    let result = client.store_execution_trace(&make_trace()).await;

    // After exhausting all retries the SDK must surface an error or
    // return a stored:false dead-letter response — it must NOT silently succeed.
    match result {
        Err(AceError::Http { status, .. }) => {
            assert_eq!(status, 503, "Final error must be 503");
        }
        Ok(resp) if !resp.stored => {
            // Acceptable: dead-letter response with stored:false
        }
        Ok(resp) if resp.stored => {
            panic!("SDK must NOT return stored:true after 3x 503 exhaustion");
        }
        other => panic!("Unexpected result: {:?}", other),
    }
}

// ─── (e) store_execution_trace_stream retries on 503 ────────────────────────

/// The SSE path must also apply the retry before falling back.
/// Two 503s then a 200 SSE response — SDK succeeds via retry,
/// does NOT trigger the fallback to /traces.
#[tokio::test]
async fn should_retry_sse_stream_on_503_before_falling_back_to_plain_traces() {
    let mut server = mockito::Server::new_async().await;

    // Two 503s on the SSE endpoint
    server
        .mock("POST", "/traces/stream")
        .with_status(503)
        .with_body("Service Unavailable")
        .create_async()
        .await;

    server
        .mock("POST", "/traces/stream")
        .with_status(503)
        .with_body("Service Unavailable")
        .create_async()
        .await;

    // Third attempt succeeds with minimal SSE body
    let sse_body = "data: {\"stage\":\"done\",\"message\":\"done\",\"timestamp\":\"2026-06-11T00:00:00Z\",\"data\":{\"stored\":true,\"analysis_performed\":true}}\n\n";
    let mock_ok = server
        .mock("POST", "/traces/stream")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(sse_body)
        .create_async()
        .await;

    // Ensure /traces (fallback) is never hit
    let mock_fallback = server
        .mock("POST", "/traces")
        .expect(0)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let trace = make_trace();

    let result = client
        .store_execution_trace_stream(
            &trace,
            |_ev| {},
            true, // fallback_on_error — must NOT be reached if retry succeeds
        )
        .await;

    assert!(
        result.is_ok(),
        "Expected SSE success after retry, got: {:?}",
        result.err()
    );
    mock_ok.assert_async().await;
    mock_fallback.assert_async().await; // 0 hits on /traces
}
