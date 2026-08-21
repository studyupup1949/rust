//! Serde-derived types for AAE protocol schemas.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Blast radius classification for proposed effects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlastRadius {
    /// No mutating side effects.
    ReadOnly,
    /// Affects a single service on a single host.
    SingleService,
    /// Affects a single host (multiple services possible).
    SingleHost,
    /// Affects multiple hosts.
    MultiHost,
    /// Effects cannot be cleanly undone.
    Irreversible,
}

/// Policy engine decision values.
///
/// v0.8: the enum is closed at exactly these three values. Unknown decision
/// values — including the two refined v0.3 values retired in v0.8 — MUST
/// fail deserialization; consumers reject rather than coerce.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    /// Plan may proceed; capability token MUST be minted.
    Allow,
    /// Plan is rejected; lifecycle terminates.
    Deny,
    /// Plan requires human approval before proceeding.
    RequireApproval,
}

/// v0.3: whether a tool's effects can be undone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reversibility {
    /// Effects can be fully undone.
    Reversible,
    /// Some effects can be undone.
    PartiallyReversible,
    /// Effects cannot be undone.
    Irreversible,
}

/// v0.7: the standard `ext.confidence` shape on policy decisions.
///
/// Answers "how sure is the engine?" without new decision values: a
/// low-confidence allow is still an allow (G-2-safe); hosts MAY route it to
/// spot checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Confidence {
    /// Confidence in [0, 1].
    pub score: f64,
    /// What the score is derived from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basis: Option<String>,
}

/// v0.3: the standard `ext.cost_estimate` shape in step previews.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    /// ISO 4217 code or deployment-defined unit.
    pub currency: String,
    /// Best estimate.
    pub amount: f64,
    /// Upper bound if known; cost-aware policies should use it conservatively.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_max: Option<f64>,
    /// What the estimate is derived from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basis: Option<String>,
    /// Confidence in [0, 1].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

/// Core strictness mode: gateway executes exactly the approved steps in
/// order; deviation aborts. This is the REQUIRED default (SPEC §5.3).
pub const STRICTNESS_STRICT_LITERAL: &str = "strict_literal";

/// Core strictness mode: gateway executes steps matching templates with
/// parameter substitution (SPEC §5.3, OPTIONAL).
pub const STRICTNESS_STRICT_TEMPLATE: &str = "strict_template";

fn default_strictness() -> String {
    STRICTNESS_STRICT_LITERAL.to_string()
}

/// A single tool invocation within a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Name of a registered tool.
    pub tool: String,
    /// Arguments matching the tool's `plan_schema`.
    pub args: serde_json::Value,
    /// Optional success criteria.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<serde_json::Value>,
    /// Blast radius classification.
    pub blast_radius: BlastRadius,
}

/// Context fields on a proposal. Open shape; rationale is required.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// Human-readable explanation of why this plan was proposed.
    pub rationale: String,
    /// Optional reference to upstream cause.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triggered_by: Option<String>,
    /// If this is a re-proposal, the previous `proposal_id`. Also the carrier
    /// of the v0.8 negotiation idiom (SPEC §5.5): a proposal submitted in
    /// response to a guided `deny` references the original here.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_from: Option<String>,
    /// Host-specific extensions (use namespaced keys).
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A typed plan submitted by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Protocol version.
    pub aae_version: String,
    /// ULID uniquely identifying this proposal.
    pub proposal_id: String,
    /// Stable identifier for the agent submitting this proposal.
    pub agent_id: String,
    /// v0.3, optional (G-1): ordered provenance for proposals submitted on
    /// behalf of other agents. Last entry must equal `agent_id`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_chain: Option<Vec<String>>,
    /// Tenant scope for multitenant hosts.
    pub tenant_id: String,
    /// Slug describing the agent's intent.
    pub intent: String,
    /// Context fields.
    pub context: Context,
    /// Steps to execute.
    pub steps: Vec<Step>,
    /// Submission timestamp.
    pub submitted_at: DateTime<Utc>,
}

/// A predicted effect of executing a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    /// Effect type (e.g., `service_state_change`, `file_write`, `read_only`).
    #[serde(rename = "type")]
    pub effect_type: String,
    /// Resource being affected.
    pub target: String,
    /// Prior state, if applicable.
    #[serde(skip_serializing_if = "Option::is_none", rename = "from")]
    pub from_state: Option<String>,
    /// Predicted state, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// Effect-specific details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Preview output for a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepPreview {
    /// Index of this step within the proposal's steps array.
    pub step_index: usize,
    /// Predicted effects.
    pub predicted_effects: Vec<Effect>,
    /// Estimated duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_duration_ms: Option<u64>,
    /// Optional human-readable diff.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
    /// Warnings produced during preview.
    #[serde(default)]
    pub warnings: Vec<String>,
    /// If this step's preview was unsupported, the reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_unsupported_reason: Option<String>,
    /// v0.3: non-normative extension fields (`ext.*` namespace).
    /// Standardized shapes: `cost_estimate` ([`CostEstimate`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext: Option<serde_json::Value>,
}

/// Predicted effects of executing a proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preview {
    /// Protocol version.
    pub aae_version: String,
    /// ULID of this preview.
    pub preview_id: String,
    /// ULID of the proposal this previews.
    pub proposal_id: String,
    /// Per-step preview outputs.
    pub step_previews: Vec<StepPreview>,
    /// Aggregate (worst-case) blast radius.
    pub aggregate_blast_radius: BlastRadius,
    /// True if any step's preview was unsupported.
    pub preview_unsupported: bool,
    /// Generation timestamp.
    pub generated_at: DateTime<Utc>,
}

/// Output of policy engine evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// Protocol version.
    pub aae_version: String,
    /// ULID of this decision.
    pub decision_id: String,
    /// ULID of the proposal this evaluates.
    pub proposal_id: String,
    /// The decision value.
    pub decision: Decision,
    /// Identifier of the policy bundle/version used.
    pub policy_version: String,
    /// Identifiers of rules consulted during evaluation.
    pub rules_evaluated: Vec<String>,
    /// Human-readable explanation.
    pub reason: String,
    /// Required approvers, if decision is `RequireApproval`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_approvers: Option<Vec<String>>,
    /// Expiration time, if decision is Allow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    /// Plan-to-execution binding strictness. v0.8: an open string. Core
    /// normatively defines [`STRICTNESS_STRICT_LITERAL`] (default) and
    /// [`STRICTNESS_STRICT_TEMPLATE`]; any other value is a
    /// companion/extension mode, and a host that does not implement the
    /// requested mode MUST fail closed (C-16), never downgrade.
    #[serde(default = "default_strictness")]
    pub strictness: String,
    /// Decision timestamp.
    pub decided_at: DateTime<Utc>,
    /// v0.7: non-normative extension fields (`ext.*`). Standardized shape:
    /// `confidence` ([`Confidence`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext: Option<serde_json::Value>,
}

/// Capability token scope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityScope {
    /// Tool name authorized by this token.
    pub tool: String,
    /// SHA-256 hash of approved steps (for plan integrity verification).
    pub approved_steps_hash: String,
    /// Tool-specific constraints (e.g., for `ssh_exec`: host, user).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_constraints: Option<serde_json::Value>,
}

/// Capability token claims (signed externally as JWT/PASETO/biscuit).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    /// Protocol version.
    pub aae_version: String,
    /// ULID of this token.
    pub token_id: String,
    /// ULID of the proposal this authorizes.
    pub proposal_id: String,
    /// ULID of the policy decision this derives from.
    pub decision_id: String,
    /// Token scope.
    pub scope: CapabilityScope,
    /// Issued-at (Unix seconds).
    pub iat: i64,
    /// Expiration (Unix seconds).
    pub exp: i64,
    /// Issuer identifier.
    pub iss: String,
    /// Subject (`agent_id`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    /// Maximum number of times this token may be presented.
    pub max_uses: u32,
}

/// Optional digital signature on an audit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSignature {
    /// Signature algorithm.
    pub alg: String,
    /// Signing key identifier.
    pub key_id: String,
    /// Base64-encoded signature value.
    pub value: String,
}

/// Event type binding an out-of-chain artifact to a chain event by content
/// hash (v0.8). Required payload fields: `artifact_hash` (algorithm-prefixed
/// canonical hash), `bound_event_id`, `artifact_kind` (free-form string,
/// e.g. `legible_record`, `session_recording`). Artifacts are advisory; the
/// chain event is authoritative.
pub const EVENT_TYPE_ARTIFACT_ATTESTED: &str = "artifact_attested";

/// A single hash-chained audit log record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Protocol version.
    pub aae_version: String,
    /// ULID of this event.
    pub event_id: String,
    /// Canonical event type (or namespaced extension). Open string; core
    /// v0.8 adds [`EVENT_TYPE_ARTIFACT_ATTESTED`]. Consumers MUST ignore
    /// unknown event types.
    pub event_type: String,
    /// ULID of the proposal this relates to.
    pub proposal_id: String,
    /// Tenant scope.
    pub tenant_id: String,
    /// Agent identifier.
    pub agent_id: String,
    /// Who emitted this event (agent, host, `policy_engine`, gateway, `audit_sink`, human:<id>).
    pub actor: String,
    /// Event timestamp.
    pub ts: DateTime<Utc>,
    /// Event-specific payload.
    pub payload: serde_json::Value,
    /// Hash of the previous event in the chain (None for genesis).
    pub prev_event_hash: Option<String>,
    /// SHA-256 of `canonicalize(event_without_this_hash)` || `prev_event_hash`.
    pub this_event_hash: String,
    /// Optional digital signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<EventSignature>,
}

/// Tool registration declaring the tool's plan/preview contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRegistration {
    /// Protocol version.
    pub aae_version: String,
    /// Tool name (matched against Step.tool).
    pub name: String,
    /// Optional human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema for the args object of a step using this tool.
    pub plan_schema: serde_json::Value,
    /// Optional JSON Schema for `predicted_effects` produced by this tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_schema: Option<serde_json::Value>,
    /// Whether this tool supports preview.
    pub preview_supported: bool,
    /// Default blast radius if step does not declare one.
    pub default_blast_radius: BlastRadius,
    /// Whether this tool requires the AAE lifecycle.
    pub aae_required: bool,
    /// v0.3 optional (G-4): opaque data-class identifiers this tool may touch.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub data_classes_touched: Vec<String>,
    /// v0.3 optional (G-4): whether operations are compliance-relevant.
    #[serde(default)]
    pub compliance_relevant: bool,
    /// v0.3 optional (G-4): systems of record this tool can modify.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub systems_of_record: Vec<String>,
    /// v0.3 optional (G-4): whether this tool's effects can be undone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reversibility: Option<Reversibility>,
}
