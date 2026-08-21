---
title: Caching
description: SQLite graph cache (ACE 1.5), local pattern cache, session storage, and TaskSession F-080 anchor store
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
| `TaskSession` anchor store | Per-pin JSON files | 24 h (configurable) | F-080 retrieval_id + retrieval_log_ids, multi-process accumulation |
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

### File location

The DB file uses a double-underscore separator:

```
~/.ace-cache/<org_id>__<project_id>.db
```

One file per `(org, project)` pair — project-scoped. The org and project IDs
come from `AceConfig::default_org_id` and `AceConfig::project_id`
respectively. All five language SDKs write the same schema to this file, so
ace-desktop can read any SDK's cache directly.

### Opening the cache

```rust
use ace_sdk_core::cache::GraphCache;

let cache = GraphCache::new(
    "org_123",
    "prj_456",
    None, // default: ~/.ace-cache/org_123__prj_456.db
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

`AceClient::search_patterns15` runs three best-effort cache steps after every
successful search — none of them can break the call:

1. **Upsert patterns** — every returned `Pattern` is written into the local
   `patterns` table with a 7-day TTL. You do not need to call `upsert_pattern`
   manually for search results.
2. **Throttled edge refresh** — calls `GET /patterns/graph?min_weight=5` at
   most once per `graph_edges_throttle_ms` window (default 1 hour), gated on
   `sync_state("graph_edges_synced_at")`. The HTTP task is spawned
   fire-and-forget; `search_patterns15` returns immediately after stamping the
   optimistic marker.
3. **Expand neighbors** — runs the 2-hop `neighbors()` query and attaches
   results to `SearchResponse15::expanded` (deduped against primary hits,
   id-only stubs). Disabled when `SearchRequest15::expand_neighbors = Some(false)`.

```rust
// All three populate steps happen automatically.
let response = client.search_patterns15(request).await?;

// Inspect the expanded 2-hop neighbours attached client-side:
if let Some(expanded) = &response.expanded {
    for entry in expanded {
        println!(
            "neighbour {} (reward={:.2}, cached={})",
            entry.pattern_id, entry.cumulative_reward, entry.cached
        );
    }
}
```

Each `ExpandedNeighborEntry` is an id-only stub:

```rust
pub struct ExpandedNeighborEntry {
    pub pattern_id: String,
    pub cumulative_reward: f64,
    pub cached: bool,      // true = found in patterns table; false = edge-only
    pub payload_json: Option<String>, // always None — token-efficient path
}
```

To skip neighbour expansion on a single call:

```rust
let request = SearchRequest15 {
    expand_neighbors: Some(false), // skip step 3
    // ... other fields
    ..Default::default()
};
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

### Refreshing graph edges from the server (`refresh_from_server`)

`GraphCache::refresh_from_server` pulls CO_APPLIED edge topology from the
live server endpoint and upserts it into the local `edges` table. This is
the data source for the ace-desktop Brain-Graph view.

```rust
use ace_sdk_core::cache::GraphCache;

let cache = GraphCache::new("org_123", "prj_456", None)?;

// Call at startup or on demand — best-effort, never throws.
let result = cache
    .refresh_from_server(
        client.http_client(),          // shared reqwest::Client
        "https://ace-api.code-engine.app",
        client.auth_headers(),         // Bearer + X-ACE-Project headers
        None,                          // min_weight — defaults to 5
        None,                          // since_ms — omitted (full refresh)
    )
    .await?;

println!(
    "Refreshed {} edges (truncated: {})",
    result.edges_upserted, result.truncated
);
```

**Endpoint:** `GET /patterns/graph?min_weight=<n>[&since=<ms>]`

**Authentication:** Bearer token plus `X-ACE-Project` (required — multi-project
users receive HTTP 400 without it) and `X-ACE-Org` for user tokens. The client
injects both headers automatically from the `(org_id, project_id)` pair the
`GraphCache` was opened with.

> The `GET /patterns/graph` endpoint is **project-scoped**. Always ensure the
> `X-ACE-Project` header matches the project you are fetching edges for.

**Response shape:**
```json
{
  "edges": [
    { "src": "pat_abc", "dst": "pat_xyz", "weight": 12 },
    ...
  ],
  "truncated": false
}
```

#### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `min_weight` | `Option<i64>` | `5` | Minimum edge weight filter. The default keeps the response safely under the server's 50 000-edge cap. |
| `since_ms` | `Option<i64>` | `None` | Millisecond-epoch lower bound. Pass the last refresh timestamp for incremental updates. |

#### Return value — `GraphRefreshResult`

```rust
pub struct GraphRefreshResult {
    /// Number of edges successfully upserted into the local `edges` table.
    pub edges_upserted: usize,
    /// Whether the server indicated the response was truncated (>50 000 edges).
    pub truncated: bool,
}
```

#### Best-effort semantics

Any network failure, non-200 status, JSON parse error, or empty edge list
returns `Ok(GraphRefreshResult { edges_upserted: 0, truncated: false })` —
the method never panics or propagates errors. The local cache is left intact
on every error path.

When `truncated` is `true` a warning is printed to stderr. To narrow the
window, raise `min_weight` or pass a `since_ms` timestamp.

#### After a refresh — `neighbors()` returns live data

Once edges are in the local DB, the existing `neighbors()` call returns the
CO_APPLIED 2-hop set:

```rust
// Refresh once (e.g. at startup or on a background timer)
cache.refresh_from_server(&http, base_url, &headers, None, None).await?;

// neighbors() now reflects server edge topology — no network round-trip
let neighbours = cache.neighbors("pat_abc123", 2, 5)?;
println!("2-hop neighbours: {}", neighbours.len());
```

> **Live data:** a warm project returns approximately 11 500 edges with the
> default `min_weight=5`. The ace-desktop Brain-Graph reads the same SQLite
> file, so a `refresh_from_server` call immediately updates what the desktop
> app shows.

### Client-level cache options (`AceClientOptions`)

`AceClientOptions` lets you tune the automatic populate path at client
construction time:

```rust
use ace_sdk_core::{AceClient, AceClientOptions, AceConfig};

let options = AceClientOptions {
    // Throttle the fire-and-forget edge refresh.
    // Default: 3_600_000 ms (1 hour).
    // Set to 0 to refresh on every search call.
    graph_edges_throttle_ms: Some(1_800_000), // 30 minutes
    ..Default::default()
};

let client = AceClient::new(config, options)?;
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `graph_edges_throttle_ms` | `Option<u64>` | `3_600_000` (1 h) | Minimum milliseconds between `GET /patterns/graph` refresh calls. `0` disables throttling. |
| `auto_refresh` | `Option<bool>` | `true` for user tokens | Auto-refresh expired auth tokens. |
| `custom_headers` | `Option<HashMap<String, String>>` | `None` | Extra HTTP headers on every request. |

The `GraphCache` is opened automatically using `AceConfig::default_org_id`
(or `"default"` when absent) and `AceConfig::project_id`. To override the DB
directory (e.g. in tests) set `AceConfig::graph_cache_dir`.

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

## TaskSession anchor store (F-080)

The `TaskSession` helper writes one small JSON file **per `pin_search` call**
into a dedicated sessions directory. This is an append-only store: no file is
ever read-modified-written, so concurrent hooks on the same task accumulate
pins safely even across separate OS processes.

### File layout

```
~/.ace-cache/sessions/<safe_org>__<safe_project>/<session_id>__<pin_uuid>.json
```

- `<safe_org>` and `<safe_project>` are the org/project IDs with filesystem-unsafe
  characters (`/ \ : * ? " < > |` and NUL) replaced by `_`. The double-underscore
  separator matches the `.db` file convention.
- `<session_id>` is the per-task UUID (either generated by `begin_task_session`
  or injected by the caller for multi-process continuity).
- `<pin_uuid>` is a fresh UUID generated for each `pin_search` call, ensuring
  the two files for the same session never collide.

### Anchor JSON shape

Each file stores one `TaskAnchor` object. The schema is **byte-identical**
across all five language SDKs:

```json
{
  "session_id":        "<uuid4>",
  "org_id":            "<string>",
  "project_id":        "<string>",
  "retrieval_id":      "<string | null>",
  "retrieval_log_ids": [<i64>, ...],
  "created_at_ms":     <i64>,
  "expires_at_ms":     <i64>
}
```

### TTL and garbage collection

The default TTL is **24 hours** (`DEFAULT_ANCHOR_TTL_MS = 86_400_000`). Both
`begin_task_session` and `load_task_session` run a best-effort GC sweep on
construction, deleting anchor files whose `expires_at_ms` is in the past.
Expired pins are also pruned opportunistically inside `read_f080` and `anchor_trace`.

To extend the TTL for long-running tasks:

```rust
use ace_sdk_core::{begin_task_session, TaskSessionOptions};

let ts = begin_task_session(
    "org_123",
    "prj_456",
    Some(TaskSessionOptions {
        ttl_ms: Some(48 * 60 * 60 * 1_000), // 48 hours
        ..Default::default()
    }),
);
```

### Accumulation across processes

Because each `pin_search` call writes a separate file, a domain-shift re-pin
from a second process appends a new file rather than overwriting the original.
`anchor_trace` globs all `<session_id>__*.json` files, unions their
`retrieval_log_ids`, and picks the earliest pin's `retrieval_id` — so the
accumulated set from all pins is sent to the server in a single trace call.

See the [ACE Client guide](/ace-sdk/rust/core/guide/ace-client/#tasksession-f-080-helper)
for the full begin/pin/anchor end-to-end example.

---

## File paths at a glance

| File | Purpose |
|------|---------|
| `~/.ace-cache/<org>__<project>.db` | GraphCache + LocalCacheService (double underscore) |
| `~/.ace-cache/<org>__<project>_index.db` | ProjectIndex |
| `~/.ace-cache/sessions.db` | SessionStorage |
| `~/.ace-cache/sessions/<org>__<project>/<session_id>__<pin_uuid>.json` | TaskSession F-080 per-pin anchor |
