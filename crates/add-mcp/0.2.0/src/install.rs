use crate::agent::agent_def;
use crate::config::{merge_server, read_config, write_config};
use crate::error::Result;
use crate::paths::{config_path, config_path_with_home};
use crate::transform::transform;
use crate::types::{Agent, InstallResult, McpServerConfig, Scope};
use std::path::Path;

/// Install an MCP server config into one or more agents.
pub fn install(
    config: &McpServerConfig,
    agents: &[Agent],
    scope: Scope,
) -> Vec<Result<InstallResult>> {
    agents
        .iter()
        .map(|&agent| install_one(config, agent, scope, None))
        .collect()
}

/// Install with an explicit home directory (for testing).
pub fn install_with_home(
    config: &McpServerConfig,
    agents: &[Agent],
    scope: Scope,
    home: &Path,
) -> Vec<Result<InstallResult>> {
    agents
        .iter()
        .map(|&agent| install_one(config, agent, scope, Some(home)))
        .collect()
}

fn install_one(
    config: &McpServerConfig,
    agent: Agent,
    scope: Scope,
    home: Option<&Path>,
) -> Result<InstallResult> {
    let def = agent_def(agent);
    let path = match home {
        Some(h) => config_path_with_home(agent, scope, h)?,
        None => config_path(agent, scope)?,
    };

    let created = !path.exists();

    let mut root = read_config(&path, def.format)?;
    let server_value = transform(agent, config);
    let already_existed = merge_server(&mut root, def.section_key, &config.name, server_value);
    write_config(&path, def.format, &root)?;

    Ok(InstallResult {
        agent,
        scope,
        path: path.display().to_string(),
        created,
        already_existed,
    })
}
