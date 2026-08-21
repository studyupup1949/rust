---
title: Caching
description: SQLite-based local caching and session storage
---

## Overview

The SDK uses a 3-tier cache to minimize API calls:

1. **RAM** -- In-memory playbook copy (fastest)
2. **SQLite** -- Local database with configurable TTL (default 5 min)
3. **Server** -- Remote ACE API (authoritative source)

## LocalCacheService

```rust
use ace_sdk_core::cache::LocalCacheService;

let cache = LocalCacheService::new(
    "org_123",
    "prj_456",
    10,  // TTL in minutes
    None // Default cache dir: ~/.ace-cache/
)?;
```

## SessionStorage

Persistent pattern pinning for context compaction (24h TTL):

```rust
use ace_sdk_core::cache::{SessionStorage, SessionStorageConfig};

let storage = SessionStorage::new(Some(SessionStorageConfig {
    cache_dir: None, // Default: ~/.ace-cache/
}))?;

// Pin patterns to a session
storage.pin_session("sess-1", "error handling", &patterns, 0.7, 10)?;

// Recall later without server round-trip
if let Some(result) = storage.recall_session("sess-1")? {
    println!("Recalled {} patterns", result.count);
}

// List active sessions
let sessions = storage.list_sessions()?;
```

## ProjectIndex

SQLite-based file index for smart bootstrap file selection:

```rust
use ace_sdk_core::cache::{ProjectIndex, ProjectIndexConfig};

let index = ProjectIndex::new(ProjectIndexConfig {
    org_id: "org_123".to_string(),
    project_id: "prj_456".to_string(),
    cache_dir: None,
})?;

// Query the index
let hubs = index.get_hub_files(20);
let entries = index.get_entry_points();
let stats = index.get_stats();
println!("Files: {}, Hubs: {}", stats.total_files, stats.hub_files);
```
