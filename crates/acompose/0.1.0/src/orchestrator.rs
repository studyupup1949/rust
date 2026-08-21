use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use agent_client_protocol::schema::v1::ToolKind;
use tokio::sync::{Mutex as TokioMutex, RwLock as TokioRwLock};
use tokio::time::{sleep, timeout};
use tracing::{error, info, warn};

use crate::acp_client::{
    PromptResult, SessionHandle, SessionStatus, send_prompt, shutdown_session,
};
use crate::state::State;

/// Serializable information about an active session.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionInfo {
    pub name: String,
    pub session_id: String,
    pub cwd: PathBuf,
    pub status: SessionStatus,
}

impl From<&SessionHandle> for SessionInfo {
    fn from(handle: &SessionHandle) -> Self {
        let status = handle
            .status
            .read()
            .map(|s| *s)
            .unwrap_or(SessionStatus::Error);
        Self {
            name: handle.name.clone(),
            session_id: handle.session_id.clone(),
            cwd: handle.cwd.clone(),
            status,
        }
    }
}

/// Status of an asynchronous prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptStatus {
    Pending,
    Completed,
    Error,
}

/// Stored result of an asynchronous prompt.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PromptJob {
    pub target: String,
    pub content: String,
    pub status: PromptStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<PromptResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Outcome of a synchronous prompt call with a timeout.
#[derive(Debug, Clone)]
pub enum PromptOutcome {
    Completed(PromptResult),
    Timeout { prompt_id: String },
}

/// Shared registry of active ACP sessions and the `kimi` binary used to spawn them.
#[derive(Debug, Clone)]
pub struct Orchestrator {
    kimi_binary: String,
    state_path: PathBuf,
    state: Arc<RwLock<State>>,
    sessions: Arc<RwLock<HashMap<String, SessionHandle>>>,
    next_prompt_id: Arc<AtomicU64>,
    pending: Arc<TokioRwLock<HashMap<String, Arc<TokioMutex<PromptJob>>>>>,
}

impl Orchestrator {
    /// Create a new orchestrator.
    pub fn new(kimi_binary: String, state_path: PathBuf) -> anyhow::Result<Self> {
        let state = State::load(&state_path)?;
        Ok(Self {
            kimi_binary,
            state_path,
            state: Arc::new(RwLock::new(state)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            next_prompt_id: Arc::new(AtomicU64::new(1)),
            pending: Arc::new(TokioRwLock::new(HashMap::new())),
        })
    }

    /// Create an orchestrator and spawn all sessions from a config file.
    /// Also resumes any additional sessions persisted in `state.json`.
    pub async fn from_config(config_path: PathBuf) -> anyhow::Result<(Self, Vec<SessionInfo>, crate::config::McpServerConfig)> {
        let config = crate::config::Config::from_file(config_path.clone())?;
        let state_path = config_path
            .parent()
            .map(|p| p.join("state.json"))
            .unwrap_or_else(|| PathBuf::from("state.json"));
        let orchestrator = Self::new(config.kimi_binary, state_path)?;

        let mut infos = Vec::new();
        for session in &config.sessions {
            match orchestrator.create_session(
                &session.name,
                session.cwd.clone(),
                &session.charter,
                session.allowed_tool_kinds.clone(),
            ).await {
                Ok(info) => infos.push(info),
                Err(e) => {
                    error!(name = %session.name, error = %e, "failed to start session from config");
                }
            }
        }

        let persisted = orchestrator.persisted_sessions()?;
        for (name, session_state) in persisted {
            if config.sessions.iter().any(|s| s.name == name) {
                continue;
            }
            let Some(cwd) = session_state.cwd else {
                warn!(name, "skipping persisted session with missing cwd");
                continue;
            };
            let charter = session_state.charter.unwrap_or_default();
            match orchestrator.create_session(&name, cwd, &charter, vec![]).await {
                Ok(info) => infos.push(info),
                Err(e) => {
                    error!(name, error = %e, "failed to resume state session");
                }
            }
        }

        Ok((orchestrator, infos, config.mcp_server))
    }

    /// Load sessions from a config file and add them to an existing orchestrator.
    /// Skips sessions that are already registered.
    pub async fn load_config(&self, config_path: PathBuf) -> anyhow::Result<Vec<SessionInfo>> {
        let config = crate::config::Config::from_file(config_path)?;
        let mut infos = Vec::new();
        for session in &config.sessions {
            let exists = self.list_sessions()?.iter().any(|s| s.name == session.name);
            if exists {
                warn!(name = %session.name, "session already exists, skipping");
                continue;
            }
            match self.create_session(
                &session.name,
                session.cwd.clone(),
                &session.charter,
                session.allowed_tool_kinds.clone(),
            ).await {
                Ok(info) => infos.push(info),
                Err(e) => {
                    error!(name = %session.name, error = %e, "failed to start session from config");
                }
            }
        }
        Ok(infos)
    }

    /// Spawn or resume a session and register it.
    pub async fn create_session(
        &self,
        name: &str,
        cwd: PathBuf,
        charter: &str,
        allowed_tool_kinds: Vec<ToolKind>,
    ) -> anyhow::Result<SessionInfo> {
        // Try to resume an existing session if we have a persisted session ID.
        let existing_id = {
            let state = self
                .state
                .read()
                .map_err(|_| anyhow::anyhow!("session state poisoned"))?;
            state.sessions.get(name).map(|s| s.session_id.clone())
        };

        if let Some(session_id) = existing_id {
            info!(name, session_id, "attempting to resume persisted session");
            match crate::acp_client::spawn_session(
                &self.kimi_binary,
                name,
                cwd.clone(),
                None,
                allowed_tool_kinds.clone(),
                Some(session_id.clone()),
            )
            .await
            {
                Ok(handle) => {
                    let info = SessionInfo::from(&handle);
                    {
                        let mut sessions = self
                            .sessions
                            .write()
                            .map_err(|_| anyhow::anyhow!("session registry poisoned"))?;
                        sessions.insert(name.to_string(), handle);
                    }
                    {
                        let mut state = self
                            .state
                            .write()
                            .map_err(|_| anyhow::anyhow!("session state poisoned"))?;
                        state.insert(
                            &self.state_path,
                            name,
                            crate::state::SessionState {
                                session_id: info.session_id.clone(),
                                cwd: Some(cwd.clone()),
                                charter: Some(charter.to_string()),
                            },
                        );
                    }
                    info!(name, session_id = %info.session_id, "resumed session registered");
                    return Ok(info);
                }
                Err(e) => {
                    warn!(name, error = %e, "resume failed, clearing stale state and creating new session");
                    {
                        let mut state = self
                            .state
                            .write()
                            .map_err(|_| anyhow::anyhow!("session state poisoned"))?;
                        state.remove(&self.state_path, name);
                    }
                }
            }
        }

        // Create a brand-new session.
        let handle = crate::acp_client::spawn_session(
            &self.kimi_binary,
            name,
            cwd.clone(),
            Some(charter),
            allowed_tool_kinds.clone(),
            None,
        )
        .await?;
        let info = SessionInfo::from(&handle);

        {
            let mut sessions = self
                .sessions
                .write()
                .map_err(|_| anyhow::anyhow!("session registry poisoned"))?;
            sessions.insert(name.to_string(), handle);
        }

        {
            let mut state = self
                .state
                .write()
                .map_err(|_| anyhow::anyhow!("session state poisoned"))?;
            state.insert(
                &self.state_path,
                name,
                crate::state::SessionState {
                    session_id: info.session_id.clone(),
                    cwd: Some(cwd),
                    charter: Some(charter.to_string()),
                },
            );
        }

        info!(name, session_id = %info.session_id, "session registered");
        Ok(info)
    }

    /// List all registered sessions.
    pub fn list_sessions(&self) -> anyhow::Result<Vec<SessionInfo>> {
        let sessions = self
            .sessions
            .read()
            .map_err(|_| anyhow::anyhow!("session registry poisoned"))?;
        Ok(sessions.values().map(SessionInfo::from).collect())
    }

    /// Return all persisted sessions from state.
    pub fn persisted_sessions(&self) -> anyhow::Result<Vec<(String, crate::state::SessionState)>> {
        let state = self
            .state
            .read()
            .map_err(|_| anyhow::anyhow!("session state poisoned"))?;
        Ok(state.sessions.clone().into_iter().collect())
    }

    /// Get a specific session by name or session id.
    pub fn get_session(&self, name_or_id: &str) -> anyhow::Result<Option<SessionInfo>> {
        let sessions = self
            .sessions
            .read()
            .map_err(|_| anyhow::anyhow!("session registry poisoned"))?;
        Ok(sessions
            .get(name_or_id)
            .map(SessionInfo::from)
            .or_else(|| {
                sessions
                    .values()
                    .find(|h| h.session_id == name_or_id)
                    .map(SessionInfo::from)
            }))
    }

    /// Send a message to a session by name or session id and await the response.
    pub async fn send_message(&self, target: &str, content: &str) -> anyhow::Result<PromptResult> {
        let handle = {
            let sessions = self
                .sessions
                .read()
                .map_err(|_| anyhow::anyhow!("session registry poisoned"))?;

            sessions
                .get(target)
                .cloned()
                .or_else(|| sessions.values().find(|h| h.session_id == target).cloned())
                .ok_or_else(|| anyhow::anyhow!("session '{}' not found", target))?
        };

        send_prompt(&handle, content).await
    }

    /// Send a message asynchronously and return a prompt id that can be polled later.
    pub async fn send_message_async(&self, target: &str, content: &str) -> String {
        let prompt_id = self
            .next_prompt_id
            .fetch_add(1, Ordering::SeqCst)
            .to_string();
        let job = Arc::new(TokioMutex::new(PromptJob {
            target: target.to_string(),
            content: content.to_string(),
            status: PromptStatus::Pending,
            result: None,
            error: None,
        }));

        {
            let mut pending = self.pending.write().await;
            pending.insert(prompt_id.clone(), job.clone());
        }

        let self_clone = self.clone();
        let target = target.to_string();
        let content = content.to_string();
        let prompt_id_for_cleanup = prompt_id.clone();
        tokio::spawn(async move {
            let result = self_clone.send_message(&target, &content).await;
            let pending = self_clone.pending.write().await;
            if let Some(job) = pending.get(&prompt_id_for_cleanup) {
                let mut j = job.lock().await;
                match result {
                    Ok(r) => {
                        j.status = PromptStatus::Completed;
                        j.result = Some(r);
                    }
                    Err(e) => {
                        j.status = PromptStatus::Error;
                        j.error = Some(e.to_string());
                    }
                }
            }
        });

        prompt_id
    }

    /// Send a message and block until it completes or the timeout expires.
    /// On timeout returns `PromptOutcome::Timeout` with the prompt id for later polling.
    pub async fn send_message_with_timeout(
        &self,
        target: &str,
        content: &str,
        timeout_ms: u64,
    ) -> anyhow::Result<PromptOutcome> {
        let prompt_id = self.send_message_async(target, content).await;
        let job = {
            let pending = self.pending.read().await;
            pending
                .get(&prompt_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("prompt {} disappeared", prompt_id))?
        };

        let poll = async {
            loop {
                {
                    let j = job.lock().await;
                    if j.status != PromptStatus::Pending {
                        break;
                    }
                }
                sleep(Duration::from_millis(100)).await;
            }
        };

        match timeout(Duration::from_millis(timeout_ms), poll).await {
            Ok(()) => {
                let j = job.lock().await;
                match j.status {
                    PromptStatus::Completed => Ok(PromptOutcome::Completed(
                        j.result.clone().expect("completed prompt has result"),
                    )),
                    PromptStatus::Error => Err(anyhow::anyhow!(
                        j.error
                            .clone()
                            .unwrap_or_else(|| "unknown prompt error".to_string())
                    )),
                    PromptStatus::Pending => unreachable!(),
                }
            }
            Err(_) => Ok(PromptOutcome::Timeout { prompt_id }),
        }
    }

    /// Retrieve the result of an asynchronous prompt. Pending jobs are kept so they
    /// can be polled repeatedly; completed or errored jobs are removed once fetched.
    pub async fn get_prompt_result(&self, prompt_id: &str) -> anyhow::Result<Option<PromptJob>> {
        let pending = self.pending.read().await;
        let job = if let Some(job) = pending.get(prompt_id) {
            let j = job.lock().await;
            if j.status != PromptStatus::Pending {
                drop(j);
                drop(pending);
                let mut pending = self.pending.write().await;
                if let Some(job) = pending.remove(prompt_id) {
                    let j = job.lock().await;
                    Ok(Some(j.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(Some(j.clone()))
            }
        } else {
            Ok(None)
        };
        job
    }

    /// Shut down all registered sessions.
    pub async fn shutdown(&self) {
        let handles: Vec<SessionHandle> = {
            let Ok(sessions) = self.sessions.write() else {
                error!("session registry poisoned during shutdown");
                return;
            };
            sessions.values().cloned().collect()
        };

        for handle in handles {
            shutdown_session(&handle).await;
        }
    }
}
