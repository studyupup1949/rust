use serde::{Deserialize, Serialize};
use std::fmt;

/// One of the 10 supported AI clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Agent {
    ClaudeCode,
    ClaudeDesktop,
    Codex,
    Cursor,
    GeminiCli,
    Goose,
    GithubCopilot,
    OpenCode,
    VsCode,
    Zed,
}

impl Agent {
    pub const ALL: &[Agent] = &[
        Agent::ClaudeCode,
        Agent::ClaudeDesktop,
        Agent::Codex,
        Agent::Cursor,
        Agent::GeminiCli,
        Agent::Goose,
        Agent::GithubCopilot,
        Agent::OpenCode,
        Agent::VsCode,
        Agent::Zed,
    ];

    pub fn from_str_loose(s: &str) -> Option<Agent> {
        match s.to_lowercase().replace(' ', "-").as_str() {
            "claude-code" | "claudecode" => Some(Agent::ClaudeCode),
            "claude-desktop" | "claudedesktop" => Some(Agent::ClaudeDesktop),
            "codex" => Some(Agent::Codex),
            "cursor" => Some(Agent::Cursor),
            "gemini-cli" | "gemini" | "geminicli" => Some(Agent::GeminiCli),
            "goose" => Some(Agent::Goose),
            "github-copilot" | "copilot" | "githubcopilot" => Some(Agent::GithubCopilot),
            "opencode" | "open-code" => Some(Agent::OpenCode),
            "vscode" | "vs-code" => Some(Agent::VsCode),
            "zed" => Some(Agent::Zed),
            _ => None,
        }
    }
}

impl fmt::Display for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Agent::ClaudeCode => write!(f, "Claude Code"),
            Agent::ClaudeDesktop => write!(f, "Claude Desktop"),
            Agent::Codex => write!(f, "Codex"),
            Agent::Cursor => write!(f, "Cursor"),
            Agent::GeminiCli => write!(f, "Gemini CLI"),
            Agent::Goose => write!(f, "Goose"),
            Agent::GithubCopilot => write!(f, "GitHub Copilot"),
            Agent::OpenCode => write!(f, "OpenCode"),
            Agent::VsCode => write!(f, "VS Code"),
            Agent::Zed => write!(f, "Zed"),
        }
    }
}

/// Global (user-level) or Local (project-level) scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    Global,
    Local,
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scope::Global => write!(f, "global"),
            Scope::Local => write!(f, "local"),
        }
    }
}

/// MCP transport type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    Stdio,
    Http,
    Sse,
}

/// Package manager for installing MCP server packages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Npm,
    Pip,
    Go,
    Cargo,
}

impl PackageManager {
    pub fn from_str_loose(s: &str) -> Option<PackageManager> {
        match s.to_lowercase().as_str() {
            "npm" | "npx" => Some(PackageManager::Npm),
            "pip" | "uv" | "uvx" | "python" => Some(PackageManager::Pip),
            "go" => Some(PackageManager::Go),
            "cargo" | "rust" => Some(PackageManager::Cargo),
            _ => None,
        }
    }
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageManager::Npm => write!(f, "npm"),
            PackageManager::Pip => write!(f, "pip"),
            PackageManager::Go => write!(f, "go"),
            PackageManager::Cargo => write!(f, "cargo"),
        }
    }
}

/// Parsed source of an MCP server.
#[derive(Debug, Clone)]
pub enum Source {
    /// A command (binary path or bare command name) with optional args.
    Command { command: String, args: Vec<String> },
    /// An HTTP/SSE URL endpoint.
    Url { url: String, transport: Transport },
    /// A package from a supported package manager.
    Package {
        manager: PackageManager,
        package: String,
    },
}

/// Configuration for an MCP server entry to be installed.
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    pub name: String,
    pub source: Source,
    pub env: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
}

/// Result of a single installation into one agent config.
#[derive(Debug, Clone, Serialize)]
pub struct InstallResult {
    pub agent: Agent,
    pub scope: Scope,
    pub path: String,
    pub created: bool,
    pub already_existed: bool,
}

/// Config file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
}
