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
    pub confidence: f64,
    pub helpful: i32,
    pub harmful: i32,
    pub observations: u32,
    pub domain: Option<String>,
    pub evidence: Vec<String>,
    pub created_at: String,
    pub last_used: String,
    /// WHY - underlying principle (populated by Reflector, server v5.2.0+)
    #[serde(default)]
    pub root_cause: String,
    /// WHAT - specific error/problem addressed (used for search boosting)
    #[serde(default)]
    pub error_context: String,
}
```

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
