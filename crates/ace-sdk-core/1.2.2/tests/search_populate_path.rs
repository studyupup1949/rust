//! TDD RED → GREEN: Populate path parity with TS unified cache.
//!
//! Three assertion groups matching the TS `search-populate-path.test.ts`:
//!
//! (a) Pattern upsert after search — already wired in search_patterns15,
//!     but we add coverage that confirms the field under test.
//!
//! (b) Throttled edge refresh — `search_patterns15` must call
//!     `refresh_from_server` at most ONCE per throttle window, gated on a
//!     `sync_state` row keyed `graph_edges_synced_at`.
//!
//! (c) result.expanded — seeded edges + patterns → non-empty `expanded`
//!     field on `SearchResponse15`, with cached neighbors id-only stubs,
//!     deduped against primary, disabled when `expand_neighbors = false`.
//!
//! Written FIRST (RED).  All tests that touch unimplemented features fail
//! with `[E0609]` / `[E0425]` / assertion failures before the implementation
//! lands.

use ace_sdk_core::{AceClient, AceClientOptions, AceConfig, SearchRequest15};
use tempfile::TempDir;

const FIXTURE: &str = include_str!("../../../../spec/fixtures/search-1.5.json");

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Build a test client pointing at the given mock server URL.
/// Graph cache lives in a TempDir — never touches ~/.ace-cache.
fn make_client(server_url: &str) -> (AceClient, TempDir) {
    let tmp = TempDir::new().expect("tempdir");
    let config = AceConfig {
        server_url: server_url.to_string(),
        api_token: "ace_user_populate_path_test".to_string(),
        project_id: "prj-populate-test".to_string(),
        default_org_id: Some("org-populate-test".to_string()),
        graph_cache_dir: Some(tmp.path().to_path_buf()),
        ..Default::default()
    };
    let client = AceClient::new(config, AceClientOptions::default()).expect("client");
    (client, tmp)
}

/// Build a test client with a custom graph-edge throttle window.
fn make_client_with_throttle(server_url: &str, throttle_ms: u64) -> (AceClient, TempDir) {
    let tmp = TempDir::new().expect("tempdir");
    let config = AceConfig {
        server_url: server_url.to_string(),
        api_token: "ace_user_throttle_test".to_string(),
        project_id: "prj-throttle-test".to_string(),
        default_org_id: Some("org-throttle-test".to_string()),
        graph_cache_dir: Some(tmp.path().to_path_buf()),
        ..Default::default()
    };
    let opts = AceClientOptions {
        graph_edges_throttle_ms: Some(throttle_ms),
        ..Default::default()
    };
    let client = AceClient::new(config, opts).expect("client");
    (client, tmp)
}

fn search_req(content: &str) -> SearchRequest15 {
    SearchRequest15 {
        pattern: serde_json::json!({
            "id": "temp-populate-test",
            "content": content,
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
        expand_neighbors: None, // default = true
    }
}

// =============================================================================
// (a) Pattern upsert after search — already partially wired, extend coverage
// =============================================================================

/// After search_patterns15 all returned patterns are in the graph cache.
/// cumulative_v15_reward is stored as cumulative_reward; absent → 0.0.
#[tokio::test]
async fn test_search_upserts_patterns_into_cache() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .create_async()
        .await;

    // Also allow the graph refresh call (throttle window is 1h so it fires once)
    let _graph_mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    client
        .search_patterns15(search_req("upsert coverage test"))
        .await
        .expect("search must succeed");

    let gc_arc = client.get_graph_cache().expect("graph cache must exist");
    let gc = gc_arc.lock().expect("lock");

    // fixture has 6 patterns; all must be retrievable
    let expected_ids = [
        "33623e44-54ed-519a-b596-ffa7314fb69f",
        "f32f94ea-408b-5a82-a899-32efa966428f",
        "16ccd980-9a00-5eeb-92b5-03d9fbd7f57a",
        "5742a613-9381-580b-977e-b083f63bd222",
        "ctx-8328763857-6733",
        "0f22864c-d035-5683-96dc-cfa58bd7447c",
    ];
    for id in &expected_ids {
        let entry = gc.get_pattern(id).expect("get_pattern must not error");
        assert!(
            entry.is_some(),
            "pattern {id} must be in graph cache after search_patterns15"
        );
    }

    mock.assert_async().await;
}

/// Expiry is ~7 days (604_800_000 ms ± 1s) from now.
#[tokio::test]
async fn test_search_sets_7day_expiry_on_upserted_patterns() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .create_async()
        .await;

    let _graph_mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    let before_ms = chrono::Utc::now().timestamp_millis();
    client
        .search_patterns15(search_req("7day expiry test"))
        .await
        .expect("search");

    let gc_arc = client.get_graph_cache().expect("cache");
    let gc = gc_arc.lock().expect("lock");

    let expires = gc
        .get_expires_at_ms("33623e44-54ed-519a-b596-ffa7314fb69f")
        .expect("get_expires_at_ms")
        .expect("must be in cache");

    let expected_min = before_ms + 604_800_000 - 2_000;
    let expected_max = before_ms + 604_800_000 + 2_000;
    assert!(
        expires >= expected_min && expires <= expected_max,
        "expires_at_ms={expires} expected in [{expected_min},{expected_max}]"
    );
}

/// cumulative_v15_reward = 6.7 for pattern[1] is stored correctly.
/// Legacy pattern with absent reward → stored as 0.0.
#[tokio::test]
async fn test_search_stores_cumulative_reward_correctly() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .create_async()
        .await;

    let _graph_mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    client
        .search_patterns15(search_req("reward test"))
        .await
        .expect("search");

    let gc_arc = client.get_graph_cache().expect("cache");
    let gc = gc_arc.lock().expect("lock");

    let (_, reward) = gc
        .get_pattern("f32f94ea-408b-5a82-a899-32efa966428f")
        .expect("get_pattern")
        .expect("must be in cache");
    assert!(
        (reward - 6.7_f64).abs() < 1e-9,
        "reward must be 6.7 for pattern[1], got {reward}"
    );

    // Legacy row — absent cumulative_v15_reward → 0.0
    let (_, legacy_reward) = gc
        .get_pattern("ctx-8328763857-6733")
        .expect("get_pattern legacy")
        .expect("legacy must be cached");
    assert_eq!(legacy_reward, 0.0, "legacy reward should be 0.0");
}

// =============================================================================
// (b) Throttled edge refresh
// =============================================================================

/// First search triggers graph refresh (no prior sync_state).
/// The graph mock must be called exactly once.
#[tokio::test]
async fn test_first_search_calls_refresh_from_server() {
    let mut server = mockito::Server::new_async().await;

    let search_mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .create_async()
        .await;

    let graph_mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .expect(1) // must be called exactly once
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    client
        .search_patterns15(search_req("first search triggers refresh"))
        .await
        .expect("search");

    // Wait briefly to allow fire-and-forget async refresh to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    graph_mock.assert_async().await;
    search_mock.assert_async().await;
}

/// Second immediate search must NOT call refresh (within throttle window).
#[tokio::test]
async fn test_second_immediate_search_does_not_call_refresh() {
    let mut server = mockito::Server::new_async().await;

    let _search_mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .expect(2) // two search calls
        .create_async()
        .await;

    // Graph endpoint: expect exactly 1 call total (only from first search)
    let graph_mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .expect(1)
        .create_async()
        .await;

    // Use a large throttle window so the second search definitely hits the gate
    let (client, _tmp) = make_client_with_throttle(&server.url(), 3_600_000);

    client
        .search_patterns15(search_req("first search"))
        .await
        .expect("first search");

    // Allow async fire-and-forget to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    client
        .search_patterns15(search_req("second search immediate"))
        .await
        .expect("second search");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Graph must have been called exactly once (from first search only)
    graph_mock.assert_async().await;
}

/// sync_state key used for throttle gate is `graph_edges_synced_at`.
/// After a search this key must exist in the graph cache sync_state table.
#[tokio::test]
async fn test_sync_state_key_graph_edges_synced_at_written_after_refresh() {
    let mut server = mockito::Server::new_async().await;

    let _search_mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .create_async()
        .await;

    let _graph_mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    client
        .search_patterns15(search_req("sync_state key test"))
        .await
        .expect("search");

    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

    // The sync_state key must be present after the fire-and-forget refresh
    let gc_arc = client.get_graph_cache().expect("cache");
    let gc = gc_arc.lock().expect("lock");
    let value = gc
        .get_sync_state("graph_edges_synced_at")
        .expect("get_sync_state must not error");
    assert!(
        value.is_some(),
        "sync_state key 'graph_edges_synced_at' must be written after first refresh"
    );
}

/// After throttle window expires a subsequent search fires refresh again.
#[tokio::test]
async fn test_refresh_fires_again_after_throttle_window_expires() {
    let mut server = mockito::Server::new_async().await;

    let _search_mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .expect(2)
        .create_async()
        .await;

    // Two graph calls expected: once per search when throttle is 0ms
    let graph_mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .expect(2)
        .create_async()
        .await;

    // Throttle of 0ms means every search triggers a refresh
    let (client, _tmp) = make_client_with_throttle(&server.url(), 0);

    client
        .search_patterns15(search_req("first — throttle 0"))
        .await
        .expect("first search");

    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

    client
        .search_patterns15(search_req("second — throttle expired"))
        .await
        .expect("second search");

    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

    // Both calls should have triggered a refresh
    graph_mock.assert_async().await;
}

// =============================================================================
// (c) result.expanded — seeded graph cache → non-empty expanded field
// =============================================================================

/// Seeded edges + patterns in the cache → expanded field is non-empty after
/// search (the primary result patterns have cached neighbors via the seed).
#[tokio::test]
async fn test_expanded_is_non_empty_with_seeded_cache() {
    // The fixture returns 6 patterns; we'll seed edges that connect two of them
    // BEFORE the search so the neighbor query returns results.
    //
    // Pattern IDs from fixture:
    //   [0] 33623e44-54ed-519a-b596-ffa7314fb69f  (primary)
    //   [1] f32f94ea-408b-5a82-a899-32efa966428f  (primary)
    //   neighbor: seed a separate pattern not in the primary result
    let neighbor_id = "seed-neighbor-aabbcc";

    let mut server = mockito::Server::new_async().await;

    let _search_mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .create_async()
        .await;

    let _graph_mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    // Pre-seed the graph cache with a neighbor pattern and an edge
    {
        let gc_arc = client.get_graph_cache().expect("cache");
        let gc = gc_arc.lock().expect("lock");
        let now_ms = chrono::Utc::now().timestamp_millis();
        let payload = format!(r#"{{"id":"{}","content":"seeded neighbor"}}"#, neighbor_id);
        gc.upsert_pattern_at(neighbor_id, &payload, 3.0, now_ms)
            .expect("upsert neighbor");
        // Edge: primary pattern[0] → neighbor
        gc.upsert_edge("33623e44-54ed-519a-b596-ffa7314fb69f", neighbor_id, 10)
            .expect("upsert edge");
    }

    let response = client
        .search_patterns15(search_req("expanded neighbors test"))
        .await
        .expect("search");

    // expanded must be present and non-empty
    let expanded = response
        .expanded
        .as_ref()
        .expect("response.expanded must be Some when neighbors exist");
    assert!(
        !expanded.is_empty(),
        "expanded must be non-empty when seeded edges connect primary patterns to neighbors"
    );

    // The seeded neighbor must appear in expanded
    let found = expanded.iter().any(|e| e.pattern_id == neighbor_id);
    assert!(
        found,
        "seeded neighbor {neighbor_id} must appear in expanded"
    );
}

/// Cached neighbors (in the patterns table) appear as id-only stubs
/// (cached: true, no payload re-delivery).
/// Uncached neighbors (get_pattern returns None) also appear id-only
/// (no payload_json leakage — matches TS fix for UncachedNeighborEntry).
#[tokio::test]
async fn test_cached_neighbors_are_id_only_stubs() {
    let neighbor_id = "cached-neighbor-ddeeff";

    let mut server = mockito::Server::new_async().await;

    let _search_mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .create_async()
        .await;

    let _graph_mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    {
        let gc_arc = client.get_graph_cache().expect("cache");
        let gc = gc_arc.lock().expect("lock");
        let now_ms = chrono::Utc::now().timestamp_millis();
        let payload = format!(
            r#"{{"id":"{}","content":"cached neighbor payload","cumulative_v15_reward":5.0}}"#,
            neighbor_id
        );
        gc.upsert_pattern_at(neighbor_id, &payload, 5.0, now_ms)
            .expect("upsert");
        gc.upsert_edge("33623e44-54ed-519a-b596-ffa7314fb69f", neighbor_id, 8)
            .expect("upsert edge");
    }

    let response = client
        .search_patterns15(search_req("cached stub test"))
        .await
        .expect("search");

    let expanded = response.expanded.as_ref().expect("expanded must exist");
    let entry = expanded
        .iter()
        .find(|e| e.pattern_id == neighbor_id)
        .expect("cached neighbor must be in expanded");

    assert!(
        entry.cached,
        "neighbor in patterns table must be cached=true"
    );
    // cached=true entries must NOT carry payload_json (id-only stub to save tokens)
    assert!(
        entry.payload_json.is_none(),
        "cached neighbor must not carry payload_json (id-only stub)"
    );
}

/// Primary result patterns must be deduped from expanded.
#[tokio::test]
async fn test_expanded_deduplicates_primary_patterns() {
    let mut server = mockito::Server::new_async().await;

    let _search_mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .create_async()
        .await;

    let _graph_mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    // Seed an edge from pattern[0] → pattern[1] (both are in the primary result)
    // pattern[1] should be deduped from expanded since it's already primary
    {
        let gc_arc = client.get_graph_cache().expect("cache");
        let gc = gc_arc.lock().expect("lock");
        gc.upsert_edge(
            "33623e44-54ed-519a-b596-ffa7314fb69f",
            "f32f94ea-408b-5a82-a899-32efa966428f",
            12,
        )
        .expect("edge");
    }

    let response = client
        .search_patterns15(search_req("dedup test"))
        .await
        .expect("search");

    // Primary patterns must NOT appear in expanded
    if let Some(ref expanded) = response.expanded {
        let in_expanded = expanded
            .iter()
            .any(|e| e.pattern_id == "f32f94ea-408b-5a82-a899-32efa966428f");
        assert!(
            !in_expanded,
            "pattern that is already in primary result must NOT appear in expanded"
        );
    }
}

/// No edges → expanded is None or empty.
#[tokio::test]
async fn test_expanded_is_empty_with_no_edges() {
    let mut server = mockito::Server::new_async().await;

    let _search_mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .create_async()
        .await;

    let _graph_mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    // Do NOT seed any edges — expanded should be None or empty

    let response = client
        .search_patterns15(search_req("no edges test"))
        .await
        .expect("search");

    let is_empty = response
        .expanded
        .as_ref()
        .map(|v| v.is_empty())
        .unwrap_or(true);
    assert!(
        is_empty,
        "expanded must be None or empty when no edges exist"
    );
}

/// expand_neighbors = false → expanded must be None.
#[tokio::test]
async fn test_expand_neighbors_false_omits_expanded_field() {
    let mut server = mockito::Server::new_async().await;

    let _search_mock = server
        .mock("POST", "/patterns/search")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(FIXTURE)
        .create_async()
        .await;

    let _graph_mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .create_async()
        .await;

    let (client, _tmp) = make_client(&server.url());

    // Seed a neighbor so expansion would normally fire
    {
        let gc_arc = client.get_graph_cache().expect("cache");
        let gc = gc_arc.lock().expect("lock");
        let now_ms = chrono::Utc::now().timestamp_millis();
        gc.upsert_pattern_at("nb-disabled", r#"{"id":"nb-disabled"}"#, 1.0, now_ms)
            .expect("upsert");
        gc.upsert_edge("33623e44-54ed-519a-b596-ffa7314fb69f", "nb-disabled", 7)
            .expect("edge");
    }

    let mut req = search_req("expand_neighbors false test");
    // expand_neighbors = false suppresses the field
    req.expand_neighbors = Some(false);

    let response = client.search_patterns15(req).await.expect("search");

    assert!(
        response.expanded.is_none(),
        "expanded must be None when expand_neighbors = false"
    );
}
