---
title: Caching
description: SQLite graph cache (ACE 1.5), local pattern cache, and session storage
---

## Overview

ACE 1.5 introduces the **SQLite graph cache** (`GraphCache`) as the primary
local data layer. It replaces the old 5-minute KV cache with a 7-day TTL,
2-hop neighbour queries, and per-`(org, project)` DB file isolation — the same
schema the ace-desktop Brain-Graphs reader depends on.

The full caching stack in ACE 1.5:

| Layer | Type | TTL | Use |
|-------|------|-----|-----|
| `GraphCache` | SQLite (patterns + edges) | 7 days | Search results, 2-hop neighbours, Brain-Graphs |
| `LocalCacheService` | SQLite (playbook + sync state) | configurable | Playbook bullets, session-local notes |
| `SessionStorage` | SQLite (pinned results) | 24 h | Context-compaction, multi-agent session pins |
| RAM | In-memory | process lifetime | Playbook copy held by `AceClient` |

---

## GraphCache (ACE 1.5)

`GraphCache` is the new local data source for the co-application graph. Its
schema is **byte-identical** across all five language SDKs so ace-desktop can
read any language's cache file.

### Schema

```sql
CREATE TABLE IF NOT EXISTS patterns (
  pattern_id        TEXT PRIMARY KEY,
  payload_json      TEXT    NOT NULL,
  cumulative_reward REAL    NOT NULL DEFAULT 0,
  fetched_at_ms     INTEGER NOT NULL,
  expires_at_ms     INTEGER NOT NULL   -- fetched_at_ms + 604_800_000 (7 days)
);
CREATE TABLE IF NOT EXISTS edges (
  src    TEXT    NOT NULL,
  dst    TEXT    NOT NULL,
  weight INTEGER NOT NULL,
  PRIMARY KEY (src, dst)
);
CREATE INDEX IF NOT EXISTS idx_edges_src_weight_dst ON edges(src, weight, dst);
CREATE INDEX IF NOT EXISTS idx_patterns_expires     ON patterns(expires_at_ms);
-- WAL mode is set at connection open
```

### Opening the cache

```rust
use ace_sdk_core::cache::GraphCache;

let cache = GraphCache::new(
    "org_123",
    "prj_456",
    None, // default: ~/.ace-cache/<org>__<project>.db
)?;
```

The DB file path follows the double-underscore convention:
`~/.ace-cache/<org_id>__<project_id>.db`.

If an old 5-minute-KV schema file exists at that path the new tables are
added with `CREATE TABLE IF NOT EXISTS` — no crash, no data loss (migration).

### Facade API

| Method | Description |
|--------|-------------|
| `get_pattern(id)` | Returns `(payload_json, cumulative_reward)` if non-expired; lazily prunes expired rows. |
| `neighbors(id, hops=2, min_weight=5)` | Flat 2-hop join (see query below). Returns `Vec<(pattern_id, payload_json, cumulative_reward)>`. |
| `upsert_pattern(id, payload_json, cumulative_reward)` | Write/refresh a pattern with current-time TTL. |
| `upsert_pattern_at(id, payload_json, cumulative_reward, fetched_at_ms)` | Write with explicit timestamp (test-friendly). |
| `upsert_edge(src, dst, weight)` | Write a directed co-application edge. |
| `prune()` | Explicitly evict expired patterns and orphan edges. Returns number of rows deleted. |

### 2-hop neighbour query

The neighbours query uses a flat `UNION` join (not a recursive CTE) for
p95 latency of ~0.17 ms at typical dataset sizes versus ~48 ms for the CTE
form. The covering index `idx_edges_src_weight_dst` makes both hops index-only.

```rust
// Get 2-hop co-applied neighbours, min edge weight 5, non-expired only
let neighbours = cache.neighbors("pat_abc123", 2, 5)?;
for (pattern_id, payload_json, reward) in neighbours {
    println!("{pattern_id}: reward={reward:.2}");
}
```

### Automatic population on search

`AceClient::search_patterns15` upserts every returned pattern into the graph
cache automatically after a successful search. You do not need to call
`upsert_pattern` manually for search results.

```rust
// After this call the returned patterns are in the graph cache (7-day TTL)
let response = client.search_patterns15(request).await?;
```

Accessing the cache directly from the client:

```rust
if let Some(gc_arc) = client.get_graph_cache() {
    let gc = gc_arc.lock().unwrap();
    if let Ok(Some((payload, reward))) = gc.get_pattern("pat_abc123") {
        println!("cached, reward={reward}");
    }
}
```

### Server refresh hook (TODO)

Edges (co-application graph links) will be pulled from the server
`/graph/co-applied` endpoint when it ships. The refresh hook in
`GraphCache` is marked `TODO(#104)` — the cache layer itself is fully wired
and unit-tested; only the server-side edge fetch is pending.

---

## LocalCacheService

Stores the playbook and sync metadata. TTL is configurable (default from
`AceConfig.cache_ttl_minutes`).

```rust
use ace_sdk_core::cache::LocalCacheService;

let cache = LocalCacheService::new(
    "org_123",
    "prj_456",
    10,  // TTL in minutes
    None // default cache dir: ~/.ace-cache/
)?;
```

---

## SessionStorage

Persistent pattern pinning for context compaction (24-hour TTL).

```rust
use ace_sdk_core::cache::{SessionStorage, SessionStorageConfig};

let storage = SessionStorage::new(Some(SessionStorageConfig {
    cache_dir: None, // default: ~/.ace-cache/
}))?;

// Pin patterns to a session
storage.pin_session("sess-1", "error handling", &patterns, 0.7, 10)?;

// Recall later without a server round-trip
if let Some(result) = storage.recall_session("sess-1")? {
    println!("Recalled {} patterns", result.count);
}

// List active sessions
let sessions = storage.list_sessions()?;
```

---

## ProjectIndex

SQLite-based file index for smart bootstrap file selection.

```rust
use ace_sdk_core::cache::{ProjectIndex, ProjectIndexConfig};

let index = ProjectIndex::new(ProjectIndexConfig {
    org_id: "org_123".to_string(),
    project_id: "prj_456".to_string(),
    cache_dir: None,
})?;

let hubs = index.get_hub_files(20);
let entries = index.get_entry_points();
let stats = index.get_stats();
println!("Files: {}, Hubs: {}", stats.total_files, stats.hub_files);
```

---

## File paths at a glance

| File | Purpose |
|------|---------|
| `~/.ace-cache/<org>__<project>.db` | GraphCache + LocalCacheService (double underscore) |
| `~/.ace-cache/<org>__<project>_index.db` | ProjectIndex |
| `~/.ace-cache/sessions.db` | SessionStorage |
