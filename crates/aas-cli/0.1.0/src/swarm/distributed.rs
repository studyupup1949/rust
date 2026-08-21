use crate::rsi::RSIEngine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMetrics {
    pub swarm_id: String,
    pub timestamp: String,
    pub agent_metrics: HashMap<String, AgentMetrics>,
    pub overall_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub name: String,
    pub success_rate: f64,
    pub threshold: f64,
    pub interval_secs: u64,
    pub cycles_completed: u64,
}

pub struct DistributedRSI {
    local_rsi: Arc<RSIEngine>,
    swarm_id: String,
    peer_metrics: dashmap::DashMap<String, SwarmMetrics>,
}

impl DistributedRSI {
    pub fn new(swarm_id: String, rsi: Arc<RSIEngine>) -> Self {
        DistributedRSI {
            local_rsi: rsi,
            swarm_id,
            peer_metrics: dashmap::DashMap::new(),
        }
    }

    // Export local metrics for sharing with other swarms
    pub async fn export_metrics(
        &self,
        agents: &[String],
        store: Arc<crate::memory::store::MemoryStore>,
    ) -> SwarmMetrics {
        let mut agent_metrics = HashMap::new();

        for agent_name in agents {
            let success_rate = store.get_agent_success_rate(agent_name, 20).await;
            let threshold = self.local_rsi.get_threshold(agent_name);
            let interval_secs = self.local_rsi.get_interval(agent_name).as_secs();

            let cycles = store.get_cycle_history(agent_name, 100).await;
            let cycles_completed = cycles.len() as u64;

            agent_metrics.insert(
                agent_name.to_string(),
                AgentMetrics {
                    name: agent_name.clone(),
                    success_rate,
                    threshold,
                    interval_secs,
                    cycles_completed,
                },
            );
        }

        let overall_success_rate = agent_metrics
            .values()
            .map(|m| m.success_rate)
            .sum::<f64>()
            / agent_metrics.len().max(1) as f64;

        SwarmMetrics {
            swarm_id: self.swarm_id.clone(),
            timestamp: Utc::now().to_rfc3339(),
            agent_metrics,
            overall_success_rate,
        }
    }

    // Import metrics from peer swarms for evolutionary comparison
    pub fn import_peer_metrics(&self, peer_metrics: SwarmMetrics) {
        self.peer_metrics
            .insert(peer_metrics.swarm_id.clone(), peer_metrics);
    }

    // Evolutionary tuning: compare local agents against best peers and adopt improvements
    pub fn evolutionary_tune(&self, agent_name: &str) {
        let local_threshold = self.local_rsi.get_threshold(agent_name);

        // Find best peer with higher success rate for this agent
        let best_peer_config = self
            .peer_metrics
            .iter()
            .filter_map(|entry| {
                let peer_metrics = entry.value();
                peer_metrics
                    .agent_metrics
                    .get(agent_name)
                    .map(|m| (m.success_rate, m.threshold))
            })
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((peer_success, peer_threshold)) = best_peer_config {
            // If peer is significantly better (>75% success), adopt blend of configs
            if peer_success > 0.75 && peer_threshold != local_threshold {
                let blended = (local_threshold + peer_threshold) / 2.0;
                self.local_rsi.set_threshold(agent_name, blended);
                tracing::info!(
                    "{}: evolutionary tune adopted peer config ({}→{:.2})",
                    agent_name,
                    local_threshold,
                    blended
                );
            }
        }
    }

    // Clear old peer metrics to avoid memory bloat
    pub fn prune_old_metrics(&self, max_age_hours: i64) {
        self.peer_metrics.retain(|_, metrics| {
            if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&metrics.timestamp) {
                let age = Utc::now().signed_duration_since(timestamp);
                age.num_hours() < max_age_hours
            } else {
                false
            }
        });
    }
}
