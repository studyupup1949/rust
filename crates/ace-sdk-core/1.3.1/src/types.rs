//! ACE SDK type definitions
//!
//! All types used throughout the SDK, with serde serialization support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SDK version constant
pub const CORE_VERSION: &str = "1.3.1";

// =============================================================================
// ACE 1.5 — Pattern (native reward model) + MatchFactors + PatternEffectiveness
// =============================================================================

/// Recommendation label from `effectiveness.recommendation`.
///
/// Server 5.7.1 does NOT emit this yet — field is optional. Unknown values
/// (future server additions) decode to `Unknown` via `#[serde(other)]`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationLabel {
    HighlyReliable,
    Reliable,
    UseWithCaution,
    Unreliable,
    /// Fallback for unknown / absent values — `#[serde(other)]` catches
    /// any string the server sends that isn't one of the above variants.
    #[serde(other)]
    Unknown,
}

/// Pattern effectiveness info, embedding `recommendation` (optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternEffectiveness {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<RecommendationLabel>,
}

/// Per-result `match_factors` (search results only, absent on /playbook).
///
/// All fields are optional because:
/// - `ucb_score` / `bandit_rank` only appear when LinUCB is warm.
/// - `retrieval_log_id` is an INTEGER (i64) — the F-080 key.
/// - Empty `{}` or absent key → `None` on every field, never throws.
/// - `#[serde(default)]` handles absent; no `deny_unknown_fields` for forward-compat.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchFactors {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_boost: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_relevance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_context_boost: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formula_boost_applied: Option<bool>,
    /// Only present when LinUCB is warm (absent on cold/shadow rows).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ucb_score: Option<f64>,
    /// Only present when LinUCB is warm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bandit_rank: Option<i64>,
    /// Per-pattern INTEGER key for F-080 feedback loop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval_log_id: Option<i64>,
    /// UUID string, duplicates top-level retrieval_id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow_mode: Option<bool>,
}

/// ACE 1.5 Pattern (search result wrapper).
///
/// Carries all 1.5 reward fields + `match_factors`. Legacy 1.0 rows (no
/// `payload_version`, no tier counters, no `match_factors`) are decoded
/// gracefully — missing fields default to `None`.
///
/// `#[serde(default)]` on every 1.5 field ensures legacy rows never throw.
/// No `deny_unknown_fields` so forward-compat is preserved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    pub content: String,
    #[serde(default)]
    pub confidence: f64,
    // Legacy 1.0 fields (deprecated — use tier counters for derived values)
    #[serde(default)]
    pub observations: f64,
    /// Deprecated — present in legacy 1.0 responses.
    #[serde(default)]
    pub helpful: f64,
    /// Deprecated — present in legacy 1.0 responses.
    #[serde(default)]
    pub harmful: f64,
    pub section: String,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
    #[serde(default)]
    pub evidence: Vec<String>,
    #[serde(default)]
    pub retrieval_count: i64,
    #[serde(default)]
    pub root_cause: String,
    #[serde(default)]
    pub error_context: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_project_name: Option<String>,
    #[serde(default)]
    pub local_helpful: f64,
    #[serde(default)]
    pub local_harmful: f64,

    // ---- ACE 1.5 reward fields (all optional for legacy-1.0 graceful degrade) ----
    /// 15 when 1.5 payload; absent on legacy 1.0 rows → None.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_version: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_hot_pos: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_hot_neg: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_warm_pos: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_warm_neg: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_cold_pos: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_cold_neg: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cumulative_v15_reward: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_retrieval_no_apply: Option<i64>,
    /// Optional — server 5.7.1 doesn't emit it. Present on some patterns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_intent: Option<String>,
    /// Effectiveness metadata including optional recommendation label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effectiveness: Option<PatternEffectiveness>,
    /// Per-result match factors (search only, absent on /playbook).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_factors: Option<MatchFactors>,

    // Decode-tolerant passthrough for other 1.5 metadata fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_cause_present: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_error_context: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_primary_lang: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain_cluster_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abstract_domain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_cause_cluster_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_first_tool_bucket: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_n_steps_bucket: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_has_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_citation_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citation_score_ema_30d: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_winner_count: Option<i64>,
    #[serde(default)]
    pub merged_from: Vec<String>,
}

impl Pattern {
    /// Deprecated derived getter: `helpful` computed from tier counters.
    ///
    /// Formula: `n_hot_pos*1.0 + n_warm_pos*0.7 + n_cold_pos*0.1` (COLD=0.1).
    /// Read-only. For 1.5 patterns only; legacy rows return 0.0.
    pub fn legacy_helpful(&self) -> f64 {
        self.n_hot_pos.unwrap_or(0) as f64 * 1.0
            + self.n_warm_pos.unwrap_or(0) as f64 * 0.7
            + self.n_cold_pos.unwrap_or(0) as f64 * 0.1
    }

    /// Deprecated derived getter: `harmful` computed from tier counters.
    ///
    /// Formula: `n_hot_neg*1.0 + n_warm_neg*0.7 + n_cold_neg*0.1` (COLD=0.1).
    /// Read-only. For 1.5 patterns only; legacy rows return 0.0.
    pub fn legacy_harmful(&self) -> f64 {
        self.n_hot_neg.unwrap_or(0) as f64 * 1.0
            + self.n_warm_neg.unwrap_or(0) as f64 * 0.7
            + self.n_cold_neg.unwrap_or(0) as f64 * 0.1
    }

    /// Returns `true` when `cumulative_v15_reward < 0.0` (net-negative / harmful pattern).
    ///
    /// `reward == 0.0` means uncredited/neutral (fresh pattern with no feedback yet) — NOT at-risk.
    /// `reward == None` means legacy row with no reward data — NOT at-risk.
    pub fn is_at_risk(&self) -> bool {
        self.cumulative_v15_reward.map(|r| r < 0.0).unwrap_or(false)
    }
}

/// A single entry in `SearchResponse15::expanded`.
///
/// Matches the TS `ExpandedNeighborEntry` contract (cross-language §5b):
/// - `cached = true`  → pattern was found in the graph-cache `patterns` table
///   → id-only stub, `payload_json` is `None` (token-efficient path).
/// - `cached = false` → pattern was found only as an edge endpoint, not yet
///   in the patterns table → also id-only stub; `payload_json` is `None`
///   (mirrors the TS fix for `UncachedNeighborEntry`: no payload leakage).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpandedNeighborEntry {
    pub pattern_id: String,
    pub cumulative_reward: f64,
    pub cached: bool,
    /// Always `None` — id-only stubs for both cached and uncached entries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_json: Option<String>,
}

/// ACE 1.5 `/patterns/search` response.
///
/// Wraps `similar_patterns` as `Vec<Pattern>` (1.5 native) and captures
/// the search-scoped `retrieval_id` at the top level (F-080).
///
/// The `expanded` field is populated by the client-side populate path
/// (CONTRACT §5c): after a successful search the core layer runs a 2-hop
/// neighbors query against the local graph cache and attaches the result
/// here. Absent when `expand_neighbors = false` or when no edges exist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse15 {
    pub similar_patterns: Vec<Pattern>,
    /// Search-scoped UUID used in F-080 feedback loop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval_id: Option<String>,
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub local_count: u32,
    #[serde(default)]
    pub shared_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domains_summary: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_params: Option<serde_json::Value>,
    #[serde(default)]
    pub tokens_in_response: u32,
    /// Client-side 2-hop neighbors attached after search (populate path).
    /// `None` when no edges exist or `expand_neighbors = false`.
    #[serde(skip)]
    pub expanded: Option<Vec<ExpandedNeighborEntry>>,
}

/// ACE 1.5 search request body.
///
/// `task_intent`, `exploration_enabled`, and `exploration_rate` are omitted
/// when `None` — server default is preserved (skip_serializing_if).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest15 {
    pub pattern: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    /// Optional task intent hint — omitted when not set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_intent: Option<String>,
    /// Optional exploration toggle — omitted when not set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exploration_enabled: Option<bool>,
    /// Optional exploration rate — omitted when not set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exploration_rate: Option<f64>,
    /// Optional session identifier — omitted when not set.
    ///
    /// When provided, the server stamps `retrieval_log_v15.session_id` so that
    /// all search calls within a session are grouped for intent classification.
    /// Without this field the column is NULL on every row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Client-side option: whether to run the 2-hop neighbor expansion and
    /// attach results to `SearchResponse15::expanded`.
    ///
    /// Defaults to `true` when `None`. Set to `Some(false)` to disable.
    /// Never serialized to the server — purely a client-side hint.
    #[serde(skip)]
    pub expand_neighbors: Option<bool>,
}

// =============================================================================
// Execution Trace
// =============================================================================

/// A single step in an execution trajectory.
///
/// Timing fields (`start_ms`, `end_ms`, `duration_ms`) are optional and
/// additive. When `None` they are omitted from the serialized JSON, keeping
/// payloads byte-compatible with older servers. Server-side validation is
/// lenient (Pydantic `extra='ignore'`), so adding these does not break.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStep {
    pub step: u32,
    pub action: String,
    pub args: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Absolute Unix epoch ms (UTC) when this step started. Optional, additive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_ms: Option<i64>,
    /// Absolute Unix epoch ms (UTC) when this step ended. Optional, additive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_ms: Option<i64>,
    /// Step duration in ms (ground-truth — may differ from `end_ms - start_ms`
    /// due to client clock skew). Optional, additive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
}

/// Execution trace capturing task details for pattern learning.
///
/// F-080 fields added (ACE 1.5):
/// - `retrieval_id`: the search-scoped UUID captured from the prior
///   `/patterns/search` response.
/// - `applied_log_ids`: the `retrieval_log_id` integers of the patterns
///   the agent actually applied. Both are omitted when unset (None).
///
/// Serialized on BOTH `/traces` and `/traces/stream`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub task: String,
    pub trajectory: Vec<serde_json::Value>,
    pub result: ExecutionResult,
    pub playbook_used: Vec<String>,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<GitContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_agent_id: Option<String>,
    /// F-080: search-scoped UUID from the prior `/patterns/search` response.
    /// Omitted when not set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieval_id: Option<String>,
    /// F-080: per-pattern `retrieval_log_id` integers of patterns applied.
    /// Omitted when not set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_log_ids: Option<Vec<i64>>,
}

/// Result of an execution trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

// =============================================================================
// Git Context
// =============================================================================

/// Git context for correlating execution traces with commits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitContext {
    pub commit_hash: String,
    pub branch: String,
    pub files_changed: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insertions: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletions: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_commits: Option<Vec<String>>,
}

// =============================================================================
// Playbook Types
// =============================================================================

/// Playbook section names.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BulletSection {
    StrategiesAndHardRules,
    UsefulCodeSnippets,
    TroubleshootingAndPitfalls,
    ApisToUse,
}

/// A single bullet (pattern) in the playbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookBullet {
    pub id: String,
    pub section: BulletSection,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concrete_domain: Option<String>,
    /// ACE 1.5: deprecated read-only passthrough from server (legacy Qdrant field).
    /// Deserialized from server responses; NEVER serialized into write/POST bodies.
    #[serde(default, skip_serializing)]
    pub helpful: f64,
    /// ACE 1.5: deprecated read-only passthrough from server (legacy Qdrant field).
    /// Deserialized from server responses; NEVER serialized into write/POST bodies.
    #[serde(default, skip_serializing)]
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
    /// WHY - underlying principle (default: "")
    #[serde(default)]
    pub root_cause: String,
    /// WHAT - specific error/problem addressed (default: "")
    #[serde(default)]
    pub error_context: String,
    /// ACE 1.5: net reward signal for this pattern. Server hydrates this on
    /// /playbook and /patterns/top responses (v5.8.18+).
    /// `None` = legacy row with no reward data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cumulative_v15_reward: Option<f64>,
}

impl PlaybookBullet {
    /// Returns `true` when `cumulative_v15_reward < 0.0` (net-negative / harmful pattern).
    ///
    /// Mirrors `Pattern::is_at_risk` — identical semantics.
    /// `reward == 0.0` means uncredited/neutral (fresh pattern, no feedback yet) — NOT at-risk.
    /// `reward == None` means legacy row with no reward data — NOT at-risk.
    pub fn is_at_risk(&self) -> bool {
        self.cumulative_v15_reward.map(|r| r < 0.0).unwrap_or(false)
    }
}

/// Summary of domains in results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainsSummary {
    #[serde(rename = "abstract")]
    pub abstract_domains: Vec<String>,
    pub concrete: Vec<String>,
}

/// Structured playbook with four sections.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StructuredPlaybook {
    #[serde(default)]
    pub strategies_and_hard_rules: Vec<PlaybookBullet>,
    #[serde(default)]
    pub useful_code_snippets: Vec<PlaybookBullet>,
    #[serde(default)]
    pub troubleshooting_and_pitfalls: Vec<PlaybookBullet>,
    #[serde(default)]
    pub apis_to_use: Vec<PlaybookBullet>,
}

// =============================================================================
// Delta Operations
// =============================================================================

/// Type of delta operation from Reflector.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeltaOperationType {
    ADD,
    UPDATE,
    DELETE,
}

/// Incremental update operation.
///
/// CONTRACT §4 / §7: `helpful_delta` and `harmful_delta` are intentionally
/// absent — the write path must NEVER carry helpful/harmful vote fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaOperation {
    #[serde(rename = "type")]
    pub op_type: DeltaOperationType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<BulletSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bullet_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

// =============================================================================
// Analytics
// =============================================================================

/// Playbook statistics.
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

    // ---- ACE 1.5 reward-aggregate fields (all optional, additive) ----
    /// Sum of `cumulative_v15_reward` across all patterns in the playbook.
    #[serde(default)]
    pub cumulative_reward_total: f64,
    /// Count of patterns with reward tier = "hot".
    #[serde(default)]
    pub hot_total: i64,
    /// Count of patterns with reward tier = "warm".
    #[serde(default)]
    pub warm_total: i64,
    /// Count of patterns with reward tier = "cold".
    #[serde(default)]
    pub cold_total: i64,
    /// Count of patterns where `cumulative_v15_reward < 0.0` (at-risk).
    #[serde(default)]
    pub at_risk_count: i64,
    /// Count of patterns that carry at least one 1.5 reward observation.
    #[serde(default)]
    pub patterns_with_v15_reward: i64,
}

// =============================================================================
// Token Metadata
// =============================================================================

/// Token usage metadata from ACE server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub tokens_in_response: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_saved_vs_full_playbook: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub efficiency_gain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_playbook_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

// =============================================================================
// Search Response
// =============================================================================

/// Search response with optional token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponseWithMetadata {
    pub similar_patterns: Vec<PlaybookBullet>,
    pub count: u32,
    pub threshold: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domains_summary: Option<DomainsSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TokenMetadata>,
}

/// Playbook response with optional token metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookResponseWithMetadata {
    pub playbook: StructuredPlaybook,
    #[serde(default)]
    pub total_bullets: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TokenMetadata>,
}

// =============================================================================
// Bootstrap Types
// =============================================================================

/// Bootstrap response from server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapResponse {
    pub success: bool,
    pub blocks_received: u32,
    pub patterns_extracted: u32,
    pub compression_percentage: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patterns_after_dedup: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_ratio: Option<String>,
    #[serde(default)]
    pub by_section: HashMap<String, u32>,
    #[serde(default)]
    pub average_confidence: f64,
    #[serde(default)]
    pub analysis_time_seconds: f64,
}

/// Bootstrap mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum BootstrapMode {
    #[default]
    Hybrid,
    Both,
    LocalFiles,
    GitHistory,
    DocsOnly,
}

/// Thoroughness level for bootstrap.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThoroughnessLevel {
    Light,
    #[default]
    Medium,
    Deep,
}

/// Thoroughness preset configuration.
#[derive(Debug, Clone)]
pub struct ThoroughnessPreset {
    pub max_files: i32,
    pub commit_limit: u32,
    pub days_back: u32,
}

/// Get thoroughness preset by level.
pub fn get_thoroughness_preset(level: &ThoroughnessLevel) -> ThoroughnessPreset {
    match level {
        ThoroughnessLevel::Light => ThoroughnessPreset {
            max_files: 1000,
            commit_limit: 100,
            days_back: 30,
        },
        ThoroughnessLevel::Medium => ThoroughnessPreset {
            max_files: 5000,
            commit_limit: 500,
            days_back: 90,
        },
        ThoroughnessLevel::Deep => ThoroughnessPreset {
            max_files: -1,
            commit_limit: 1000,
            days_back: 180,
        },
    }
}

// =============================================================================
// Learning Types
// =============================================================================

/// Learning statistics returned from server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStatistics {
    pub patterns_created: u32,
    pub patterns_updated: u32,
    pub patterns_pruned: u32,
    pub patterns_deduplicated: u32,
    #[serde(default)]
    pub by_section: HashMap<String, u32>,
    #[serde(default)]
    pub average_confidence: f64,
    #[serde(default)]
    pub helpful_delta: i32,
    #[serde(default)]
    pub helpful_count: u32,
    #[serde(default)]
    pub harmful_count: u32,
    #[serde(default)]
    pub analysis_time_seconds: f64,

    // ---- ACE 1.5 reward fields (all optional, additive) ----
    /// Net change in `cumulative_v15_reward` across all patterns touched
    /// during this learning pass. Absent on older server responses → 0.0.
    #[serde(default)]
    pub cumulative_v15_reward_delta: f64,
    /// Number of patterns that received a reward update during this pass.
    #[serde(default)]
    pub patterns_rewarded: i64,
    /// Dominant reward tier of the updated patterns ("hot", "warm", "cold").
    /// Empty string when absent — omitted from serialized output.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reward_tier: String,
}

/// Response from /traces endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningResponse {
    pub stored: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    pub analysis_performed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_learning_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_statistics: Option<LearningStatistics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_queued: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_exceeded: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_error_code: Option<String>,
}

// =============================================================================
// Trace Read Types (spec-21)
// =============================================================================

/// Filters for listing execution traces (spec-21).
///
/// `project_id` is required. All other fields are optional filter chips.
/// `limit` defaults to 50 server-side; max is 200. `cursor` is opaque base64
/// from a prior `TraceListResponse.next_cursor` — pass through unchanged.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TraceFilters {
    pub project_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Summary of a single trace as returned by `GET /traces`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSummary {
    pub id: String,
    pub task: String,
    pub status: String,
    pub timestamp: String,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub step_count: Option<u32>,
}

/// Paginated list response from `GET /traces`.
///
/// `next_cursor` is opaque — pass through to the next request unchanged.
/// `next_cursor == None` means no more pages.
/// `total` is always `None` in v1 (not computed server-side).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceListResponse {
    #[serde(default)]
    pub traces: Vec<TraceSummary>,
    #[serde(default)]
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub total: Option<u64>,
}

/// A single step in the parsed trajectory of a trace detail.
///
/// `duration_ms` is `None` on responses from servers that have not yet
/// adopted the matching read-side schema extension. In that case, use
/// `start_ms` and `end_ms` to compute the duration on the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    pub step: u32,
    pub action: String,
    #[serde(default)]
    pub args: serde_json::Value,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub start_ms: Option<u64>,
    #[serde(default)]
    pub end_ms: Option<u64>,
    /// Step duration in ms. Optional. `None` on responses from servers
    /// that have not yet adopted the read-side schema extension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
}

/// A linked pattern referenced from the trace's playbook context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedPattern {
    pub id: String,
    pub content: String,
    pub domain: String,
    pub helpful_score: i32,
}

/// Metadata about how linked_patterns were resolved (best-effort, 200ms cap).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedPatternsMeta {
    pub requested: u32,
    pub resolved: u32,
    #[serde(default)]
    pub missing_reason: Option<String>,
}

/// Full trace detail from `GET /traces/{trace_id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceDetail {
    pub id: String,
    pub task: String,
    pub status: String,
    pub timestamp: String,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub trajectory: Vec<TraceStep>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub linked_patterns: Vec<LinkedPattern>,
    pub linked_patterns_meta: LinkedPatternsMeta,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

// =============================================================================
// Configuration Types
// =============================================================================

/// Verbosity level for learn command output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum VerbosityLevel {
    #[default]
    Compact,
    Detailed,
}

/// ACE Configuration (loaded from config file).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AceConfig {
    #[serde(default = "default_server_url")]
    pub server_url: String,
    #[serde(default)]
    pub api_token: String,
    #[serde(default)]
    pub project_id: String,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_minutes: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orgs: Option<HashMap<String, OrgConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<VerbosityLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<UserAuth>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_org_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    /// Override the graph cache directory (tests only — use `TempDir` to avoid
    /// writing to `~/.ace-cache` during `cargo test`).
    #[serde(skip)]
    pub graph_cache_dir: Option<std::path::PathBuf>,
}

fn default_server_url() -> String {
    "https://ace-api.code-engine.app".to_string()
}

fn default_cache_ttl() -> u32 {
    120
}

impl Default for AceConfig {
    fn default() -> Self {
        Self {
            server_url: default_server_url(),
            api_token: String::new(),
            project_id: String::new(),
            cache_ttl_minutes: default_cache_ttl(),
            orgs: None,
            verbosity: None,
            auth: None,
            default_org_id: None,
            device_id: None,
            graph_cache_dir: None,
        }
    }
}

/// Organization-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgConfig {
    pub org_name: String,
    pub api_token: String,
    pub projects: Vec<String>,
}

/// Server-side configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedup_similarity_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedup_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constitution_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pruning_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_playbook_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_budget_enforcement: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_batch_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_learning_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reflector_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curator_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_settings: Option<AceRuntimeSettings>,
}

/// Server-side runtime settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AceRuntimeSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_min_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_min_confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summarization_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summarization_max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_min_helpful: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_default_section: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap_default_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap_default_thoroughness: Option<String>,
}

/// ACE runtime context combining config with server settings.
#[derive(Debug, Clone)]
pub struct AceContext {
    pub server_url: String,
    pub api_token: String,
    pub project_id: String,
    pub org_id: Option<String>,
    pub cache_ttl_minutes: u32,
    pub runtime_settings: AceRuntimeSettings,
}

// =============================================================================
// Auth Types
// =============================================================================

/// Organization membership info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgMembership {
    pub org_id: String,
    pub name: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// User authentication state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuth {
    pub token: String,
    pub user_id: String,
    pub email: String,
    #[serde(default)]
    pub organizations: Vec<OrgMembership>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub absolute_expires_at: Option<String>,
}

/// Device code response from /api/v1/auth/device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Token response from /api/v1/auth/device/token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<TokenUser>,
    // Flat shape (current server contract): user fields at top level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(default)]
    pub organizations: Vec<OrgMembership>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_expires_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub absolute_expires_at: Option<String>,
}

/// User info within token response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUser {
    pub user_id: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

impl TokenResponse {
    /// Resolve user from nested `user` field if present, else from flat top-level fields.
    ///
    /// Matches the TS SDK fallback in device-auth.ts:238 — the server currently
    /// returns flat shape but nested is kept for back-compat.
    pub fn resolved_user(&self) -> Result<TokenUser, String> {
        if let Some(user) = &self.user {
            return Ok(user.clone());
        }
        let user_id = self
            .user_id
            .clone()
            .ok_or_else(|| "TokenResponse missing both nested user and flat user_id".to_string())?;
        let email = self.email.clone().ok_or_else(|| {
            "TokenResponse missing both nested user.email and flat email".to_string()
        })?;
        Ok(TokenUser {
            user_id,
            email,
            name: self.name.clone(),
            image_url: self.image_url.clone(),
        })
    }
}

/// Refresh token response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub refresh_expires_in: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub absolute_expires_at: Option<String>,
}

/// Current user info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    pub user_id: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(default)]
    pub organizations: Vec<OrgMembership>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_org_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authenticated_at: Option<String>,
}

// =============================================================================
// Subscription Types
// =============================================================================

/// A single usage metric with current usage and limit.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageMetric {
    pub used: u32,
    pub limit: i32,
}

/// Plan tier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PlanTier {
    Free,
    Basic,
    Pro,
}

/// Subscription type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionType {
    Individual,
    Team,
}

/// Subscription status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    Trialing,
    ReadOnly,
    Blocked,
}

/// Complete usage information parsed from X-ACE-* headers.
#[derive(Debug, Clone)]
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

// =============================================================================
// Token Type Utilities
// =============================================================================

/// Token type detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    User,
    Org,
    Unknown,
}

/// Detect token type from format.
pub fn detect_token_type(token: &str) -> TokenType {
    if token.is_empty() {
        return TokenType::Unknown;
    }
    if token.starts_with("ace_user_") {
        return TokenType::User;
    }
    if token.starts_with("ace_") {
        return TokenType::Org;
    }
    TokenType::Unknown
}

/// Check if token is a user token.
pub fn is_user_token(token: &str) -> bool {
    detect_token_type(token) == TokenType::User
}

/// Check if token is an org token.
pub fn is_org_token(token: &str) -> bool {
    detect_token_type(token) == TokenType::Org
}

// =============================================================================
// Usage Helpers
// =============================================================================

/// Parse plan string into type and tier.
pub fn parse_plan(plan: &str) -> (SubscriptionType, PlanTier) {
    let parts: Vec<&str> = plan.split('/').collect();
    let sub_type = if parts.first() == Some(&"team") {
        SubscriptionType::Team
    } else {
        SubscriptionType::Individual
    };
    let tier = match parts.get(1) {
        Some(&"pro") => PlanTier::Pro,
        Some(&"basic") => PlanTier::Basic,
        _ => PlanTier::Free,
    };
    (sub_type, tier)
}

/// Calculate usage percentage.
pub fn get_usage_percentage(metric: &UsageMetric) -> u32 {
    if metric.limit <= 0 {
        return 0;
    }
    std::cmp::min(
        100,
        (metric.used as f64 / metric.limit as f64 * 100.0).round() as u32,
    )
}

/// Check if a metric is near limit (>80%).
pub fn is_near_limit(metric: &UsageMetric) -> bool {
    get_usage_percentage(metric) >= 80
}

/// Check if a metric has exceeded its limit.
pub fn is_over_limit(metric: &UsageMetric) -> bool {
    metric.limit > 0 && metric.used >= metric.limit as u32
}

/// Get feature availability based on plan.
pub fn get_features(sub_type: &SubscriptionType, tier: &PlanTier) -> PlanFeatures {
    PlanFeatures {
        teams: *sub_type == SubscriptionType::Team,
        sharing: if *sub_type == SubscriptionType::Team {
            *tier != PlanTier::Free
        } else {
            *tier == PlanTier::Pro
        },
        api_access: *tier != PlanTier::Free,
        priority_support: *tier == PlanTier::Pro,
    }
}

/// Plan features based on subscription.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanFeatures {
    pub teams: bool,
    pub sharing: bool,
    pub api_access: bool,
    pub priority_support: bool,
}

// =============================================================================
// SSE Streaming Types
// =============================================================================

/// Stage identifiers for SSE learning stream events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LearningStreamStage {
    Received,
    Analyzing,
    Synthesizing,
    Merging,
    Done,
    Error,
}

/// Individual SSE event from /traces/stream endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStreamEvent {
    pub stage: LearningStreamStage,
    pub message: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// =============================================================================
// Reflection Types
// =============================================================================

/// Output from Reflector agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reflection {
    pub operations: Vec<DeltaOperation>,
    pub summary: String,
}

// =============================================================================
// Token Expiry Types
// =============================================================================

/// Token expiry information from X-ACE-Token-Expires-In header.
#[derive(Debug, Clone)]
pub struct TokenExpiryInfo {
    pub expires_in_seconds: u64,
    pub received_at: u64,
}

// =============================================================================
// Context Resolution Types
// =============================================================================

/// Resolved context with org and project IDs.
#[derive(Debug, Clone)]
pub struct ResolvedContext {
    pub org_id: String,
    pub project_id: String,
    pub source: ContextSource,
}

/// Source of context resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextSource {
    Flags,
    Env,
    File,
    Default,
}

/// Options for context resolution.
#[derive(Debug, Clone, Default)]
pub struct ResolveContextOptions {
    pub org: Option<String>,
    pub project: Option<String>,
    pub cwd: Option<String>,
}

/// Default runtime settings used when server doesn't provide overrides.
pub fn default_runtime_settings() -> AceRuntimeSettings {
    AceRuntimeSettings {
        search_top_k: Some(10),
        search_threshold: Some(0.75),
        learning_enabled: Some(true),
        learning_min_tokens: Some(100),
        learning_min_confidence: Some(0.30),
        summarization_style: Some("detailed".to_string()),
        summarization_max_tokens: Some(1000),
        pattern_min_helpful: Some(0),
        pattern_default_section: None,
        bootstrap_default_mode: Some("hybrid".to_string()),
        bootstrap_default_thoroughness: Some("medium".to_string()),
    }
}

// =============================================================================
// Device Management Types
// =============================================================================

/// Authorized device information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub device_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    pub first_seen_at: String,
    pub last_seen_at: String,
    #[serde(default)]
    pub clients: Vec<String>,
    #[serde(default)]
    pub is_current: bool,
}

/// Device limit information for the user's account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLimit {
    pub current_devices: u32,
    pub max_devices: u32,
    #[serde(default)]
    pub is_custom: bool,
}

/// Result of removing a device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveDeviceResult {
    pub revoked_count: u32,
}

// =============================================================================
// Project Management Types
// =============================================================================

/// Project information accessible to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub project_id: String,
    pub project_name: String,
    pub org_id: String,
    pub org_name: String,
    pub created_at: String,
}

/// Response from /api/v1/auth/projects endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectsResponse {
    pub projects: Vec<Project>,
    #[serde(default)]
    pub count: u32,
}

// =============================================================================
// Batch Get Patterns Response
// =============================================================================

/// Response from batch pattern retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchGetPatternsResponse {
    #[serde(default)]
    pub patterns: Vec<PlaybookBullet>,
    #[serde(default)]
    pub found_count: u32,
    #[serde(default)]
    pub not_found: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_token_type() {
        assert_eq!(detect_token_type("ace_user_abc123"), TokenType::User);
        assert_eq!(detect_token_type("ace_12345678abc"), TokenType::Org);
        assert_eq!(detect_token_type("invalid"), TokenType::Unknown);
        assert_eq!(detect_token_type(""), TokenType::Unknown);
    }

    #[test]
    fn test_parse_plan() {
        let (t, tier) = parse_plan("individual/free");
        assert_eq!(t, SubscriptionType::Individual);
        assert_eq!(tier, PlanTier::Free);

        let (t, tier) = parse_plan("team/pro");
        assert_eq!(t, SubscriptionType::Team);
        assert_eq!(tier, PlanTier::Pro);
    }

    #[test]
    fn test_usage_percentage() {
        let m = UsageMetric {
            used: 80,
            limit: 100,
        };
        assert_eq!(get_usage_percentage(&m), 80);
        assert!(is_near_limit(&m));
        assert!(!is_over_limit(&m));

        let m2 = UsageMetric {
            used: 100,
            limit: 100,
        };
        assert!(is_over_limit(&m2));
    }

    #[test]
    fn test_thoroughness_presets() {
        let preset = get_thoroughness_preset(&ThoroughnessLevel::Light);
        assert_eq!(preset.max_files, 1000);
        assert_eq!(preset.commit_limit, 100);

        let preset = get_thoroughness_preset(&ThoroughnessLevel::Deep);
        assert_eq!(preset.max_files, -1);
    }

    #[test]
    fn test_ace_config_default() {
        let config = AceConfig::default();
        assert_eq!(config.server_url, "https://ace-api.code-engine.app");
        assert_eq!(config.cache_ttl_minutes, 120);
    }

    // =========================================================================
    // TrajectoryStep / TraceStep timing fields (additive, optional)
    //
    // Mirrors the TypeScript reference on branch
    // `feat/trajectory-timing-fields-ts` (commit c60ec7f). Server confirmed
    // lenient validation, so these are pure-additive optional fields.
    // =========================================================================

    #[test]
    fn trajectory_step_timing_fields_round_trip_when_present() {
        let step = TrajectoryStep {
            step: 1,
            action: "Read".to_string(),
            args: HashMap::new(),
            result: None,
            start_ms: Some(1_700_000_000_000),
            end_ms: Some(1_700_000_000_150),
            duration_ms: Some(150),
        };

        let json = serde_json::to_string(&step).expect("serialize");
        assert!(json.contains("\"start_ms\":1700000000000"), "json: {json}");
        assert!(json.contains("\"end_ms\":1700000000150"), "json: {json}");
        assert!(json.contains("\"duration_ms\":150"), "json: {json}");

        let decoded: TrajectoryStep = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.start_ms, Some(1_700_000_000_000));
        assert_eq!(decoded.end_ms, Some(1_700_000_000_150));
        assert_eq!(decoded.duration_ms, Some(150));
    }

    #[test]
    fn trajectory_step_timing_fields_omitted_when_none() {
        let step = TrajectoryStep {
            step: 2,
            action: "Write".to_string(),
            args: HashMap::new(),
            result: None,
            start_ms: None,
            end_ms: None,
            duration_ms: None,
        };

        let json = serde_json::to_string(&step).expect("serialize");
        assert!(
            !json.contains("start_ms"),
            "start_ms must be omitted when None: {json}"
        );
        assert!(
            !json.contains("end_ms"),
            "end_ms must be omitted when None: {json}"
        );
        assert!(
            !json.contains("duration_ms"),
            "duration_ms must be omitted when None: {json}"
        );
    }

    #[test]
    fn trajectory_step_preserves_clock_skew_round_trip() {
        // Ground-truth duration may diverge from end-start due to client
        // clock skew; SDK must preserve the values byte-for-byte and not
        // "correct" them.
        let step = TrajectoryStep {
            step: 3,
            action: "Bash".to_string(),
            args: HashMap::new(),
            result: None,
            start_ms: Some(1000),
            end_ms: Some(900),
            duration_ms: Some(150),
        };

        let json = serde_json::to_string(&step).expect("serialize");
        let decoded: TrajectoryStep = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(decoded.start_ms, Some(1000));
        assert_eq!(decoded.end_ms, Some(900));
        assert_eq!(decoded.duration_ms, Some(150));
    }

    #[test]
    fn trace_step_duration_ms_deserializes_when_present() {
        let body = r#"{
            "step": 1,
            "action": "Read",
            "args": {"path": "src/lib.rs"},
            "result": null,
            "start_ms": 1000,
            "end_ms": 1200,
            "duration_ms": 200
        }"#;
        let step: TraceStep = serde_json::from_str(body).expect("deserialize");
        assert_eq!(step.duration_ms, Some(200));
        assert_eq!(step.start_ms, Some(1000));
        assert_eq!(step.end_ms, Some(1200));
    }

    #[test]
    fn trace_step_duration_ms_defaults_to_none_when_missing() {
        // Older servers that have not adopted the read-side schema extension
        // will omit duration_ms entirely. Deserialization must NOT fail.
        let body = r#"{
            "step": 1,
            "action": "Read",
            "args": {"path": "src/lib.rs"},
            "result": null,
            "start_ms": 1000,
            "end_ms": 1200
        }"#;
        let step: TraceStep = serde_json::from_str(body).expect("deserialize");
        assert_eq!(step.duration_ms, None);
    }

    #[test]
    fn trace_step_tolerates_unknown_fields_forward_compat() {
        // Regression-guard: serde's default behavior is to ignore unknown
        // fields. If anyone adds #[serde(deny_unknown_fields)] in the
        // future, this test will fail loudly — that attribute would block
        // forward-compat with new server-side fields.
        let body = r#"{
            "step": 1,
            "action": "Read",
            "args": {},
            "duration_ms": 42,
            "future_field_added_by_server": "totally fine"
        }"#;
        let step: TraceStep =
            serde_json::from_str(body).expect("must tolerate unknown fields for forward-compat");
        assert_eq!(step.duration_ms, Some(42));
    }
}
