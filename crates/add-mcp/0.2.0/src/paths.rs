use crate::error::{AddMcpError, Result};
use crate::types::{Agent, Scope};
use std::path::{Path, PathBuf};

/// Resolve the config file path for a given agent and scope.
pub fn config_path(agent: Agent, scope: Scope) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or(AddMcpError::HomeDirNotFound)?;
    config_path_with_home(agent, scope, &home)
}

/// Resolve config path with an explicit home directory (useful for testing).
pub fn config_path_with_home(agent: Agent, scope: Scope, home: &Path) -> Result<PathBuf> {
    match scope {
        Scope::Global => global_path(agent, home),
        Scope::Local => local_path(agent),
    }
}

fn config_dir(home: &Path) -> PathBuf {
    home.join(".config")
}

fn global_path(agent: Agent, home: &Path) -> Result<PathBuf> {
    let config = config_dir(home);

    let path = match agent {
        Agent::ClaudeCode => home.join(".claude.json"),
        Agent::ClaudeDesktop => {
            #[cfg(target_os = "macos")]
            {
                home.join("Library/Application Support/Claude/claude_desktop_config.json")
            }
            #[cfg(not(target_os = "macos"))]
            {
                config.join("Claude/claude_desktop_config.json")
            }
        }
        Agent::Codex => home.join(".codex/config.toml"),
        Agent::Cursor => home.join(".cursor/mcp.json"),
        Agent::GeminiCli => home.join(".gemini/settings.json"),
        Agent::Goose => config.join("goose/config.yaml"),
        Agent::GithubCopilot => home.join(".copilot/mcp-config.json"),
        Agent::OpenCode => config.join("opencode/opencode.json"),
        Agent::VsCode => config.join("Code/User/mcp.json"),
        Agent::Zed => config.join("zed/settings.json"),
    };

    Ok(path)
}

fn local_path(agent: Agent) -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;

    let path = match agent {
        Agent::ClaudeCode => cwd.join(".mcp.json"),
        Agent::Codex => cwd.join(".codex/config.toml"),
        Agent::Cursor => cwd.join(".cursor/mcp.json"),
        Agent::GeminiCli => cwd.join(".gemini/settings.json"),
        Agent::GithubCopilot => cwd.join(".vscode/mcp.json"),
        Agent::VsCode => cwd.join(".vscode/mcp.json"),
        Agent::OpenCode => cwd.join("opencode.json"),
        Agent::Zed => cwd.join(".zed/settings.json"),
        Agent::ClaudeDesktop | Agent::Goose => {
            return Err(AddMcpError::ConfigPathNotFound {
                agent: agent.to_string(),
                scope: "local".into(),
            });
        }
    };

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_claude_code() {
        let p = config_path(Agent::ClaudeCode, Scope::Global).unwrap();
        assert!(p.to_str().unwrap().ends_with(".claude.json"));
    }

    #[test]
    fn local_no_desktop() {
        let r = config_path(Agent::ClaudeDesktop, Scope::Local);
        assert!(r.is_err());
    }

    #[test]
    fn local_claude_code() {
        let p = config_path(Agent::ClaudeCode, Scope::Local).unwrap();
        assert!(p.to_str().unwrap().ends_with(".mcp.json"));
    }

    #[test]
    fn with_home_override() {
        let home = PathBuf::from("/tmp/fakehome");
        let p = config_path_with_home(Agent::ClaudeCode, Scope::Global, &home).unwrap();
        assert_eq!(p, PathBuf::from("/tmp/fakehome/.claude.json"));
    }
}
