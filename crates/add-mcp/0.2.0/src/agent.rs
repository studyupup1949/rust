use crate::types::{Agent, ConfigFormat, Scope};

/// Static definition of how each agent stores its MCP config.
pub struct AgentDef {
    pub agent: Agent,
    /// The top-level key in the config file where servers live.
    pub section_key: &'static str,
    pub format: ConfigFormat,
    pub has_local: bool,
}

impl AgentDef {
    pub fn supports_scope(&self, scope: Scope) -> bool {
        match scope {
            Scope::Global => true,
            Scope::Local => self.has_local,
        }
    }
}

/// Get the agent definition for a given agent.
pub fn agent_def(agent: Agent) -> AgentDef {
    match agent {
        Agent::ClaudeCode => AgentDef {
            agent,
            section_key: "mcpServers",
            format: ConfigFormat::Json,
            has_local: true,
        },
        Agent::ClaudeDesktop => AgentDef {
            agent,
            section_key: "mcpServers",
            format: ConfigFormat::Json,
            has_local: false,
        },
        Agent::Codex => AgentDef {
            agent,
            section_key: "mcp_servers",
            format: ConfigFormat::Toml,
            has_local: true,
        },
        Agent::Cursor => AgentDef {
            agent,
            section_key: "mcpServers",
            format: ConfigFormat::Json,
            has_local: true,
        },
        Agent::GeminiCli => AgentDef {
            agent,
            section_key: "mcpServers",
            format: ConfigFormat::Json,
            has_local: true,
        },
        Agent::Goose => AgentDef {
            agent,
            section_key: "extensions",
            format: ConfigFormat::Yaml,
            has_local: false,
        },
        Agent::GithubCopilot => AgentDef {
            agent,
            section_key: "mcpServers",
            format: ConfigFormat::Json,
            has_local: true,
        },
        Agent::OpenCode => AgentDef {
            agent,
            section_key: "mcp",
            format: ConfigFormat::Json,
            has_local: true,
        },
        Agent::VsCode => AgentDef {
            agent,
            section_key: "servers",
            format: ConfigFormat::Json,
            has_local: true,
        },
        Agent::Zed => AgentDef {
            agent,
            section_key: "context_servers",
            format: ConfigFormat::Json,
            has_local: true,
        },
    }
}
