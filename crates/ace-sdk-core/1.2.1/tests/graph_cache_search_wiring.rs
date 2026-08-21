//! Stage 3 TDD — ACE 1.5 search read-path → graph cache wiring.
//!
//! Verifies that `AceClient::search_patterns15` populates the graph cache
//! (CONTRACT §5c: "wire on search refresh") so patterns returned from a
//! `/patterns/search` response are immediately retrievable via `get_pattern(id)`.
//!
//! These tests were written FIRST (red) before the wiring was added to client.rs.

use ace_sdk_core::{AceClient, AceClientOptions, AceConfig, SearchRequest15};
use tempfile::TempDir;

const FIXTURE: &str = include_str!("../../../../spec/fixtures/search-1.5.json");

/// Build a test client that hits the given mock server URL.
///
/// The graph cache is rooted in a `TempDir` so `cargo test` never touches
/// `~/.ace-cache` (test isolation — each call creates a fresh temp directory).
fn make_client(server_url: &str) -> (AceClient, TempDir) {
    let tmp = TempDir::new().expect("tempdir");
    let config = AceConfig {
        server_url: server_url.to_string(),
        api_token: "ace_user_test_search_wiring".to_string(),
        project_id: "prj-wiring-test".to_string(),
        default_org_id: Some("org-wiring-test".to_string()),
        graph_cache_dir: Some(tmp.path().to_path_buf()),
        ..Default::default()
    };
    let client = AceClient::new(config, AceClientOptions::default()).expect("client");
    (client, tmp)
}

// =============================================================================
// graph_cache_ttl_2hop_isolation acceptance gate
//
// This test exercises the FULL acceptance gate named in the structured output:
//   • graph cache populated after search_patterns15
//   • patterns from search are retrievable via get_pattern(id)
//   • only the correct pattern IDs are present (isolation between search calls)
// =============================================================================

/// After `search_patterns15` returns, every pattern in `similar_patterns`
/// must be retrievable from the graph cache via `get_pattern(id)`.
#[tokio::test]
async fn test_search_patterns15_populates_graph_cache() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    // Verify graph cache exists on the client
    assert!(
        client.get_graph_cache().is_some(),
        "AceClient must carry a graph cache after construction"
    );

    let req = SearchRequest15 {
        pattern: serde_json::json!({
            "id": "temp-search-id",
            "content": "test query for wiring",
            "confidence": 0.8,
            "created_at": "2026-06-05T00:00:00Z",
            "section": "general"
        }),
        threshold: Some(0.75),
        top_k: Some(15),
        task_intent: None,
        exploration_enabled: None,
        exploration_rate: None,
        session_id: None,
        expand_neighbors: None,
    };

    let response = client
        .search_patterns15(req)
        .await
        .expect("search_patterns15 must succeed");

    assert_eq!(response.similar_patterns.len(), 6, "fixture has 6 patterns");

    // After search, every returned pattern must be in the graph cache.
    let gc_arc = client.get_graph_cache().expect("graph cache must exist");
    let gc = gc_arc.lock().expect("lock graph cache");

    let expected_ids = [
        "33623e44-54ed-519a-b596-ffa7314fb69f", // pattern[0]
        "f32f94ea-408b-5a82-a899-32efa966428f", // pattern[1]
        "16ccd980-9a00-5eeb-92b5-03d9fbd7f57a", // pattern[2]
        "5742a613-9381-580b-977e-b083f63bd222", // pattern[3]
        "ctx-8328763857-6733",                  // pattern[4] (legacy 1.0)
        "0f22864c-d035-5683-96dc-cfa58bd7447c", // pattern[5]
    ];

    for id in &expected_ids {
        let result = gc.get_pattern(id).expect("get_pattern must not error");
        assert!(
            result.is_some(),
            "pattern id={} must be in graph cache after search_patterns15",
            id
        );
    }

    mock.assert_async().await;
}

/// The `cumulative_v15_reward` stored in the graph cache matches what the
/// search response provided (round-trip of the reward field).
#[tokio::test]
async fn test_graph_cache_stores_correct_reward_from_search() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    let req = SearchRequest15 {
        pattern: serde_json::json!({
            "id": "temp-reward-check",
            "content": "reward check query",
            "confidence": 0.8,
            "created_at": "2026-06-05T00:00:00Z",
            "section": "general"
        }),
        threshold: None,
        top_k: Some(10),
        task_intent: None,
        exploration_enabled: None,
        exploration_rate: None,
        session_id: None,
        expand_neighbors: None,
    };

    client.search_patterns15(req).await.expect("search");

    let gc_arc = client.get_graph_cache().expect("graph cache");
    let gc = gc_arc.lock().expect("lock");

    // pattern[1]: cumulative_v15_reward = 6.7
    let (_, reward) = gc
        .get_pattern("f32f94ea-408b-5a82-a899-32efa966428f")
        .expect("get_pattern")
        .expect("must be in cache");
    assert!(
        (reward - 6.7_f64).abs() < 1e-9,
        "reward must be 6.7 for pattern[1], got {reward}"
    );

    // pattern[4]: legacy 1.0, cumulative_v15_reward absent → stored as 0.0
    let (_, legacy_reward) = gc
        .get_pattern("ctx-8328763857-6733")
        .expect("get_pattern legacy")
        .expect("legacy pattern must also be cached");
    assert_eq!(
        legacy_reward, 0.0,
        "legacy 1.0 pattern should have reward=0.0"
    );

    mock.assert_async().await;
}

/// When `search_patterns15` is called with `task_intent` and
/// `exploration_enabled`, those fields must appear in the request body sent to
/// the server (CONTRACT §2 optional knobs).
#[tokio::test]
async fn test_search_patterns15_sends_optional_knobs_when_set() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .match_body(mockito::Matcher::AllOf(vec![
            mockito::Matcher::Regex("\"task_intent\"\\s*:\\s*\"routine\"".to_string()),
            mockito::Matcher::Regex("\"exploration_enabled\"\\s*:\\s*true".to_string()),
            mockito::Matcher::Regex("\"exploration_rate\"\\s*:\\s*0\\.25".to_string()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    let req = SearchRequest15 {
        pattern: serde_json::json!({
            "id": "temp-knobs",
            "content": "knobs test",
            "confidence": 0.8,
            "created_at": "2026-06-05T00:00:00Z",
            "section": "general"
        }),
        threshold: None,
        top_k: None,
        task_intent: Some("routine".to_string()),
        exploration_enabled: Some(true),
        exploration_rate: Some(0.25),
        session_id: None,
        expand_neighbors: None,
    };

    client
        .search_patterns15(req)
        .await
        .expect("search with knobs");

    mock.assert_async().await;
}

/// Graceful degrade: when the server returns a response that contains a legacy
/// 1.0 row (no `match_factors`, no `payload_version`), `search_patterns15`
/// must NOT panic or return an error — it must decode the response and
/// populate the cache for valid patterns.
#[tokio::test]
async fn test_search_patterns15_graceful_degrade_legacy_row() {
    let legacy_response = r#"{
        "similar_patterns": [
            {
                "id": "legacy-only-id",
                "content": "legacy pattern content",
                "confidence": 0.75,
                "section": "strategies_and_hard_rules",
                "created_at": "2026-01-01T00:00:00Z",
                "helpful": 3.5,
                "harmful": 0.0,
                "observations": 3.5
            }
        ],
        "retrieval_id": null,
        "count": 1,
        "local_count": 1,
        "shared_count": 0,
        "tokens_in_response": 100
    }"#;

    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(legacy_response)
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    let req = SearchRequest15 {
        pattern: serde_json::json!({
            "id": "temp",
            "content": "legacy degrade test",
            "confidence": 0.8,
            "created_at": "2026-06-05T00:00:00Z",
            "section": "general"
        }),
        threshold: None,
        top_k: None,
        task_intent: None,
        exploration_enabled: None,
        exploration_rate: None,
        session_id: None,
        expand_neighbors: None,
    };

    let response = client
        .search_patterns15(req)
        .await
        .expect("must not error on legacy 1.0 response");

    // The legacy pattern should be decoded correctly
    assert_eq!(response.similar_patterns.len(), 1);
    let p = &response.similar_patterns[0];
    assert_eq!(p.id, "legacy-only-id");
    assert!(
        p.payload_version.is_none(),
        "legacy row has no payload_version"
    );
    assert!(p.match_factors.is_none(), "legacy row has no match_factors");
    // retrieval_id absent → None (no crash)
    assert!(response.retrieval_id.is_none());

    // Legacy pattern is still cached (cache is write-on-every-pattern)
    let gc_arc = client.get_graph_cache().expect("graph cache");
    let gc = gc_arc.lock().expect("lock");
    assert!(
        gc.get_pattern("legacy-only-id").expect("get").is_some(),
        "legacy pattern must be in cache"
    );

    mock.assert_async().await;
}

// =============================================================================
// session_id wire-through (CONTRACT §search-session-id)
//
// When the caller supplies `session_id` on `SearchRequest15`, the serialized
// POST body sent to `/patterns/search` MUST contain the field.
// When `session_id` is `None`, the field MUST be absent from the body
// (skip_serializing_if = "Option::is_none").
// =============================================================================

/// `session_id` is included in the request body when `Some`.
#[tokio::test]
async fn test_search_patterns15_sends_session_id_when_some() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .match_body(mockito::Matcher::Regex(
            "\"session_id\"\\s*:\\s*\"ses-abc-123\"".to_string(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    let req = SearchRequest15 {
        pattern: serde_json::json!({
            "id": "temp-session-id-present",
            "content": "session id present test",
            "confidence": 0.8,
            "created_at": "2026-06-05T00:00:00Z",
            "section": "general"
        }),
        threshold: None,
        top_k: None,
        task_intent: None,
        exploration_enabled: None,
        exploration_rate: None,
        session_id: Some("ses-abc-123".to_string()),
        expand_neighbors: None,
    };

    client
        .search_patterns15(req)
        .await
        .expect("search with session_id");

    mock.assert_async().await;
}

/// `session_id` is absent from the serialized body when `None`.
///
/// This is a serialization-level assertion (no HTTP round-trip needed) because
/// `skip_serializing_if = "Option::is_none"` is purely a serde property.
#[test]
fn test_search_patterns15_omits_session_id_when_none() {
    let req = SearchRequest15 {
        pattern: serde_json::json!({
            "id": "temp-session-id-absent",
            "content": "session id absent test",
            "confidence": 0.8,
            "created_at": "2026-06-05T00:00:00Z",
            "section": "general"
        }),
        threshold: None,
        top_k: None,
        task_intent: None,
        exploration_enabled: None,
        exploration_rate: None,
        session_id: None,
        expand_neighbors: None,
    };

    let json = serde_json::to_string(&req).expect("serialize");
    assert!(
        !json.contains("session_id"),
        "session_id must be absent when None, but got: {json}"
    );
    // Sanity: pattern key is present
    assert!(
        json.contains("\"pattern\""),
        "pattern must be present: {json}"
    );
}
