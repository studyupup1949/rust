---
title: Traces
description: Store and retrieve execution traces; F-080 feedback loop (ACE 1.5)
---

The traces API covers two concerns:

1. **Writing** — `store_trace` (POST `/traces`) and `store_trace_stream`
   (POST `/traces/stream`). ACE 1.5 adds F-080 fields to `ExecutionTrace`.
2. **Reading** — `list_traces` / `get_trace` for paginated retrieval and
   full trace detail.

---

## F-080 Feedback Loop (ACE 1.5)

The F-080 loop closes the search → apply → learn cycle so the server can
update pattern scores based on which patterns an agent actually used.

```
search_patterns15()
  └─ response.retrieval_id          (search-scoped UUID)
  └─ pattern.match_factors
       └─ retrieval_log_id          (per-pattern i64, the F-080 key)
            │
            │  agent applies subset of patterns
            ▼
store_trace(ExecutionTrace {
    retrieval_id:    Some("<UUID from search>"),
    applied_log_ids: Some(vec![47870, 47891, ...]),
    ...
})
```

Both `retrieval_id` and `applied_log_ids` are omitted when `None` (old
single-agent traces remain byte-compatible with older servers).

### Full F-080 example

```rust
use ace_sdk_core::{AceClient, AceClientOptions, AceConfig};
use ace_sdk_core::types::{ExecutionTrace, ExecutionResult, SearchRequest15};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AceConfig {
        server_url: "https://ace-api.code-engine.app".to_string(),
        api_token: "ace_user_...".to_string(),
        project_id: "prj_abc".to_string(),
        ..Default::default()
    };
    let client = AceClient::new(config, AceClientOptions::default())?;

    // Step 1: search
    let now = chrono::Utc::now().to_rfc3339();
    let search_request = SearchRequest15 {
        pattern: json!({
            "id": "tmp", "content": "retry logic", "confidence": 0.8,
            "created_at": now, "section": "general"
        }),
        top_k: Some(10),
        threshold: None,
        task_intent: None,
        exploration_enabled: None,
        exploration_rate: None,
    };
    let search_resp = client.search_patterns15(search_request).await?;

    let retrieval_id = search_resp.retrieval_id.clone();

    // Step 2: collect retrieval_log_ids of patterns the agent applies
    let mut applied_log_ids: Vec<i64> = Vec::new();
    for pattern in &search_resp.similar_patterns {
        // Your selection logic here — this example applies the top two patterns
        if let Some(ref mf) = pattern.match_factors {
            if let Some(log_id) = mf.retrieval_log_id {
                applied_log_ids.push(log_id);
                if applied_log_ids.len() >= 2 { break; }
            }
        }
    }

    // Step 3: store trace with F-080 fields
    let trace = ExecutionTrace {
        task: "Implement retry logic".to_string(),
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
        session_id: None,
        agent_id: None,
        agent_type: None,
        parent_agent_id: None,
        retrieval_id,
        applied_log_ids: if applied_log_ids.is_empty() { None } else { Some(applied_log_ids) },
    };
    client.store_trace(&trace).await?;
    Ok(())
}
```

### Graceful degrade (legacy 1.0 search responses)

If a search response does not contain `match_factors` (legacy 1.0 pattern row)
`retrieval_log_id` will be `None`. In that case simply omit `applied_log_ids`
from the trace — the server accepts it and falls back to the heuristic learning
path.

```rust
// Legacy path: no match_factors → applied_log_ids stays None → trace still accepted
let trace = ExecutionTrace {
    retrieval_id: None,
    applied_log_ids: None,
    // ... rest of fields
    # task: "".to_string(),
    # trajectory: vec![],
    # result: ExecutionResult { success: true, output: "".to_string(), error: None, summary: None },
    # playbook_used: vec![],
    # timestamp: "".to_string(),
    # git: None,
    # session_id: None,
    # agent_id: None,
    # agent_type: None,
    # parent_agent_id: None,
};
```

---

## ExecutionTrace fields

```rust
pub struct ExecutionTrace {
    pub task: String,
    pub trajectory: Vec<serde_json::Value>,
    pub result: ExecutionResult,
    pub playbook_used: Vec<String>,
    pub timestamp: String,
    // Optional attribution
    pub git: Option<GitContext>,
    pub session_id: Option<String>,
    pub agent_id: Option<String>,
    pub agent_type: Option<String>,
    pub parent_agent_id: Option<String>,
    // F-080 (ACE 1.5) — omitted when None
    pub retrieval_id: Option<String>,
    pub applied_log_ids: Option<Vec<i64>>,
}
```

---

## Reading Traces

### Authentication & Headers

- `Authorization: Bearer <token>` — user (`ace_user_*`) or org (`ace_*`) token.
- `X-ACE-Org` — sent automatically from `AceConfig.org_id` when present.
- `X-ACE-Project` — set per call to match the `project_id` argument.

### list_traces

Paginated list of trace summaries.

```rust
pub async fn list_traces(
    &self,
    filters: TraceFilters,
) -> Result<TraceListResponse, AceError>
```

#### TraceFilters

| Field | Type | Description |
|-------|------|-------------|
| `project_id` | `String` | Required. Project scope. |
| `start` | `Option<String>` | ISO-8601 lower bound (inclusive). |
| `end` | `Option<String>` | ISO-8601 upper bound (exclusive). |
| `status` | `Option<String>` | `"success"` or `"failed"`. |
| `agent_type` | `Option<String>` | Filter by agent role (e.g. `"reviewer"`). |
| `session_id` | `Option<String>` | Group traces from a multi-agent session. |
| `git_branch` | `Option<String>` | Filter by git branch. |
| `limit` | `Option<u32>` | Page size. Server default 50, max 200. |
| `cursor` | `Option<String>` | Opaque cursor from `next_cursor` on a prior page. |

#### Example

```rust
use ace_sdk_core::types::TraceFilters;

let filters = TraceFilters {
    project_id: "prj_abc".to_string(),
    status: Some("success".to_string()),
    limit: Some(50),
    ..Default::default()
};

let page = client.list_traces(filters).await?;
for summary in &page.traces {
    println!("{} [{}] task={}", summary.id, summary.status, summary.task);
}
```

### get_trace

Fetch a single trace with full trajectory and linked patterns.

```rust
pub async fn get_trace(
    &self,
    trace_id: &str,
    project_id: &str,
) -> Result<TraceDetail, AceError>
```

```rust
let detail = match client.get_trace("trc_xyz789", "prj_abc").await {
    Ok(d) => d,
    Err(ace_sdk_core::AceError::TraceUnavailable(msg)) => {
        // 410 Gone — pre-migration ghost trace, safe to skip
        eprintln!("trace unavailable: {msg}");
        return Ok(());
    }
    Err(e) => return Err(e.into()),
};

println!("task: {}", detail.task);
for step in &detail.trajectory {
    println!("  {:>3}. {}", step.step, step.action);
    if let Some(d) = step.duration_ms {
        println!("       duration: {d}ms");
    } else if let (Some(s), Some(e)) = (step.start_ms, step.end_ms) {
        println!("       duration: {}ms (computed)", e.saturating_sub(s));
    }
}
```

### Cursor Pagination

```rust
use ace_sdk_core::types::TraceFilters;

let mut filters = TraceFilters {
    project_id: "prj_abc".to_string(),
    limit: Some(100),
    ..Default::default()
};

loop {
    let page = client.list_traces(filters.clone()).await?;
    for summary in &page.traces {
        println!("{}", summary.id);
    }
    match page.next_cursor {
        Some(cursor) => filters.cursor = Some(cursor),
        None => break,
    }
}
```

### Error Handling

| Variant | HTTP | Meaning |
|---------|------|---------|
| `AceError::TraceUnavailable` | 410 | Pre-migration ghost — skip, do not retry. |
| `AceError::Unauthorized` | 401 | Missing/expired token. |
| `AceError::Forbidden` | 403 | Token lacks access. |
| `AceError::BadRequest` | 400/422 | Invalid filter or malformed cursor. |
| `AceError::ServiceUnavailable` | 503 | Transient — retry with backoff. |

### LinkedPatternsMeta

```rust
pub struct LinkedPatternsMeta {
    pub requested: u32,
    pub resolved: u32,
    pub missing_reason: Option<String>,
}
```

Intended UI string: `"{resolved} of {requested} still active"`. Patterns in
`requested` but not `resolved` were deleted, out-of-scope, or hit the
200 ms resolution budget; `missing_reason` carries an optional hint.

```rust
let meta = &detail.linked_patterns_meta;
println!("{} of {} still active", meta.resolved, meta.requested);
if let Some(reason) = &meta.missing_reason {
    println!("  ({reason})");
}
```
