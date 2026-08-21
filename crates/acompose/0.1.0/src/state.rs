use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, error};

/// Persistent state for a single session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: String,
    #[serde(default)]
    pub cwd: Option<PathBuf>,
    #[serde(default)]
    pub charter: Option<String>,
}

/// Persistent orchestrator state, stored next to the config file.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub sessions: HashMap<String, SessionState>,
}

impl State {
    /// Load state from disk. Returns an empty state if the file does not exist.
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        if !path.exists() {
            debug!(path = %path.display(), "no state file found, starting empty");
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read state file {:?}: {}", path, e))?;
        let state: State = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("failed to parse state file {:?}: {}", path, e))?;
        debug!(path = %path.display(), sessions = state.sessions.len(), "loaded state");
        Ok(state)
    }

    /// Save state to disk.
    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("failed to serialize state: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| anyhow::anyhow!("failed to write state file {:?}: {}", path, e))?;
        debug!(path = %path.display(), "saved state");
        Ok(())
    }

    /// Insert or update a session mapping and persist.
    pub fn insert(&mut self, path: &PathBuf, name: &str, session_state: SessionState) {
        self.sessions.insert(name.to_string(), session_state);
        if let Err(e) = self.save(path) {
            error!(error = %e, "failed to persist state after insert");
        }
    }

    /// Remove a session mapping and persist.
    pub fn remove(&mut self, path: &PathBuf, name: &str) {
        self.sessions.remove(name);
        if let Err(e) = self.save(path) {
            error!(error = %e, "failed to persist state after remove");
        }
    }
}
