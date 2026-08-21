---
title: ACE Client
description: Using the main API client (ACE 1.5)
---

## Creating a Client

```rust
use ace_sdk_core::{AceClient, AceClientOptions, AceConfig};

let config = AceConfig {
    server_url: "https://ace-api.code-engine.app".to_string(),
    api_token: "ace_user_...".to_string(),
    project_id: "prj_...".to_string(),
    org_id: Some("org_...".to_string()),
    ..Default::default()
};

let client = AceClient::new(config, AceClientOptions::default())?;
```

The client automatically initializes a `GraphCache` for the `(org, project)`
pair so search results are persisted to the local SQLite graph on every
`search_patterns15` call.

---

## Operations

### Get Playbook

```rust
let playbook = client.get_playbook(false).await?;
for bullet in &playbook.playbook.strategies_and_hard_rules {
    println!("{} (confidence: {})", bullet.content, bullet.confidence);
}
```

### Search Patterns — ACE 1.5 native (`search_patterns15`)

Use `search_patterns15` for the full ACE 1.5 reward model, `match_factors`,
and F-080 feedback data. The returned patterns are also upserted into the local
`GraphCache` automatically.

```rust
use ace_sdk_core::types::SearchRequest15;
use serde_json::json;

let now = chrono::Utc::now().to_rfc3339();
let request = SearchRequest15 {
    pattern: json!({
        "id": "tmp", "content": "error handling", "confidence": 0.8,
        "created_at": now, "section": "general"
    }),
    top_k: Some(10),
    threshold: None,
    // Optional: bias ranking toward refactoring tasks
    task_intent: Some("refactor".to_string()),
    // Optional: enable bandit exploration
    exploration_enabled: Some(true),
    exploration_rate: None, // use server default
    // Optional: link this search to a session for F-080 session-scoped credit
    session_id: Some("sess_abc123".to_string()),
};

let response = client.search_patterns15(request).await?;

// Top-level search-scoped UUID — capture for F-080
let retrieval_id = response.retrieval_id.clone();

for pattern in &response.similar_patterns {
    println!(
        "[{}] reward={:.2}  at_risk={}  recommendation={:?}",
        pattern.id,
        pattern.cumulative_v15_reward.unwrap_or(0.0),
        pattern.is_at_risk(),
        pattern.effectiveness.as_ref()
            .and_then(|e| e.recommendation.as_ref()),
    );
    // match_factors — present only when LinUCB is warm
    if let Some(ref mf) = pattern.match_factors {
        println!(
            "  semantic={:.3}  ucb={:?}  bandit_rank={:?}  log_id={:?}",
            mf.semantic_score.unwrap_or(0.0),
            mf.ucb_score,
            mf.bandit_rank,
            mf.retrieval_log_id,
        );
    }
}
```

#### task_intent values

| Value | Meaning |
|-------|---------|
| `"refactor"` | Refactoring or code restructuring tasks |
| `"routine"` | Routine / maintenance tasks |
| `"explore"` | Exploratory or open-ended tasks |
| `"spec_strict"` | Strict specification / compliance tasks |

Omit `task_intent` (pass `None`) to let the server apply its default ranking.

#### session_id — F-080 session-scoped credit

Pass `session_id` in `SearchRequest15` to stamp the server-side
`retrieval_log_v15.session_id` column. This links the search retrieval row
to the subsequent `store_trace` call that carries the same `session_id`,
enabling ACE to attribute reward credit correctly within a session (F-080
session-scoped credit).

```rust
let request = SearchRequest15 {
    pattern: json!({ "id": "tmp", "content": "auth bug", "confidence": 0.8,
                     "created_at": now, "section": "general" }),
    top_k: Some(5),
    threshold: None,
    task_intent: None,
    exploration_enabled: None,
    exploration_rate: None,
    session_id: Some("sess_abc123".to_string()), // links search → trace
};
let response = client.search_patterns15(request).await?;

// Later, store the trace with the SAME session_id
client.store_trace(&ExecutionTrace {
    session_id: Some("sess_abc123".to_string()),
    retrieval_id: response.retrieval_id,
    // ... other fields
    ..Default::default()
}).await?;
```

When `session_id` is `None` (the field is absent from the JSON body) the
server leaves the column `NULL` — no credit linking occurs. Omit it for
one-off searches that are not part of a named session.

### Search Patterns — Legacy (`search_patterns`)

The original `search_patterns` method remains available for compatibility. It
returns `SearchResponseWithMetadata` (typed around `PlaybookBullet` rather than
`Pattern`) and does not populate `match_factors` or `retrieval_id`.

```rust
let results = client.search_patterns("error handling", None, Some(10), None, None).await?;
for pattern in &results.similar_patterns {
    println!("{}", pattern.content);
}
```

### Store Execution Trace (with F-080)

See the [Traces guide](/ace-sdk/rust/core/guide/traces/) for the full F-080 round-trip example.

```rust
use ace_sdk_core::types::{ExecutionTrace, ExecutionResult};

let trace = ExecutionTrace {
    task: "Fix login bug".to_string(),
    trajectory: vec![],
    result: ExecutionResult {
        success: true,
        output: "Fixed".to_string(),
        error: None,
        summary: None,
    },
    playbook_used: vec![],
    timestamp: chrono::Utc::now().to_rfc3339(),
    git: None,
    session_id: None,
    agent_id: None,
    agent_type: None,
    parent_agent_id: None,
    // F-080 (ACE 1.5) — omit when not using search_patterns15
    retrieval_id: None,
    applied_log_ids: None,
};
client.store_trace(&trace).await?;
```

### Access the Graph Cache

```rust
if let Some(gc_arc) = client.get_graph_cache() {
    let gc = gc_arc.lock().unwrap();

    // Direct pattern lookup (7-day TTL, lazy prune on miss)
    if let Ok(Some((payload, reward))) = gc.get_pattern("pat_abc123") {
        println!("hit: reward={reward}");
    }

    // 2-hop neighbours (min edge weight 5)
    let neighbours = gc.neighbors("pat_abc123", 2, 5)?;
    for (pid, _payload, reward) in neighbours {
        println!("  neighbour {pid}: reward={reward:.2}");
    }
}
```

#### Refreshing edge topology from the server

After initialization, call `refresh_from_server` to populate the `edges`
table from `GET /patterns/graph`. This is what feeds the 2-hop neighbours
query and the ace-desktop Brain-Graph.

```rust
if let Some(gc_arc) = client.get_graph_cache() {
    let gc = gc_arc.lock().unwrap();
    let result = gc
        .refresh_from_server(
            client.http_client(),
            &client.base_url(),
            client.auth_headers(),
            None,  // min_weight defaults to 5
            None,  // since_ms — full refresh
        )
        .await?;
    println!("graph edges loaded: {}", result.edges_upserted);
}
```

See the [Caching guide](/ace-sdk/rust/core/guide/caching/#refreshing-graph-edges-from-the-server-refresh_from_server)
for the full parameter reference and best-effort semantics.

---

## Multi-Agent Attribution

Populate `session_id`, `agent_id`, `agent_type`, and `parent_agent_id` to
group traces by session and track parent/child relationships.

```rust
use ace_sdk_core::types::{ExecutionTrace, ExecutionResult};

let trace = ExecutionTrace {
    task: "Review PR #42".to_string(),
    trajectory: vec![],
    result: ExecutionResult {
        success: true,
        output: "LGTM".to_string(),
        error: None,
        summary: None,
    },
    playbook_used: vec![],
    timestamp: chrono::Utc::now().to_rfc3339(),
    git: None,
    session_id: Some("sess_abc123".to_string()),
    agent_id: Some("agent_reviewer_1".to_string()),
    agent_type: Some("reviewer".to_string()),
    parent_agent_id: Some("agent_orchestrator".to_string()),
    retrieval_id: None,
    applied_log_ids: None,
};
client.store_trace(&trace).await?;
```

---

## TaskSession (F-080 helper)

`TaskSession` is the ergonomic alternative to wiring `retrieval_id` and
`applied_log_ids` by hand. It stores a per-pin JSON file after every search and
reads back the accumulated union at trace time — safely across separate OS
processes (e.g. a `SubagentStart` hook in process A and a `SubagentStop` hook
in process B).

All public types are re-exported from the crate root:

```rust
use ace_sdk_core::{
    begin_task_session, load_task_session, anchor_trace, read_f080,
    TaskSession, TaskSessionOptions, F080View,
};
```

### Typical single-process flow

```rust
use ace_sdk_core::{
    AceClient, AceClientOptions, AceConfig,
    begin_task_session, TaskSessionOptions,
};
use ace_sdk_core::types::{ExecutionTrace, ExecutionResult, SearchRequest15};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), ace_sdk_core::AceError> {
    let config = AceConfig {
        server_url: "https://ace-api.code-engine.app".to_string(),
        api_token:  "ace_user_...".to_string(),
        project_id: "prj_abc".to_string(),
        org_id:     Some("org_123".to_string()),
        ..Default::default()
    };
    let client = AceClient::new(config, AceClientOptions::default())?;

    // ── Step 1: begin — one TaskSession per task ───────────────────────────
    let ts = begin_task_session("org_123", "prj_abc", None);

    // ── Step 2: search, then pin the response ─────────────────────────────
    let now = chrono::Utc::now().to_rfc3339();
    let response = client.search_patterns15(SearchRequest15 {
        pattern: json!({
            "id": "tmp", "content": "retry logic", "confidence": 0.8,
            "created_at": now, "section": "general"
        }),
        top_k: Some(10),
        // Pass the task session_id so the server links this retrieval row
        session_id: Some(ts.session_id.clone()),
        threshold: None,
        task_intent: None,
        exploration_enabled: None,
        exploration_rate: None,
        expand_neighbors: None,
    }).await?;

    ts.pin_search(&response); // persists retrieval_id + log_ids to disk

    // ── Step 3: (agent does work) ──────────────────────────────────────────

    // ── Step 4: anchor — stamps retrieval_id + applied_log_ids onto trace ─
    let trace = ExecutionTrace {
        task:       "Implement retry logic".to_string(),
        trajectory: vec![],
        result: ExecutionResult {
            success: true,
            output:  "Done".to_string(),
            error:   None,
            summary: None,
        },
        playbook_used: vec![],
        timestamp: chrono::Utc::now().to_rfc3339(),
        git: None,
        // Leave session_id/retrieval_id/applied_log_ids as None —
        // anchor_trace fills them from the pinned anchor.
        session_id:      None,
        agent_id:        None,
        agent_type:      None,
        parent_agent_id: None,
        retrieval_id:    None,
        applied_log_ids: None,
    };

    let trace = ts.anchor_trace(trace); // fills + reaps pin files
    client.store_trace(&trace).await?;
    Ok(())
}
```

### Multi-process re-entry (SubagentStop hook)

When the search runs in process A (e.g. `SubagentStart` hook) and the
`store_trace` runs in process B (e.g. `SubagentStop` hook), there is no
shared memory. Use `load_task_session` in process B to bind by `session_id`,
or call the module-level `anchor_trace` convenience function which does the
same in one line:

```rust
use ace_sdk_core::{anchor_trace, TaskSessionOptions};
use ace_sdk_core::types::{ExecutionTrace, ExecutionResult};

// In process B — only (org, project, session_id) are known from the stop event.
let session_id = "the-same-id-that-process-a-used".to_string();

let trace = ExecutionTrace {
    task:            "Implement retry logic".to_string(),
    trajectory:      vec![],
    result: ExecutionResult {
        success: true,
        output:  "Done".to_string(),
        error:   None,
        summary: None,
    },
    playbook_used:   vec![],
    timestamp:       chrono::Utc::now().to_rfc3339(),
    git:             None,
    session_id:      Some(session_id),
    agent_id:        None,
    agent_type:      None,
    parent_agent_id: None,
    retrieval_id:    None, // filled by anchor_trace
    applied_log_ids: None, // filled by anchor_trace
};

// Module-level convenience — derives session from trace.session_id.
let trace = anchor_trace("org_123", "prj_abc", trace, None);
client.store_trace(&trace).await?;
```

### Injecting a caller-provided session_id

If your plugin framework supplies a stable sub-agent identity token, pass it
via `TaskSessionOptions::session_id` so process A and process B both refer to
the same anchor files:

```rust
use ace_sdk_core::{begin_task_session, TaskSessionOptions};

let ts = begin_task_session(
    "org_123",
    "prj_abc",
    Some(TaskSessionOptions {
        session_id: Some("my-plugin-subagent-id".to_string()),
        ..Default::default()
    }),
);
// ts.session_id == "my-plugin-subagent-id"
```

### Non-reaping peek with `read_f080`

`read_f080` computes the same view that `anchor_trace` would stamp **without
deleting** any pin files. Use it for debugging or when you need the F-080 fields
but want to keep the anchor alive for a later `anchor_trace` call:

```rust
use ace_sdk_core::read_f080;

let view = read_f080("org_123", "prj_abc", &ts.session_id, None);
println!("retrieval_id:    {:?}", view.retrieval_id);
println!("applied_log_ids: {:?}", view.applied_log_ids);
// Pin files are NOT deleted — anchor_trace can still be called later.
```

### Anchor store — where files live

```
~/.ace-cache/sessions/<org>__<project>/<session_id>__<pin_uuid>.json
```

Each `pin_search` call writes one file. `anchor_trace` globs all files matching
`<session_id>__*.json`, unions their `retrieval_log_ids`, and reaps them all.
The 24-hour TTL is configurable via `TaskSessionOptions::ttl_ms`. See the
[Caching guide](/ace-sdk/rust/core/guide/caching/#tasksession-anchor-store-f-080)
for the full anchor JSON schema and GC details.

---

## get_last_usage

Returns the most recent `UsageInfo` parsed from `X-ACE-*` response headers,
or `None` until the first authenticated response containing `X-ACE-Plan`
arrives.

```rust
pub async fn get_last_usage(&self) -> Option<UsageInfo>
```

```rust
let _ = client.get_playbook(false).await?;

if let Some(usage) = client.get_last_usage().await {
    println!("Plan: {} ({:?}/{:?})", usage.plan, usage.subscription_type, usage.plan_tier);
    println!("API calls: {}/{}", usage.api_calls.used, usage.api_calls.limit);
}
```

### Parsed headers

| Header | Field |
|--------|-------|
| `X-ACE-Plan` | `plan`, `subscription_type`, `plan_tier` (required — gates parsing) |
| `X-ACE-Status` | `status` |
| `X-ACE-Patterns` | `patterns` |
| `X-ACE-Patterns-Total` | `patterns_total` |
| `X-ACE-Projects` | `projects` |
| `X-ACE-Domains` | `domains` |
| `X-ACE-Templates` | `templates` |
| `X-ACE-API-Calls` | `api_calls` |
| `X-ACE-Traces` | `traces_today` |
| `X-ACE-Subscription-Updated-At` | `subscription_updated_at` |

---

## get_org_usage_hourly

Fetch hourly/daily usage buckets for a specific org.

```rust
pub async fn get_org_usage_hourly(
    &self,
    org_id: &str,
    window: UsageWindow,
    project_id: Option<&str>,
) -> Result<UsageHistoryResponse, AceError>
```

| Parameter | Description |
|-----------|-------------|
| `org_id` | Organization ID. Sent via `X-ACE-Org`, overriding the client default. |
| `window` | `"1h"` / `"6h"` / `"12h"` / `"1d"` / `"7d"` / `"14d"` / `"30d"` |
| `project_id` | Restrict to one project; `None` for org-wide totals. |

```rust
use ace_sdk_core::types::UsageWindow;

let usage = client
    .get_org_usage_hourly("org_abc123", UsageWindow::SixHours, None)
    .await?;

if let Some(first) = usage.buckets.first() {
    println!("Period: {}, API calls: {}", first.period, first.api_calls_total);
}
```

### 0.3.0 type rename (non-breaking)

- `UsageHistoryWindow` → `UsageWindow`
- `UsageHistoryBucket` → `UsageBucket`
- `UsageHistoryGranularity` → `UsageGranularity`

Old names are preserved as aliases — existing code compiles without changes.
