//! Integration tests for `AceClient::get_last_usage()` and the
//! response-side `X-ACE-*` header parser (parity with TS, Kotlin, Go,
//! Python `getLastUsage`).
//!
//! Uses `mockito` to stand up a local HTTP server that returns the same
//! `X-ACE-*` headers the production ACE server emits on every
//! authenticated response. Exercises:
//!
//! 1. `get_last_usage()` returns `None` before any request is made
//! 2. After a request returning `X-ACE-Plan` etc., `get_last_usage()`
//!    returns a fully populated `UsageInfo` (plan, plan_tier,
//!    subscription_type, status, all metrics, subscription_updated_at)
//! 3. A response with NO `X-ACE-Plan` header leaves `last_usage`
//!    untouched (no spurious updates)

use ace_sdk_core::{
    AceClient, AceClientOptions, AceConfig, PlanTier, SubscriptionStatus, SubscriptionType,
};

fn make_client(server_url: &str) -> AceClient {
    let config = AceConfig {
        server_url: server_url.to_string(),
        api_token: "ace_user_testtoken".to_string(),
        project_id: "test-project".to_string(),
        ..Default::default()
    };
    AceClient::new(config, AceClientOptions::default()).expect("client")
}

const EMPTY_PLAYBOOK_BODY: &str = r#"{
    "playbook": {
        "strategies_and_hard_rules": [],
        "useful_code_snippets": [],
        "troubleshooting_and_pitfalls": [],
        "apis_to_use": []
    },
    "total_bullets": 0
}"#;

#[tokio::test]
async fn last_usage_is_none_before_any_request() {
    let server = mockito::Server::new_async().await;
    let client = make_client(&server.url());

    assert!(
        client.get_last_usage().await.is_none(),
        "expected get_last_usage() to be None before any request"
    );
}

#[tokio::test]
async fn last_usage_is_populated_after_response_with_usage_headers() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/playbook?include_metadata=true")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_header("X-ACE-Plan", "team/pro")
        .with_header("X-ACE-Status", "active")
        .with_header("X-ACE-Patterns", "10/50")
        .with_header("X-ACE-Patterns-Total", "100/500")
        .with_header("X-ACE-Projects", "3/10")
        .with_header("X-ACE-Domains", "1/5")
        .with_header("X-ACE-Templates", "2/20")
        .with_header("X-ACE-API-Calls", "1500/10000")
        .with_header("X-ACE-Traces", "42/1000")
        .with_header("X-ACE-Subscription-Updated-At", "2026-05-05T12:00:00Z")
        .with_body(EMPTY_PLAYBOOK_BODY)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let _ = client
        .get_playbook(true)
        .await
        .expect("get_playbook should succeed");

    let usage = client
        .get_last_usage()
        .await
        .expect("get_last_usage() must return Some after a response with X-ACE-Plan");

    assert_eq!(usage.plan, "team/pro");
    assert_eq!(usage.subscription_type, SubscriptionType::Team);
    assert_eq!(usage.plan_tier, PlanTier::Pro);
    assert_eq!(usage.status, SubscriptionStatus::Active);

    assert_eq!(usage.patterns.used, 10);
    assert_eq!(usage.patterns.limit, 50);
    assert_eq!(usage.patterns_total.used, 100);
    assert_eq!(usage.patterns_total.limit, 500);
    assert_eq!(usage.projects.used, 3);
    assert_eq!(usage.projects.limit, 10);
    assert_eq!(usage.domains.used, 1);
    assert_eq!(usage.domains.limit, 5);
    assert_eq!(usage.templates.used, 2);
    assert_eq!(usage.templates.limit, 20);
    assert_eq!(usage.api_calls.used, 1500);
    assert_eq!(usage.api_calls.limit, 10000);
    assert_eq!(usage.traces_today.used, 42);
    assert_eq!(usage.traces_today.limit, 1000);

    assert_eq!(
        usage.subscription_updated_at.as_deref(),
        Some("2026-05-05T12:00:00Z")
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn response_without_plan_header_leaves_last_usage_untouched() {
    let mut server = mockito::Server::new_async().await;

    // No X-ACE-Plan header on this response — parser must short-circuit.
    let mock = server
        .mock("GET", "/playbook?include_metadata=true")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(EMPTY_PLAYBOOK_BODY)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let _ = client.get_playbook(true).await.expect("get_playbook ok");

    assert!(
        client.get_last_usage().await.is_none(),
        "expected get_last_usage() to remain None when response had no X-ACE-Plan header"
    );

    mock.assert_async().await;
}

#[tokio::test]
async fn last_usage_handles_individual_free_plan() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/playbook?include_metadata=true")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_header("X-ACE-Plan", "individual/free")
        .with_header("X-ACE-Status", "trialing")
        .with_header("X-ACE-Patterns", "0/10")
        .with_body(EMPTY_PLAYBOOK_BODY)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let _ = client.get_playbook(true).await.expect("get_playbook ok");

    let usage = client.get_last_usage().await.expect("usage");
    assert_eq!(usage.plan, "individual/free");
    assert_eq!(usage.subscription_type, SubscriptionType::Individual);
    assert_eq!(usage.plan_tier, PlanTier::Free);
    assert_eq!(usage.status, SubscriptionStatus::Trialing);
    assert_eq!(usage.patterns.used, 0);
    assert_eq!(usage.patterns.limit, 10);

    mock.assert_async().await;
}
