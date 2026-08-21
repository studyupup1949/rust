use crate::swarm::agent::{Agent, AgentContext};
use crate::swarm::types::*;
use async_trait::async_trait;
use chrono::Utc;
use std::fs;
use uuid::Uuid;

pub struct LogsAgent;

#[async_trait]
impl Agent for LogsAgent {
    fn name(&self) -> &str {
        "logs"
    }

    fn domain(&self) -> Domain {
        Domain::Logs
    }

    fn description(&self) -> &str {
        "Monitors application and system logs for errors, anomalies, and security events"
    }

    fn detection_interval(&self) -> &str {
        "continuous"
    }

    async fn detect(&self, ctx: &AgentContext) -> Vec<Issue> {
        let mut issues = Vec::new();

        if let Some(ref logs_config) = ctx.config.agents.logs {
            if !logs_config.enabled {
                return issues;
            }

            for source in &logs_config.sources {
                let path = &source.path;
                if let Ok(contents) = fs::read_to_string(path) {
                    let error_count = contents.lines()
                        .filter(|l| {
                            let lower = l.to_lowercase();
                            lower.contains("error") || lower.contains("exception")
                                || lower.contains("fatal") || lower.contains("panic")
                                || lower.contains("stack trace")
                        })
                        .count();

                    let warning_count = contents.lines()
                        .filter(|l| {
                            let lower = l.to_lowercase();
                            lower.contains("warn") || lower.contains("timeout")
                        })
                        .count();

                    let threshold = logs_config.error_threshold.count as usize;
                    if error_count >= threshold {
                        let sample_errors: Vec<&str> = contents
                            .lines()
                            .filter(|l| l.to_lowercase().contains("error"))
                            .take(3)
                            .collect();

                        issues.push(Issue {
                            id: Uuid::new_v4().to_string(),
                            domain: Domain::Logs,
                            agent_name: self.name().to_string(),
                            title: format!("High error rate in {}", path),
                            description: format!(
                                "Found {} errors in last 100 lines (threshold: {}).\nSample errors:\n{}",
                                error_count,
                                threshold,
                                sample_errors.join("\n")
                            ),
                            severity: if error_count > threshold * 2 {
                                Severity::Critical
                            } else {
                                Severity::High
                            },
                            source: path.clone(),
                            timestamp: Utc::now(),
                            metadata: [
                                ("path".to_string(), path.clone()),
                                ("error_count".to_string(), error_count.to_string()),
                                ("warning_count".to_string(), warning_count.to_string()),
                            ].into(),
                            signature: format!("high_error_rate:{}:{}", path, error_count),
                            stage: Stage::Detected,
                        });
                    }

                    if warning_count > threshold * 2 {
                        issues.push(Issue {
                            id: Uuid::new_v4().to_string(),
                            domain: Domain::Logs,
                            agent_name: self.name().to_string(),
                            title: format!("Elevated warning count in {}", path),
                            description: format!(
                                "Found {} warnings in last 100 lines, indicating potential issues",
                                warning_count
                            ),
                            severity: Severity::Medium,
                            source: path.clone(),
                            timestamp: Utc::now(),
                            metadata: [
                                ("path".to_string(), path.clone()),
                                ("warning_count".to_string(), warning_count.to_string()),
                            ].into(),
                            signature: format!("high_warnings:{}:{}", path, warning_count),
                            stage: Stage::Detected,
                        });
                    }
                }
            }
        }

        issues
    }

    async fn analyze(&self, issue: &Issue, ctx: &AgentContext) -> Option<Analysis> {
        let context = format!(
            "Log issue detected:\nTitle: {}\nDescription: {}\nSource: {}\nSeverity: {}",
            issue.title, issue.description, issue.source, issue.severity
        );

        let prompt = "Analyze this log error pattern. Identify the root cause, potential impact, and recommend fixes.";

        let provider = ctx.router.route(crate::llm::router::TaskType::FastAnalysis);
        let llm_response = provider.analyze(prompt, &context).await.unwrap_or_else(|_| String::from("Log error analysis by RSI"));

        Some(Analysis {
            issue_id: issue.id.clone(),
            agent_name: self.name().to_string(),
            root_cause: "Error pattern detected in logs - see description for details".to_string(),
            impact: "May indicate system instability or bugs".to_string(),
            suggested_approaches: vec![
                Approach {
                    name: "Diagnose and fix".to_string(),
                    description: "Investigate error pattern and apply targeted fix".to_string(),
                    expected_success_rate: 0.7,
                    execution_time_estimate: "10-30 minutes".to_string(),
                    risks: vec!["Fix may not address root cause".to_string()],
                },
            ],
            confidence: 0.65,
            reasoning: llm_response,
            timestamp: Utc::now(),
        })
    }

    async fn plan(&self, analysis: &Analysis, _ctx: &AgentContext) -> Option<Vec<Action>> {
        let action = Action {
            id: Uuid::new_v4().to_string(),
            issue_id: analysis.issue_id.clone(),
            agent_name: self.name().to_string(),
            approach_name: "diagnose_and_fix".to_string(),
            description: format!("Investigate and fix: {}", analysis.root_cause),
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
