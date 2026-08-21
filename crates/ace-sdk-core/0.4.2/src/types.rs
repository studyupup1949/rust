//! ACE SDK type definitions
//!
//! All types used throughout the SDK, with serde serialization support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SDK version constant
pub const CORE_VERSION: &str = "0.2.0";

// =============================================================================
// Execution Trace
// =============================================================================

/// A single step in an execution trajectory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStep {
    pub step: u32,
    pub action: String,
    pub args: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}

/// Execution trace capturing task details for pattern learning.
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
    /// WHY - underlying principle (default: "")
    #[serde(default)]
    pub root_cause: String,
    /// WHAT - specific error/problem addressed (default: "")
    #[serde(default)]
    pub error_context: String,
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
    pub helpful_delta: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub harmful_delta: Option<i32>,
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
}
