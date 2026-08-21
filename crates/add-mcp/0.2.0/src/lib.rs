pub mod agent;
pub mod config;
pub mod detect;
pub mod error;
pub mod install;
pub mod paths;
pub mod source;
pub mod transform;
pub mod types;

pub use error::{AddMcpError, Result};
pub use types::{
    Agent, ConfigFormat, InstallResult, McpServerConfig, PackageManager, Scope, Source, Transport,
};

/// Install a command-based MCP server into one or more agents.
///
/// This is the primary convenience function for Rust MCP servers
/// that want to self-install.
///
/// # Example
/// ```no_run
/// use add_mcp::{install_command, Agent, Scope};
///
/// let results = install_command(
///     "my-server",
///     "/usr/local/bin/my-server",
///     &[],
///     &[Agent::ClaudeCode],
///     Scope::Global,
/// );
/// ```
pub fn install_command(
    name: &str,
    command: &str,
    args: &[&str],
    agents: &[Agent],
    scope: Scope,
) -> Vec<Result<InstallResult>> {
    let config = McpServerConfig {
        name: name.to_string(),
        source: Source::Command {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        },
        env: vec![],
        headers: vec![],
    };
    install::install(&config, agents, scope)
}

/// Install a URL-based MCP server into one or more agents.
pub fn install_url(
    name: &str,
    url: &str,
    transport: Transport,
    headers: &[(&str, &str)],
    agents: &[Agent],
    scope: Scope,
) -> Vec<Result<InstallResult>> {
    let config = McpServerConfig {
        name: name.to_string(),
        source: Source::Url {
            url: url.to_string(),
            transport,
        },
        env: vec![],
        headers: headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    };
    install::install(&config, agents, scope)
}

/// Install a package-based MCP server into one or more agents.
///
/// Supports npm, pip (via uvx), go (via go run), and cargo packages.
pub fn install_package(
    name: &str,
    package: &str,
    manager: PackageManager,
    agents: &[Agent],
    scope: Scope,
) -> Vec<Result<InstallResult>> {
    let config = McpServerConfig {
        name: name.to_string(),
        source: Source::Package {
            manager,
            package: package.to_string(),
        },
        env: vec![],
        headers: vec![],
    };
    install::install(&config, agents, scope)
}

/// Install from a fully-specified McpServerConfig.
pub fn install(
    config: &McpServerConfig,
    agents: &[Agent],
    scope: Scope,
) -> Vec<Result<InstallResult>> {
    install::install(config, agents, scope)
}

/// Detect which AI clients have config files present.
pub fn detect_agents(include_local: bool) -> Vec<detect::DetectedAgent> {
    detect::detect(include_local)
}

/// Detect with an explicit home directory (for testing).
pub fn detect_agents_with_home(
    include_local: bool,
    home: &std::path::Path,
) -> Vec<detect::DetectedAgent> {
    detect::detect_with_home(include_local, home)
}

/// Install with an explicit home directory (for testing).
pub fn install_with_home(
    config: &McpServerConfig,
    agents: &[Agent],
    scope: Scope,
    home: &std::path::Path,
) -> Vec<Result<InstallResult>> {
    install::install_with_home(config, agents, scope, home)
}

/// List all supported agents.
pub fn list_agents() -> &'static [Agent] {
    Agent::ALL
}
