use crate::swarm::agent::{Agent, AgentContext};
use crate::swarm::types::*;
use async_trait::async_trait;
use chrono::Utc;
use sysinfo::System;
use uuid::Uuid;

pub struct MetricsAgent;

#[async_trait]
impl Agent for MetricsAgent {
    fn name(&self) -> &str {
        "metrics"
    }

    fn domain(&self) -> Domain {
        Domain::Metrics
    }

    fn description(&self) -> &str {
        "Monitors system metrics including CPU, memory, disk, and latency"
    }

    fn detection_interval(&self) -> &str {
        "1m"
    }

    async fn detect(&self, ctx: &AgentContext) -> Vec<Issue> {
        let mut issues = Vec::new();

        if let Some(ref metrics_config) = ctx.config.agents.metrics {
            if !metrics_config.enabled {
                return issues;
            }

            let mut sys = System::new_all();
            sys.refresh_all();

            let cpu_percent = sys.global_cpu_info().cpu_usage() as f64;
            let memory_percent = sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0;

            let disk_percent = match tokio::process::Command::new("df")
                .arg("-k")
                .arg("/")
                .output()
                .await
            {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let line = stdout.lines().nth(1).unwrap_or("");
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 5 {
                        let used: f64 = fields[2].parse().unwrap_or(0.0);
                        let total: f64 = fields[1].parse().unwrap_or(1.0);
                        if total > 0.0 { (used / total) * 100.0 } else { 0.0 }
                    } else {
                        0.0
                    }
                }
                Err(_) => 0.0,
            };

            let thresholds = &metrics_config.thresholds;

            if cpu_percent > thresholds.cpu_percent {
                issues.push(Issue {
                    id: Uuid::new_v4().to_string(),
                    domain: Domain::Metrics,
                    agent_name: self.name().to_string(),
                    title: format!("High CPU usage: {:.1}% (threshold: {}%)", cpu_percent, thresholds.cpu_percent),
                    description: format!(
                        "CPU usage is at {:.1}%, exceeding the threshold of {}%. This may indicate a runaway process or insufficient capacity.",
                        cpu_percent, thresholds.cpu_percent
                    ),
                    severity: if cpu_percent > 95.0 { Severity::Critical } else { Severity::High },
                    source: "system".to_string(),
                    timestamp: Utc::now(),
                    metadata: [
                        ("metric".to_string(), "cpu".to_string()),
                        ("value".to_string(), format!("{:.1}", cpu_percent)),
                        ("threshold".to_string(), thresholds.cpu_percent.to_string()),
                    ].into(),
                    signature: format!("high_cpu:{:.0}", cpu_percent),
                    stage: Stage::Detected,
                });
            }

            if memory_percent > thresholds.memory_percent {
                issues.push(Issue {
                    id: Uuid::new_v4().to_string(),
                    domain: Domain::Metrics,
                    agent_name: self.name().to_string(),
                    title: format!("High memory usage: {:.1}% (threshold: {}%)", memory_percent, thresholds.memory_percent),
                    description: format!(
                        "Memory usage is at {:.1}%, exceeding the threshold of {}%. This may lead to OOM conditions.",
                        memory_percent, thresholds.memory_percent
                    ),
                    severity: if memory_percent > 95.0 { Severity::Critical } else { Severity::High },
                    source: "system".to_string(),
                    timestamp: Utc::now(),
                    metadata: [
                        ("metric".to_string(), "memory".to_string()),
                        ("value".to_string(), format!("{:.1}", memory_percent)),
                        ("threshold".to_string(), thresholds.memory_percent.to_string()),
                    ].into(),
                    signature: format!("high_memory:{:.0}", memory_percent),
                    stage: Stage::Detected,
                });
            }

            if disk_percent > thresholds.disk_percent {
                issues.push(Issue {
                    id: Uuid::new_v4().to_string(),
                    domain: Domain::Metrics,
                    agent_name: self.name().to_string(),
                    title: format!("High disk usage: {:.1}% (threshold: {}%)", disk_percent, thresholds.disk_percent),
                    description: format!(
                        "Disk usage is at {:.1}%, exceeding the threshold of {}%. Service may fail if disk fills up.",
                        disk_percent, thresholds.disk_percent
                    ),
                    severity: if disk_percent > 97.0 { Severity::Critical } else { Severity::High },
                    source: "system".to_string(),
                    timestamp: Utc::now(),
                    metadata: [
                        ("metric".to_string(), "disk".to_string()),
                        ("value".to_string(), format!("{:.1}", disk_percent)),
                        ("threshold".to_string(), thresholds.disk_percent.to_string()),
                    ].into(),
                    signature: format!("high_disk:{:.0}", disk_percent),
                    stage: Stage::Detected,
                });
            }
        }

        issues
    }

    async fn analyze(&self, issue: &Issue, ctx: &AgentContext) -> Option<Analysis> {
        let context = format!(
            "Metric issue:\nTitle: {}\nDescription: {}\nSeverity: {}",
            issue.title, issue.description, issue.severity
        );

        let prompt = "Analyze this metrics threshold breach. Determine root cause, impact, and recommended optimizations.";

        let llm_response = ctx.llm.analyze(prompt, &context).await.unwrap_or_else(|_| String::from("Metrics analysis by RSI"));

        Some(Analysis {
            issue_id: issue.id.clone(),
            agent_name: self.name().to_string(),
            root_cause: format!("Resource threshold exceeded: {}", issue.title),
            impact: "System performance may degrade, risk of outage if unaddressed".to_string(),
            suggested_approaches: vec![
                Approach {
                    name: "Resource optimization".to_string(),
                    description: "Identify and terminate resource-heavy processes or optimize configuration".to_string(),
                    expected_success_rate: 0.8,
                    execution_time_estimate: "5-15 minutes".to_string(),
                    risks: vec!["May impact running services".to_string()],
                },
            ],
            confidence: 0.75,
            reasoning: llm_response,
            timestamp: Utc::now(),
        })
    }

    async fn plan(&self, analysis: &Analysis, _ctx: &AgentContext) -> Option<Vec<Action>> {
        let action = Action {
            id: Uuid::new_v4().to_string(),
            issue_id: analysis.issue_id.clone(),
            agent_name: self.name().to_string(),
            approach_name: "resource_optimization".to_string(),
            description: format!("Optimize: {}", analysis.root_cause),
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
                .find_or_create_pattern(issue, &action.description, 0.75, result.duration_ms)
                .await;
        }
    }
}
