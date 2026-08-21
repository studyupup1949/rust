use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackPayload {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub category: AttackCategory,
    pub severity: Severity,
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AttackCategory {
    PromptInjection,
    Jailbreak,
    RoleConfusion,
    DataExfiltration,
    Custom,
}

impl std::fmt::Display for AttackCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttackCategory::PromptInjection => write!(f, "Prompt Injection"),
            AttackCategory::Jailbreak => write!(f, "Jailbreak"),
            AttackCategory::RoleConfusion => write!(f, "Role Confusion"),
            AttackCategory::DataExfiltration => write!(f, "Data Exfiltration"),
            AttackCategory::Custom => write!(f, "Custom"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn to_score(&self) -> u8 {
        match self {
            Severity::Low => 25,
            Severity::Medium => 50,
            Severity::High => 75,
            Severity::Critical => 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackSuite {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: AttackCategory,
    pub payloads: Vec<AttackPayload>,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackResult {
    pub id: Uuid,
    pub payload_id: String,
    pub payload_name: String,
    pub category: AttackCategory,
    pub severity: Severity,
    pub prompt: String,
    pub response: String,
    pub success: bool,
    pub risk_score: u8,
    pub timestamp: DateTime<Utc>,
    pub execution_time_ms: u64,
    pub detection_reason: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRun {
    pub id: Uuid,
    pub model: String,
    pub provider: String,
    pub timestamp: DateTime<Utc>,
    pub total_attacks: usize,
    pub successful_attacks: usize,
    pub failed_attacks: usize,
    pub overall_risk_score: u8,
    pub results: Vec<AttackResult>,
    pub category_summary: HashMap<AttackCategory, CategorySummary>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub category: AttackCategory,
    pub total: usize,
    pub successful: usize,
    pub average_risk_score: f64,
    pub max_severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}
