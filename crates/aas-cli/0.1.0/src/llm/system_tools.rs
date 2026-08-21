use crate::rsi::RSIEngine;
use crate::memory::store::MemoryStore;
use crate::memory::patterns::PatternEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSelfState {
    pub name: String,
    pub confidence_threshold: f64,
    pub polling_interval_secs: u64,
    pub success_rate: f64,
    pub cycles_last_20: u64,
    pub actions_succeeded: u64,
    pub actions_failed: u64,
    pub recent_patterns_cached: usize,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAnalysis {
    pub agent_name: String,
    pub current_state: AgentSelfState,
    pub identified_issues: Vec<String>,
    pub proposed_improvements: Vec<Improvement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Improvement {
    pub title: String,
    pub reason: String,
    pub action: String,
    pub expected_benefit: String,
}

pub struct SystemTools {
    rsi: Arc<RSIEngine>,
    memory: Arc<MemoryStore>,
    patterns: Arc<PatternEngine>,
}

impl SystemTools {
    pub fn new(
        rsi: Arc<RSIEngine>,
        memory: Arc<MemoryStore>,
        patterns: Arc<PatternEngine>,
    ) -> Self {
        SystemTools { rsi, memory, patterns }
    }

    pub async fn get_self_state(&self, agent_name: &str) -> AgentSelfState {
        let success_rate = self.memory.get_agent_success_rate(agent_name, 20).await;
        let history = self.memory.get_cycle_history(agent_name, 20).await;

        let actions_succeeded = history.iter().map(|c| c.actions_succeeded as u64).sum();
        let actions_failed = history.iter()
            .map(|c| (c.actions_attempted - c.actions_succeeded) as u64)
            .sum();

        // TODO: implement pattern count tracking
        let cached_patterns = 0usize;

        let recommendation = if success_rate < 0.6 {
            "CONSERVATIVE: Raise threshold, slow down polling".to_string()
        } else if success_rate > 0.85 {
            "AGGRESSIVE: Lower threshold, speed up polling".to_string()
        } else {
            "STABLE: Maintain current parameters".to_string()
        };

        AgentSelfState {
            name: agent_name.to_string(),
            confidence_threshold: self.rsi.get_threshold(agent_name),
            polling_interval_secs: self.rsi.get_interval(agent_name).as_secs(),
            success_rate,
            cycles_last_20: history.len() as u64,
            actions_succeeded,
            actions_failed,
            recent_patterns_cached: cached_patterns,
            recommendation,
        }
    }

    pub async fn self_analyze(&self, agent_name: &str) -> SystemAnalysis {
        let state = self.get_self_state(agent_name).await;
        let mut issues = Vec::new();
        let mut improvements = Vec::new();

        // Issue 1: Low success rate but high threshold
        if state.success_rate < 0.6 && state.confidence_threshold < 0.8 {
            issues.push("Low success rate despite reasonable threshold - may be task mismatch".to_string());
            improvements.push(Improvement {
                title: "Request capability review".to_string(),
                reason: "Consistent failures suggest this agent lacks capability for task type".to_string(),
                action: "emit_capability_request".to_string(),
                expected_benefit: "Identify missing capabilities needed".to_string(),
            });
        }

        // Issue 2: High success rate but aggressive polling
        if state.success_rate > 0.85 && state.polling_interval_secs < 30 {
            issues.push("High success but wasting cycles - over-polling detected".to_string());
            improvements.push(Improvement {
                title: "Increase polling interval".to_string(),
                reason: "Success rate is excellent, can afford to check less frequently".to_string(),
                action: "set_interval:120".to_string(),
                expected_benefit: "Save cycles without sacrificing quality".to_string(),
            });
        }

        // Issue 3: Many cached patterns but low pattern reuse
        if state.recent_patterns_cached > 10 && state.success_rate < 0.75 {
            issues.push("Patterns cached but not being reused effectively".to_string());
            improvements.push(Improvement {
                title: "Lower confidence threshold for pattern matching".to_string(),
                reason: "Cached patterns exist but threshold too high to use them".to_string(),
                action: "set_threshold:0.5".to_string(),
                expected_benefit: "Reuse learned solutions more aggressively".to_string(),
            });
        }

        // Issue 4: No patterns cached despite history
        if state.recent_patterns_cached == 0 && state.cycles_last_20 > 5 {
            issues.push("No patterns learned despite operational history".to_string());
            improvements.push(Improvement {
                title: "Verify pattern storage".to_string(),
                reason: "Should have learned patterns from 20 cycles of operation".to_string(),
                action: "diagnose:pattern_learning".to_string(),
                expected_benefit: "Ensure pattern system is capturing learnings".to_string(),
            });
        }

        SystemAnalysis {
            agent_name: agent_name.to_string(),
            current_state: state,
            identified_issues: issues,
            proposed_improvements: improvements,
        }
    }

    pub fn request_threshold_change(&self, agent_name: &str, new_threshold: f64, reason: &str) -> Result<f64, String> {
        if new_threshold < 0.3 || new_threshold > 0.95 {
            return Err(format!("Threshold {} out of bounds [0.3, 0.95]", new_threshold));
        }
        self.rsi.set_threshold(agent_name, new_threshold);
        tracing::info!("{}: self-requested threshold → {:.2} ({})", agent_name, new_threshold, reason);
        Ok(new_threshold)
    }

    pub fn request_interval_change(&self, agent_name: &str, new_interval_secs: u64, reason: &str) -> Result<u64, String> {
        if new_interval_secs < 5 || new_interval_secs > 3600 {
            return Err(format!("Interval {} out of bounds [5, 3600]", new_interval_secs));
        }
        self.rsi.set_interval(agent_name, std::time::Duration::from_secs(new_interval_secs));
        tracing::info!("{}: self-requested interval → {}s ({})", agent_name, new_interval_secs, reason);
        Ok(new_interval_secs)
    }

    pub async fn can_execute(&self, agent_name: &str, confidence: f64) -> bool {
        let threshold = self.rsi.get_threshold(agent_name);
        confidence >= threshold
    }

    pub fn get_status_summary(&self, agent_names: &[String]) -> String {
        format!("System has {} active agents", agent_names.len())
    }
}
