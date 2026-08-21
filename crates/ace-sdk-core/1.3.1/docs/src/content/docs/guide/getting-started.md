---
title: Getting Started
description: Install and configure the Rust ACE SDK (ACE 1.5)
---

## Migrating from ACE 1.0?

ACE 1.5 is a **clean-break MAJOR** release. There is no 1.0 backward-compat
shim in the write path. Key changes at a glance:

- The reward model is now **tier-based** (`n_hot/warm/cold` counters +
  `cumulative_v15_reward`). `helpful` / `harmful` are deprecated derived
  getters, not the primary API.
- Search results carry `match_factors` (semantic score, UCB score, LinUCB
  bandit rank, F-080 ids) and a top-level `retrieval_id`.
- `ExecutionTrace` gains `retrieval_id` + `applied_log_ids` for the F-080
  feedback loop (close the search→apply→learn cycle automatically).
- The local cache is now a **SQLite graph cache** (`GraphCache`) with a 7-day
  TTL, 2-hop neighbour queries, and per-`(org, project)` isolation — the data
  source for ace-desktop Brain-Graphs.
- Prior 1.0 patterns stored on the server are still readable; the SDK decodes
  them gracefully (missing 1.5 fields default to `None`).

## Installation

### Cargo.toml

```toml
[dependencies]
ace-sdk-core = "0.5"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use ace_sdk_core::{AceClient, AceClientOptions, AceConfig};
use ace_sdk_core::types::{
    ExecutionTrace, ExecutionResult, SearchRequest15,
};
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

    // ── 1. Search (ACE 1.5 native) ─────────────────────────────────────────
    let now = chrono::Utc::now().to_rfc3339();
    let request = SearchRequest15 {
        pattern: json!({
            "id": "tmp", "content": "error handling", "confidence": 0.8,
            "created_at": now, "section": "general"
        }),
        top_k: Some(10),
        task_intent: Some("refactor".to_string()),
        exploration_enabled: None,
        exploration_rate: None,
        threshold: None,
    };
    let response = client.search_patterns15(request).await?;

    // Capture the search-scoped UUID for the F-080 feedback loop
    let retrieval_id = response.retrieval_id.clone();

    // Collect retrieval_log_ids for patterns you actually apply
    let mut applied_log_ids: Vec<i64> = Vec::new();
    for pattern in &response.similar_patterns {
        println!(
            "[{}] {} (reward={:.2}, risk={})",
            pattern.id,
            &pattern.content[..pattern.content.len().min(60)],
            pattern.cumulative_v15_reward.unwrap_or(0.0),
            pattern.is_at_risk(),
        );
        if let Some(ref mf) = pattern.match_factors {
            if let Some(log_id) = mf.retrieval_log_id {
                // Decide which patterns to apply; collect their log ids
                applied_log_ids.push(log_id);
            }
        }
    }

    // ── 2. Playbook (unchanged) ────────────────────────────────────────────
    let playbook = client.get_playbook(false).await?;
    println!("Total patterns: {}", playbook.total_bullets);

    // ── 3. Store trace with F-080 feedback ────────────────────────────────
    let trace = ExecutionTrace {
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
        session_id: None,
        agent_id: None,
        agent_type: None,
        parent_agent_id: None,
        // F-080 fields: close the search→apply→learn loop
        retrieval_id,
        applied_log_ids: if applied_log_ids.is_empty() { None } else { Some(applied_log_ids) },
    };
    client.store_trace(&trace).await?;

    Ok(())
}
```

## Requirements

- Rust 1.75+ (2021 edition)
- tokio runtime
- SQLite (bundled via `rusqlite`)
