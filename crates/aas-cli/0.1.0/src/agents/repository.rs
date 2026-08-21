use crate::integrations::github;
use crate::swarm::agent::{Agent, AgentContext};
use crate::swarm::types::*;
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

pub struct RepositoryAgent;

#[async_trait]
impl Agent for RepositoryAgent {
    fn name(&self) -> &str {
        "repository"
    }

    fn domain(&self) -> Domain {
        Domain::Repository
    }

    fn description(&self) -> &str {
        "Monitors git repositories for failing tests, code quality issues, and dependency vulnerabilities"
    }

    fn detection_interval(&self) -> &str {
        "5m"
    }

    async fn detect(&self, ctx: &AgentContext) -> Vec<Issue> {
        let mut issues = Vec::new();

        if let Some(ref repo_config) = ctx.config.agents.repository {
            if !repo_config.enabled {
                return issues;
            }

            for repo_path in &repo_config.local_repos {
                let path = std::path::Path::new(repo_path);
                if !path.exists() {
                    continue;
                }

                if let Ok((_modified, _staged, _untracked)) =
                    crate::integrations::git::GitOps::status_summary(path).await
                {
                    if _modified > 50 {
                        issues.push(Issue {
                            id: Uuid::new_v4().to_string(),
                            domain: Domain::Repository,
                            agent_name: self.name().to_string(),
                            title: "Large diff detected".to_string(),
                            description: format!(
                                "Repository {} has {} files changed, which may indicate need for refactoring",
                                repo_path, _modified
                            ),
                            severity: Severity::Medium,
                            source: repo_path.to_string(),
                            timestamp: Utc::now(),
                            metadata: [("repo".to_string(), repo_path.to_string())].into(),
                            signature: format!("large_diff:{}", repo_path),
                            stage: Stage::Detected,
                        });
                    }

                    if _modified > 20 {
                        issues.push(Issue {
                            id: Uuid::new_v4().to_string(),
                            domain: Domain::Repository,
                            agent_name: self.name().to_string(),
                            title: "Uncommitted changes accumulating".to_string(),
                            description: format!(
                                "{} files modified/untracked in {}",
                                _modified, repo_path
                            ),
                            severity: Severity::Low,
                            source: repo_path.to_string(),
                            timestamp: Utc::now(),
                            metadata: [
                                ("repo".to_string(), repo_path.to_string()),
                                ("modified".to_string(), _modified.to_string()),
                            ].into(),
                            signature: format!("uncommitted:{}", repo_path),
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
            "Domain: {}\nIssue: {}\nDescription: {}\nSource: {}",
            issue.domain, issue.title, issue.description, issue.source
        );

        let prompt = "Analyze this repository issue and provide root cause analysis, impact assessment, and suggested approaches to fix it.";

        // Route deep reasoning to best provider
        let provider = ctx.router.route(crate::llm::router::TaskType::DeepReasoning);
        let llm_response = provider.analyze(prompt, &context).await.unwrap_or_else(|_| String::from("Repository analysis by RSI"));

        Some(Analysis {
            issue_id: issue.id.clone(),
            agent_name: self.name().to_string(),
            root_cause: format!("Analysis based on: {}", issue.description),
            impact: "Impact determined by severity and scope".to_string(),
            suggested_approaches: vec![
                Approach {
                    name: "Automated fix".to_string(),
                    description: format!("Apply automated fix for: {}", issue.title),
                    expected_success_rate: 0.75,
                    execution_time_estimate: "5-15 minutes".to_string(),
                    risks: vec!["May need manual review".to_string()],
                },
            ],
            confidence: 0.7,
            reasoning: llm_response,
            timestamp: Utc::now(),
        })
    }

    async fn plan(&self, analysis: &Analysis, ctx: &AgentContext) -> Option<Vec<Action>> {
        let issues = ctx.memory.get_recent_issues_for_agent(self.name(), 20).await;
        let issue = issues.iter().find(|i| i.id == analysis.issue_id);
        let repo_path = issue.map(|i| i.source.as_str()).unwrap_or("").to_string();
        let auto_commit = ctx.config.agents.repository
            .as_ref().map(|r| r.auto_commit).unwrap_or(false);

        let (commands, rollback_commands) = if repo_path.is_empty() {
            (vec![], vec![])
        } else {
            let is_large_diff = issue
                .map(|i| i.signature.starts_with("large_diff:"))
                .unwrap_or(false);
            if is_large_diff {
                (
                    vec![
                        format!("git -C '{}' diff --stat", repo_path),
                        format!("git -C '{}' log --oneline -10", repo_path),
                    ],
                    vec![],
                )
            } else if auto_commit {
                (
                    vec![
                        format!("git -C '{}' add -A", repo_path),
                        format!("git -C '{}' commit -m 'aas: resolve uncommitted changes'", repo_path),
                    ],
                    vec![format!("git -C '{}' revert --no-edit HEAD", repo_path)],
                )
            } else {
                (
                    vec![format!("git -C '{}' status --short", repo_path)],
                    vec![],
                )
            }
        };

        Some(vec![Action {
            id: Uuid::new_v4().to_string(),
            issue_id: analysis.issue_id.clone(),
            agent_name: self.name().to_string(),
            approach_name: "automated_fix".to_string(),
            description: format!("Fix: {}", analysis.root_cause),
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
        let mut result = ctx.execution.execute_staged(action, ctx).await;

        if result.success {
            if let Some(repo_cfg) = &ctx.config.agents.repository {
                if let Some(gh) = &repo_cfg.github {
                    if let (Some(token), Some(org)) = (&gh.token, &gh.organization) {
                        let client = github::GitHubClient::new(token);
                        for repo in &gh.repositories {
                            let issue = github::GitHubIssue {
                                title: format!("AAS: {}", action.description),
                                body: format!(
                                    "**Automated action taken**\n\n{}\n\n**Commands:**\n```\n{}\n```\n\n**Output:**\n```\n{}\n```",
                                    action.description,
                                    action.commands.join("\n"),
                                    result.output
                                ),
                                labels: vec!["aas-automated".to_string()],
                            };
                            if let Ok(url) = client.create_issue(org, repo, &issue).await {
                                tracing::info!("GitHub issue created: {}", url);
                                result.output = format!("{}\nGitHub: {}", result.output, url);
                            }
                        }
                    }
                }
            }
        }

        result
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
                .find_or_create_pattern(
                    issue,
                    &action.description,
                    result.verification_passed as u32 as f64 * 0.9 + 0.1,
                    result.duration_ms,
                )
                .await;
        }
    }

    async fn react_to_event(&self, event: &AgentEvent, ctx: &AgentContext) {
        if let AgentEvent::IssueDetected { agent, issue, .. } = event {
            if agent == "health" && issue.severity == Severity::Critical {
                // Critical service down: check for recent repo commits
                ctx.event_bus
                    .emit(AgentEvent::HyperfocusRequest {
                        agent: self.name().to_string(),
                        reason: format!("Critical issue from health agent: {}", issue.title),
                        duration_secs: 120,
                        timestamp: Utc::now(),
                    })
                    .await;
            }
        }
    }
}
