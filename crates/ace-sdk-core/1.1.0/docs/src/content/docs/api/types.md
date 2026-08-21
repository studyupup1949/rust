---
title: Types Reference
description: Structs and type definitions (ACE 1.5)
---

## Pattern Types (ACE 1.5)

ACE 1.5 introduces a **tier-based reward model**. The primary fields are the
six tier counters and `cumulative_v15_reward`. The legacy `helpful` / `harmful`
wire fields are still decoded for backward-compat with old stored patterns, but
they are **deprecated** — use `legacy_helpful()` / `legacy_harmful()` when you
need the derived scalar and `PatternEffectiveness` / tier counters for new code.

### Pattern

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub name: String,
    pub domain: Option<String>,
    pub content: String,
    pub confidence: f64,
    pub section: String,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub last_used: Option<String>,
    pub evidence: Vec<String>,
    pub retrieval_count: i64,
    pub root_cause: String,
    pub error_context: String,
    // Legacy 1.0 wire fields — deprecated, read-only
    pub helpful: f64,
    pub harmful: f64,
    pub observations: f64,
    pub local_helpful: f64,
    pub local_harmful: f64,
    // ── ACE 1.5 reward fields (all Option — absent on legacy 1.0 rows) ──
    pub payload_version: Option<i64>,      // 15 when 1.5; None on 1.0 rows
    pub n_hot_pos: Option<i64>,
    pub n_hot_neg: Option<i64>,
    pub n_warm_pos: Option<i64>,
    pub n_warm_neg: Option<i64>,
    pub n_cold_pos: Option<i64>,
    pub n_cold_neg: Option<i64>,
    pub cumulative_v15_reward: Option<f64>, // 0.0 → pattern is at-risk
    pub n_retrieval_no_apply: Option<i64>,
    pub task_intent: Option<String>,
    pub effectiveness: Option<PatternEffectiveness>,
    pub match_factors: Option<MatchFactors>, // search results only; None on /playbook
    // forward-compat passthrough fields (decode-tolerant)
    pub root_cause_present: Option<bool>,
    pub has_error_context: Option<bool>,
    pub birth_primary_lang: Option<String>,
    // ... additional 1.5 metadata fields decoded as Option
}
```

#### Derived getters

```rust
impl Pattern {
    /// Deprecated. Derived from tier counters.
    /// Formula: n_hot_pos * 1.0 + n_warm_pos * 0.7 + n_cold_pos * 0.1
    pub fn legacy_helpful(&self) -> f64 { ... }

    /// Deprecated. Derived from tier counters.
    /// Formula: n_hot_neg * 1.0 + n_warm_neg * 0.7 + n_cold_neg * 0.1
    pub fn legacy_harmful(&self) -> f64 { ... }

    /// True when cumulative_v15_reward == 0.0 (at-risk pattern).
    pub fn is_at_risk(&self) -> bool { ... }
}
```

> **Decode rule:** all ACE 1.5 fields carry `#[serde(default)]`. A legacy 1.0
> response (missing tier counters, missing `match_factors`) deserializes
> cleanly — all 1.5 fields default to `None`. Never throws on partial data.

### PatternEffectiveness

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternEffectiveness {
    /// Reliability label. Server 5.7.1 does NOT emit this yet — field is
    /// optional; absent or unknown values decode to `RecommendationLabel::Unknown`.
    pub recommendation: Option<RecommendationLabel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationLabel {
    HighlyReliable,
    Reliable,
    UseWithCaution,
    Unreliable,
    #[serde(other)]
    Unknown, // fallback for new server values — never panics
}
```

### MatchFactors

Present on `/patterns/search` results only; absent on `/playbook` responses.
All fields are optional to handle cold/shadow patterns and legacy rows.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchFactors {
    pub semantic_score: Option<f64>,
    pub domain_boost: Option<bool>,
    pub domain_relevance: Option<f64>,
    pub error_context_boost: Option<bool>,
    pub formula_boost_applied: Option<bool>,
    /// Only present when LinUCB is warm (absent on cold/shadow rows).
    pub ucb_score: Option<f64>,
    /// Only present when LinUCB is warm.
    pub bandit_rank: Option<i64>,
    /// Per-pattern INTEGER key for the F-080 feedback loop.
    pub retrieval_log_id: Option<i64>,
    /// UUID string — duplicates the top-level retrieval_id.
    pub retrieval_id: Option<String>,
    pub shadow_mode: Option<bool>,
}
```

> `retrieval_log_id` is an **integer** (`i64`), not a string.
> It is the key you collect and send as `applied_log_ids` in the subsequent
> `ExecutionTrace` to close the F-080 loop.

### SearchResponse15

Returned by `AceClient::search_patterns15`.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse15 {
    pub similar_patterns: Vec<Pattern>,
    /// Search-scoped UUID — capture for F-080.
    pub retrieval_id: Option<String>,
    pub count: u32,
    pub local_count: u32,
    pub shared_count: u32,
    pub domains_summary: Option<serde_json::Value>,
    pub search_params: Option<serde_json::Value>,
    pub tokens_in_response: u32,
}
```

### SearchRequest15

All optional fields are omitted from the JSON body when `None` —
`#[serde(skip_serializing_if = "Option::is_none")]` is applied to each.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest15 {
    pub pattern: serde_json::Value,
    /// Omitted when None — server default preserved.
    pub threshold: Option<f64>,
    /// Omitted when None — server default preserved.
    pub top_k: Option<u32>,
    /// Hint for context-aware ranking: "refactor" | "routine" | "explore" | "spec_strict"
    pub task_intent: Option<String>,
    /// Enable/disable bandit exploration. Omitted when None.
    pub exploration_enabled: Option<bool>,
    /// Exploration rate. Omitted when None.
    pub exploration_rate: Option<f64>,
    /// F-080 session-scoped credit (ACE 1.5).
    ///
    /// When set, the server stamps `retrieval_log_v15.session_id` so ACE can
    /// group the search row with the trace that follows. Pass the same value
    /// in `ExecutionTrace::session_id` to close the credit loop.
    /// Omitted from the POST body when `None` — server leaves the column NULL.
    pub session_id: Option<String>,
}
```

### PlaybookBullet

Used in `/playbook` responses. Carries 1.5 reward fields; no `match_factors`
(search-only). `helpful` / `harmful` are legacy wire fields — use
`legacy_helpful()` / `legacy_harmful()` for the derived scalar.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookBullet {
    pub id: String,
    pub section: BulletSection,
    pub content: String,
    pub domain: Option<String>,
    pub concrete_domain: Option<String>,
    /// Deprecated legacy wire field.
    pub helpful: f64,
    /// Deprecated legacy wire field.
    pub harmful: f64,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub observations: f64,
    pub created_at: String,
    pub last_used: Option<String>,
    pub root_cause: String,
    pub error_context: String,
}
```

### PlaybookStats

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookStats {
    pub total_bullets: Option<u32>,
    pub total_patterns: Option<u32>,
    pub by_section: HashMap<String, u32>,
    pub by_domain: Option<HashMap<String, u32>>,
    pub top_helpful: Vec<PlaybookBullet>,
    pub top_harmful: Vec<PlaybookBullet>,
    pub avg_confidence: f64,
    pub helpful_total: Option<f64>,
    pub harmful_total: Option<f64>,
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

---

## Trace Types

### ExecutionTrace

ACE 1.5 adds `retrieval_id` and `applied_log_ids` for the F-080 feedback loop.
Both fields are omitted when `None` — byte-compatible with older servers.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub task: String,
    pub trajectory: Vec<serde_json::Value>,
    pub result: ExecutionResult,
    pub playbook_used: Vec<String>,
    pub timestamp: String,
    pub git: Option<GitContext>,
    pub session_id: Option<String>,
    pub agent_id: Option<String>,
    pub agent_type: Option<String>,
    pub parent_agent_id: Option<String>,
    /// F-080 (ACE 1.5): search-scoped UUID from the prior search response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval_id: Option<String>,
    /// F-080 (ACE 1.5): retrieval_log_id integers of patterns actually applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_log_ids: Option<Vec<i64>>,
}
```

> Both `retrieval_id` and `applied_log_ids` are serialized on **both**
> `POST /traces` and `POST /traces/stream`. They are never sent in the
> write path via `helpful_delta` / `harmful_delta` — those vote fields are
> intentionally absent from `ExecutionTrace` in ACE 1.5.

### TrajectoryStep (write side)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStep {
    pub step: u32,
    pub action: String,
    pub args: HashMap<String, serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub start_ms: Option<i64>,
    pub end_ms: Option<i64>,
    pub duration_ms: Option<i64>,
}
```

### TraceStep (read side)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    pub step: u32,
    pub action: String,
    pub args: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub start_ms: Option<u64>,
    pub end_ms: Option<u64>,
    /// None on responses from servers that have not yet adopted the timing
    /// schema extension — compute from end_ms - start_ms in that case.
    pub duration_ms: Option<i64>,
}
```

---

## Subscription / Usage Types

### UsageInfo

Parsed from `X-ACE-*` response headers on every authenticated request.

```rust
pub struct UsageInfo {
    pub plan: String,
    pub subscription_type: SubscriptionType,
    pub plan_tier: PlanTier,
    pub status: SubscriptionStatus,
    pub patterns: UsageMetric,
    pub patterns_total: UsageMetric,
    pub projects: UsageMetric,
    pub domains: UsageMetric,
    pub templates: UsageMetric,
    pub api_calls: UsageMetric,
    pub traces_today: UsageMetric,
    pub subscription_updated_at: Option<String>,
}
```

### UsageMetric

```rust
pub struct UsageMetric {
    pub used: u32,
    pub limit: i32,  // -1 == unlimited / unparseable sentinel
}
```

---

## Config Types

See `ace_sdk_core::types` for `AceConfig`, `AceContext`, `ServerConfig`.

## Auth Types

See `ace_sdk_core::types` for `UserAuth`, `TokenResponse`, `DeviceCodeResponse`.

## Cache Types

### GraphCacheEntry (returned by `GraphCache::get_pattern`)

```rust
// Returns (payload_json: String, cumulative_reward: f64)
```

### GraphRefreshResult (returned by `GraphCache::refresh_from_server`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphRefreshResult {
    /// Number of edges successfully upserted into the local `edges` table.
    pub edges_upserted: usize,
    /// Whether the server indicated the response was truncated (>50 000 edges).
    pub truncated: bool,
}
```

The method is **best-effort** — it always returns `Ok(...)`. Inspect
`edges_upserted` to confirm edges landed. When `truncated` is `true` the
server capped the response; raise `min_weight` or pass `since_ms` to narrow
the window. See `GraphCache::refresh_from_server` in the
[Caching guide](/ace-sdk/rust/core/guide/caching/#refreshing-graph-edges-from-the-server-refresh_from_server).

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
