use agent_client_protocol::schema::v1::ToolKind;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Top-level configuration for the acompose compositor.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Path to the `kimi` binary. Defaults to "kimi" (resolved via PATH).
    #[serde(default = "default_kimi_binary")]
    pub kimi_binary: String,

    /// Sessions to spawn and initialize.
    #[serde(default, alias = "session")]
    pub sessions: Vec<SessionConfig>,

    /// MCP server configuration for acompose control plane.
    #[serde(default)]
    pub acompose_control_mcp: AcomposeControlMcpConfig,

    /// Compose server WebSocket server configuration.
    #[serde(default)]
    pub acp_proxy: AcpProxyConfig,

    /// Optional path to the persisted state file. Relative paths are resolved
    /// against the configuration file's directory.
    #[serde(default)]
    pub state_path: Option<PathBuf>,

    /// MCP servers available to agent sessions (referenced by name).
    #[serde(default)]
    pub mcp_servers: Vec<McpServer>,

    #[serde(skip)]
    mcp_server_index: std::collections::HashMap<String, McpServer>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            kimi_binary: default_kimi_binary(),
            sessions: Vec::new(),
            acompose_control_mcp: AcomposeControlMcpConfig::default(),
            acp_proxy: AcpProxyConfig::default(),
            state_path: None,
            mcp_servers: Vec::new(),
            mcp_server_index: std::collections::HashMap::new(),
        }
    }
}

fn default_kimi_binary() -> String {
    "kimi".to_string()
}

/// Configuration for the acompose control-plane MCP server.
#[derive(Debug, Clone, Deserialize)]
pub struct AcomposeControlMcpConfig {
    /// Whether the MCP server should be started.
    #[serde(default = "default_mcp_server_enabled")]
    pub enabled: bool,

    /// Bind address for the HTTP/SSE MCP server.
    #[serde(default = "default_mcp_server_bind_address")]
    pub bind_address: String,
}

impl Default for AcomposeControlMcpConfig {
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

/// Configuration for the Compose server WebSocket server.
#[derive(Debug, Clone, Deserialize)]
pub struct AcpProxyConfig {
    /// Whether the Compose server WebSocket server should be started.
    #[serde(default = "default_acp_proxy_enabled")]
    pub enabled: bool,

    /// Bind address for the WebSocket proxy server.
    #[serde(default = "default_acp_proxy_bind_address")]
    pub bind_address: String,
}

impl Default for AcpProxyConfig {
    fn default() -> Self {
        Self {
            enabled: default_acp_proxy_enabled(),
            bind_address: default_acp_proxy_bind_address(),
        }
    }
}

fn default_acp_proxy_enabled() -> bool {
    true
}

fn default_acp_proxy_bind_address() -> String {
    "127.0.0.1:19095".to_string()
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

    /// MCP servers to expose to this agent session (referenced by name from the global `mcp_servers` list).
    #[serde(default)]
    pub mcp_servers: Vec<String>,

    /// Cron jobs that periodically send prompts to this session.
    #[serde(default, alias = "cron")]
    pub cron_jobs: Vec<CronJobConfig>,
}

/// Configuration for a single cron job attached to a session.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema, serde::Serialize)]
pub struct CronJobConfig {
    /// Stable identifier for the job.
    pub name: String,

    /// 5-field cron expression (minute hour day-of-month month day-of-week).
    /// Either `schedule` or `run_at` must be provided.
    #[serde(default)]
    pub schedule: Option<String>,

    /// Prompt text sent to the session when the job fires.
    pub prompt: String,

    /// IANA timezone name. Defaults to UTC. Used for cron expressions.
    #[serde(default = "default_timezone")]
    pub timezone: String,

    /// What to do if the server was down and missed scheduled runs.
    #[serde(default)]
    pub misfire_policy: MisfirePolicy,

    /// One-shot ISO 8601 timestamp. When set, the prompt fires once at this
    /// exact time and the job stops afterwards. `schedule` is ignored.
    #[serde(default)]
    pub run_at: Option<String>,
}

fn default_timezone() -> String {
    "UTC".to_string()
}

/// Policy for missed runs after a long shutdown.
#[derive(
    Debug, Clone, Copy, Deserialize, schemars::JsonSchema, serde::Serialize, Default, PartialEq, Eq,
)]
#[serde(rename_all = "snake_case")]
pub enum MisfirePolicy {
    /// Skip missed runs and wait for the next scheduled time.
    #[default]
    Skip,
    /// Fire once immediately, then resume the normal schedule.
    FireOnce,
}

/// MCP server transport type.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema, serde::Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum McpServerTransport {
    /// HTTP transport (streamable HTTP or plain HTTP).
    #[default]
    Http,
    /// SSE transport.
    Sse,
    /// Stdio transport (command is executed as a subprocess).
    Stdio,
}

/// MCP server declaration for agent sessions.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema, serde::Serialize)]
pub struct McpServer {
    /// Human-readable name for the MCP server.
    pub name: String,

    /// URL of the MCP server (e.g. `http://localhost:9091/mcp`).
    /// Required for `http` and `sse` transports.
    pub url: String,

    /// Transport type. Defaults to `http`.
    #[serde(default)]
    pub transport: McpServerTransport,

    /// Command to execute for `stdio` transport.
    #[serde(default)]
    pub command: Option<String>,

    /// Arguments for `stdio` transport command.
    #[serde(default)]
    pub args: Vec<String>,
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read config file {}: {}", path.display(), e))?;
        let mut config: Self = toml::from_str(&content).map_err(|e| {
            anyhow::anyhow!("failed to parse config file {}: {}", path.display(), e)
        })?;
        config.mcp_server_index = config
            .mcp_servers
            .iter()
            .map(|s| (s.name.clone(), s.clone()))
            .collect();
        Ok(config)
    }

    /// Resolve MCP server references by name into full `McpServer` definitions.
    /// Supports the special `acompose` reference which points to the
    /// acompose control-plane MCP server.
    ///
    /// When `session_name` is provided, the acompose control-plane URL includes
    /// an `agent` query parameter so the MCP server can identify the caller.
    pub fn resolve_mcp_servers(
        &self,
        refs: &[String],
        session_name: Option<&str>,
    ) -> Vec<McpServer> {
        let mut resolved = Vec::new();
        for ref_name in refs {
            if ref_name == "acompose" {
                let bind = &self.acompose_control_mcp.bind_address;
                let mut url = if bind.starts_with("http://") || bind.starts_with("https://") {
                    format!("{}/mcp", bind.trim_end_matches('/'))
                } else {
                    format!("http://{}/mcp", bind)
                };
                if let Some(name) = session_name {
                    url.push_str(&format!("?agent={}", urlencoding::encode(name)));
                }
                resolved.push(McpServer {
                    name: "acompose".to_string(),
                    url,
                    transport: McpServerTransport::Http,
                    command: None,
                    args: Vec::new(),
                });
            } else if let Some(server) = self.mcp_server_index.get(ref_name) {
                resolved.push(server.clone());
            } else {
                tracing::warn!(mcp_server = %ref_name, "unknown MCP server reference");
            }
        }
        resolved
    }
}
