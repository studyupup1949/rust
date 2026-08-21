use crate::swarm::agent::{Agent, AgentContext};
use crate::swarm::types::*;
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use uuid::Uuid;

pub struct HealthAgent {
    client: Client,
}

impl HealthAgent {
    pub fn new() -> Self {
        HealthAgent {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }
}

#[async_trait]
impl Agent for HealthAgent {
    fn name(&self) -> &str {
        "health"
    }

    fn domain(&self) -> Domain {
        Domain::Health
    }

    fn description(&self) -> &str {
        "Monitors service health endpoints for availability and response time"
    }

    fn detection_interval(&self) -> &str {
        "30s"
    }

    async fn detect(&self, ctx: &AgentContext) -> Vec<Issue> {
        let mut issues = Vec::new();

        if let Some(ref health_config) = ctx.config.agents.health {
            if !health_config.enabled {
                return issues;
            }

            for endpoint in &health_config.endpoints {
                let start = std::time::Instant::now();
                match self.client.get(endpoint).send().await {
                    Ok(resp) => {
                        let latency = start.elapsed().as_millis() as u64;
                        let status = resp.status();

                        if !status.is_success() {
                            issues.push(Issue {
                                id: Uuid::new_v4().to_string(),
                                domain: Domain::Health,
                                agent_name: self.name().to_string(),
                                title: format!("Health check failed: {}", endpoint),
                                description: format!(
                                    "Endpoint {} returned HTTP {}",
                                    endpoint, status.as_u16()
                                ),
                                severity: Severity::Critical,
                                source: endpoint.clone(),
                                timestamp: Utc::now(),
                                metadata: [
                                    ("endpoint".to_string(), endpoint.clone()),
                                    ("status".to_string(), status.as_u16().to_string()),
                                    ("latency_ms".to_string(), latency.to_string()),
                                ].into(),
                                signature: format!("health_fail:{}", endpoint),
                                stage: Stage::Detected,
                            });
                        } else if latency > 1000 {
                            issues.push(Issue {
                                id: Uuid::new_v4().to_string(),
                                domain: Domain::Health,
                                agent_name: self.name().to_string(),
                                title: format!("Slow health check: {} ({}ms)", endpoint, latency),
                                description: format!(
                                    "Endpoint {} responded in {}ms, which exceeds the warning threshold",
                                    endpoint, latency
                                ),
                                severity: Severity::Medium,
                                source: endpoint.clone(),
                                timestamp: Utc::now(),
                                metadata: [
                                    ("endpoint".to_string(), endpoint.clone()),
                                    ("latency_ms".to_string(), latency.to_string()),
                                ].into(),
                                signature: format!("health_slow:{}", endpoint),
                                stage: Stage::Detected,
                            });
                        }
                    }
                    Err(e) => {
                        issues.push(Issue {
                            id: Uuid::new_v4().to_string(),
                            domain: Domain::Health,
                            agent_name: self.name().to_string(),
                            title: format!("Health check unreachable: {}", endpoint),
                            description: format!(
                                "Cannot connect to endpoint {}: {}",
                                endpoint, e
                            ),
                            severity: Severity::Critical,
                            source: endpoint.clone(),
                            timestamp: Utc::now(),
                            metadata: [
                                ("endpoint".to_string(), endpoint.clone()),
                                ("error".to_string(), e.to_string()),
                            ].into(),
                            signature: format!("health_down:{}", endpoint),
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
            "Health check issue:\nTitle: {}\nDescription: {}\nEndpoint: {}\nSeverity: {}",
            issue.title, issue.description, issue.source, issue.severity
        );

        let prompt = "Analyze this health check failure. Determine if this is a transient issue, service crash, or network problem.";

        // Use fast analysis provider for quick diagnosis
        let provider = ctx.router.route(crate::llm::router::TaskType::FastAnalysis);
        let llm_response = provider.analyze(prompt, &context).await.unwrap_or_else(|_| String::from("Health check analysis by RSI"));

        Some(Analysis {
            issue_id: issue.id.clone(),
            agent_name: self.name().to_string(),
            root_cause: format!("Service health check failed: {}", issue.title),
            impact: "Service may be unavailable to users".to_string(),
            suggested_approaches: vec![
                Approach {
                    name: "Service restart".to_string(),
                    description: "Attempt to restart the failing service".to_string(),
                    expected_success_rate: 0.6,
                    execution_time_estimate: "1-5 minutes".to_string(),
                    risks: vec!["May cause brief downtime".to_string()],
                },
                Approach {
                    name: "Diagnose and fix".to_string(),
                    description: "Investigate service logs and apply targeted fix".to_string(),
                    expected_success_rate: 0.7,
                    execution_time_estimate: "15-60 minutes".to_string(),
                    risks: vec!["Requires deeper investigation".to_string()],
                },
            ],
            confidence: 0.6,
            reasoning: llm_response,
            timestamp: Utc::now(),
        })
    }

    async fn plan(&self, analysis: &Analysis, ctx: &AgentContext) -> Option<Vec<Action>> {
        let issues = ctx.memory.get_recent_issues_for_agent(self.name(), 20).await;
        let issue = issues.iter().find(|i| i.id == analysis.issue_id);
        let endpoint = issue.map(|i| i.source.as_str()).unwrap_or("").to_string();
        let auto_restart = ctx.config.agents.health
            .as_ref().map(|h| h.auto_restart).unwrap_or(false);

        let (commands, rollback_commands, approach_name) = if auto_restart && !endpoint.is_empty() {
            let host = endpoint
                .trim_start_matches("https://")
                .trim_start_matches("http://")
                .split(':').next().unwrap_or("")
                .split('/').next().unwrap_or("")
                .to_string();
            (
                vec![
                    format!("docker restart '{}' 2>/dev/null || echo 'not a docker container'", host),
                    format!("sleep 2"),
                    format!("curl -sf '{}' && echo 'recovered' || echo 'still down'", endpoint),
                ],
                vec![format!("docker stop '{}' 2>/dev/null || true", host)],
                "service_restart",
            )
        } else {
            (
                vec![
                    format!("curl -sf -o /dev/null -w '%{{http_code}}' '{}' || echo 'unreachable'", endpoint),
                ],
                vec![],
                "diagnose",
            )
        };

        Some(vec![Action {
            id: Uuid::new_v4().to_string(),
            issue_id: analysis.issue_id.clone(),
            agent_name: self.name().to_string(),
            approach_name: approach_name.to_string(),
            description: format!("Restart service: {}", analysis.root_cause),
            commands,
            rollback_commands,
            files_to_modify: vec![],
            stage: Stage::Planned,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            confidence: analysis.confidence,
        }])
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
                .find_or_create_pattern(issue, &action.description, 0.6, result.duration_ms)
                .await;
        }
    }

    async fn react_to_event(&self, event: &AgentEvent, ctx: &AgentContext) {
        // Logs agent can react to health issues too
        if let AgentEvent::IssueDetected { agent, issue, .. } = event {
            if agent == "repository" && issue.severity == Severity::Critical {
                // Repo issue found: check service health
                ctx.event_bus
                    .emit(AgentEvent::HyperfocusRequest {
                        agent: self.name().to_string(),
                        reason: format!("Critical repo issue detected: {}", issue.title),
                        duration_secs: 60,
                        timestamp: Utc::now(),
                    })
                    .await;
            }
        }
    }
}
