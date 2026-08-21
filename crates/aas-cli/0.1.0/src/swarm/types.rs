use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Domain {
    Repository,
    Logs,
    Metrics,
    Health,
    Task,
    Trace,
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Domain::Repository => write!(f, "repository"),
            Domain::Logs => write!(f, "logs"),
            Domain::Metrics => write!(f, "metrics"),
            Domain::Health => write!(f, "health"),
            Domain::Task => write!(f, "task"),
            Domain::Trace => write!(f, "trace"),
        }
    }
}

impl Domain {
    pub fn all() -> Vec<Domain> {
        vec![
            Domain::Repository,
            Domain::Logs,
            Domain::Metrics,
            Domain::Health,
            Domain::Task,
            Domain::Trace,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Critical => write!(f, "critical"),
            Severity::High => write!(f, "high"),
            Severity::Medium => write!(f, "medium"),
            Severity::Low => write!(f, "low"),
            Severity::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Stage {
    Detected,
    Analyzing,
    Planned,
    Testing,
    Validating,
    Executing,
    Verifying,
    Completed,
    Failed,
    RolledBack,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stage::Detected => write!(f, "detected"),
            Stage::Analyzing => write!(f, "analyzing"),
            Stage::Planned => write!(f, "planned"),
            Stage::Testing => write!(f, "testing"),
            Stage::Validating => write!(f, "validating"),
            Stage::Executing => write!(f, "executing"),
            Stage::Verifying => write!(f, "verifying"),
            Stage::Completed => write!(f, "completed"),
            Stage::Failed => write!(f, "failed"),
            Stage::RolledBack => write!(f, "rolled_back"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub domain: Domain,
    pub agent_name: String,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub signature: String,
    pub stage: Stage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub issue_id: String,
    pub agent_name: String,
    pub root_cause: String,
    pub impact: String,
    pub suggested_approaches: Vec<Approach>,
    pub confidence: f64,
    pub reasoning: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approach {
    pub name: String,
    pub description: String,
    pub expected_success_rate: f64,
    pub execution_time_estimate: String,
    pub risks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub issue_id: String,
    pub agent_name: String,
    pub approach_name: String,
    pub description: String,
    pub commands: Vec<String>,
    pub rollback_commands: Vec<String>,
    pub files_to_modify: Vec<String>,
    pub stage: Stage,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action_id: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub stage: Stage,
    pub verification_passed: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub issue: Issue,
    pub analysis: Option<Analysis>,
    pub action: Option<Action>,
    pub result: Option<ActionResult>,
    pub status: DecisionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecisionStatus {
    Detected,
    Analyzing,
    AwaitingApproval,
    InProgress,
    Completed,
    Failed,
    RolledBack,
    Rejected,
}

impl fmt::Display for DecisionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecisionStatus::Detected => write!(f, "detected"),
            DecisionStatus::Analyzing => write!(f, "analyzing"),
            DecisionStatus::AwaitingApproval => write!(f, "awaiting_approval"),
            DecisionStatus::InProgress => write!(f, "in_progress"),
            DecisionStatus::Completed => write!(f, "completed"),
            DecisionStatus::Failed => write!(f, "failed"),
            DecisionStatus::RolledBack => write!(f, "rolled_back"),
            DecisionStatus::Rejected => write!(f, "rejected"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub domain: Domain,
    pub indicators: Vec<String>,
    pub solution_description: String,
    pub confidence: f64,
    pub occurrences: u32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub avg_execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub id: String,
    pub agent_name: String,
    pub predicted_issue: String,
    pub description: String,
    pub confidence: f64,
    pub time_until_expected: String,
    pub suggested_action: String,
    pub based_on_pattern: Option<String>,
    pub created_at: DateTime<Utc>,
    pub status: PredictionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PredictionStatus {
    Active,
    Averted,
    Occurred,
    Expired,
}

impl fmt::Display for PredictionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PredictionStatus::Active => write!(f, "active"),
            PredictionStatus::Averted => write!(f, "averted"),
            PredictionStatus::Occurred => write!(f, "occurred"),
            PredictionStatus::Expired => write!(f, "expired"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CyclePerformance {
    pub id: String,
    pub agent_name: String,
    pub cycle_duration_ms: u64,
    pub issues_found: u32,
    pub actions_attempted: u32,
    pub actions_succeeded: u32,
    pub confidence_threshold: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    AgentStarted {
        agent: String,
        domain: Domain,
        timestamp: DateTime<Utc>,
    },
    AgentStopped {
        agent: String,
        timestamp: DateTime<Utc>,
    },
    AgentError {
        agent: String,
        error: String,
        timestamp: DateTime<Utc>,
    },
    IssueDetected {
        agent: String,
        issue: Issue,
        timestamp: DateTime<Utc>,
    },
    IssueAnalyzed {
        agent: String,
        issue_id: String,
        analysis: Analysis,
        timestamp: DateTime<Utc>,
    },
    ActionPlanned {
        agent: String,
        action: Action,
        timestamp: DateTime<Utc>,
    },
    ActionStarted {
        agent: String,
        action_id: String,
        stage: Stage,
        timestamp: DateTime<Utc>,
    },
    ActionCompleted {
        agent: String,
        action_id: String,
        result: ActionResult,
        timestamp: DateTime<Utc>,
    },
    ActionFailed {
        agent: String,
        action_id: String,
        stage: Stage,
        error: String,
        timestamp: DateTime<Utc>,
    },
    ActionRolledBack {
        agent: String,
        action_id: String,
        timestamp: DateTime<Utc>,
    },
    EscalationNeeded {
        agent: String,
        issue_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    PredictionMade {
        agent: String,
        prediction: Prediction,
        timestamp: DateTime<Utc>,
    },
    Learned {
        agent: String,
        pattern_id: String,
        confidence: f64,
        timestamp: DateTime<Utc>,
    },
    HyperfocusRequest {
        agent: String,
        reason: String,
        duration_secs: u64,
        timestamp: DateTime<Utc>,
    },
    ContextSwitch {
        from_agent: String,
        to_agent: String,
        trigger_issue_id: String,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub name: String,
    pub domain: Domain,
    pub running: bool,
    pub healthy: bool,
    pub uptime_seconds: u64,
    pub issues_detected: u64,
    pub actions_taken: u64,
    pub success_rate: f64,
    pub last_check: Option<DateTime<Utc>>,
    pub last_decision: Option<DateTime<Utc>>,
    pub current_issue: Option<String>,
    pub memory_usage_mb: f64,
    // RSI observability
    pub rsi_confidence_threshold: Option<f64>,
    pub rsi_interval_secs: Option<u64>,
}
