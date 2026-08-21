---
title: Getting Started
description: Install and configure the Rust ACE SDK
---

## Installation

### Cargo.toml

```toml
[dependencies]
ace-sdk-core = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use ace_sdk_core::{AceClient, AceClientOptions, AceConfig};

#[tokio::main]
async fn main() -> Result<(), ace_sdk_core::AceError> {
    let config = AceConfig {
        server_url: "https://ace-api.code-engine.app".to_string(),
        api_token: "ace_user_xxx".to_string(),
        project_id: "my-project".to_string(),
        cache_ttl_minutes: 120,
        ..Default::default()
    };

    let client = AceClient::new(config, AceClientOptions::default())?;
    let playbook = client.get_playbook(false).await?;
    println!("Total patterns: {}", playbook.total_bullets);
    Ok(())
}
```

## Requirements

- Rust 1.75+ (2021 edition)
- tokio runtime
- SQLite (bundled via `rusqlite`)
