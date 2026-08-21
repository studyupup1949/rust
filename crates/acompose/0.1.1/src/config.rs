use agent_client_protocol::schema::v1::ToolKind;
use serde::Deserialize;
use std::path::PathBuf;

/// Top-level configuration for the acompose orchestrator.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Path to the `kimi` binary. Defaults to "kimi" (resolved via PATH).
    #[serde(default = "default_kimi_binary")]
    pub kimi_binary: String,

    /// Sessions to spawn and initialize.
    #[serde(default, alias = "session")]
    pub sessions: Vec<SessionConfig>,

    /// MCP server configuration.
    #[serde(default)]
    pub mcp_server: McpServerConfig,
}

fn default_kimi_binary() -> String {
    "kimi".to_string()
}

/// Configuration for the integrated MCP server.
#[derive(Debug, Clone, Deserialize)]
pub struct McpServerConfig {
    /// Whether the MCP server should be started.
    #[serde(default = "default_mcp_server_enabled")]
    pub enabled: bool,

    /// Bind address for the HTTP/SSE MCP server.
    #[serde(default = "default_mcp_server_bind_address")]
    pub bind_address: String,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            enabled: default_mcp_server_enabled(),
            bind_address: default_mcp_server_bind_address(),
        }
    }
}

fn default_mcp_server_enabled() -> bool {
    true
}

fn default_mcp_server_bind_address() -> String {
    "127.0.0.1:19094".to_string()
}

/// Configuration for a single persistent agent session.
#[derive(Debug, Clone, Deserialize)]
pub struct SessionConfig {
    /// Human-readable name for logging.
    pub name: String,

    /// Working directory where the session should be created.
    pub cwd: PathBuf,

    /// Initial system message / charter sent to the agent.
    pub charter: String,

    /// Allowed tool kinds for this session. If empty, all tool kinds are allowed.
    /// Tool kinds: read, edit, delete, move, search, execute, think, fetch, switch_mode, other.
    #[serde(default)]
    pub allowed_tool_kinds: Vec<ToolKind>,
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn from_file(path: PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("failed to read config file {:?}: {}", path, e))?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("failed to parse config file {:?}: {}", path, e))?;
        Ok(config)
    }
}
