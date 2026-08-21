# ace-sdk-core

Rust client library for the ACE (Agentic Context Engineering) API.

[![crates.io](https://img.shields.io/crates/v/ace-sdk-core.svg)](https://crates.io/crates/ace-sdk-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Installation

```toml
[dependencies]
ace-sdk-core = "0.2"
```

## Features

- **ACE Client**: Async HTTP client for ACE Server API (reqwest + tokio)
- **3-Tier Caching**: RAM → SQLite → Server for optimal performance
- **Device Code Auth**: RFC 8628 device authorization with automatic token refresh
- **Configuration**: Flexible config resolution (CLI args → env vars → files)

## Quick Start

```rust,no_run
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

    let client = AceClient::new(config, None)?;

    // Fetch playbook
    let playbook = client.get_playbook(None).await?;
    println!("Loaded {} patterns", playbook.patterns.len());

    // Search for relevant patterns
    let results = client.search("authentication patterns", None).await?;
    Ok(())
}
```

## Org Usage Analytics

### `get_org_usage_hourly`

```rust,no_run
get_org_usage_hourly(org_id: &str, window: UsageWindow, project_id: Option<&str>) -> Result<UsageHistoryResponse, AceError>
```

Fetch hourly/daily usage buckets for a specific org. Sets `X-ACE-Org` header from the `org_id` argument (use for multi-org admin contexts where the caller's default org differs from the target org). Valid `UsageWindow` values: `"1h"`, `"6h"`, `"12h"`, `"1d"`, `"7d"`, `"14d"`, `"30d"`. Calls `GET /api/v1/usage/history`.

```rust,no_run
let resp = client.get_org_usage_hourly("org_abc", "1d".into(), None).await?;
if let Some(b) = resp.buckets.first() {
    println!("{}: {}", b.period, b.api_calls_total);
}
```

## 0.3.0 — type rename (non-breaking)

Several analytics types were renamed for cross-language consistency with the
other SDKs. The previous names are preserved as aliases, so existing code
continues to compile:

- `UsageHistoryWindow` → `UsageWindow` (old name kept as alias)
- `UsageHistoryBucket` → `UsageBucket` (old name kept as alias)
- `UsageHistoryGranularity` → `UsageGranularity` (old name kept as alias)

## Documentation

Full documentation: [sdks/rust/core/docs](./docs)

## License

MIT © [CE.NET Team](mailto:ace@code-engine.net)
