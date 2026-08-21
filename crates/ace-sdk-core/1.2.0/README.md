# ace-sdk-core

Rust client library for the ACE (Agentic Context Engineering) API — **ACE 1.5**.

[![crates.io](https://img.shields.io/crates/v/ace-sdk-core.svg)](https://crates.io/crates/ace-sdk-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Installation

```toml
[dependencies]
ace-sdk-core = "0.5"
tokio = { version = "1", features = ["full"] }
```

## What's new in ACE 1.5

ACE 1.5 is a **clean-break MAJOR** release (no 1.0 backward-compat shim in
the write path). Prior 1.0 patterns stored on the server are still readable —
missing 1.5 fields default to `None`.

| Area | Change |
|------|--------|
| Reward model | Tier-based (`n_hot/warm/cold_pos/neg` counters + `cumulative_v15_reward`). `is_at_risk()` returns `true` when `cumulative_v15_reward == 0.0`. `helpful` / `harmful` are deprecated read-only derived getters. |
| Search | `search_patterns15()` returns `SearchResponse15` with typed `Pattern`, top-level `retrieval_id`, per-result `match_factors`, and `expanded` 2-hop graph-cache neighbours. |
| Search request | Optional `task_intent`, `exploration_enabled`, `exploration_rate`, `session_id`, and client-side `expand_neighbors` fields. |
| F-080 feedback | `ExecutionTrace` gains `retrieval_id` + `applied_log_ids` — close the search→apply→learn cycle automatically. `session_id` on `SearchRequest15` links the retrieval row for session-scoped credit. |
| Graph cache | `GraphCache` (SQLite, 7-day TTL, flat 2-hop `neighbors()`, per-`(org,project)` DB at `~/.ace-cache/<org>__<project>.db`). `refresh_from_server` pulls CO_APPLIED edges from `GET /patterns/graph` — project-scoped, requires `X-ACE-Project`. Throttle configurable via `AceClientOptions::graph_edges_throttle_ms` (default 1 h). Same schema as all other language SDKs — local data source for ace-desktop Brain-Graphs. |

## Features

- **ACE 1.5 Reward Model** — tier counters (`n_hot/warm/cold_pos/neg`), `cumulative_v15_reward`, `PatternEffectiveness`, `is_at_risk()`; legacy `helpful`/`harmful` are deprecated derived getters
- **`search_patterns15`** — native 1.5 search with `match_factors`, top-level `retrieval_id`, optional `session_id`, and automatic populate path (pattern upsert + throttled edge refresh + `expanded` 2-hop neighbours)
- **`expanded` neighbors** — `SearchResponse15::expanded` carries up to 2-hop graph-cache neighbours as id-only stubs, deduped against primary results; controlled per-call via `SearchRequest15::expand_neighbors`
- **F-080 Feedback Loop** — `retrieval_id` + `applied_log_ids` on `ExecutionTrace`; `session_id` on search links retrieval for session-scoped credit
- **SQLite Graph Cache** — `GraphCache` (7-day TTL, flat 2-hop `neighbors()`, per-`(org,project)` DB file at `~/.ace-cache/<org>__<project>.db`); `refresh_from_server` pulls CO_APPLIED edges from `GET /patterns/graph` (project-scoped — requires `X-ACE-Project`); fire-and-forget throttle tunable via `AceClientOptions::graph_edges_throttle_ms`
- **Async HTTP Client** — reqwest + tokio
- **Device Code Auth** — RFC 8628 with automatic token refresh
- **Flexible Configuration** — CLI args → env vars → XDG config file

## Quick Start (ACE 1.5)

```rust,no_run
use ace_sdk_core::{AceClient, AceClientOptions, AceConfig};
use ace_sdk_core::types::{ExecutionTrace, ExecutionResult, SearchRequest15};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), ace_sdk_core::AceError> {
    let config = AceConfig {
        server_url: "https://ace-api.code-engine.app".to_string(),
        api_token: "ace_user_xxx".to_string(),
        project_id: "my-project".to_string(),
        org_id: Some("org_xxx".to_string()),
        ..Default::default()
    };

    let client = AceClient::new(config, AceClientOptions::default())?;

    // ── 1. ACE 1.5 search ─────────────────────────────────────────────────
    let now = chrono::Utc::now().to_rfc3339();
    let request = SearchRequest15 {
        pattern: json!({
            "id": "tmp", "content": "error handling", "confidence": 0.8,
            "created_at": now, "section": "general"
        }),
        top_k: Some(10),
        task_intent: Some("refactor".to_string()), // optional ranking hint
        threshold: None,
        exploration_enabled: None,
        exploration_rate: None,
        // Optional: link search → trace for F-080 session-scoped credit
        session_id: Some("sess_abc123".to_string()),
    };
    let response = client.search_patterns15(request).await?;
    // Patterns are also upserted into GraphCache (7-day TTL) automatically.

    let retrieval_id = response.retrieval_id.clone();
    let mut applied_log_ids: Vec<i64> = Vec::new();

    for pattern in &response.similar_patterns {
        println!(
            "[{}] reward={:.2}  at_risk={}",
            pattern.id,
            pattern.cumulative_v15_reward.unwrap_or(0.0),
            pattern.is_at_risk(),
        );
        // Collect F-080 keys for patterns you actually apply
        if let Some(ref mf) = pattern.match_factors {
            if let Some(log_id) = mf.retrieval_log_id {
                applied_log_ids.push(log_id);
            }
        }
    }

    // ── 2. Refresh graph edges from server, then query 2-hop neighbours ──────
    if let Some(gc_arc) = client.get_graph_cache() {
        let gc = gc_arc.lock().unwrap();

        // Pull CO_APPLIED topology from GET /patterns/graph (best-effort).
        // A warm project returns ~11 500 edges at min_weight=5.
        let refresh = gc
            .refresh_from_server(
                client.http_client(),
                "https://ace-api.code-engine.app",
                client.auth_headers(),
                None,  // min_weight defaults to 5
                None,  // since_ms — full refresh
            )
            .await?;
        println!("graph edges loaded: {} (truncated: {})", refresh.edges_upserted, refresh.truncated);

        // neighbors() now returns live server topology — no extra network call
        if let Some(first) = response.similar_patterns.first() {
            let neighbours = gc.neighbors(&first.id, 2, 5)?;
            println!("2-hop neighbours of {}: {}", first.id, neighbours.len());
        }
    }

    // ── 3. Store trace with F-080 feedback ────────────────────────────────
    client.store_execution_trace(&ExecutionTrace {
        task: "Fix error handling".to_string(),
        trajectory: vec![],
        result: ExecutionResult {
            success: true,
            output: "Done".to_string(),
            error: None,
            summary: None,
        },
        playbook_used: vec![],
        timestamp: chrono::Utc::now().to_rfc3339(),
        git: None,
        session_id: Some("sess_abc123".to_string()), // same as search — closes F-080 loop
        agent_id: None,
        agent_type: None,
        parent_agent_id: None,
        retrieval_id,
        applied_log_ids: if applied_log_ids.is_empty() { None } else { Some(applied_log_ids) },
    }).await?;

    Ok(())
}
```

## Deprecated: helpful / harmful

`helpful` and `harmful` are **read-only derived getters** in ACE 1.5, not
primary API. They compute from tier counters using COLD weight 0.1:

```
legacy_helpful() = n_hot_pos * 1.0 + n_warm_pos * 0.7 + n_cold_pos * 0.1
legacy_harmful() = n_hot_neg * 1.0 + n_warm_neg * 0.7 + n_cold_neg * 0.1
```

Do not send `helpful_delta` / `harmful_delta` in any write path — those vote
fields are intentionally absent from `ExecutionTrace`.

## Org Usage Analytics

```rust,no_run
use ace_sdk_core::types::UsageWindow;

let resp = client.get_org_usage_hourly("org_abc", UsageWindow::OneDay, None).await?;
if let Some(b) = resp.buckets.first() {
    println!("{}: {}", b.period, b.api_calls_total);
}
```

## 0.3.0 type rename (non-breaking)

- `UsageHistoryWindow` → `UsageWindow`
- `UsageHistoryBucket` → `UsageBucket`
- `UsageHistoryGranularity` → `UsageGranularity`

Old names are preserved as aliases.

## Documentation

Full documentation: [sdks/rust/core/docs](./docs)

## License

MIT © [CE.NET Team](mailto:ace@code-engine.net)
