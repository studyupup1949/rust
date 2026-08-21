use std::path::PathBuf;
use std::sync::Arc;

use agent_client_protocol::schema::v1::ToolKind;
use http::request::Parts;
use rmcp::{
    ServerHandler,
    handler::server::{
        common::{AsRequestContext, FromContextPart},
        router::tool::ToolRouter,
        tool::Extension,
        wrapper::Parameters,
    },
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;
use tracing::{error, info};

use crate::compositor::Compositor;
use crate::config::CronJobConfig;
use crate::cron::format_remaining;

const README: &str = include_str!("../README.md");

/// Extractor that returns an extension if it is present, without failing when
/// it is absent. This lets tools work both over HTTP (where `http::request::Parts`
/// is available) and over plain in-memory transports.
pub struct OptionalExtension<T>(pub Option<T>);

impl<C, T> FromContextPart<C> for OptionalExtension<T>
where
    C: AsRequestContext,
    T: Send + Sync + 'static + Clone,
{
    fn from_context_part(context: &mut C) -> Result<Self, rmcp::ErrorData> {
        Ok(Self(
            context.as_request_context().extensions.get::<T>().cloned(),
        ))
    }
}

/// Input for `create_session`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateSessionRequest {
    /// Unique name for the session.
    pub name: String,
    /// Working directory for the session.
    pub cwd: String,
    /// Initial charter / system prompt sent to the agent.
    pub charter: String,
    /// Allowed tool kinds. If empty, all tool kinds are allowed.
    #[serde(default)]
    pub allowed_tool_kinds: Vec<ToolKind>,
    /// MCP servers to expose to the agent session.
    #[serde(default)]
    pub mcp_servers: Vec<crate::config::McpServer>,
}

fn default_true() -> bool {
    true
}

/// Input for `send_message`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendMessageRequest {
    /// Target session name or session id.
    pub target: String,
    /// Message content to send.
    pub content: String,
    /// Whether to send the prompt result back to the caller as a new message
    /// once the target agent finishes its turn. Defaults to true.
    #[serde(default = "default_true")]
    pub need_result: bool,
}

/// Input for `recreate_session`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RecreateSessionRequest {
    /// Name of the existing session to recreate.
    pub name: String,
    /// Additional text appended to the resulting charter. Use this to extend
    /// the session's instructions without replacing the original charter.
    #[serde(default)]
    pub extra_charter: Option<String>,
}
/// Input for `delete_session`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteSessionRequest {
    /// Name or session id of the session to delete.
    pub name: String,
}

/// Input for `list_cron_jobs`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListCronJobsRequest {
    /// Session name or session id. If omitted, uses the caller's session
    /// identified from the `agent` query parameter.
    #[serde(default)]
    pub session: Option<String>,
}

/// Input for `add_cron_job`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddCronJobRequest {
    /// Session name or session id. If omitted, uses the caller's session
    /// identified from the `agent` query parameter.
    #[serde(default)]
    pub session: Option<String>,
    /// Stable identifier for the cron job.
    pub name: String,
    /// 5-field cron expression (minute hour day-of-month month day-of-week).
    /// Either `schedule` or `run_at` must be provided.
    #[serde(default)]
    pub schedule: Option<String>,
    /// Prompt text sent to the session when the job fires.
    pub prompt: String,
    /// IANA timezone name. Defaults to UTC.
    #[serde(default = "default_timezone")]
    pub timezone: String,
    /// What to do if the server was down and missed scheduled runs.
    #[serde(default)]
    pub misfire_policy: crate::config::MisfirePolicy,
    /// One-shot ISO 8601 timestamp. When set, the prompt fires once at this
    /// exact time and the job stops afterwards. `schedule` is ignored.
    #[serde(default)]
    pub run_at: Option<String>,
}

fn default_timezone() -> String {
    "UTC".to_string()
}

/// Input for `remove_cron_job`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RemoveCronJobRequest {
    /// Session name or session id. If omitted, uses the caller's session
    /// identified from the `agent` query parameter.
    #[serde(default)]
    pub session: Option<String>,
    /// Name of the cron job to remove.
    pub job_name: String,
}

/// MCP server exposing compose tools.
#[derive(Debug, Clone)]
pub struct ComposeMcpServer {
    tool_router: ToolRouter<Self>,
    compositor: Arc<Compositor>,
}

impl ComposeMcpServer {
    /// Create a new compose MCP server backed by the given compositor.
    #[must_use]
    pub fn new(compositor: Arc<Compositor>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            compositor,
        }
    }
}

#[tool_router]
impl ComposeMcpServer {
    /// List all active sessions managed by the compositor.
    #[tool(description = "List all active acompose sessions")]
    async fn list_sessions(
        &self,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, String> {
        let current_agent = parts.uri.query().and_then(|q| {
            q.split('&').find_map(|p| {
                let (k, v) = p.split_once('=')?;
                if k == "agent" {
                    Some(v.to_string())
                } else {
                    None
                }
            })
        });
        info!(?current_agent, "handling list_sessions");
        match self.compositor.list_sessions().await {
            Ok(sessions) => {
                let text = if sessions.is_empty() {
                    "No active sessions found".to_string()
                } else {
                    let mut lines = vec![format!("Active sessions ({}):", sessions.len())];
                    for s in &sessions {
                        let marker = if current_agent.as_ref().is_some_and(|a| a == &s.name) {
                            " <- you"
                        } else {
                            ""
                        };
                        lines.push(format!(
                            "- name: {} | id: {} | cwd: {} | status: {:?}{}",
                            s.name,
                            s.session_id,
                            s.cwd.display(),
                            s.status,
                            marker
                        ));
                        if let Some(ref p) = s.current_prompt {
                            let preview: String = p.content.chars().take(50).collect();
                            lines.push(format!(
                                "  current prompt: {} (id: {})",
                                preview, p.prompt_id
                            ));
                        }
                    }
                    lines.join("\n")
                };
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(format!("failed to list sessions: {}", e)),
        }
    }

    /// Create and charter a new agent session.
    #[tool(description = "Create a new persistent agent session")]
    async fn create_session(
        &self,
        Parameters(req): Parameters<CreateSessionRequest>,
    ) -> Result<CallToolResult, String> {
        info!(name = %req.name, cwd = %req.cwd, ?req.allowed_tool_kinds, "handling create_session");
        let cwd = if std::path::Path::new(&req.cwd).is_absolute() {
            PathBuf::from(&req.cwd)
        } else {
            std::env::current_dir().unwrap_or_default().join(&req.cwd)
        };
        match self
            .compositor
            .create_session(
                &req.name,
                cwd,
                &req.charter,
                req.allowed_tool_kinds,
                req.mcp_servers,
            )
            .await
        {
            Ok((info, charter_prompt_id)) => {
                let mut text = format!(
                    "Session '{}' created\nid: {}\ncwd: {}\nstatus: {:?}",
                    req.name,
                    info.session_id,
                    info.cwd.display(),
                    info.status
                );
                if let Some(prompt_id) = charter_prompt_id {
                    text.push_str(&format!("\nCharter prompt id: {}", prompt_id));
                }
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => {
                error!(error = %e, "create_session failed");
                Err(format!("failed to create session: {}", e))
            }
        }
    }

    /// Send a message to an existing session by name or id.
    /// The caller is identified from the `agent` query parameter on the MCP
    /// request URL, when available. By default the target agent's response is
    /// sent back to the caller as a new message once it finishes its turn. Set
    /// `need_result` to false for fire-and-forget messages. If the caller cannot
    /// be identified, `need_result` is treated as false because there is no
    /// session to forward the result to.
    #[tool(description = "Send a message to an active session")]
    async fn send_message(
        &self,
        OptionalExtension(parts): OptionalExtension<Parts>,
        Parameters(req): Parameters<SendMessageRequest>,
    ) -> Result<CallToolResult, String> {
        let current_agent = parts.and_then(|parts| {
            parts.uri.query().and_then(|q| {
                q.split('&').find_map(|p| {
                    let (k, v) = p.split_once('=')?;
                    if k == "agent" {
                        Some(v.to_string())
                    } else {
                        None
                    }
                })
            })
        });
        info!(
            target = %req.target,
            need_result = req.need_result,
            ?current_agent,
            "handling send_message"
        );

        let effective_need_result = req.need_result && current_agent.is_some();
        let prompt_id = self
            .compositor
            .send_message_async(
                &req.target,
                &req.content,
                current_agent.as_deref(),
                effective_need_result,
            )
            .await
            .map_err(|e| format!("failed to send message: {}", e))?;

        let text = if effective_need_result {
            if let Some(ref agent) = current_agent {
                format!(
                    "Message queued (prompt id: {}). The agent's response will be sent back to you ({}) as a new message when the prompt completes.",
                    prompt_id, agent
                )
            } else {
                // Unreachable because effective_need_result is false when current_agent is None.
                format!(
                    "Message queued (prompt id: {}). The agent's response will be sent back as a new message when the prompt completes.",
                    prompt_id
                )
            }
        } else if req.need_result {
            format!(
                "Message queued (prompt id: {}). The caller could not be identified, so the result will not be forwarded.",
                prompt_id
            )
        } else {
            format!("Message queued (prompt id: {}).", prompt_id)
        };

        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    /// Get information about the acompose system (configuration, MCP tools, architecture).
    #[tool(description = "Get acompose system documentation and configuration guide")]
    async fn agent_info(&self) -> Result<CallToolResult, String> {
        info!("handling agent_info");
        Ok(CallToolResult::success(vec![Content::text(README)]))
    }

    /// Recreate a session by shutting it down, removing it, and spawning a new one.
    #[tool(description = "Recreate an existing session with optional new parameters")]
    async fn recreate_session(
        &self,
        Parameters(req): Parameters<RecreateSessionRequest>,
    ) -> Result<CallToolResult, String> {
        info!(name = %req.name, "handling recreate_session");
        match self
            .compositor
            .recreate_session(&req.name, req.extra_charter.as_deref())
            .await
        {
            Ok((info, charter_prompt_id)) => {
                let mut text = format!(
                    "Session '{}' recreated\nid: {}\ncwd: {}\nstatus: {:?}",
                    req.name,
                    info.session_id,
                    info.cwd.display(),
                    info.status
                );
                if let Some(prompt_id) = charter_prompt_id {
                    text.push_str(&format!("\nCharter prompt id: {}", prompt_id));
                }
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => {
                error!(error = %e, "recreate_session failed");
                Err(format!("failed to recreate session: {}", e))
            }
        }
    }

    /// Delete a session by name or id, shutting it down and removing it from state.
    #[tool(description = "Delete an existing session")]
    async fn delete_session(
        &self,
        Parameters(req): Parameters<DeleteSessionRequest>,
    ) -> Result<CallToolResult, String> {
        info!(name = %req.name, "handling delete_session");
        match self.compositor.delete_session(&req.name).await {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Session '{}' deleted",
                req.name
            ))])),
            Err(e) => {
                error!(error = %e, "delete_session failed");
                Err(format!("failed to delete session: {}", e))
            }
        }
    }

    /// List cron jobs attached to a session.
    #[tool(description = "List cron jobs for a session")]
    async fn list_cron_jobs(
        &self,
        OptionalExtension(parts): OptionalExtension<Parts>,
        Parameters(req): Parameters<ListCronJobsRequest>,
    ) -> Result<CallToolResult, String> {
        let current_agent = parts.and_then(|parts| extract_agent_from_query(&parts));
        let session = req.session.or(current_agent).ok_or_else(|| {
            "session name is required; either pass it explicitly or call from an identified agent"
                .to_string()
        })?;
        info!(session = %session, "handling list_cron_jobs");
        match self.compositor.list_cron_jobs(&session).await {
            Ok(jobs) => {
                if jobs.is_empty() {
                    Ok(CallToolResult::success(vec![Content::text(format!(
                        "No cron jobs found for session '{}'",
                        session
                    ))]))
                } else {
                    let mut lines = vec![format!(
                        "Cron jobs for session '{}' ({}):",
                        session,
                        jobs.len()
                    )];
                    for j in jobs {
                        let last = j
                            .last_run_at
                            .map_or_else(|| "never".to_string(), |d| d.to_rfc3339());
                        let next = j
                            .next_run_at
                            .map_or_else(|| "unknown".to_string(), |d| d.to_rfc3339());
                        let remaining = j
                            .next_run_at
                            .map_or_else(|| "unknown".to_string(), format_remaining);
                        let schedule_or_run_at = j
                            .config
                            .run_at
                            .as_ref()
                            .map(|r| format!("run_at: {}", r))
                            .or_else(|| {
                                j.config
                                    .schedule
                                    .as_ref()
                                    .map(|s| format!("schedule: {}", s))
                            })
                            .unwrap_or_else(|| "unknown".to_string());
                        lines.push(format!(
                            "- name: {}\n  {}\n  timezone: {}\n  description: {}\n  misfire: {:?}\n  last_run: {}\n  next_run: {} ({})",
                            j.config.name,
                            schedule_or_run_at,
                            j.config.timezone,
                            j.description,
                            j.config.misfire_policy,
                            last,
                            next,
                            remaining
                        ));
                    }
                    Ok(CallToolResult::success(vec![Content::text(
                        lines.join("\n"),
                    )]))
                }
            }
            Err(e) => {
                error!(error = %e, "list_cron_jobs failed");
                Err(format!("failed to list cron jobs: {}", e))
            }
        }
    }

    /// Add or replace a cron job for a session.
    #[tool(description = "Add or replace a cron job for a session")]
    async fn add_cron_job(
        &self,
        OptionalExtension(parts): OptionalExtension<Parts>,
        Parameters(req): Parameters<AddCronJobRequest>,
    ) -> Result<CallToolResult, String> {
        let current_agent = parts.and_then(|parts| extract_agent_from_query(&parts));
        let session = req.session.or(current_agent).ok_or_else(|| {
            "session name is required; either pass it explicitly or call from an identified agent"
                .to_string()
        })?;
        let job_name = req.name.clone();
        let schedule = req.schedule.clone();
        let run_at = req.run_at.clone();
        let timezone = req.timezone.clone();

        if schedule.is_none() && run_at.is_none() {
            return Err("either schedule or run_at must be provided".to_string());
        }

        let job = CronJobConfig {
            name: req.name,
            schedule: req.schedule,
            prompt: req.prompt,
            timezone: req.timezone,
            misfire_policy: req.misfire_policy,
            run_at: req.run_at,
        };
        info!(session = %session, job_name = %job.name, "handling add_cron_job");
        match self.compositor.add_cron_job(&session, job).await {
            Ok(info) => {
                let next = info
                    .next_run_at
                    .map_or_else(|| "unknown".to_string(), |d| d.to_rfc3339());
                let remaining = info
                    .next_run_at
                    .map_or_else(|| "unknown".to_string(), format_remaining);
                let schedule_or_run_at = run_at
                    .as_ref()
                    .map(|r| format!("run_at: {}", r))
                    .or_else(|| schedule.as_ref().map(|s| format!("schedule: {}", s)))
                    .unwrap_or_else(|| "unknown".to_string());
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Cron job '{}' added to session '{}'\n{}\ntimezone: {}\ndescription: {}\nnext_run: {} ({})",
                    job_name,
                    session,
                    schedule_or_run_at,
                    timezone,
                    info.description,
                    next,
                    remaining
                ))]))
            }
            Err(e) => {
                error!(error = %e, "add_cron_job failed");
                Err(format!("failed to add cron job: {}", e))
            }
        }
    }

    /// Remove a cron job from a session.
    #[tool(description = "Remove a cron job from a session")]
    async fn remove_cron_job(
        &self,
        OptionalExtension(parts): OptionalExtension<Parts>,
        Parameters(req): Parameters<RemoveCronJobRequest>,
    ) -> Result<CallToolResult, String> {
        let current_agent = parts.and_then(|parts| extract_agent_from_query(&parts));
        let session = req.session.or(current_agent).ok_or_else(|| {
            "session name is required; either pass it explicitly or call from an identified agent".to_string()
        })?;
        info!(session = %session, job_name = %req.job_name, "handling remove_cron_job");
        match self
            .compositor
            .remove_cron_job(&session, &req.job_name)
            .await
        {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Cron job '{}' removed from session '{}'",
                req.job_name, session
            ))])),
            Err(e) => {
                error!(error = %e, "remove_cron_job failed");
                Err(format!("failed to remove cron job: {}", e))
            }
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ComposeMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Compose compositor MCP server. Manage persistent Kimi ACP sessions.",
        )
    }
}

fn extract_agent_from_query(parts: &Parts) -> Option<String> {
    parts.uri.query().and_then(|q| {
        q.split('&').find_map(|p| {
            let (k, v) = p.split_once('=')?;
            if k == "agent" {
                Some(v.to_string())
            } else {
                None
            }
        })
    })
}
