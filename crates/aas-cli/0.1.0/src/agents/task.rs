use crate::swarm::agent::{Agent, AgentContext};
use crate::swarm::types::*;
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

pub struct TaskAgent;

#[async_trait]
impl Agent for TaskAgent {
    fn name(&self) -> &str {
        "task"
    }

    fn domain(&self) -> Domain {
        Domain::Task
    }

    fn description(&self) -> &str {
        "Manages personal workflows, monitors pending tasks, and tracks deadlines"
    }

    fn detection_interval(&self) -> &str {
        "10m"
    }

    async fn detect(&self, ctx: &AgentContext) -> Vec<Issue> {
        let mut issues = Vec::new();

        if let Some(ref task_config) = ctx.config.agents.task {
            if !task_config.enabled {
                return issues;
            }

            let recent_decisions = ctx.memory.get_recent_issues_for_agent("task", 50).await;
            let pending_count = recent_decisions
                .iter()
                .filter(|i| i.stage != Stage::Completed && i.stage != Stage::Failed)
                .count();

            if pending_count > 5 {
                issues.push(Issue {
                    id: Uuid::new_v4().to_string(),
                    domain: Domain::Task,
                    agent_name: self.name().to_string(),
                    title: format!("Task backlog: {} pending items", pending_count),
                    description: format!(
                        "There are {} pending tasks that need attention. Consider prioritizing or delegating.",
                        pending_count
                    ),
                    severity: Severity::Medium,
                    source: "internal".to_string(),
                    timestamp: Utc::now(),
                    metadata: [
                        ("pending_count".to_string(), pending_count.to_string()),
                    ].into(),
                    signature: format!("task_backlog:{}", pending_count),
                    stage: Stage::Detected,
                });
            }
        }

        issues
    }

    async fn analyze(&self, issue: &Issue, ctx: &AgentContext) -> Option<Analysis> {
        let context = format!(
            "Task issue:\nTitle: {}\nDescription: {}\nSeverity: {}",
            issue.title, issue.description, issue.severity
        );

        let prompt = "Analyze this task management issue and suggest a plan to address it.";

        let llm_response = ctx.llm.analyze(prompt, &context).await.unwrap_or_else(|_| String::from("Task analysis by RSI"));

        Some(Analysis {
            issue_id: issue.id.clone(),
            agent_name: self.name().to_string(),
            root_cause: "Pending task backlog identified".to_string(),
            impact: "May lead to missed deadlines or forgotten work".to_string(),
            suggested_approaches: vec![
                Approach {
                    name: "Task triage".to_string(),
                    description: "Review and prioritize pending tasks, auto-complete routine ones".to_string(),
                    expected_success_rate: 0.85,
                    execution_time_estimate: "5-10 minutes".to_string(),
                    risks: vec!["May mis-prioritize important tasks".to_string()],
                },
            ],
            confidence: 0.7,
            reasoning: llm_response,
            timestamp: Utc::now(),
        })
    }

    async fn plan(&self, analysis: &Analysis, _ctx: &AgentContext) -> Option<Vec<Action>> {
        let action = Action {
            id: Uuid::new_v4().to_string(),
            issue_id: analysis.issue_id.clone(),
            agent_name: self.name().to_string(),
            approach_name: "task_triage".to_string(),
            description: format!("Triage tasks: {}", analysis.root_cause),
            commands: vec![],
            rollback_commands: vec![],
            files_to_modify: vec![],
            stage: Stage::Planned,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            confidence: analysis.confidence,
        };

        Some(vec![action])
    }

    async fn execute(&self, action: &Action, ctx: &AgentContext) -> ActionResult {
        ctx.execution.execute_staged(action, ctx).await
    }

    async fn verify(&self, _action: &Action, result: &ActionResult, _ctx: &AgentContext) -> bool {
        result.success && result.verification_passed
    }

    async fn learn(&self, issue: &Issue, action: &Action, result: &ActionResult, ctx: &AgentContext) {
        if result.success {
            ctx.memory
                .store_decision(&Decision {
                    id: issue.id.clone(),
                    issue: issue.clone(),
                    analysis: None,
                    action: Some(action.clone()),
                    result: Some(result.clone()),
                    status: DecisionStatus::Completed,
                    created_at: issue.timestamp,
                    updated_at: Utc::now(),
                })
                .await;

            ctx.pattern_engine
                .find_or_create_pattern(issue, &action.description, 0.7, result.duration_ms)
                .await;
        }
    }
}
