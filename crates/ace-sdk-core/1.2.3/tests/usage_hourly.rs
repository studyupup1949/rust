//! Integration tests for `AceClient::get_org_usage_hourly` and
//! the public `UsageWindow` enum.
//!
//! Uses `mockito` to stand up a local HTTP server so we can exercise
//! happy path, auth/not-found/server errors, and verify that the
//! `window=` query string is built correctly.

use std::str::FromStr;

use ace_sdk_core::{
    AceClient, AceClientOptions, AceConfig, AceError, UsageGranularity, UsageWindow,
};
use mockito::Matcher;

fn make_client(server_url: &str) -> AceClient {
    let config = AceConfig {
        server_url: server_url.to_string(),
        api_token: "ace_user_testtoken".to_string(),
        project_id: "test-project".to_string(),
        ..Default::default()
    };
    AceClient::new(config, AceClientOptions::default()).expect("client")
}

const HAPPY_BODY: &str = r#"{
    "org_id": "org_abc",
    "project_id": "prj_xyz",
    "window": "1h",
    "granularity": "hourly",
    "buckets": [
        {
            "period": "2026-02-17T14:00:00Z",
            "api_calls_total": 42,
            "api_calls_patterns": 10,
            "api_calls_traces": 20,
            "api_calls_playbook": 12,
            "patterns_created": 3,
            "patterns_updated": 2,
            "patterns_deleted": 1,
            "patterns_searched": 7,
            "traces_submitted": 5,
            "bootstrap_runs": 1
        }
    ],
    "totals": {
        "api_calls_total": 42,
        "patterns_created": 3,
        "traces_submitted": 5
    }
}"#;

#[tokio::test]
async fn test_happy_path() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/v1/usage/history?window=1h")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(HAPPY_BODY)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let resp = client
        .get_org_usage_hourly("org_abc", UsageWindow::H1, None)
        .await
        .expect("happy path response");

    assert_eq!(resp.org_id, "org_abc");
    assert_eq!(resp.project_id.as_deref(), Some("prj_xyz"));
    assert_eq!(resp.window, UsageWindow::H1);
    assert_eq!(resp.granularity, UsageGranularity::Hourly);
    assert_eq!(resp.buckets.len(), 1);

    let bucket = &resp.buckets[0];
    assert_eq!(bucket.period, "2026-02-17T14:00:00Z");
    assert_eq!(bucket.api_calls_total, 42);
    assert_eq!(bucket.api_calls_patterns, 10);
    assert_eq!(bucket.api_calls_traces, 20);
    assert_eq!(bucket.api_calls_playbook, 12);
    assert_eq!(bucket.patterns_created, 3);
    assert_eq!(bucket.patterns_updated, 2);
    assert_eq!(bucket.patterns_deleted, 1);
    assert_eq!(bucket.patterns_searched, 7);
    assert_eq!(bucket.traces_submitted, 5);
    assert_eq!(bucket.bootstrap_runs, 1);

    assert_eq!(resp.totals.api_calls_total, 42);
    assert_eq!(resp.totals.patterns_created, 3);
    assert_eq!(resp.totals.traces_submitted, 5);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_unauthorized() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/v1/usage/history?window=7d")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"Unauthorized"}"#)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .get_org_usage_hourly("org_abc", UsageWindow::D7, None)
        .await
        .expect_err("should fail on 401");

    assert!(err.is_auth_error(), "expected auth error, got: {:?}", err);
    assert!(matches!(err, AceError::Auth(_)));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_not_found() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/v1/usage/history?window=1d")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"org not found","code":"NOT_FOUND"}"#)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .get_org_usage_hourly("org_missing", UsageWindow::D1, None)
        .await
        .expect_err("should fail on 404");

    match err {
        AceError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("expected Http(404), got {:?}", other),
    }

    mock.assert_async().await;
}

#[tokio::test]
async fn test_server_error() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/api/v1/usage/history?window=30d")
        .with_status(500)
        .with_header("content-type", "text/plain")
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .get_org_usage_hourly("org_abc", UsageWindow::D30, None)
        .await
        .expect_err("should fail on 500");

    match err {
        AceError::Http { status, .. } => assert_eq!(status, 500),
        other => panic!("expected Http(500), got {:?}", other),
    }

    mock.assert_async().await;
}

#[test]
fn test_invalid_window_via_fromstr() {
    let err = UsageWindow::from_str("2h").expect_err("2h is not a valid window");
    match err {
        AceError::Config(msg) => {
            assert!(msg.contains("2h"), "message should mention input: {}", msg);
        }
        other => panic!("expected Config error, got {:?}", other),
    }

    // sanity: known values still parse
    assert_eq!(UsageWindow::from_str("1h").unwrap(), UsageWindow::H1);
    assert_eq!(UsageWindow::from_str("14d").unwrap(), UsageWindow::D14);
}

#[tokio::test]
async fn test_query_string_roundtrip() {
    let mut server = mockito::Server::new_async().await;

    // Match on the method, path, AND the raw query string. mockito's
    // Matcher::UrlEncoded inspects the query portion so we can assert
    // `?window=1h&project_id=prj_xyz` is present.
    let mock = server
        .mock("GET", "/api/v1/usage/history")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("window".into(), "1h".into()),
            Matcher::UrlEncoded("project_id".into(), "prj_xyz".into()),
        ]))
        .match_header("x-ace-org", "org_abc")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(HAPPY_BODY)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let resp = client
        .get_org_usage_hourly("org_abc", UsageWindow::H1, Some("prj_xyz"))
        .await
        .expect("response");

    assert_eq!(resp.window, UsageWindow::H1);
    mock.assert_async().await;
}
