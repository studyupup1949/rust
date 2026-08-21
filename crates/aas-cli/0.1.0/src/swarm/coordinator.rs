use crate::agents::create_agents;
use crate::config::settings::Config;
use crate::execution::staged::ExecutionEngine;
use crate::llm::traits::LLMProvider;
use crate::memory::patterns::PatternEngine;
use crate::memory::predictions::PredictionEngine;
use crate::memory::store::MemoryStore;
use crate::swarm::agent::{Agent, AgentContext};
use crate::swarm::event_bus::EventBus;
use crate::swarm::types::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{self, Duration};
use tracing::{error, info};

pub struct Coordinator {
    agents: Vec<Arc<Box<dyn Agent>>>,
    context: Arc<AgentContext>,
    running: Arc<AtomicBool>,
    event_bus: Arc<EventBus>,
    shutdown_notify: Arc<Notify>,
    in_flight_actions: Arc<std::sync::atomic::AtomicUsize>,
}

impl Coordinator {
    pub fn new(
        config: Arc<Config>,
        memory: Arc<MemoryStore>,
        llm: Arc<dyn LLMProvider>,
        router: Arc<crate::llm::router::LLMRouter>,
    ) -> Self {
        let event_bus = EventBus::new_persisted();
        let execution = Arc::new(ExecutionEngine::new(&config.execution));
        let pattern_engine = Arc::new(PatternEngine::new(memory.clone()));
        let prediction_engine = Arc::new(PredictionEngine::new(memory.clone()));
        let rsi_engine = Arc::new(crate::rsi::RSIEngine::new(memory.clone()));

        let ctx = Arc::new(AgentContext::new(
            config.clone(),
            event_bus.clone(),
            memory.clone(),
            llm.clone(),
            execution.clone(),
            pattern_engine.clone(),
            prediction_engine.clone(),
            Some(rsi_engine),
            router,
        ));

        let agents: Vec<Arc<Box<dyn Agent>>> = create_agents(&config)
            .into_iter()
            .map(|a| Arc::new(a))
            .collect();

        Coordinator {
            agents,
            context: ctx,
            running: Arc::new(AtomicBool::new(false)),
            event_bus: event_bus.clone(),
            shutdown_notify: Arc::new(Notify::new()),
            in_flight_actions: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    pub fn context(&self) -> Arc<AgentContext> {
        self.context.clone()
    }

    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }

    pub async fn start(&self) {
        self.running.store(true, Ordering::SeqCst);
        info!("Starting agent swarm with {} agents", self.agents.len());

        let mut handles = Vec::new();

        for agent in &self.agents {
            let ctx = self.context.clone();
            let agent_clone = agent.clone();
            let interval = agent_clone.detection_interval().to_string();
            let running = self.running.clone();
            let event_bus = self.event_bus.clone();

            let handle = tokio::spawn(async move {
                info!("Agent '{}' started (interval: {})", agent_clone.name(), interval);
                let initial_duration = parse_interval(&interval).unwrap_or(Duration::from_secs(300));
                let mut timer = tokio::time::interval(initial_duration);
                let mut rx = event_bus.subscribe();

                while running.load(Ordering::SeqCst) {
                    tokio::select! {
                        // Normal cycle on timer
                        _ = timer.tick() => {
                            agent_clone.run_cycle(&ctx).await;
                            // Get next interval from RSI (may have been adjusted)
                            let next_interval = if let Some(rsi) = ctx.rsi_engine.as_ref() {
                                rsi.get_interval(agent_clone.name())
                            } else {
                                initial_duration
                            };
                            timer = tokio::time::interval(next_interval);
                        }
                        // Hyperfocus event override
                        Ok(AgentEvent::HyperfocusRequest { agent, duration_secs, .. }) = rx.recv() => {
                            if agent == agent_clone.name() {
                                info!("Agent '{}' entering hyperfocus mode for {} seconds", agent_clone.name(), duration_secs);
                                let hyperfocus_end = std::time::Instant::now() + Duration::from_secs(duration_secs);
                                while std::time::Instant::now() < hyperfocus_end && running.load(Ordering::SeqCst) {
                                    agent_clone.run_cycle(&ctx).await;
                                    time::sleep(Duration::from_secs(1)).await;
                                }
                                info!("Agent '{}' exiting hyperfocus mode", agent_clone.name());
                                let next_interval = if let Some(rsi) = ctx.rsi_engine.as_ref() {
                                    rsi.get_interval(agent_clone.name())
                                } else {
                                    initial_duration
                                };
                                timer = tokio::time::interval(next_interval);
                            }
                        }
                    }
                }
                info!("Agent '{}' stopped", agent_clone.name());
            });

            handles.push(handle);
        }

        for handle in handles {
            if let Err(e) = handle.await {
                error!("Agent task failed: {}", e);
            }
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("Stopping agent swarm...");
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn agent_names(&self) -> Vec<String> {
        self.agents.iter().map(|a| a.name().to_string()).collect()
    }

    pub fn increment_action(&self) {
        self.in_flight_actions.fetch_add(1, Ordering::SeqCst);
    }

    pub fn decrement_action(&self) {
        self.in_flight_actions.fetch_sub(1, Ordering::SeqCst);
        self.shutdown_notify.notify_one();
    }

    pub fn action_count(&self) -> usize {
        self.in_flight_actions.load(Ordering::SeqCst)
    }

    pub async fn drain_and_shutdown(&self, timeout_secs: u64) {
        info!("Starting graceful shutdown with {}s drain timeout", timeout_secs);
        self.stop();

        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            let count = self.action_count();
            if count == 0 {
                info!("All actions completed");
                break;
            }
            if start.elapsed() > timeout {
                error!("Shutdown timeout: {} actions still in flight", count);
                break;
            }
            info!("Draining: {} actions in flight, waiting...", count);
            tokio::select! {
                _ = self.shutdown_notify.notified() => {
                    continue;
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    continue;
                }
            }
        }
        info!("Graceful shutdown complete");
    }

    pub async fn get_statuses(&self) -> Vec<AgentStatus> {
        let mut statuses = Vec::new();
        for agent in &self.agents {
            let name = agent.name().to_string();
            let recent_issues = self
                .context
                .memory
                .get_recent_issues_for_agent(&name, 10)
                .await;
            let recent_actions = self
                .context
                .memory
                .get_recent_actions_for_agent(&name, 10)
                .await;

            let success_count = recent_actions
                .iter()
                .filter(|a| a.1.success)
                .count();
            let total = recent_actions.len();
            let success_rate = if total > 0 {
                success_count as f64 / total as f64 * 100.0
            } else {
                100.0
            };

            let rsi_threshold = self.context.rsi_engine.as_ref().map(|rsi| rsi.get_threshold(&name));
            let rsi_interval = self.context.rsi_engine.as_ref().map(|rsi| rsi.get_interval(&name).as_secs());

            statuses.push(AgentStatus {
                name: name.clone(),
                domain: agent.domain(),
                running: self.running.load(Ordering::SeqCst),
                healthy: recent_issues.iter().filter(|i| i.severity == Severity::Critical).count() < 3,
                uptime_seconds: 0,
                issues_detected: recent_issues.len() as u64,
                actions_taken: recent_actions.len() as u64,
                success_rate,
                last_check: recent_issues.first().map(|i| i.timestamp),
                last_decision: recent_actions.first().map(|a| a.1.timestamp),
                current_issue: recent_issues.first().filter(|i| i.stage != Stage::Completed).map(|i| i.title.clone()),
                memory_usage_mb: 0.0,
                rsi_confidence_threshold: rsi_threshold,
                rsi_interval_secs: rsi_interval,
            });
        }
        statuses
    }

    pub async fn trigger_agent(&self, agent_name: &str) -> Result<String, String> {
        for agent in &self.agents {
            if agent.name() == agent_name {
                info!("Manually triggering agent '{}'", agent_name);
                agent.run_cycle(&self.context).await;
                return Ok(format!("Agent '{}' cycle completed", agent_name));
            }
        }
        Err(format!("Agent '{}' not found", agent_name))
    }
}

fn parse_interval(s: &str) -> Option<Duration> {
    let s = s.trim();
    if let Some(val) = s.strip_suffix("s").and_then(|n| n.parse::<u64>().ok()) {
        Some(Duration::from_secs(val))
    } else if let Some(val) = s.strip_suffix("m").and_then(|n| n.parse::<u64>().ok()) {
        Some(Duration::from_secs(val * 60))
    } else if let Some(val) = s.strip_suffix("h").and_then(|n| n.parse::<u64>().ok()) {
        Some(Duration::from_secs(val * 3600))
    } else if let Some(val) = s.strip_suffix("ms").and_then(|n| n.parse::<u64>().ok()) {
        Some(Duration::from_millis(val))
    } else if s == "continuous" {
        Some(Duration::from_secs(10))
    } else {
        s.parse::<u64>().ok().map(Duration::from_secs)
    }
}
