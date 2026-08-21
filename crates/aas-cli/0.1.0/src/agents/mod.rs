pub mod health;
pub mod logs;
pub mod metrics;
pub mod repository;
pub mod task;
pub mod trace;

use crate::config::settings::Config;
use crate::swarm::agent::Agent;

pub fn create_agents(config: &Config) -> Vec<Box<dyn Agent>> {
    let mut agents: Vec<Box<dyn Agent>> = Vec::new();

    if let Some(ref c) = config.agents.repository {
        if c.enabled {
            agents.push(Box::new(repository::RepositoryAgent));
        }
    }
    if let Some(ref c) = config.agents.logs {
        if c.enabled {
            agents.push(Box::new(logs::LogsAgent));
        }
    }
    if let Some(ref c) = config.agents.metrics {
        if c.enabled {
            agents.push(Box::new(metrics::MetricsAgent));
        }
    }
    if let Some(ref c) = config.agents.health {
        if c.enabled {
            agents.push(Box::new(health::HealthAgent::new()));
        }
    }
    if let Some(ref c) = config.agents.task {
        if c.enabled {
            agents.push(Box::new(task::TaskAgent));
        }
    }
    if let Some(ref c) = config.agents.trace {
        if c.enabled {
            agents.push(Box::new(trace::TraceAgent));
        }
    }

    agents
}
