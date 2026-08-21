use std::path::PathBuf;
use std::sync::Arc;

use agent_client_protocol::schema::v1::ToolKind;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;
use tracing::{error, info};

use crate::orchestrator::Orchestrator;

/// Input for `compose_create_session`.
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
}

fn default_true() -> bool {
    true
}

/// Input for `compose_send_message`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendMessageRequest {
    /// Target session name or session id.
    pub target: String,
    /// Message content to send.
    pub content: String,
    /// Whether to wait for the prompt to complete. If false, returns a prompt_id immediately.
    #[serde(default = "default_true")]
    pub wait: bool,
    /// Optional timeout in milliseconds. If the prompt does not finish in time, returns a prompt_id for polling.
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

/// Input for `compose_get_prompt_result`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPromptResultRequest {
    /// Prompt id returned by `compose_send_message` (when not waiting) or `compose_send_message_async`.
    pub prompt_id: String,
}

/// MCP server exposing compose orchestration tools.
#[derive(Debug, Clone)]
pub struct ComposeMcpServer {
    tool_router: ToolRouter<Self>,
    orchestrator: Arc<Orchestrator>,
}

impl ComposeMcpServer {
    /// Create a new compose MCP server backed by the given orchestrator.
    pub fn new(orchestrator: Arc<Orchestrator>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            orchestrator,
        }
    }
}

#[tool_router]
impl ComposeMcpServer {
    /// List all active sessions managed by the orchestrator.
    #[tool(description = "List all active acompose sessions")]
    async fn compose_list_sessions(&self) -> Result<String, String> {
        info!("handling compose_list_sessions");
        match self.orchestrator.list_sessions() {
            Ok(sessions) => serde_json::to_string_pretty(&sessions)
                .map_err(|e| format!("failed to serialize sessions: {}", e)),
            Err(e) => Err(format!("failed to list sessions: {}", e)),
        }
    }

    /// Create and charter a new agent session.
    #[tool(description = "Create a new persistent agent session")]
    async fn compose_create_session(
        &self,
        Parameters(req): Parameters<CreateSessionRequest>,
    ) -> Result<String, String> {
        info!(name = %req.name, cwd = %req.cwd, ?req.allowed_tool_kinds, "handling compose_create_session");
        let cwd = PathBuf::from(&req.cwd);
        match self
            .orchestrator
            .create_session(&req.name, cwd, &req.charter, req.allowed_tool_kinds)
            .await
        {
            Ok(info) => serde_json::to_string_pretty(&info)
                .map_err(|e| format!("failed to serialize session info: {}", e)),
            Err(e) => {
                error!(error = %e, "compose_create_session failed");
                Err(format!("failed to create session: {}", e))
            }
        }
    }

    /// Send a message to an existing session by name or id.
    /// By default waits for the agent's turn to finish. Use `wait=false` or `timeout_ms`
    /// to run the prompt in the background and poll later with `compose_get_prompt_result`.
    #[tool(description = "Send a message to an active session")]
    async fn compose_send_message(
        &self,
        Parameters(req): Parameters<SendMessageRequest>,
    ) -> Result<String, String> {
        info!(target = %req.target, wait = req.wait, timeout_ms = ?req.timeout_ms, "handling compose_send_message");

        let outcome = if !req.wait {
            let prompt_id = self
                .orchestrator
                .send_message_async(&req.target, &req.content)
                .await;
            serde_json::json!({
                "status": "pending",
                "prompt_id": prompt_id,
            })
        } else if let Some(timeout_ms) = req.timeout_ms {
            match self
                .orchestrator
                .send_message_with_timeout(&req.target, &req.content, timeout_ms)
                .await
            {
                Ok(crate::orchestrator::PromptOutcome::Completed(result)) => {
                    serde_json::json!({
                        "status": "completed",
                        "stop_reason": format!("{:?}", result.stop_reason),
                        "text": result.text.join(""),
                    })
                }
                Ok(crate::orchestrator::PromptOutcome::Timeout { prompt_id }) => {
                    serde_json::json!({
                        "status": "pending",
                        "prompt_id": prompt_id,
                    })
                }
                Err(e) => {
                    error!(error = %e, "compose_send_message failed");
                    return Err(format!("failed to send message: {}", e));
                }
            }
        } else {
            match self
                .orchestrator
                .send_message(&req.target, &req.content)
                .await
            {
                Ok(result) => serde_json::json!({
                    "status": "completed",
                    "stop_reason": format!("{:?}", result.stop_reason),
                    "text": result.text.join(""),
                }),
                Err(e) => {
                    error!(error = %e, "compose_send_message failed");
                    return Err(format!("failed to send message: {}", e));
                }
            }
        };

        serde_json::to_string_pretty(&outcome)
            .map_err(|e| format!("failed to serialize response: {}", e))
    }

    /// Retrieve the result of a previously started asynchronous prompt.
    #[tool(description = "Get the result of an async prompt by prompt_id")]
    async fn compose_get_prompt_result(
        &self,
        Parameters(req): Parameters<GetPromptResultRequest>,
    ) -> Result<String, String> {
        info!(prompt_id = %req.prompt_id, "handling compose_get_prompt_result");
        match self.orchestrator.get_prompt_result(&req.prompt_id).await {
            Ok(Some(job)) => serde_json::to_string_pretty(&job)
                .map_err(|e| format!("failed to serialize prompt result: {}", e)),
            Ok(None) => Err(format!("prompt_id '{}' not found", req.prompt_id)),
            Err(e) => {
                error!(error = %e, "compose_get_prompt_result failed");
                Err(format!("failed to get prompt result: {}", e))
            }
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ComposeMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Compose orchestrator MCP server. Manage persistent Kimi ACP sessions.",
        )
    }
}
