use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::agent::session_actor::PromptResult;
use crate::config::CronJobConfig;

/// Status of an asynchronous prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptStatus {
    /// Job is queued but has not been sent to the agent yet.
    Queued,
    /// Job has been sent to the agent and is waiting for a response.
    Pending,
    Completed,
    Error,
}

/// Stored result of an asynchronous prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptJob {
    pub target: String,
    pub content: String,
    pub status: PromptStatus,
    /// Optional session name that should receive the result once the prompt
    /// completes. If set, the compositor worker sends a follow-up message
    /// to that session with the outcome of this prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_result_to: Option<String>,
    /// Optional cron job name that produced this prompt. Used to enforce
    /// overlap policies for cron jobs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron_job_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<PromptResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Creation timestamp used to preserve prompt ordering across restarts.
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

/// Persistent state for a single session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    #[serde(default)]
    pub cwd: PathBuf,
    #[serde(default)]
    pub charter: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub allowed_tool_kinds: Vec<agent_client_protocol::schema::v1::ToolKind>,
    #[serde(default)]
    pub mcp_servers: Vec<crate::config::McpServer>,
    #[serde(default)]
    pub cron_jobs: HashMap<String, CronJobState>,
    /// Pending and queued prompt jobs owned by this session. Active jobs are
    /// replayed as a "continue" message after the session is loaded.
    #[serde(default)]
    pub jobs: Vec<PromptJob>,
}

/// Persistent state for a single cron job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobState {
    pub config: CronJobConfig,
    #[serde(default)]
    pub last_run_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub next_run_at: Option<DateTime<Utc>>,
}

/// Persistent compositor state.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub sessions: HashMap<String, SessionState>,
}

/// Message sent to the persistence actor to update or remove a session's state.
#[derive(Debug)]
pub struct PersistSession {
    pub name: String,
    pub state: Option<SessionState>,
}
/// Storage backend for [`State`].
#[async_trait]
pub trait StateStore: Send + Sync {
    /// Load state from the store. Returns an empty state if no state exists.
    async fn load(&self) -> anyhow::Result<State>;

    /// Save state to the store atomically.
    async fn save(&self, state: &State) -> anyhow::Result<()>;
}

/// File-backed [`StateStore`] that persists state as JSON.
#[derive(Debug, Clone)]
pub struct FileStateStore {
    path: PathBuf,
}

impl FileStateStore {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[async_trait]
impl StateStore for FileStateStore {
    async fn load(&self) -> anyhow::Result<State> {
        if !self.path.exists() {
            tracing::debug!(path = %self.path.display(), "no state file found, starting empty");
            return Ok(State::default());
        }

        let content = tokio::fs::read_to_string(&self.path).await.map_err(|e| {
            anyhow::anyhow!("failed to read state file {}: {}", self.path.display(), e)
        })?;
        let state: State = serde_json::from_str(&content).map_err(|e| {
            anyhow::anyhow!("failed to parse state file {}: {}", self.path.display(), e)
        })?;
        tracing::debug!(
            path = %self.path.display(),
            sessions = state.sessions.len(),
            "loaded state"
        );
        Ok(state)
    }

    async fn save(&self, state: &State) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(state)
            .map_err(|e| anyhow::anyhow!("failed to serialize state: {}", e))?;
        let tmp = self.path.with_extension("tmp");
        tokio::fs::write(&tmp, content).await.map_err(|e| {
            anyhow::anyhow!("failed to write temp state file {}: {}", tmp.display(), e)
        })?;
        tokio::fs::rename(&tmp, &self.path).await.map_err(|e| {
            anyhow::anyhow!(
                "failed to rename temp state file {} -> {}: {}",
                tmp.display(),
                self.path.display(),
                e
            )
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = tokio::fs::metadata(&self.path).await.map_err(|e| {
                anyhow::anyhow!("failed to get metadata for {}: {}", self.path.display(), e)
            })?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            tokio::fs::set_permissions(&self.path, permissions)
                .await
                .map_err(|e| {
                    anyhow::anyhow!(
                        "failed to set permissions for {}: {}",
                        self.path.display(),
                        e
                    )
                })?;
        }
        tracing::debug!(path = %self.path.display(), "saved state");
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct MemoryStateStore {
    state: std::sync::Arc<std::sync::Mutex<Option<State>>>,
}

impl MemoryStateStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Load the current state from the store.
    pub async fn load(&self) -> anyhow::Result<State> {
        let guard = self
            .state
            .lock()
            .map_err(|e| anyhow::anyhow!("state store lock poisoned: {}", e))?;
        Ok(guard.clone().unwrap_or_default())
    }

    /// Save the current state to the store.
    pub async fn save(&self, state: &State) -> anyhow::Result<()> {
        let mut guard = self
            .state
            .lock()
            .map_err(|e| anyhow::anyhow!("state store lock poisoned: {}", e))?;
        *guard = Some(state.clone());
        drop(guard);
        Ok(())
    }

    /// Get the current state without going through the async trait.
    #[must_use]
    pub fn current(&self) -> Option<State> {
        let Ok(guard) = self.state.lock() else {
            return None;
        };
        guard.clone()
    }
}

#[async_trait]
impl StateStore for MemoryStateStore {
    async fn load(&self) -> anyhow::Result<State> {
        let guard = self
            .state
            .lock()
            .map_err(|e| anyhow::anyhow!("state store lock poisoned: {}", e))?;
        Ok(guard.clone().unwrap_or_default())
    }

    async fn save(&self, state: &State) -> anyhow::Result<()> {
        let mut guard = self
            .state
            .lock()
            .map_err(|e| anyhow::anyhow!("state store lock poisoned: {}", e))?;
        *guard = Some(state.clone());
        drop(guard);
        Ok(())
    }
}

/// No-op state store that always returns an empty state and ignores saves.
#[derive(Debug, Clone, Default)]
pub struct NoopStateStore;

#[async_trait]
impl StateStore for NoopStateStore {
    async fn load(&self) -> anyhow::Result<State> {
        Ok(State::default())
    }

    async fn save(&self, _state: &State) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::session_actor::PromptResult;
    use agent_client_protocol::schema::v1::StopReason;
    use std::path::PathBuf;

    #[tokio::test]
    async fn pending_jobs_roundtrip_through_state_file() {
        let path = PathBuf::from("/tmp/acompose_state_pending_test.json");
        let _ = tokio::fs::remove_file(&path).await;

        let store = FileStateStore::new(&path);
        let mut state = State::default();
        let mut session = SessionState {
            session_id: "sid".to_string(),
            cwd: PathBuf::new(),
            charter: None,
            model: None,
            allowed_tool_kinds: vec![],
            mcp_servers: vec![],
            cron_jobs: HashMap::new(),
            jobs: Vec::new(),
        };
        session.jobs.push(PromptJob {
            target: "test-session".to_string(),
            content: "hello".to_string(),
            status: PromptStatus::Pending,
            send_result_to: Some("sender".to_string()),
            cron_job_name: None,
            result: None,
            error: None,
            created_at: Utc::now(),
        });
        session.jobs.push(PromptJob {
            target: "test-session".to_string(),
            content: "world".to_string(),
            status: PromptStatus::Completed,
            send_result_to: None,
            cron_job_name: None,
            result: Some(PromptResult {
                stop_reason: StopReason::EndTurn,
                text: "done".to_string(),
            }),
            error: None,
            created_at: Utc::now(),
        });
        state.sessions.insert("test-session".to_string(), session);
        store.save(&state).await.expect("state should save");

        let loaded = store.load().await.expect("state should load");
        let session = loaded
            .sessions
            .get("test-session")
            .expect("session should exist");
        assert_eq!(session.jobs.len(), 2);

        let pending = session
            .jobs
            .iter()
            .find(|j| j.status == PromptStatus::Pending)
            .expect("pending job");
        assert_eq!(pending.content, "hello");
        assert_eq!(pending.send_result_to.as_deref(), Some("sender"));

        let completed = session
            .jobs
            .iter()
            .find(|j| j.status == PromptStatus::Completed)
            .expect("completed job");
        assert!(completed.result.is_some());

        let _ = tokio::fs::remove_file(&path).await;
    }
}
