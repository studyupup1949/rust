---
title: Architecture
description: How the Rust ACE SDK is structured
---

## Module Overview

The SDK mirrors the TypeScript `@ace-sdk/core` architecture:

| Module | Description |
|--------|-------------|
| `types` | Structs with serde Serialize/Deserialize |
| `client` | AceClient + HTTP client (reqwest) |
| `auth` | Device code OAuth (RFC 8628) |
| `config` | XDG config loading + context resolution |
| `cache` | LocalCacheService, SessionStorage, ProjectIndex |
| `services` | Bootstrap streaming, LanguageDetector, ImportGraph |
| `errors` | Error types via thiserror |
| `logger` | Logger trait + NoopLogger |
| `utils` | Semver parsing, code extraction |

## 3-Tier Cache

```
RAM Cache (AceClient instance)
    | miss
SQLite Cache (~5 min TTL)
    | miss
ACE Server (ChromaDB)
```

## Dependencies

- **reqwest 0.12** -- HTTP client with JSON support
- **serde 1** -- Serialization with derive macros
- **tokio 1** -- Async runtime
- **rusqlite 0.32** -- SQLite with bundled build
- **chrono 0.4** -- Date/time handling
- **thiserror 2** -- Error derive macros
- **regex 1** -- Pattern matching for code extraction
