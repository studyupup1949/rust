/// Stage 2 TDD — ACE 1.5 SQLite Graph Cache (Rust)
///
/// RED tests written first; they must FAIL before the implementation is added.
/// Contract: /tmp/ace15-contract/CONTRACT.md §5 — schema + flat-2-hop SQL must
/// be byte-identical across all 5 language SDKs.
///
/// Covers acceptance criteria from CONTRACT §5d:
///   (a) 7d TTL — 6d-old pattern = hit; 8d-old pattern = miss + lazily pruned
///   (b) neighbors(id, hops=2, minWeight=5) flat-2-hop, weight filter, TTL filter
///   (c) isolation — project-A db never returns project-B rows
///   (d) migration — old 5-min-KV schema present → new schema created, no crash
use ace_sdk_core::cache::GraphCache;
use tempfile::TempDir;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

const DAY_MS: i64 = 86_400_000; // 24 h in ms
const TTL_7D_MS: i64 = 7 * DAY_MS; // 604_800_000

/// Build a simple pattern JSON payload.
fn make_payload(content: &str) -> String {
    format!(
        r#"{{"id":"","content":"{}","confidence":0.9,"cumulative_v15_reward":1.0}}"#,
        content
    )
}

/// Insert a pattern directly with a controlled `fetched_at_ms`.
///
/// `fetched_at_ms` determines `expires_at_ms = fetched_at_ms + 604_800_000`.
fn insert_pattern_at(cache: &GraphCache, id: &str, fetched_at_ms: i64) {
    let payload = make_payload(id);
    // Use upsert with overridden fetched_at_ms via the raw helper (or standard upsert + adjust).
    // The test uses the raw upsert_with_time helper exposed for testing.
    cache
        .upsert_pattern_at(id, &payload, 1.0, fetched_at_ms)
        .expect("upsert_pattern_at failed");
}

// ─────────────────────────────────────────────────────────────────────────────
// (a) 7d TTL tests
// ─────────────────────────────────────────────────────────────────────────────

/// A pattern fetched exactly 6 days ago is still within the 7-day window → HIT.
#[test]
fn test_ttl_6d_ago_is_hit() {
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    let now_ms = chrono::Utc::now().timestamp_millis();
    let fetched_6d_ago = now_ms - 6 * DAY_MS;

    insert_pattern_at(&cache, "pat-ttl-hit", fetched_6d_ago);

    let result = cache.get_pattern("pat-ttl-hit").unwrap();
    assert!(
        result.is_some(),
        "Pattern fetched 6d ago should be a cache HIT (within 7d TTL)"
    );
}

/// A pattern fetched 8 days ago is expired → MISS, and the row must be lazily pruned.
#[test]
fn test_ttl_8d_ago_is_miss_and_pruned() {
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    let now_ms = chrono::Utc::now().timestamp_millis();
    let fetched_8d_ago = now_ms - 8 * DAY_MS;

    insert_pattern_at(&cache, "pat-ttl-miss", fetched_8d_ago);

    // get_pattern must return None (expired) AND lazily prune the row.
    let result = cache.get_pattern("pat-ttl-miss").unwrap();
    assert!(
        result.is_none(),
        "Pattern fetched 8d ago should be a cache MISS (expired TTL)"
    );

    // Verify the row is actually gone after the lazy prune.
    let count = cache.count_all_patterns().unwrap();
    assert_eq!(
        count, 0,
        "Expired row must be lazily pruned on get_pattern miss"
    );
}

/// `expires_at_ms` stored = `fetched_at_ms + 604_800_000` (exact TTL constant).
#[test]
fn test_ttl_expires_at_ms_calculation() {
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    let fetched_at = 1_000_000_000_000_i64; // arbitrary fixed ms
    cache
        .upsert_pattern_at("pat-expires", &make_payload("expires"), 1.0, fetched_at)
        .unwrap();

    let stored_expires = cache.get_expires_at_ms("pat-expires").unwrap().unwrap();
    assert_eq!(
        stored_expires,
        fetched_at + TTL_7D_MS,
        "expires_at_ms must equal fetched_at_ms + 604_800_000"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// (b) neighbors() — flat 2-hop, weight filter, TTL filter
// ─────────────────────────────────────────────────────────────────────────────

/// Basic 1-hop and 2-hop reachability with minWeight=5.
///
/// Graph:
///   A –w10→ B –w8→ C
///   A –w3→ D          (weight < 5, filtered)
///   A –w6→ E –w2→ F   (E reachable, F's edge weight < 5 → F NOT reachable)
///
/// Expected neighbors(A, hops=2, minWeight=5): {B, C, E}
#[test]
fn test_neighbors_2hop_weight_filter() {
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    let now_ms = chrono::Utc::now().timestamp_millis();
    for id in ["A", "B", "C", "D", "E", "F"] {
        insert_pattern_at(&cache, id, now_ms);
    }

    cache.upsert_edge("A", "B", 10).unwrap();
    cache.upsert_edge("B", "C", 8).unwrap();
    cache.upsert_edge("A", "D", 3).unwrap(); // filtered by weight
    cache.upsert_edge("A", "E", 6).unwrap();
    cache.upsert_edge("E", "F", 2).unwrap(); // 2nd hop filtered by weight

    let neighbors = cache.neighbors("A", 2, 5).unwrap();
    let ids: Vec<String> = neighbors.into_iter().map(|p| p.0).collect();

    assert!(
        ids.contains(&"B".to_string()),
        "B must be in neighbors (1-hop, w=10)"
    );
    assert!(
        ids.contains(&"C".to_string()),
        "C must be in neighbors (2-hop, w=8)"
    );
    assert!(
        ids.contains(&"E".to_string()),
        "E must be in neighbors (1-hop, w=6)"
    );
    assert!(
        !ids.contains(&"D".to_string()),
        "D must NOT be in neighbors (w=3 < minWeight=5)"
    );
    assert!(
        !ids.contains(&"F".to_string()),
        "F must NOT be in neighbors (2nd hop w=2 < minWeight=5)"
    );
    assert!(
        !ids.contains(&"A".to_string()),
        "A (root) must NOT appear in its own neighbors"
    );
}

/// Expired neighbors are excluded from results.
#[test]
fn test_neighbors_excludes_expired() {
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    let now_ms = chrono::Utc::now().timestamp_millis();
    // Root: fresh
    insert_pattern_at(&cache, "root", now_ms);
    // Neighbor: expired (8d ago)
    insert_pattern_at(&cache, "expired-neighbor", now_ms - 8 * DAY_MS);
    // Neighbor: fresh
    insert_pattern_at(&cache, "fresh-neighbor", now_ms);

    cache.upsert_edge("root", "expired-neighbor", 10).unwrap();
    cache.upsert_edge("root", "fresh-neighbor", 10).unwrap();

    let neighbors = cache.neighbors("root", 2, 1).unwrap();
    let ids: Vec<String> = neighbors.into_iter().map(|p| p.0).collect();

    assert!(ids.contains(&"fresh-neighbor".to_string()));
    assert!(
        !ids.contains(&"expired-neighbor".to_string()),
        "Expired neighbor must be excluded from 2-hop results"
    );
}

/// `neighbors()` with `hops=2` returns the DISTINCT set (no duplicates even when
/// a node is reachable via both a 1-hop AND a 2-hop path).
#[test]
fn test_neighbors_distinct_results() {
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    let now_ms = chrono::Utc::now().timestamp_millis();
    for id in ["root", "X", "Y"] {
        insert_pattern_at(&cache, id, now_ms);
    }

    // root→X (1-hop) AND root→Y→X (2-hop) — X reachable via both paths.
    cache.upsert_edge("root", "X", 10).unwrap();
    cache.upsert_edge("root", "Y", 10).unwrap();
    cache.upsert_edge("Y", "X", 10).unwrap();

    let neighbors = cache.neighbors("root", 2, 1).unwrap();
    let ids: Vec<String> = neighbors.iter().map(|p| p.0.clone()).collect();

    // X should appear exactly once
    let x_count = ids.iter().filter(|s| *s == "X").count();
    assert_eq!(x_count, 1, "DISTINCT: X must appear exactly once");
}

/// `neighbors()` returns the payload_json and cumulative_reward columns.
#[test]
fn test_neighbors_returns_payload_and_reward() {
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    let now_ms = chrono::Utc::now().timestamp_millis();
    insert_pattern_at(&cache, "src", now_ms);

    let payload = r#"{"id":"dst-1","content":"neighbor content","cumulative_v15_reward":3.5}"#;
    cache
        .upsert_pattern_at("dst-1", payload, 3.5, now_ms)
        .unwrap();
    cache.upsert_edge("src", "dst-1", 7).unwrap();

    let neighbors = cache.neighbors("src", 2, 5).unwrap();
    assert_eq!(neighbors.len(), 1);
    let (nid, njson, nreward) = &neighbors[0];
    assert_eq!(nid, "dst-1");
    assert_eq!(*nreward, 3.5f64);
    assert!(njson.contains("neighbor content"));
}

// ─────────────────────────────────────────────────────────────────────────────
// (c) Project isolation — one DB file per (org, project)
// ─────────────────────────────────────────────────────────────────────────────

/// project-A cache must NEVER return project-B rows.
#[test]
fn test_project_isolation() {
    let tmp = TempDir::new().unwrap();

    let cache_a = GraphCache::new("org1", "proj-A", Some(tmp.path().to_path_buf())).unwrap();
    let cache_b = GraphCache::new("org1", "proj-B", Some(tmp.path().to_path_buf())).unwrap();

    let now_ms = chrono::Utc::now().timestamp_millis();
    insert_pattern_at(&cache_a, "only-in-A", now_ms);
    insert_pattern_at(&cache_b, "only-in-B", now_ms);

    // project-A sees its own pattern
    assert!(cache_a.get_pattern("only-in-A").unwrap().is_some());
    // project-A does NOT see project-B's pattern
    assert!(
        cache_a.get_pattern("only-in-B").unwrap().is_none(),
        "project-A must not return project-B rows"
    );
    // project-B sees its own pattern
    assert!(cache_b.get_pattern("only-in-B").unwrap().is_some());
    // project-B does NOT see project-A's pattern
    assert!(
        cache_b.get_pattern("only-in-A").unwrap().is_none(),
        "project-B must not return project-A rows"
    );
}

/// Different orgs with the same project id also get separate DB files.
#[test]
fn test_isolation_different_orgs() {
    let tmp = TempDir::new().unwrap();

    let cache_org1 = GraphCache::new("org-alpha", "proj", Some(tmp.path().to_path_buf())).unwrap();
    let cache_org2 = GraphCache::new("org-beta", "proj", Some(tmp.path().to_path_buf())).unwrap();

    let now_ms = chrono::Utc::now().timestamp_millis();
    insert_pattern_at(&cache_org1, "alpha-pattern", now_ms);

    assert!(cache_org1.get_pattern("alpha-pattern").unwrap().is_some());
    assert!(
        cache_org2.get_pattern("alpha-pattern").unwrap().is_none(),
        "org-beta must not see org-alpha patterns"
    );
}

/// DB file naming uses double-underscore separator: `<org>__<project>.db`.
#[test]
fn test_db_file_naming_double_underscore() {
    let tmp = TempDir::new().unwrap();
    GraphCache::new("myorg", "myproj", Some(tmp.path().to_path_buf())).unwrap();

    let db_file = tmp.path().join("myorg__myproj.db");
    assert!(
        db_file.exists(),
        "DB file must be named <org>__<project>.db (double underscore), got: {:?}",
        tmp.path()
            .read_dir()
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect::<Vec<_>>()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// (d) Migration — old KV cache file present → new schema created, no crash
// ─────────────────────────────────────────────────────────────────────────────

/// Old-style 5-minute KV DB file with the old schema (playbook_bullets + sync_state)
/// is present at the same location → `GraphCache::new()` creates the new schema
/// without crashing. This simulates the migration path from the old `LocalCacheService`.
#[test]
fn test_migration_old_kv_schema_no_crash() {
    let tmp = TempDir::new().unwrap();

    // Pre-create the DB file with the OLD 5-min-KV schema (playbook_bullets + sync_state).
    {
        let old_db_path = tmp.path().join("org1__proj1.db");
        let conn = rusqlite::Connection::open(&old_db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS playbook_bullets (
                id TEXT PRIMARY KEY,
                section TEXT NOT NULL,
                content TEXT NOT NULL,
                helpful REAL DEFAULT 0,
                harmful REAL DEFAULT 0,
                confidence REAL DEFAULT 0.5,
                observations REAL DEFAULT 0,
                evidence TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                last_used TEXT,
                synced_at TEXT
            );
            CREATE TABLE IF NOT EXISTS sync_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            INSERT INTO sync_state (key, value, updated_at) VALUES ('last_sync', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
            ",
        )
        .unwrap();
    }

    // Opening GraphCache on the same file must succeed (CREATE TABLE IF NOT EXISTS
    // adds new tables without destroying old data).
    let result = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf()));
    assert!(
        result.is_ok(),
        "GraphCache::new() must not crash on a DB file that contains the old KV schema: {:?}",
        result.err()
    );

    let cache = result.unwrap();

    // New tables must be usable.
    let now_ms = chrono::Utc::now().timestamp_millis();
    insert_pattern_at(&cache, "migrated-pat", now_ms);
    assert!(cache.get_pattern("migrated-pat").unwrap().is_some());
}

// ─────────────────────────────────────────────────────────────────────────────
// (e) Schema DDL — tables and indexes exist after init
// ─────────────────────────────────────────────────────────────────────────────

/// After `GraphCache::new()`, both `patterns` and `edges` tables must exist,
/// and the required indexes must be present.
#[test]
fn test_schema_tables_and_indexes_exist() {
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    // Validate via the raw connection helper exposed for testing.
    let table_names = cache.list_table_names().unwrap();
    assert!(
        table_names.contains(&"patterns".to_string()),
        "patterns table must exist"
    );
    assert!(
        table_names.contains(&"edges".to_string()),
        "edges table must exist"
    );

    let index_names = cache.list_index_names().unwrap();
    assert!(
        index_names.contains(&"idx_edges_src_weight_dst".to_string()),
        "covering index idx_edges_src_weight_dst must exist"
    );
    assert!(
        index_names.contains(&"idx_patterns_expires".to_string()),
        "index idx_patterns_expires must exist"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// (f) prune() — explicit prune removes all expired rows
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_prune_removes_expired() {
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    let now_ms = chrono::Utc::now().timestamp_millis();
    insert_pattern_at(&cache, "fresh", now_ms);
    insert_pattern_at(&cache, "expired-1", now_ms - 8 * DAY_MS);
    insert_pattern_at(&cache, "expired-2", now_ms - 10 * DAY_MS);

    let pruned = cache.prune().unwrap();
    assert_eq!(pruned, 2, "prune() must return count of removed rows");

    let total = cache.count_all_patterns().unwrap();
    assert_eq!(total, 1, "Only the fresh pattern should remain after prune");
}

// ─────────────────────────────────────────────────────────────────────────────
// (g) upsert replaces payload (idempotent)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_upsert_replaces_existing() {
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    let now_ms = chrono::Utc::now().timestamp_millis();
    cache
        .upsert_pattern_at("upsert-id", &make_payload("v1"), 0.5, now_ms)
        .unwrap();
    cache
        .upsert_pattern_at("upsert-id", &make_payload("v2"), 2.0, now_ms)
        .unwrap();

    let result = cache.get_pattern("upsert-id").unwrap().unwrap();
    assert!(result.0.contains("v2"), "Upsert must replace payload");
    assert_eq!(result.1, 2.0f64, "Upsert must replace cumulative_reward");
}

// ─────────────────────────────────────────────────────────────────────────────
// (h) WAL mode is active
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_wal_mode_is_active() {
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    let journal_mode = cache.get_journal_mode().unwrap();
    assert_eq!(
        journal_mode.to_lowercase(),
        "wal",
        "WAL journal mode must be active"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// (i) REFRESH HOOK — clearly marked placeholder
// ─────────────────────────────────────────────────────────────────────────────

/// The CO_APPLIED graph endpoint is not yet wired to the Rust SDK.
///
/// When it is wired, this test should exercise:
///   1. Call server `GET /graph/co-applied` (or equivalent) to fetch edges.
///   2. Call `cache.upsert_edge(src, dst, weight)` for each returned edge.
///   3. Call `cache.upsert_pattern_at(id, payload, reward, now_ms)` for each node.
///
/// For now, the test validates the refresh hook exists and is callable.
#[test]
fn test_refresh_hook_placeholder() {
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    // TODO(#104): Wire to actual server CO_APPLIED graph endpoint.
    // When wired, call: cache.refresh_from_server(&client, org_id, project_id).await
    //
    // The refresh hook must:
    //   a) fetch edges from server
    //   b) upsert patterns via upsert_pattern_at()
    //   c) upsert edges via upsert_edge()
    //   d) call prune() to evict expired rows

    // For now, just verify the cache is functional and the upsert API is available.
    let now_ms = chrono::Utc::now().timestamp_millis();
    assert!(cache
        .upsert_pattern_at("hook-test", &make_payload("test"), 1.0, now_ms)
        .is_ok());
    assert!(cache.upsert_edge("hook-test", "hook-test", 5).is_ok());
}

// ─────────────────────────────────────────────────────────────────────────────
// (j) Configurable TTL — issue #98 AC2
// ─────────────────────────────────────────────────────────────────────────────

/// Default TTL via `GraphCache::new()` is exactly GRAPH_TTL_MS (7 days).
#[test]
fn test_default_ttl_is_7_days() {
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new("org1", "proj1", Some(tmp.path().to_path_buf())).unwrap();

    let fetched_at: i64 = 1_000_000_000_000;
    cache
        .upsert_pattern_at(
            "default-ttl-pat",
            &make_payload("default-ttl"),
            1.0,
            fetched_at,
        )
        .unwrap();

    let expires_at = cache.get_expires_at_ms("default-ttl-pat").unwrap().unwrap();
    assert_eq!(
        expires_at,
        fetched_at + TTL_7D_MS,
        "Default TTL must be 604_800_000 ms (7 days)"
    );
}

/// Custom TTL via `GraphCache::new_with_ttl()`: a short 1-hour TTL causes
/// a pattern upserted 2 hours ago to be expired.
#[test]
fn test_custom_ttl_short_expires_old_pattern() {
    use ace_sdk_core::cache::{GraphCache, GRAPH_TTL_MS};

    let one_hour_ms: i64 = 3_600_000;
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new_with_ttl(
        "org1",
        "proj1",
        Some(tmp.path().to_path_buf()),
        Some(one_hour_ms),
    )
    .unwrap();

    let now_ms = chrono::Utc::now().timestamp_millis();
    // Upserted 2 hours ago — should be expired under 1-hour TTL.
    let fetched_2h_ago = now_ms - 2 * one_hour_ms;
    cache
        .upsert_pattern_at(
            "short-ttl-pat",
            &make_payload("short-ttl"),
            1.0,
            fetched_2h_ago,
        )
        .unwrap();

    let result = cache.get_pattern("short-ttl-pat").unwrap();
    assert!(
        result.is_none(),
        "Pattern upserted 2h ago must be expired under a 1-hour TTL"
    );

    // Sanity: the default 7-day constant is still 604_800_000.
    assert_eq!(GRAPH_TTL_MS, 604_800_000);
}

/// Custom TTL via `GraphCache::new_with_ttl()`: a long 30-day TTL keeps
/// a pattern upserted 8 days ago alive (would be expired under the default 7d TTL).
#[test]
fn test_custom_ttl_long_keeps_pattern_alive() {
    let thirty_days_ms: i64 = 30 * DAY_MS;
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new_with_ttl(
        "org1",
        "proj1",
        Some(tmp.path().to_path_buf()),
        Some(thirty_days_ms),
    )
    .unwrap();

    let now_ms = chrono::Utc::now().timestamp_millis();
    // Upserted 8 days ago — would be expired under 7d TTL, but alive under 30d TTL.
    let fetched_8d_ago = now_ms - 8 * DAY_MS;
    cache
        .upsert_pattern_at(
            "long-ttl-pat",
            &make_payload("long-ttl"),
            1.0,
            fetched_8d_ago,
        )
        .unwrap();

    let result = cache.get_pattern("long-ttl-pat").unwrap();
    assert!(
        result.is_some(),
        "Pattern upserted 8d ago must be a HIT under a 30-day TTL"
    );
}

/// `None` TTL in `new_with_ttl()` falls back to the default 7-day TTL.
#[test]
fn test_none_ttl_falls_back_to_default() {
    let tmp = TempDir::new().unwrap();
    let cache = GraphCache::new_with_ttl(
        "org1",
        "proj1",
        Some(tmp.path().to_path_buf()),
        None, // explicit None → must use GRAPH_TTL_MS
    )
    .unwrap();

    let fetched_at: i64 = 2_000_000_000_000;
    cache
        .upsert_pattern_at("none-ttl-pat", &make_payload("none-ttl"), 1.0, fetched_at)
        .unwrap();

    let expires_at = cache.get_expires_at_ms("none-ttl-pat").unwrap().unwrap();
    assert_eq!(
        expires_at,
        fetched_at + TTL_7D_MS,
        "None TTL must fall back to GRAPH_TTL_MS (604_800_000 ms)"
    );
}
