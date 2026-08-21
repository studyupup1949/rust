use crate::agent::agent_def;
use crate::paths::{config_path, config_path_with_home};
use crate::types::{Agent, Scope};
use std::path::Path;

/// Info about a detected agent installation.
#[derive(Debug, Clone)]
pub struct DetectedAgent {
    pub agent: Agent,
    pub scope: Scope,
    pub path: String,
    pub has_servers: bool,
}

/// Detect which AI clients have config files present.
pub fn detect(include_local: bool) -> Vec<DetectedAgent> {
    detect_inner(include_local, None)
}

/// Detect with an explicit home directory (for testing).
pub fn detect_with_home(include_local: bool, home: &Path) -> Vec<DetectedAgent> {
    detect_inner(include_local, Some(home))
}

fn detect_inner(include_local: bool, home: Option<&Path>) -> Vec<DetectedAgent> {
    let mut found = Vec::new();

    for &agent in Agent::ALL {
        // Check global
        let global_path = match home {
            Some(h) => config_path_with_home(agent, Scope::Global, h),
            None => config_path(agent, Scope::Global),
        };
        if let Ok(path) = global_path {
            if path.exists() {
                let has_servers = check_has_servers(&path, agent);
                found.push(DetectedAgent {
                    agent,
                    scope: Scope::Global,
                    path: path.display().to_string(),
                    has_servers,
                });
            }
        }

        // Check local
        if include_local {
            let def = agent_def(agent);
            if def.has_local {
                if let Ok(path) = config_path(agent, Scope::Local) {
                    if path.exists() {
                        let has_servers = check_has_servers(&path, agent);
                        found.push(DetectedAgent {
                            agent,
                            scope: Scope::Local,
                            path: path.display().to_string(),
                            has_servers,
                        });
                    }
                }
            }
        }
    }

    found
}

fn check_has_servers(path: &std::path::Path, agent: Agent) -> bool {
    let def = agent_def(agent);
    let config = crate::config::read_config(path, def.format);
    match config {
        Ok(val) => val
            .as_object()
            .and_then(|o| o.get(def.section_key))
            .and_then(|s| s.as_object())
            .is_some_and(|m| !m.is_empty()),
        Err(_) => false,
    }
}
