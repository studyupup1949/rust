---
title: ACE Client
description: Using the main API client
---

## Creating a Client

```rust
use ace_sdk_core::{AceClient, AceClientOptions, AceConfig};

let config = AceConfig {
    server_url: "https://ace-api.code-engine.app".to_string(),
    api_token: "ace_user_...".to_string(),
    project_id: "prj_...".to_string(),
    ..Default::default()
};

let client = AceClient::new(config, AceClientOptions::default())?;
```

## Operations

### Get Playbook

```rust
let playbook = client.get_playbook(false).await?;
for bullet in &playbook.playbook.strategies_and_hard_rules {
    println!("{} (confidence: {})", bullet.content, bullet.confidence);
}
```

### Search Patterns

```rust
let results = client.search_patterns("error handling", 10, None, None).await?;
for pattern in &results.similar_patterns {
    println!("{}", pattern.content);
}
```

### Store Execution Trace

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
};
```

## Multi-Agent Attribution

Traces can be attributed to a specific agent in a multi-agent workflow. Populate
the optional `session_id`, `agent_id`, `agent_type`, and `parent_agent_id` fields
on `ExecutionTrace` to group traces by session and track parent/child agent
relationships. `AceClient::search_patterns` accepts an optional `agent_type`
filter to retrieve patterns scoped to a specific agent role.

```rust
use ace_sdk_core::types::{ExecutionTrace, ExecutionResult};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AceClient::new(config, AceClientOptions::default())?;

    // Search patterns scoped to a particular agent type
    let results = client
        .search_patterns("retry logic", 10, None, Some("reviewer"))
        .await?;
    for pattern in &results.similar_patterns {
        println!("{}", pattern.content);
    }

    // Store a trace attributed to a child agent within a session
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
    };
    client.store_trace(&trace).await?;
    Ok(())
}
```

Fields are serialized with `#[serde(skip_serializing_if = "Option::is_none")]`,
so omitted values are not sent on the wire — existing single-agent traces stay
byte-compatible.
