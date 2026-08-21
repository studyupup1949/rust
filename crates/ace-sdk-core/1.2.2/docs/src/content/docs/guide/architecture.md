---
title: Architecture
description: How the Rust ACE SDK is structured (ACE 1.5)
---

## Module Overview

The SDK mirrors the TypeScript `@ace-sdk/core` architecture:

| Module | Description |
|--------|-------------|
| `types` | Structs with serde Serialize/Deserialize — includes ACE 1.5 reward types |
| `client` | `AceClient` + HTTP client (reqwest) |
| `auth` | Device code OAuth (RFC 8628) |
| `config` | XDG config loading + context resolution |
| `cache` | `GraphCache`, `LocalCacheService`, `SessionStorage`, `ProjectIndex` |
| `services` | Bootstrap streaming, `LanguageDetector`, `ImportGraph` |
| `errors` | Error types via thiserror |
| `logger` | Logger trait + `NoopLogger` |
| `utils` | Semver parsing, code extraction |

---

## ACE 1.5 Cache Stack

```
AceClient (RAM — playbook copy)
    │
    ├─ GraphCache (~/.ace-cache/<org>__<project>.db)
    │    patterns table (7-day TTL, cumulative_reward)
    │    edges table    (co-application graph, 2-hop neighbours)
    │    → populated on every search_patterns15() call
    │    → read by ace-desktop Brain-Graphs (schema is cross-SDK byte-identical)
    │
    ├─ LocalCacheService (<org>__<project>.db, configurable TTL)
    │    playbook bullets + sync state
    │
    └─ ACE Server (authoritative)
```

The old 5-minute KV cache is replaced by `GraphCache` in ACE 1.5. The schema
(`patterns` + `edges` tables + two indexes) is **identical across all five
language SDKs** so ace-desktop can read any language's DB file without
conversion.

### Per-project isolation

Each `(org, project)` pair has its own DB file:

```
~/.ace-cache/
  org_abc__prj_123.db          ← GraphCache + LocalCacheService
  org_abc__prj_123_index.db    ← ProjectIndex
  sessions.db                  ← SessionStorage
```

The double-underscore separator prevents collisions when org or project IDs
contain a single underscore.

---

## ACE 1.5 Reward Model

```
Pattern
  ├── n_hot_pos / n_hot_neg    (hot tier — weight 1.0)
  ├── n_warm_pos / n_warm_neg  (warm tier — weight 0.7)
  ├── n_cold_pos / n_cold_neg  (cold tier — weight 0.1)
  ├── cumulative_v15_reward    (0.0 → is_at_risk())
  └── effectiveness
        └── recommendation     (HighlyReliable | Reliable | UseWithCaution |
                                 Unreliable | Unknown)

legacy_helpful() = n_hot_pos*1.0 + n_warm_pos*0.7 + n_cold_pos*0.1  (deprecated)
legacy_harmful() = n_hot_neg*1.0 + n_warm_neg*0.7 + n_cold_neg*0.1  (deprecated)
```

---

## F-080 Feedback Loop

```
search_patterns15(query)
  → SearchResponse15 {
      retrieval_id: Option<String>,          ← search-scoped UUID
      similar_patterns: Vec<Pattern {
          match_factors: Option<MatchFactors {
              retrieval_log_id: Option<i64>, ← per-pattern F-080 key
              ucb_score: Option<f64>,        ← only when LinUCB warm
              bandit_rank: Option<i64>,
              ...
          }>,
          ...
      }>,
    }

store_trace(ExecutionTrace {
    retrieval_id:    Some("<UUID>"),
    applied_log_ids: Some(vec![log_id_1, log_id_2, ...]),
    ...
})
  → POST /traces (or /traces/stream)
  → server updates pattern scores for applied log ids
```

---

## Dependencies

- **reqwest 0.12** — HTTP client with JSON support
- **serde 1** — Serialization with derive macros
- **tokio 1** — Async runtime
- **rusqlite 0.32** — SQLite with bundled build (powers `GraphCache`)
- **chrono 0.4** — Date/time handling
- **thiserror 2** — Error derive macros
- **regex 1** — Pattern matching for code extraction
- **dirs** — XDG/home directory resolution
