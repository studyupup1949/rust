use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use agent_client_protocol::schema::v1::ToolKind;
use anyhow::Context;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use path_clean::PathClean;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::agent::session_actor::SessionCommand;
use crate::agent::session_factory::{SessionConfig as FactorySessionConfig, SessionFactory};
use crate::agent::session_handle::SessionHandle;
use crate::config::{Config, CronJobConfig, McpServer};
use crate::cron::worker::CronCommand;

use persistence::PersistenceActor;
use state::{FileStateStore, PromptJob, SessionState, State, StateStore};

pub mod persistence;
pub mod state;

/// Serializable information about an active session.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionInfo {
    pub name: String,
    pub session_id: String,
    pub cwd: PathBuf,
    pub status: SessionStatus,
    pub current_prompt: Option<PendingPromptSummary>,
}

/// Lifecycle status of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// The session is ready to receive follow-up prompts.
    Ready,
    /// The session encountered an error.
    Error,
}

/// Summary of a pending prompt for display in session listings.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PendingPromptSummary {
    pub prompt_id: String,
    pub content: String,
    pub status: state::PromptStatus,
}

impl From<&SessionHandle> for SessionInfo {
    fn from(handle: &SessionHandle) -> Self {
        Self {
            name: handle.name.clone(),
            session_id: handle.session_id.clone(),
            cwd: handle.cwd.clone(),
            status: SessionStatus::Ready,
            current_prompt: None,
        }
    }
}

/// Summary of a cron job for listings.
#[derive(Debug, Clone)]
pub struct CronJobInfo {
    pub config: CronJobConfig,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub description: String,
}

/// Resolve a possibly-relative path against the given base directory and
/// return an absolute, lexically-clean path. Symlinks are not resolved.
fn resolve_path_against(base: &Path, path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let path = path.as_ref();
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    Ok(absolute.clean())
}

/// Resolve relative `cwd` and `state_path` fields in `config` against
/// `base_dir`. After this call all paths in `config` are absolute.
fn resolve_config_paths(config: &mut Config, base_dir: &Path) -> std::io::Result<()> {
    for session in &mut config.sessions {
        session.cwd = resolve_path_against(base_dir, &session.cwd)?;
    }
    if let Some(state_path) = &config.state_path {
        config.state_path = Some(resolve_path_against(base_dir, state_path)?);
    }
    Ok(())
}

/// Validate that a session name is non-empty and contains only safe characters.
fn validate_session_name(name: &str) -> anyhow::Result<()> {
    if name.is_empty() {
        anyhow::bail!("session name cannot be empty");
    }
    if name.len() > 128 {
        anyhow::bail!("session name too long (max 128 characters)");
    }
    if name
        .chars()
        .any(|c| c.is_whitespace() || c == '/' || c == '\\' || c == '\0')
    {
        anyhow::bail!("session name contains invalid characters");
    }
    Ok(())
}

/// Shared registry of active ACP sessions and the factory used to spawn them.
#[derive(Clone)]
pub struct Compositor {
    session_factory: Arc<dyn SessionFactory>,
    persist_tx: mpsc::UnboundedSender<state::PersistSession>,
    pub(crate) sessions: Arc<DashMap<String, SessionHandle>>,
    next_prompt_id: Arc<AtomicU64>,
    /// Producer end of the forward channel used by sessions to send results
    /// to other sessions.
    forward_tx: mpsc::UnboundedSender<(String, PromptJob)>,
    cancel_token: CancellationToken,
}

impl std::fmt::Debug for Compositor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("compositor")
            .field("sessions", &self.sessions)
            .field("next_prompt_id", &self.next_prompt_id)
            .finish_non_exhaustive()
    }
}

impl Compositor {
    /// Create a new compositor.
    ///
    /// If `state_store` is provided, a persistence actor is spawned and will
    /// save state changes until the compositor is dropped/shutdown. If not,
    /// persistence is a no-op.
    ///
    /// If `cancel_token` is provided, it is used to coordinate shutdown of the
    /// compositor's background tasks. Otherwise a fresh token is created.
    pub fn new(
        session_factory: Arc<dyn SessionFactory>,
        state_store: Option<Arc<dyn StateStore>>,
        cancel_token: Option<CancellationToken>,
    ) -> anyhow::Result<Self> {
        let cancel_token = cancel_token.unwrap_or_default();
        let persist_tx: mpsc::UnboundedSender<state::PersistSession> =
            if let Some(store) = state_store {
                let (actor, tx) = PersistenceActor::new(store);
                let ct = cancel_token.child_token();
                tokio::spawn(async move { actor.run(ct).await });
                tx
            } else {
                let (tx, mut rx) = mpsc::unbounded_channel::<state::PersistSession>();
                let ct = cancel_token.child_token();
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = rx.recv() => {}
                            () = ct.cancelled() => break,
                        }
                    }
                });
                tx
            };

        let (forward_tx, forward_rx) = mpsc::unbounded_channel();
        let compositor = Self {
            session_factory,
            persist_tx,
            sessions: Arc::new(DashMap::new()),
            next_prompt_id: Arc::new(AtomicU64::new(1)),
            forward_tx,
            cancel_token,
        };
        let forward_compositor = compositor.clone();
        let forward_cancel = compositor.cancel_token.child_token();
        tokio::spawn(async move {
            forward_compositor
                .run_forward_task(forward_rx, forward_cancel)
                .await;
        });

        Ok(compositor)
    }

    /// Load configuration from `config_path`, resolve relative paths, load the
    /// persisted state from the effective state store, and create an
    /// compositor. The compositor does **not** spawn any sessions; call
    /// [`Self::spawn_sessions_from_config`] after starting any services (e.g.
    /// the MCP server) that spawned agents may need.
    pub async fn from_config_file(
        config_path: &Path,
        base_dir: Option<&Path>,
        state_store: Option<Arc<dyn StateStore>>,
        cancel_token: Option<CancellationToken>,
    ) -> anyhow::Result<(Self, Config, State)> {
        let mut config = Config::from_file(config_path)?;
        let config_dir = config_path
            .parent()
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        let base_dir = base_dir
            .map(|p| {
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    config_dir.join(p)
                }
            })
            .unwrap_or(config_dir);
        let base_dir = resolve_path_against(
            &std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            &base_dir,
        )?;
        resolve_config_paths(&mut config, &base_dir)?;

        let effective_state_store: Option<Arc<dyn StateStore>> = match state_store {
            Some(store) => Some(store),
            None => {
                if let Some(state_path) = &config.state_path {
                    Some(Arc::new(FileStateStore::new(state_path.clone())))
                } else {
                    None
                }
            }
        };

        let state = if let Some(store) = &effective_state_store {
            store.load().await.unwrap_or_default()
        } else {
            State::default()
        };

        let session_factory = Arc::new(crate::agent::session_factory::StdioFactory::new(
            &config.kimi_binary,
        ));
        let compositor = Self::new(session_factory, effective_state_store, cancel_token)?;
        Ok((compositor, config, state))
    }

    /// Forward task: receives `(target_session, PromptJob)` from sessions and
    /// queues the job as a new prompt on the target session.
    async fn run_forward_task(
        self,
        mut rx: mpsc::UnboundedReceiver<(String, PromptJob)>,
        cancel: CancellationToken,
    ) {
        loop {
            tokio::select! {
                Some((target, job)) = rx.recv() => {
                    if let Some(handle) = self.get_session_handle(&target) {
                        let prompt_id = self
                            .next_prompt_id
                            .fetch_add(1, Ordering::SeqCst)
                            .to_string();
                        handle.send_prompt(&prompt_id, &job.content, job.send_result_to.clone());
                    } else {
                        warn!(target, "forward target session not found, dropping result");
                    }
                }
                () = cancel.cancelled() => break,
            }
        }
    }

    /// Spawn all sessions declared in the config and load any persisted sessions.
    /// Must be called after the integrated MCP server is already listening so that
    /// spawned agents can discover the acompose control-plane tools during init.
    ///
    /// Persisted sessions take precedence over config-declared sessions: if a
    /// session name exists in `state`, it is resumed with its saved configuration
    /// instead of being recreated from the config.
    pub async fn spawn_sessions_from_config(
        &self,
        config: &Config,
        state: &State,
    ) -> anyhow::Result<Vec<SessionInfo>> {
        let mut infos = Vec::new();
        let mut loaded_names = std::collections::HashSet::<&str>::new();

        // First, resume any sessions that exist in the persisted state.
        for (name, session_state) in &state.sessions {
            match self.load_session(name, session_state).await {
                Ok((info, _)) => {
                    loaded_names.insert(name.as_str());
                    infos.push(info);
                }
                Err(e) => {
                    error!(name, error = %e, "failed to load persisted session");
                }
            }
        }

        // Then create sessions from config that were not restored from state.
        for session in &config.sessions {
            if loaded_names.contains(session.name.as_str()) {
                continue;
            }
            match self
                .create_session(
                    &session.name,
                    session.cwd.clone(),
                    &session.charter,
                    session.allowed_tool_kinds.clone(),
                    config.resolve_mcp_servers(&session.mcp_servers, Some(&session.name)),
                )
                .await
            {
                Ok((info, _)) => infos.push(info),
                Err(e) => {
                    error!(name = %session.name, error = %e, "failed to start session from config");
                }
            }
        }

        Ok(infos)
    }

    /// Spawn or load a session and register it.
    pub async fn create_session(
        &self,
        name: &str,
        cwd: PathBuf,
        charter: &str,
        allowed_tool_kinds: Vec<ToolKind>,
        mcp_servers: Vec<McpServer>,
    ) -> anyhow::Result<(SessionInfo, Option<String>)> {
        validate_session_name(name)?;

        if self.get_session_handle(name).is_some() {
            anyhow::bail!("session '{}' already exists", name);
        }

        let actor = self
            .session_factory
            .create(
                FactorySessionConfig {
                    name: name.to_string(),
                    cwd: cwd.clone(),
                    charter: charter.to_string(),
                    allowed_tool_kinds: allowed_tool_kinds.clone(),
                    mcp_servers: mcp_servers.clone(),
                    load_session_id: None,
                },
                self.persist_tx.clone(),
                self.forward_tx.clone(),
                self.cancel_token.child_token(),
            )
            .await?;

        let handle = actor.spawn(None).await;
        self.register_session(name, handle.clone());

        let info = SessionInfo::from(&handle);

        let charter_prompt_id = if charter.is_empty() {
            None
        } else {
            Some(self.send_message_async(name, charter, None, false).await?)
        };

        Ok((info, charter_prompt_id))
    }

    /// Load a session from persisted state. The session actor will restore its
    /// queued/pending jobs and cron jobs.
    pub async fn load_session(
        &self,
        name: &str,
        session_state: &SessionState,
    ) -> anyhow::Result<(SessionInfo, Option<String>)> {
        validate_session_name(name)?;

        if self.get_session_handle(name).is_some() {
            anyhow::bail!("session '{}' already exists", name);
        }

        let actor = self
            .session_factory
            .create(
                FactorySessionConfig::from_state(name, session_state),
                self.persist_tx.clone(),
                self.forward_tx.clone(),
                self.cancel_token.child_token(),
            )
            .await?;

        let handle = actor.spawn(Some(session_state)).await;
        self.register_session(name, handle.clone());

        let info = SessionInfo::from(&handle);

        Ok((info, None))
    }

    fn register_session(&self, name: &str, handle: SessionHandle) {
        self.sessions.insert(name.to_string(), handle);
    }

    /// List all registered sessions.
    pub async fn list_sessions(&self) -> anyhow::Result<Vec<SessionInfo>> {
        let handles: Vec<SessionHandle> = {
            self.sessions
                .iter()
                .map(|item| item.value().clone())
                .collect()
        };
        let mut infos = Vec::new();
        for handle in handles {
            infos.push(SessionInfo::from(&handle));
        }
        Ok(infos)
    }

    /// Get a specific session by name or session id.
    pub async fn get_session(&self, name_or_id: &str) -> Option<SessionInfo> {
        self.get_session_handle(name_or_id)
            .map(|s| SessionInfo::from(&s))
    }

    /// Recreate a session by asking its actor to replace the underlying ACP
    /// session. The actor keeps the same command channel and cron worker.
    /// If `extra_charter` is provided, it is appended to the session's charter
    /// and sent as a follow-up prompt to the recreated session.
    pub async fn recreate_session(
        &self,
        name: &str,
        extra_charter: Option<&str>,
    ) -> anyhow::Result<(SessionInfo, Option<String>)> {
        let handle = self.get_session_handle(name).context("Session not found")?;
        let (tx, rx) = oneshot::channel();
        let _ = handle.cmd_tx.send(SessionCommand::Recreate {
            extra_charter: extra_charter.map(String::from),
            respond_to: tx,
        });
        let (new_handle, charter_prompt_id) = rx
            .await
            .map_err(|_| anyhow::anyhow!("session actor dropped"))??;

        self.sessions.insert(name.to_string(), new_handle.clone());

        let info = SessionInfo::from(&new_handle);
        Ok((info, charter_prompt_id))
    }

    /// Delete a session by name or session id, shutting it down and removing it from state.
    pub async fn delete_session(&self, name_or_id: &str) -> anyhow::Result<()> {
        let handle = self
            .get_session_handle(name_or_id)
            .context("Session not found")?;

        let name = handle.name.clone();
        let _ = handle.shutdown().await;

        self.sessions.remove(&name);

        let _ = self.persist_tx.send(state::PersistSession {
            name: name.clone(),
            state: None,
        });

        info!(name, "session deleted");
        Ok(())
    }

    /// Return a clone of the compositor's shutdown cancellation token.
    #[must_use]
    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// Look up a registered session by name or session id.
    #[must_use]
    pub fn get_session_handle(&self, name_or_id: &str) -> Option<SessionHandle> {
        self.sessions
            .get(name_or_id)
            .map(|r| r.value().clone())
            .or_else(|| {
                self.sessions
                    .iter()
                    .find(|r| r.value().session_id == name_or_id)
                    .map(|r| r.value().clone())
            })
    }

    /// Send a message asynchronously and return a prompt id.
    ///
    /// The returned receiver resolves once the prompt finishes. If
    /// `send_result_to` is provided and `need_result` is true, the result is
    /// forwarded to the named session by the session actor itself.
    pub async fn send_message_async(
        &self,
        target: &str,
        content: &str,
        sender_name: Option<&str>,
        need_result: bool,
    ) -> anyhow::Result<String> {
        let prompt_id = self
            .next_prompt_id
            .fetch_add(1, Ordering::SeqCst)
            .to_string();

        let handle = self
            .get_session_handle(target)
            .context("Session not found")?;

        let formatted_content = match sender_name {
            Some(name) => format!("Message from agent '{}':\n\n{}", name, content),
            None => content.to_string(),
        };

        let send_result_to = if need_result {
            sender_name.map(String::from)
        } else {
            None
        };
        handle.send_prompt(&prompt_id, &formatted_content, send_result_to);

        Ok(prompt_id)
    }

    /// Add or replace a cron job for a session.
    pub async fn add_cron_job(
        &self,
        session_name: &str,
        job: CronJobConfig,
    ) -> anyhow::Result<CronJobInfo> {
        let handle = self
            .get_session_handle(session_name)
            .context("Session not found")?;
        let (tx, rx) = oneshot::channel();
        let _ = handle
            .cmd_tx
            .send(SessionCommand::Cron(CronCommand::AddJob {
                job,
                respond_to: tx,
            }));
        rx.await
            .map_err(|_| anyhow::anyhow!("session actor dropped"))?
    }

    /// Remove a cron job from a session.
    pub async fn remove_cron_job(&self, session_name: &str, job_name: &str) -> anyhow::Result<()> {
        let handle = self
            .get_session_handle(session_name)
            .context("Session not found")?;
        let (tx, rx) = oneshot::channel();
        let _ = handle
            .cmd_tx
            .send(SessionCommand::Cron(CronCommand::RemoveJob {
                job_name: job_name.to_string(),
                respond_to: tx,
            }));
        rx.await
            .map_err(|_| anyhow::anyhow!("session actor dropped"))?
    }

    /// List cron jobs for a session.
    pub async fn list_cron_jobs(&self, session_name: &str) -> anyhow::Result<Vec<CronJobInfo>> {
        let handle = self
            .get_session_handle(session_name)
            .context("Session not found")?;
        let (tx, rx) = oneshot::channel();
        let _ = handle
            .cmd_tx
            .send(SessionCommand::Cron(CronCommand::ListJobs {
                respond_to: tx,
            }));
        rx.await
            .map_err(|_| anyhow::anyhow!("session actor dropped"))?
    }

    pub async fn shutdown(&self) {
        self.cancel_token.cancel();
        for r in self.sessions.iter() {
            let _ = r.value().shutdown().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_path_against() {
        let base = PathBuf::from("/tmp/config");

        let cwd = base.join("subdir");
        let resolved = resolve_path_against(&base, cwd).unwrap();
        assert!(resolved.is_absolute());
        assert_eq!(resolved, PathBuf::from("/tmp/config/subdir"));

        let abs = PathBuf::from("/abs/dir");
        let resolved = resolve_path_against(&base, abs).unwrap();
        assert_eq!(resolved, PathBuf::from("/abs/dir"));

        let dot = PathBuf::from("./agents/moderator");
        let resolved = resolve_path_against(&base, dot).unwrap();
        assert_eq!(resolved, PathBuf::from("/tmp/config/agents/moderator"));
    }
}
