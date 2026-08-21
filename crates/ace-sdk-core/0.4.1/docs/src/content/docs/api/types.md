---
title: Types Reference
description: Structs and type definitions
---

## Pattern Types

### PlaybookBullet

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookBullet {
    pub id: String,
    pub section: BulletSection,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concrete_domain: Option<String>,
    #[serde(default)]
    pub helpful: f64,
    #[serde(default)]
    pub harmful: f64,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub observations: f64,
    pub created_at: String,
    #[serde(default)]
    pub last_used: Option<String>,
    /// WHY - underlying principle (populated by Reflector, server v5.2.0+)
    #[serde(default)]
    pub root_cause: String,
    /// WHAT - specific error/problem addressed (used for search boosting)
    #[serde(default)]
    pub error_context: String,
}
```

> **Note:** `helpful`, `harmful`, and `observations` are `f64` — the server applies fractional weights based on confidence, so counters may be non-integer. `last_used` is `Option<String>` and defaults to `None` when the bullet has never been retrieved.

### PlaybookStats

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bullets: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_patterns: Option<u32>,
    #[serde(default)]
    pub by_section: HashMap<String, u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by_domain: Option<HashMap<String, u32>>,
    #[serde(default)]
    pub top_helpful: Vec<PlaybookBullet>,
    #[serde(default)]
    pub top_harmful: Vec<PlaybookBullet>,
    #[serde(default)]
    pub avg_confidence: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helpful_total: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harmful_total: Option<f64>,
}
```

`helpful_total` and `harmful_total` are `Option<f64>` aggregates — fractional because server applies confidence-weighted increments per observation.

### StructuredPlaybook

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StructuredPlaybook {
    pub strategies_and_hard_rules: Vec<PlaybookBullet>,
    pub useful_code_snippets: Vec<PlaybookBullet>,
    pub troubleshooting_and_pitfalls: Vec<PlaybookBullet>,
    pub apis_to_use: Vec<PlaybookBullet>,
}
```

### BulletSection

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BulletSection {
    StrategiesAndHardRules,
    UsefulCodeSnippets,
    TroubleshootingAndPitfalls,
    ApisToUse,
}
```

## Trace Types

### ExecutionTrace

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub task: String,
    pub trajectory: Vec<TrajectoryStep>,
    pub result: ExecutionResult,
    pub playbook_used: Vec<String>,
    pub timestamp: String,
    pub git: Option<GitContext>,
    /// Groups traces from the same multi-agent session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Unique identifier for the executing agent instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// Role/type of the agent (e.g. "reviewer", "coder").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    /// Agent that spawned this one, for parent/child attribution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_agent_id: Option<String>,
}
```

The four attribution fields are optional and omitted from the serialized JSON
when `None`, keeping single-agent traces byte-compatible with older servers.

## Config Types

See `ace_sdk_core::types` for `AceConfig`, `AceContext`, `ServerConfig`.

## Auth Types

See `ace_sdk_core::types` for `UserAuth`, `TokenResponse`, `DeviceCodeResponse`.

## Cache Types

### SessionPinResult

```rust
pub struct SessionPinResult {
    pub similar_patterns: Vec<PlaybookBullet>,
    pub count: u32,
    pub threshold: f64,
    pub top_k: u32,
    pub session_id: String,
    pub pinned_at: i64,
    pub expires_at: i64,
}
```

### ProjectIndexStats

```rust
pub struct ProjectIndexStats {
    pub total_files: u32,
    pub hub_files: u32,
    pub entry_points: u32,
    pub languages: HashMap<String, u32>,
}
```

## Service Types

### ImportGraph

```rust
pub struct ImportGraph {
    pub nodes: HashMap<String, FileNode>,
    pub entry_points: Vec<String>,
    pub hub_files: Vec<String>,
    pub leaf_files: Vec<String>,
    pub circular_deps: Vec<Vec<String>>,
    pub dead_code: Vec<String>,
}
```

### CodeBlock

```rust
pub struct CodeBlock {
    pub code: String,
    pub name: String,
    pub block_type: CodeBlockType,
    pub file: String,
}
```
