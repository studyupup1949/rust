//! Integration tests for spec-21 traces-read API:
//! `AceClient::list_traces` + `AceClient::get_trace`.
//!
//! Uses `mockito` to assert the SDK:
//!   - hits the ROOT prefix `/traces` (not `/api/v1/traces`)
//!   - sends Authorization + X-ACE-Org + X-ACE-Project headers
//!   - propagates the full filter chip query string
//!   - maps 400/401/403/404/410/422/503 to typed errors
//!   - maps 410 to `AceError::TraceUnavailable`

use ace_sdk_core::{
    AceClient, AceClientOptions, AceConfig, AceError, TraceFilters,
};
use mockito::Matcher;

fn make_client(server_url: &str) -> AceClient {
    let config = AceConfig {
        server_url: server_url.to_string(),
        api_token: "ace_user_testtoken".to_string(),
        project_id: "prj_test".to_string(),
        default_org_id: Some("org_abc".to_string()),
        ..Default::default()
    };
    AceClient::new(config, AceClientOptions::default()).expect("client")
}

const LIST_BODY: &str = r#"{
    "traces": [
        {
            "id": "11111111-1111-1111-1111-111111111111",
            "task": "do thing",
            "status": "success",
            "timestamp": "2026-04-26T12:00:00Z",
            "project_id": "prj_test",
            "duration_ms": null,
            "step_count": 7
        },
        {
            "id": "22222222-2222-2222-2222-222222222222",
            "task": "do other thing",
            "status": "failure",
            "timestamp": "2026-04-26T11:59:00Z",
            "project_id": "prj_test"
        }
    ],
    "next_cursor": "Y3Vyc29yX2RlYWRiZWVm",
    "total": null
}"#;

const DETAIL_BODY: &str = r#"{
    "id": "11111111-1111-1111-1111-111111111111",
    "task": "do thing",
    "status": "success",
    "timestamp": "2026-04-26T12:00:00Z",
    "project_id": "prj_test",
    "duration_ms": 1234,
    "trajectory": [
        {
            "step": 1,
            "action": "Glob",
            "args": {"pattern": "**/*.rs"},
            "result": null,
            "start_ms": 0,
            "end_ms": 12
        },
        {
            "step": 2,
            "action": "Read",
            "args": {"path": "src/lib.rs"}
        }
    ],
    "summary": "Read some files",
    "error": null,
    "linked_patterns": [
        {
            "id": "ctx-pat-1",
            "content": "use Read tool",
            "domain": "ace-filesystem",
            "helpful_score": 12
        }
    ],
    "linked_patterns_meta": {
        "requested": 3,
        "resolved": 1,
        "missing_reason": "not_resolvable"
    },
    "metadata": {
        "git_branch": "main",
        "agent_type": "general-purpose",
        "session_id": "sess_xyz"
    }
}"#;

// =============================================================================
// list_traces
// =============================================================================

#[tokio::test]
async fn test_list_traces_happy_path() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces")
        .match_query(Matcher::UrlEncoded("project_id".into(), "prj_test".into()))
        .match_header("authorization", "Bearer ace_user_testtoken")
        .match_header("x-ace-org", "org_abc")
        .match_header("x-ace-project", "prj_test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(LIST_BODY)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let resp = client
        .list_traces(TraceFilters {
            project_id: "prj_test".to_string(),
            ..Default::default()
        })
        .await
        .expect("happy path");

    assert_eq!(resp.traces.len(), 2);
    assert_eq!(resp.traces[0].id, "11111111-1111-1111-1111-111111111111");
    assert_eq!(resp.traces[0].status, "success");
    assert_eq!(resp.traces[0].step_count, Some(7));
    assert_eq!(resp.traces[0].duration_ms, None);
    assert_eq!(resp.traces[1].status, "failure");
    assert_eq!(resp.next_cursor.as_deref(), Some("Y3Vyc29yX2RlYWRiZWVm"));
    assert_eq!(resp.total, None);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_traces_all_six_filters() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("project_id".into(), "prj_test".into()),
            Matcher::UrlEncoded("start".into(), "2026-04-01T00:00:00Z".into()),
            Matcher::UrlEncoded("end".into(), "2026-04-26T23:59:59Z".into()),
            Matcher::UrlEncoded("status".into(), "failure".into()),
            Matcher::UrlEncoded("agent_type".into(), "general-purpose".into()),
            Matcher::UrlEncoded("session_id".into(), "sess_xyz".into()),
            Matcher::UrlEncoded("git_branch".into(), "main".into()),
            Matcher::UrlEncoded("limit".into(), "75".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(LIST_BODY)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let resp = client
        .list_traces(TraceFilters {
            project_id: "prj_test".to_string(),
            start: Some("2026-04-01T00:00:00Z".to_string()),
            end: Some("2026-04-26T23:59:59Z".to_string()),
            status: Some("failure".to_string()),
            agent_type: Some("general-purpose".to_string()),
            session_id: Some("sess_xyz".to_string()),
            git_branch: Some("main".to_string()),
            limit: Some(75),
            cursor: None,
        })
        .await
        .expect("filters");

    assert_eq!(resp.traces.len(), 2);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_traces_cursor_pagination() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("project_id".into(), "prj_test".into()),
            Matcher::UrlEncoded("cursor".into(), "Y3Vyc29yX2RlYWRiZWVm".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"traces":[],"next_cursor":null,"total":null}"#)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let resp = client
        .list_traces(TraceFilters {
            project_id: "prj_test".to_string(),
            cursor: Some("Y3Vyc29yX2RlYWRiZWVm".to_string()),
            ..Default::default()
        })
        .await
        .expect("cursor page");

    assert!(resp.traces.is_empty());
    assert!(resp.next_cursor.is_none());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_traces_400_missing_project_id() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces")
        .match_query(Matcher::Any)
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"missing project_id","code":"BAD_REQUEST"}"#)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .list_traces(TraceFilters {
            project_id: "prj_test".to_string(),
            ..Default::default()
        })
        .await
        .expect_err("400");

    match err {
        AceError::Http { status, .. } => assert_eq!(status, 400),
        other => panic!("expected Http(400), got {:?}", other),
    }
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_traces_401_unauthorized() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces")
        .match_query(Matcher::Any)
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"Unauthorized"}"#)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .list_traces(TraceFilters {
            project_id: "prj_test".to_string(),
            ..Default::default()
        })
        .await
        .expect_err("401");

    assert!(err.is_auth_error());
    assert!(matches!(err, AceError::Auth(_)));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_traces_403_forbidden() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces")
        .match_query(Matcher::Any)
        .with_status(403)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"forbidden","code":"FORBIDDEN"}"#)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .list_traces(TraceFilters {
            project_id: "prj_test".to_string(),
            ..Default::default()
        })
        .await
        .expect_err("403");

    match err {
        AceError::Http { status, .. } => assert_eq!(status, 403),
        other => panic!("expected Http(403), got {:?}", other),
    }
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_traces_422_limit_out_of_range() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces")
        .match_query(Matcher::Any)
        .with_status(422)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"limit out of range","code":"VALIDATION"}"#)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .list_traces(TraceFilters {
            project_id: "prj_test".to_string(),
            limit: Some(500),
            ..Default::default()
        })
        .await
        .expect_err("422");

    match err {
        AceError::Http { status, .. } => assert_eq!(status, 422),
        other => panic!("expected Http(422), got {:?}", other),
    }
    mock.assert_async().await;
}

#[tokio::test]
async fn test_list_traces_503_unavailable() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces")
        .match_query(Matcher::Any)
        .with_status(503)
        .with_header("content-type", "text/plain")
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .list_traces(TraceFilters {
            project_id: "prj_test".to_string(),
            ..Default::default()
        })
        .await
        .expect_err("503");

    match err {
        AceError::Http { status, .. } => assert_eq!(status, 503),
        other => panic!("expected Http(503), got {:?}", other),
    }
    mock.assert_async().await;
}

// =============================================================================
// get_trace
// =============================================================================

#[tokio::test]
async fn test_get_trace_happy_path() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces/11111111-1111-1111-1111-111111111111")
        .match_query(Matcher::UrlEncoded("project_id".into(), "prj_test".into()))
        .match_header("authorization", "Bearer ace_user_testtoken")
        .match_header("x-ace-org", "org_abc")
        .match_header("x-ace-project", "prj_test")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(DETAIL_BODY)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let trace = client
        .get_trace("11111111-1111-1111-1111-111111111111", "prj_test")
        .await
        .expect("happy path");

    assert_eq!(trace.id, "11111111-1111-1111-1111-111111111111");
    assert_eq!(trace.status, "success");
    assert_eq!(trace.duration_ms, Some(1234));
    assert_eq!(trace.trajectory.len(), 2);
    assert_eq!(trace.trajectory[0].step, 1);
    assert_eq!(trace.trajectory[0].action, "Glob");
    assert_eq!(trace.trajectory[0].start_ms, Some(0));
    assert_eq!(trace.trajectory[0].end_ms, Some(12));
    assert_eq!(trace.trajectory[1].action, "Read");
    assert_eq!(trace.trajectory[1].start_ms, None);

    assert_eq!(trace.linked_patterns.len(), 1);
    assert_eq!(trace.linked_patterns[0].id, "ctx-pat-1");
    assert_eq!(trace.linked_patterns[0].helpful_score, 12);
    assert_eq!(trace.linked_patterns[0].domain, "ace-filesystem");

    assert_eq!(trace.linked_patterns_meta.requested, 3);
    assert_eq!(trace.linked_patterns_meta.resolved, 1);
    assert_eq!(
        trace.linked_patterns_meta.missing_reason.as_deref(),
        Some("not_resolvable")
    );

    assert_eq!(
        trace.metadata.get("git_branch").and_then(|v| v.as_str()),
        Some("main")
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_trace_410_trace_unavailable() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces/ghost-trace-id")
        .match_query(Matcher::Any)
        .with_status(410)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"trace content unrecoverable","code":"GONE"}"#)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .get_trace("ghost-trace-id", "prj_test")
        .await
        .expect_err("410 should map to TraceUnavailable");

    match err {
        AceError::TraceUnavailable(msg) => {
            assert!(msg.contains("unrecoverable"), "msg: {}", msg)
        }
        other => panic!("expected TraceUnavailable, got {:?}", other),
    }
    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_trace_400_missing_project_id() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces/some-id")
        .match_query(Matcher::Any)
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"missing project_id"}"#)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .get_trace("some-id", "prj_test")
        .await
        .expect_err("400");

    match err {
        AceError::Http { status, .. } => assert_eq!(status, 400),
        other => panic!("expected Http(400), got {:?}", other),
    }
    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_trace_401_unauthorized() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces/some-id")
        .match_query(Matcher::Any)
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"Unauthorized"}"#)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .get_trace("some-id", "prj_test")
        .await
        .expect_err("401");

    assert!(err.is_auth_error());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_trace_403_forbidden() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces/some-id")
        .match_query(Matcher::Any)
        .with_status(403)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"forbidden"}"#)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .get_trace("some-id", "prj_test")
        .await
        .expect_err("403");

    match err {
        AceError::Http { status, .. } => assert_eq!(status, 403),
        other => panic!("expected Http(403), got {:?}", other),
    }
    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_trace_404_not_found() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces/missing-id")
        .match_query(Matcher::Any)
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"not found"}"#)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .get_trace("missing-id", "prj_test")
        .await
        .expect_err("404");

    match err {
        AceError::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("expected Http(404), got {:?}", other),
    }
    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_trace_422_unprocessable() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces/some-id")
        .match_query(Matcher::Any)
        .with_status(422)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"validation"}"#)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .get_trace("some-id", "prj_test")
        .await
        .expect_err("422");

    match err {
        AceError::Http { status, .. } => assert_eq!(status, 422),
        other => panic!("expected Http(422), got {:?}", other),
    }
    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_trace_503_unavailable() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/traces/some-id")
        .match_query(Matcher::Any)
        .with_status(503)
        .with_header("content-type", "text/plain")
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let client = make_client(&server.url());
    let err = client
        .get_trace("some-id", "prj_test")
        .await
        .expect_err("503");

    match err {
        AceError::Http { status, .. } => assert_eq!(status, 503),
        other => panic!("expected Http(503), got {:?}", other),
    }
    mock.assert_async().await;
}
