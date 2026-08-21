use std::sync::Arc;
use std::time::Duration;
use dashmap::DashMap;
use tracing::{info, warn};
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSICheckpoint {
    pub agent_name: String,
    pub threshold: f64,
    pub interval_secs: u64,
    pub timestamp: String,
}

pub struct RSIEngine {
    store: Arc<crate::memory::store::MemoryStore>,
    thresholds: Arc<DashMap<String, f64>>,
    intervals: Arc<DashMap<String, Duration>>,
}

impl RSIEngine {
    pub fn new(store: Arc<crate::memory::store::MemoryStore>) -> Self {
        RSIEngine {
            store,
            thresholds: Arc::new(DashMap::new()),
            intervals: Arc::new(DashMap::new()),
        }
    }

    pub fn store(&self) -> Arc<crate::memory::store::MemoryStore> {
        self.store.clone()
    }

    pub fn get_threshold(&self, agent: &str) -> f64 {
        self.thresholds
            .get(agent)
            .map(|r| *r)
            .unwrap_or(0.7)
    }

    pub fn set_threshold(&self, agent: &str, threshold: f64) {
        let clamped = threshold.max(0.3).min(0.95);
        self.thresholds.insert(agent.to_string(), clamped);
    }

    pub fn get_interval(&self, agent: &str) -> Duration {
        self.intervals
            .get(agent)
            .map(|r| *r)
            .unwrap_or(Duration::from_secs(300)) // default 5 minutes
    }

    pub fn set_interval(&self, agent: &str, interval: Duration) {
        let min_secs = 5;
        let max_secs = 3600;
        let secs = interval.as_secs().max(min_secs).min(max_secs);
        self.intervals.insert(agent.to_string(), Duration::from_secs(secs));
    }

    pub async fn evaluate_and_adjust(&self, agent_name: &str) {
        let success_rate = self.store.get_agent_success_rate(agent_name, 20).await;
        let current_threshold = self.get_threshold(agent_name);
        let current_interval = self.get_interval(agent_name);

        // Poor performance — be more conservative
        if success_rate < 0.6 {
            let new_threshold = (current_threshold + 0.1).min(0.95);
            self.set_threshold(agent_name, new_threshold);

            let new_interval = current_interval * 13 / 10; // * 1.3
            self.set_interval(agent_name, new_interval);

            warn!(
                "{}: low accuracy ({:.0}%), raising threshold to {:.2} and slowing to {}s",
                agent_name,
                success_rate * 100.0,
                new_threshold,
                new_interval.as_secs()
            );
        }
        // High performance — be more aggressive
        else if success_rate > 0.85 {
            let new_threshold = (current_threshold - 0.05).max(0.4);
            self.set_threshold(agent_name, new_threshold);

            let new_interval = (current_interval * 10 / 12).max(Duration::from_secs(5)); // / 1.2
            self.set_interval(agent_name, new_interval);

            info!(
                "{}: high accuracy ({:.0}%), lowering threshold to {:.2} and speeding to {}s",
                agent_name,
                success_rate * 100.0,
                new_threshold,
                new_interval.as_secs()
            );
        }
    }

    pub fn save_checkpoint(&self, agent_name: &str) -> RSICheckpoint {
        RSICheckpoint {
            agent_name: agent_name.to_string(),
            threshold: self.get_threshold(agent_name),
            interval_secs: self.get_interval(agent_name).as_secs(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    pub fn load_checkpoint(&self, checkpoint: &RSICheckpoint) {
        self.set_threshold(&checkpoint.agent_name, checkpoint.threshold);
        self.set_interval(&checkpoint.agent_name, Duration::from_secs(checkpoint.interval_secs));
        info!(
            "{}: loaded checkpoint (threshold={:.2}, interval={}s)",
            checkpoint.agent_name, checkpoint.threshold, checkpoint.interval_secs
        );
    }

    pub fn save_all_checkpoints(&self, agents: &[String]) -> Vec<RSICheckpoint> {
        agents.iter().map(|a| self.save_checkpoint(a)).collect()
    }

    pub fn load_all_checkpoints(&self, checkpoints: Vec<RSICheckpoint>) {
        for checkpoint in checkpoints {
            self.load_checkpoint(&checkpoint);
        }
    }
}
