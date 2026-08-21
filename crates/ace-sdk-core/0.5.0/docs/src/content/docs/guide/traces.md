---
title: Traces (read)
description: List and fetch execution traces (spec-21)
---

The traces-read API exposes two methods on `AceClient` for inspecting stored
execution traces. Both endpoints live under the root `/traces` prefix and
require a valid `project_id`.

## Authentication & Headers

- `Authorization: Bearer <token>` — user (`ace_user_*`) or org (`ace_*`) token.
- `X-ACE-Org` — sent automatically from `AceConfig.org_id` when present.
- `X-ACE-Project` — overridden per call to match the `project_id` you pass in
  (filter or argument), so the server-side scope is always the call-site truth
  even if the client default differs.

## list_traces

Paginated list of trace summaries.

### Signature

```rust
pub async fn list_traces(
    &self,
    filters: TraceFilters,
) -> Result<TraceListResponse, AceError>
```

### TraceFilters

`project_id` is required; everything else is an optional filter chip.

| Field        | Type             | Description |
|--------------|------------------|-------------|
| `project_id` | `String`         | Required. Project scope. |
| `start`      | `Option<String>` | ISO-8601 timestamp lower bound (inclusive). |
| `end`        | `Option<String>` | ISO-8601 timestamp upper bound (exclusive). |
| `status`     | `Option<String>` | `"success"` or `"failed"`. |
| `agent_type` | `Option<String>` | Filter by agent role (e.g. `"reviewer"`). |
| `session_id` | `Option<String>` | Group traces from a multi-agent session. |
| `git_branch` | `Option<String>` | Filter by git branch captured on the trace. |
| `limit`      | `Option<u32>`    | Page size. Server default 50, max 200. |
| `cursor`     | `Option<String>` | Opaque cursor from `next_cursor` on a prior page. |

### Example

```rust
use ace_sdk_core::{AceClient, AceClientOptions, AceConfig};
use ace_sdk_core::types::TraceFilters;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AceConfig {
        server_url: "https://ace-api.code-engine.app".to_string(),
        api_token: "ace_user_...".to_string(),
        project_id: "prj_abc".to_string(),
        ..Default::default()
    };
    let client = AceClient::new(config, AceClientOptions::default())?;

    let filters = TraceFilters {
        project_id: "prj_abc".to_string(),
        status: Some("success".to_string()),
        agent_type: Some("reviewer".to_string()),
        limit: Some(50),
        ..Default::default()
    };

    let page = client.list_traces(filters).await?;
    for summary in &page.traces {
        println!(
            "{} [{}] task={}",
            summary.id, summary.status, summary.task
        );
    }
    Ok(())
}
```

## get_trace

Fetch a single trace with full trajectory, summary, and linked patterns.

### Signature

```rust
pub async fn get_trace(
    &self,
    trace_id: &str,
    project_id: &str,
) -> Result<TraceDetail, AceError>
```

### Example

```rust
use ace_sdk_core::AceError;

let detail = match client.get_trace("trc_xyz789", "prj_abc").await {
    Ok(d) => d,
    Err(AceError::TraceUnavailable(msg)) => {
        // 410 Gone — pre-migration ghost trace. Skip without retry.
        eprintln!("trace unavailable: {msg}");
        return Ok(());
    }
    Err(e) => return Err(e.into()),
};

println!("task: {}", detail.task);
println!("steps: {}", detail.trajectory.len());
for step in &detail.trajectory {
    println!("  {:>3}. {}", step.step, step.action);
    // Per-step timing (`duration_ms` is `None` on servers that have not
    // yet adopted the read-side schema extension — fall back to
    // `end_ms - start_ms` in that case).
    if let Some(d) = step.duration_ms {
        println!("       duration: {d}ms");
    } else if let (Some(s), Some(e)) = (step.start_ms, step.end_ms) {
        println!("       duration: {}ms (computed)", e.saturating_sub(s));
    }
}
```

## Cursor Pagination

`next_cursor` is opaque base64 — pass it through unchanged. A `None` value
means there are no more pages. `total` is `None` in v1 (not computed
server-side); use the loop instead.

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

If you prefer the `while let` form:

```rust
filters.cursor = None;
let mut first = true;
while first || filters.cursor.is_some() {
    first = false;
    let page = client.list_traces(filters.clone()).await?;
    // ... handle page.traces ...
    filters.cursor = page.next_cursor;
    if filters.cursor.is_none() {
        break;
    }
}
```

## Error Handling

| Variant                       | HTTP | Meaning |
|-------------------------------|------|---------|
| `AceError::TraceUnavailable`  | 410  | Pre-migration ghost trace — safe to skip, do **not** retry. |
| `AceError::Unauthorized`      | 401  | Missing/expired token. Re-authenticate. |
| `AceError::Forbidden`         | 403  | Token lacks access to the project/org. |
| `AceError::BadRequest`        | 400 / 422 | Invalid filter (e.g. malformed timestamp, bad cursor). |
| `AceError::ServiceUnavailable`| 503  | Transient — retry with backoff. |
| `AceError::Network` / `Other` | —    | Transport-level failures. |

`TraceUnavailable` is the only variant unique to this API. Treat it as a
terminal "skip this trace" signal — the trace existed before a migration and
its detail is no longer fetchable.

## LinkedPatternsMeta

`TraceDetail.linked_patterns_meta` reports how many of the trace's referenced
patterns were resolvable when the response was assembled (best-effort, capped
at 200 ms server-side):

```rust
pub struct LinkedPatternsMeta {
    pub requested: u32,
    pub resolved: u32,
    pub missing_reason: Option<String>,
}
```

The intended UI string is **"X of Y still active"** — for example,
`"3 of 5 still active"` when `resolved = 3` and `requested = 5`. Patterns
counted in `requested` but absent from `resolved` were either deleted, scoped
to a different project, or skipped due to the resolution time budget;
`missing_reason` carries an optional human-readable hint.

```rust
let meta = &detail.linked_patterns_meta;
println!("{} of {} still active", meta.resolved, meta.requested);
if meta.resolved < meta.requested {
    if let Some(reason) = &meta.missing_reason {
        println!("  ({reason})");
    }
}
```
