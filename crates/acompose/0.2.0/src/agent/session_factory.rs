use std::path::PathBuf;

use crate::agent::session_actor::SessionActor;
use crate::compositor::state::{PersistSession, SessionState};
use crate::config::McpServer;
use agent_client_protocol::ByteStreams;
use agent_client_protocol::schema::v1::ToolKind;
use async_trait::async_trait;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

/// Configuration for creating a new session.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub name: String,
    pub cwd: PathBuf,
    pub charter: String,
    pub allowed_tool_kinds: Vec<ToolKind>,
    pub mcp_servers: Vec<McpServer>,
    pub load_session_id: Option<String>,
}

impl SessionConfig {
    #[must_use]
    pub fn from_state(name: &str, state: &SessionState) -> Self {
        Self {
            name: name.to_string(),
            cwd: state.cwd.clone(),
            charter: state.charter.clone().unwrap_or_default(),
            allowed_tool_kinds: state.allowed_tool_kinds.clone(),
            mcp_servers: state.mcp_servers.clone(),
            load_session_id: Some(state.session_id.clone()),
        }
    }
}

/// Factory for creating ACP sessions.
///
/// Production implementations spawn real `kimi acp` processes; test
/// implementations create in-memory agents.
#[async_trait]
pub trait SessionFactory: Send + Sync {
    /// Create a session with the given configuration and return its actor.
    ///
    /// The actor is not running yet; the caller must call `spawn` on it.
    async fn create(
        &self,
        config: SessionConfig,
        persist_tx: mpsc::UnboundedSender<PersistSession>,
        forward_tx: mpsc::UnboundedSender<(String, crate::compositor::state::PromptJob)>,
        cancel_token: CancellationToken,
    ) -> anyhow::Result<SessionActor>;
}

/// Production session factory that spawns `kimi acp` processes.
#[derive(Debug, Clone)]
pub struct StdioFactory {
    kimi_binary: String,
}

impl StdioFactory {
    #[must_use]
    pub fn new(kimi_binary: impl Into<String>) -> Self {
        Self {
            kimi_binary: kimi_binary.into(),
        }
    }

    /// Spawn a `kimi acp` process and connect a session actor to it.
    #[allow(clippy::too_many_arguments)]
    pub async fn spawn(
        kimi_binary: &str,
        name: &str,
        cwd: PathBuf,
        charter: String,
        allowed_tool_kinds: Vec<ToolKind>,
        load_session_id: Option<String>,
        mcp_servers: Vec<McpServer>,
        persist_tx: mpsc::UnboundedSender<PersistSession>,
        forward_tx: mpsc::UnboundedSender<(String, crate::compositor::state::PromptJob)>,
        cancel_token: CancellationToken,
    ) -> anyhow::Result<SessionActor> {
        info!(name, cwd = %cwd.display(), ?load_session_id, "spawning agent");

        // We deliberately bypass `agent_client_protocol::AcpAgent` here and use
        // `tokio::process::Command` + `ByteStreams` directly.
        //
        // `AcpAgent` relies on `async-process` / `async-io` for stdio, which runs its own
        // reactor thread (kqueue/epoll). On macOS (and likely Linux) this reactor spams
        // `wake_by_ref` on the Tokio waker, causing a self-wake busy-loop that consumes
        // 100% CPU with 30M+ self-wakes per minute. Using `tokio::process` keeps everything
        // inside the Tokio runtime and avoids the incompatibility.
        let mut cmd = Command::new(kimi_binary);
        cmd.arg("acp")
            .current_dir(&cwd)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| anyhow::anyhow!("failed to spawn '{} acp': {}", kimi_binary, e))?;

        let child_stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to open stdin"))?;
        let child_stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to open stdout"))?;
        let child_stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to open stderr"))?;

        let name_for_stderr = name.to_string();
        tokio::spawn(async move {
            let mut stderr = tokio::io::BufReader::new(child_stderr);
            let mut buf = String::new();
            loop {
                match stderr.read_line(&mut buf).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let line = buf.trim_end();
                        if !line.is_empty() {
                            warn!(name = %name_for_stderr, "agent stderr: {}", line);
                        }
                        buf.clear();
                    }
                    Err(e) => {
                        warn!(name = %name_for_stderr, "stderr read error: {}", e);
                        break;
                    }
                }
            }
        });

        let byte_streams = ByteStreams::new(child_stdin.compat_write(), child_stdout.compat());

        let result = SessionActor::connect(
            name,
            cwd,
            charter,
            allowed_tool_kinds,
            load_session_id,
            mcp_servers,
            byte_streams,
            persist_tx,
            forward_tx,
            cancel_token.clone(),
        )
        .await;

        if result.is_err() {
            let _ = child.kill().await;
        }

        let actor = result?;

        let cancel = cancel_token.child_token();
        tokio::spawn(async move {
            cancel.cancelled().await;
            let _ = child.kill().await;
            let _ = child.wait().await;
        });

        Ok(actor)
    }
}

#[async_trait]
impl SessionFactory for StdioFactory {
    async fn create(
        &self,
        config: SessionConfig,
        persist_tx: mpsc::UnboundedSender<PersistSession>,
        forward_tx: mpsc::UnboundedSender<(String, crate::compositor::state::PromptJob)>,
        cancel_token: CancellationToken,
    ) -> anyhow::Result<SessionActor> {
        Self::spawn(
            &self.kimi_binary,
            &config.name,
            config.cwd,
            config.charter,
            config.allowed_tool_kinds,
            config.load_session_id,
            config.mcp_servers,
            persist_tx,
            forward_tx,
            cancel_token,
        )
        .await
    }
}
