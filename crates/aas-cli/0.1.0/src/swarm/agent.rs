use crate::config::settings::Config;
use crate::execution::staged::ExecutionEngine;
use crate::llm::traits::LLMProvider;
use crate::memory::patterns::PatternEngine;
use crate::memory::predictions::PredictionEngine;
use crate::memory::store::MemoryStore;
use crate::swarm::event_bus::EventBus;
use crate::swarm::types::*;
use async_trait::async_trait;
use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use dashmap::DashMap;

pub struct AgentContext {
    pub config: Arc<Config>,
    pub event_bus: Arc<EventBus>,
    pub memory: Arc<MemoryStore>,
    pub llm: Arc<dyn LLMProvider>,
    pub execution: Arc<ExecutionEngine>,
    pub pattern_engine: Arc<PatternEngine>,
    pub prediction_engine: Arc<PredictionEngine>,
    pub rsi_engine: Option<Arc<crate::rsi::RSIEngine>>,
    pub router: Arc<crate::llm::router::LLMRouter>,
    pub learned_solutions: Arc<DashMap<String, Action>>,  // issue_sig → cached action
}

impl AgentContext {
    pub fn new(
        config: Arc<Config>,
        event_bus: Arc<EventBus>,
        memory: Arc<MemoryStore>,
        llm: Arc<dyn LLMProvider>,
        execution: Arc<ExecutionEngine>,
        pattern_engine: Arc<PatternEngine>,
        prediction_engine: Arc<PredictionEngine>,
        rsi_engine: Option<Arc<crate::rsi::RSIEngine>>,
        router: Arc<crate::llm::router::LLMRouter>,
    ) -> Self {
        AgentContext {
            config,
            event_bus,
            memory,
            llm,
            execution,
            pattern_engine,
            prediction_engine,
            rsi_engine,
            router,
            learned_solutions: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &str;
    fn domain(&self) -> Domain;
    fn description(&self) -> &str;
    fn detection_interval(&self) -> &str;

    async fn detect(&self, ctx: &AgentContext) -> Vec<Issue>;
    async fn analyze(&self, issue: &Issue, ctx: &AgentContext) -> Option<Analysis>;
    async fn plan(&self, analysis: &Analysis, ctx: &AgentContext) -> Option<Vec<Action>>;
    async fn execute(&self, action: &Action, ctx: &AgentContext) -> ActionResult;
    async fn verify(&self, action: &Action, result: &ActionResult, ctx: &AgentContext) -> bool;
    async fn learn(&self, issue: &Issue, action: &Action, result: &ActionResult, ctx: &AgentContext);

    async fn react_to_event(&self, _event: &AgentEvent, _ctx: &AgentContext) {
        // Default: no-op. Agents can override to react to events from other agents.
    }

    async fn run_cycle(&self, ctx: &AgentContext) {
        let cycle_start = std::time::Instant::now();
        let issues_found;
        let mut actions_attempted = 0u32;
        let mut actions_succeeded = 0u32;

        ctx.event_bus
            .emit(AgentEvent::AgentStarted {
                agent: self.name().to_string(),
                domain: self.domain(),
                timestamp: Utc::now(),
            })
            .await;

        // Emit any active predictions at the start of cycle
        let predictions = ctx.prediction_engine.generate_predictions(self.name()).await;
        for pred in predictions {
            ctx.event_bus
                .emit(AgentEvent::PredictionMade {
                    agent: self.name().to_string(),
                    prediction: pred,
                    timestamp: Utc::now(),
                })
                .await;
        }

        let issues = self.detect(ctx).await;
        issues_found = issues.len() as u32;

        for issue in issues {
            ctx.event_bus
                .emit(AgentEvent::IssueDetected {
                    agent: self.name().to_string(),
                    issue: issue.clone(),
                    timestamp: Utc::now(),
                })
                .await;

            ctx.memory.store_issue(&issue).await;

            // MVP Learning: Check if we've solved this exact issue before
            let issue_sig = crate::memory::issue_signature(&issue);
            if let Some(cached_action) = ctx.learned_solutions.get(&issue_sig) {
                let cached_action = cached_action.value().clone();
                tracing::info!(
                    "{}: reusing learned solution for {}",
                    self.name(),
                    issue_sig
                );
                actions_attempted += 1;
                let result = self.execute(&cached_action, ctx).await;
                if result.success {
                    actions_succeeded += 1;
                    ctx.event_bus
                        .emit(AgentEvent::ActionCompleted {
                            agent: self.name().to_string(),
                            action_id: cached_action.id.clone(),
                            result: result.clone(),
                            timestamp: Utc::now(),
                        })
                        .await;
                }
                self.learn(&issue, &cached_action, &result, ctx).await;
                continue; // Skip rest of pipeline
            }

            // Check for known patterns — high-confidence cache hit skips LLM
            let known_pattern = ctx.pattern_engine.match_issue_to_pattern(&issue).await;
            if let Some(pattern) = known_pattern {
                let threshold = if let Some(rsi) = ctx.rsi_engine.as_ref() {
                    rsi.get_threshold(self.name())
                } else {
                    0.7
                };

                if pattern.confidence >= threshold {
                    // Reuse cached solution
                    tracing::info!(
                        "{}: using cached pattern (confidence {:.2})",
                        self.name(),
                        pattern.confidence
                    );
                    let cached_action = Action {
                        id: Uuid::new_v4().to_string(),
                        issue_id: issue.id.clone(),
                        agent_name: self.name().to_string(),
                        approach_name: "cached".to_string(),
                        description: pattern.solution_description.clone(),
                        commands: vec![pattern.solution_description.clone()],
                        rollback_commands: vec![],
                        files_to_modify: vec![],
                        stage: Stage::Planned,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        confidence: pattern.confidence,
                    };

                    actions_attempted += 1;
                    let result = self.execute(&cached_action, ctx).await;
                    if result.success {
                        actions_succeeded += 1;
                        ctx.event_bus
                            .emit(AgentEvent::ActionCompleted {
                                agent: self.name().to_string(),
                                action_id: cached_action.id.clone(),
                                result: result.clone(),
                                timestamp: Utc::now(),
                            })
                            .await;
                    } else {
                        ctx.event_bus
                            .emit(AgentEvent::ActionFailed {
                                agent: self.name().to_string(),
                                action_id: cached_action.id.clone(),
                                stage: result.stage.clone(),
                                error: result.error.clone().unwrap_or_default(),
                                timestamp: Utc::now(),
                            })
                            .await;
                    }
                    self.learn(&issue, &cached_action, &result, ctx).await;
                    continue; // Skip LLM pipeline
                }
            }

            // Normal LLM pipeline
            let analysis = self.analyze(&issue, ctx).await;
            if let Some(analysis) = analysis {
                ctx.memory.store_analysis(&analysis).await;

                ctx.event_bus
                    .emit(AgentEvent::IssueAnalyzed {
                        agent: self.name().to_string(),
                        issue_id: issue.id.clone(),
                        analysis: analysis.clone(),
                        timestamp: Utc::now(),
                    })
                    .await;

                if let Some(actions) = self.plan(&analysis, ctx).await {
                    for action in actions {
                        actions_attempted += 1;

                        ctx.event_bus
                            .emit(AgentEvent::ActionPlanned {
                                agent: self.name().to_string(),
                                action: action.clone(),
                                timestamp: Utc::now(),
                            })
                            .await;

                        ctx.memory.store_action(&action).await;

                        let result = self.execute(&action, ctx).await;
                        if result.success {
                            actions_succeeded += 1;
                            let verified = self.verify(&action, &result, ctx).await;
                            if verified {
                                // MVP Learning: Cache this successful solution
                                let issue_sig = crate::memory::issue_signature(&issue);
                                ctx.learned_solutions.insert(issue_sig.clone(), action.clone());
                                tracing::info!(
                                    "{}: learned solution for {}",
                                    self.name(),
                                    issue_sig
                                );

                                ctx.event_bus
                                    .emit(AgentEvent::ActionCompleted {
                                        agent: self.name().to_string(),
                                        action_id: action.id.clone(),
                                        result: result.clone(),
                                        timestamp: Utc::now(),
                                    })
                                    .await;
                            }
                        } else {
                            ctx.event_bus
                                .emit(AgentEvent::ActionFailed {
                                    agent: self.name().to_string(),
                                    action_id: action.id.clone(),
                                    stage: result.stage.clone(),
                                    error: result.error.clone().unwrap_or_default(),
                                    timestamp: Utc::now(),
                                })
                                .await;
                        }

                        self.learn(&issue, &action, &result, ctx).await;
                    }
                }
            }
        }

        // Record cycle performance for RSI
        let cycle_duration_ms = cycle_start.elapsed().as_millis() as u64;
        let threshold = if let Some(rsi) = ctx.rsi_engine.as_ref() {
            rsi.get_threshold(self.name())
        } else {
            0.7
        };

        let perf = CyclePerformance {
            id: Uuid::new_v4().to_string(),
            agent_name: self.name().to_string(),
            cycle_duration_ms,
            issues_found,
            actions_attempted,
            actions_succeeded,
            confidence_threshold: threshold,
            timestamp: Utc::now(),
        };

        ctx.memory.record_cycle(&perf).await;

        // RSI: evaluate and adjust thresholds/intervals based on performance
        if let Some(rsi) = ctx.rsi_engine.as_ref() {
            rsi.evaluate_and_adjust(self.name()).await;
        }
    }
}
