//! TDD — Stage 4: GraphCache::refresh_from_server (TODO #104)
//!
//! RED tests written FIRST.  They must FAIL before the implementation lands.
//!
//! Contract:
//!   GET /patterns/graph?min_weight=<n>[&since=<ms>]
//!   Auth: same Bearer + X-ACE-Project headers as search_patterns.
//!   200 → { "edges": [ { "src": "…", "dst": "…", "weight": <int> }, … ], "truncated"?: bool }
//!
//! Acceptance criteria:
//!   (a) Happy-path: 3 edges returned → all 3 are upserted; neighbors() returns them.
//!   (b) Truncated: server returns truncated=true → edges are still upserted, result
//!       carries truncated=true.
//!   (c) Empty edges (DB error path, HTTP 200 { "edges": [] }) → no throw, 0 upserted,
//!       pre-existing cache state unchanged.
//!   (d) Network error / non-200 → no throw, returns Err or Ok(0) (best-effort),
//!       pre-existing cache state unchanged.
//!   (e) since_ms is forwarded as a query param when Some; absent when None.
//!   (f) min_weight defaults to 5 when None.

use ace_sdk_core::cache::GraphCache;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use tempfile::TempDir;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Build an Authorization header map for mock requests.
fn auth_headers(token: &str) -> HeaderMap {
    let mut map = HeaderMap::new();
    map.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );
    map
}

/// Upsert a fresh non-expired pattern so `neighbors()` can join against it.
fn seed_pattern(cache: &GraphCache, id: &str) {
    let now_ms = chrono::Utc::now().timestamp_millis();
    let payload = format!(r#"{{"id":"{}","content":"seeded"}}"#, id);
    cache
        .upsert_pattern_at(id, &payload, 1.0, now_ms)
        .expect("upsert_pattern_at");
}

// ─────────────────────────────────────────────────────────────────────────────
// (a) Happy-path: edges upserted, neighbors() returns them
// ─────────────────────────────────────────────────────────────────────────────

/// Three edges from the server land in the local graph cache and are
/// immediately reachable via `neighbors()`.
#[tokio::test]
async fn test_refresh_upserts_edges_and_neighbors_works() {
    let mut server = mockito::Server::new_async().await;

    let body = r#"{
        "edges": [
            {"src": "pat-A", "dst": "pat-B", "weight": 10},
            {"src": "pat-A", "dst": "pat-C", "weight": 7},
            {"src": "pat-B", "dst": "pat-C", "weight": 5}
        ]
    }"#;

    let mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    // Seed patterns so neighbors() JOIN succeeds (topology-only endpoint doesn't
    // upsert nodes — only edges).
    seed_pattern(&cache, "pat-A");
    seed_pattern(&cache, "pat-B");
    seed_pattern(&cache, "pat-C");

    let http = reqwest::Client::new();
    let result = cache
        .refresh_from_server(&http, &server.url(), &auth_headers("ace_test"), None, None)
        .await;

    assert!(
        result.is_ok(),
        "refresh_from_server must not throw: {:?}",
        result.err()
    );
    let info = result.unwrap();
    assert_eq!(info.edges_upserted, 3, "all 3 edges must be upserted");
    assert!(!info.truncated, "truncated must be false");

    // neighbors(A, hops=2, min_weight=5) should now return B and C.
    let neighbors = cache.neighbors("pat-A", 2, 5).unwrap();
    let ids: Vec<String> = neighbors.into_iter().map(|n| n.0).collect();
    assert!(
        ids.contains(&"pat-B".to_string()),
        "pat-B must be reachable from pat-A"
    );
    assert!(
        ids.contains(&"pat-C".to_string()),
        "pat-C must be reachable from pat-A"
    );

    mock.assert_async().await;
}

// ─────────────────────────────────────────────────────────────────────────────
// (b) Truncated response — edges upserted, result.truncated = true
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_refresh_truncated_flag_propagated() {
    let mut server = mockito::Server::new_async().await;

    let body = r#"{
        "edges": [
            {"src": "p1", "dst": "p2", "weight": 8}
        ],
        "truncated": true
    }"#;

    let mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();
    seed_pattern(&cache, "p1");
    seed_pattern(&cache, "p2");

    let http = reqwest::Client::new();
    let result = cache
        .refresh_from_server(&http, &server.url(), &auth_headers("ace_test"), None, None)
        .await
        .expect("must not error");

    assert_eq!(result.edges_upserted, 1);
    assert!(
        result.truncated,
        "truncated must be true when server returns truncated=true"
    );

    mock.assert_async().await;
}

// ─────────────────────────────────────────────────────────────────────────────
// (c) Empty edges response (DB error path) — no throw, cache unchanged
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_refresh_empty_edges_no_throw_cache_unchanged() {
    let mut server = mockito::Server::new_async().await;

    let body = r#"{"edges": []}"#;

    let mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await;

    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    // Pre-populate an edge that must survive the empty refresh.
    seed_pattern(&cache, "pre-A");
    seed_pattern(&cache, "pre-B");
    cache.upsert_edge("pre-A", "pre-B", 10).unwrap();

    let http = reqwest::Client::new();
    let result = cache
        .refresh_from_server(&http, &server.url(), &auth_headers("ace_test"), None, None)
        .await;

    assert!(result.is_ok(), "empty edges must not throw");
    let info = result.unwrap();
    assert_eq!(info.edges_upserted, 0);

    // Pre-existing edge must still be there.
    let neighbors = cache.neighbors("pre-A", 2, 5).unwrap();
    let ids: Vec<String> = neighbors.into_iter().map(|n| n.0).collect();
    assert!(
        ids.contains(&"pre-B".to_string()),
        "pre-existing edge must survive an empty refresh"
    );

    mock.assert_async().await;
}

// ─────────────────────────────────────────────────────────────────────────────
// (d) Network error — no throw (best-effort), cache unchanged
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_refresh_network_error_no_throw() {
    // Point at a port that is not listening.
    let unreachable = "http://127.0.0.1:1"; // port 1 is reserved, will refuse

    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    // Seed a pattern + edge that must survive the failed refresh.
    seed_pattern(&cache, "survives-A");
    seed_pattern(&cache, "survives-B");
    cache.upsert_edge("survives-A", "survives-B", 9).unwrap();

    let http = reqwest::Client::new();
    // Must NOT panic — returns Ok(0) or Err (both acceptable for best-effort).
    // The critical assertion is: no panic, cache intact.
    let _ = cache
        .refresh_from_server(&http, unreachable, &auth_headers("ace_test"), None, None)
        .await;

    // Cache must be intact.
    let neighbors = cache.neighbors("survives-A", 2, 5).unwrap();
    let ids: Vec<String> = neighbors.into_iter().map(|n| n.0).collect();
    assert!(
        ids.contains(&"survives-B".to_string()),
        "pre-existing edge must survive a network-error refresh"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// (d2) Non-200 response — no throw, cache unchanged
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_refresh_non_200_no_throw() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::Any)
        .with_status(503)
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();
    seed_pattern(&cache, "x");
    seed_pattern(&cache, "y");
    cache.upsert_edge("x", "y", 6).unwrap();

    let http = reqwest::Client::new();
    let _ = cache
        .refresh_from_server(&http, &server.url(), &auth_headers("ace_test"), None, None)
        .await;

    // Cache must be intact.
    let neighbors = cache.neighbors("x", 2, 5).unwrap();
    let ids: Vec<String> = neighbors.into_iter().map(|n| n.0).collect();
    assert!(
        ids.contains(&"y".to_string()),
        "edge must survive a 503 refresh"
    );

    mock.assert_async().await;
}

// ─────────────────────────────────────────────────────────────────────────────
// (e) since_ms forwarded as query param when Some
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_refresh_since_ms_forwarded() {
    let mut server = mockito::Server::new_async().await;

    let since_ms: i64 = 1_700_000_000_000;

    let mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::AllOf(vec![
            mockito::Matcher::UrlEncoded("min_weight".into(), "5".into()),
            mockito::Matcher::UrlEncoded("since".into(), since_ms.to_string()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .create_async()
        .await;

    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    let http = reqwest::Client::new();
    let _ = cache
        .refresh_from_server(
            &http,
            &server.url(),
            &auth_headers("ace_test"),
            None,           // min_weight defaults to 5
            Some(since_ms), // since_ms forwarded
        )
        .await;

    mock.assert_async().await;
}

// ─────────────────────────────────────────────────────────────────────────────
// (f) min_weight defaults to 5 when None; explicit value forwarded
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_refresh_default_min_weight_is_5() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::UrlEncoded(
            "min_weight".into(),
            "5".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .create_async()
        .await;

    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    let http = reqwest::Client::new();
    let _ = cache
        .refresh_from_server(&http, &server.url(), &auth_headers("ace_test"), None, None)
        .await;

    mock.assert_async().await;
}

#[tokio::test]
async fn test_refresh_explicit_min_weight_forwarded() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/patterns/graph")
        .match_query(mockito::Matcher::UrlEncoded(
            "min_weight".into(),
            "12".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"edges":[]}"#)
        .create_async()
        .await;

    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    let http = reqwest::Client::new();
    let _ = cache
        .refresh_from_server(
            &http,
            &server.url(),
            &auth_headers("ace_test"),
            Some(12), // explicit min_weight
            None,
        )
        .await;

    mock.assert_async().await;
}

// ─────────────────────────────────────────────────────────────────────────────
// (g) Upsert is idempotent — calling refresh twice with the same edges leaves
//     the cache correct (no duplicates, final weight wins).
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_refresh_idempotent_upsert() {
    let body = r#"{
        "edges": [
            {"src": "q1", "dst": "q2", "weight": 9}
        ]
    }"#;

    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();
    seed_pattern(&cache, "q1");
    seed_pattern(&cache, "q2");

    let http = reqwest::Client::new();

    for _ in 0..2 {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/patterns/graph")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create_async()
            .await;

        let _ = cache
            .refresh_from_server(&http, &server.url(), &auth_headers("ace_test"), None, None)
            .await;
    }

    // Only one unique edge q1→q2 should be present.
    let neighbors = cache.neighbors("q1", 2, 5).unwrap();
    let ids: Vec<String> = neighbors.into_iter().map(|n| n.0).collect();
    assert_eq!(
        ids.iter().filter(|s| *s == "q2").count(),
        1,
        "edge must appear exactly once after idempotent double-upsert"
    );
}
