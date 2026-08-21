use crate::source::resolve_package;
use crate::types::{Agent, McpServerConfig, Source, Transport};
use serde_json::{json, Value};

/// Transform an `McpServerConfig` into the JSON shape expected by a specific agent.
///
/// Package sources are resolved to commands first, so agent transforms
/// only deal with Command and Url variants.
pub fn transform(agent: Agent, config: &McpServerConfig) -> Value {
    // Resolve Package → Command before dispatching to agent-specific transforms
    let resolved_config;
    let config = if matches!(config.source, Source::Package { .. }) {
        resolved_config = McpServerConfig {
            source: resolve_package(&config.source),
            ..config.clone()
        };
        &resolved_config
    } else {
        config
    };

    match agent {
        Agent::ClaudeCode | Agent::ClaudeDesktop | Agent::Cursor | Agent::GeminiCli => {
            standard_stdio_or_url(config)
        }
        Agent::Codex => codex_transform(config),
        Agent::Goose => goose_transform(config),
        Agent::GithubCopilot => copilot_transform(config),
        Agent::OpenCode => opencode_transform(config),
        Agent::VsCode => vscode_transform(config),
        Agent::Zed => zed_transform(config),
    }
}

/// Standard shape: { "command": "...", "args": [...], "env": {...} }
/// or { "url": "...", "transport": "..." } for URL sources.
fn standard_stdio_or_url(config: &McpServerConfig) -> Value {
    match &config.source {
        Source::Command { command, args } => {
            let mut obj = json!({
                "command": command,
                "args": args,
            });
            add_env(&mut obj, &config.env);
            obj
        }
        Source::Url { url, transport } => {
            let mut obj = json!({
                "url": url,
                "transport": transport_str(*transport),
            });
            add_headers(&mut obj, &config.headers);
            obj
        }
        Source::Package { .. } => unreachable!("Package should be resolved before transform"),
    }
}

fn codex_transform(config: &McpServerConfig) -> Value {
    match &config.source {
        Source::Command { command, args } => {
            let mut obj = json!({
                "command": command,
                "args": args,
            });
            add_env(&mut obj, &config.env);
            obj
        }
        Source::Url { url, transport } => {
            json!({
                "url": url,
                "transport": transport_str(*transport),
            })
        }
        Source::Package { .. } => unreachable!("Package should be resolved before transform"),
    }
}

fn goose_transform(config: &McpServerConfig) -> Value {
    match &config.source {
        Source::Command { command, args } => {
            let mut obj = json!({
                "type": "stdio",
                "cmd": command,
                "args": args,
                "enabled": true,
            });
            add_env_as_map(&mut obj, &config.env);
            obj
        }
        Source::Url { url, .. } => {
            json!({
                "type": "sse",
                "uri": url,
                "enabled": true,
            })
        }
        Source::Package { .. } => unreachable!("Package should be resolved before transform"),
    }
}

fn copilot_transform(config: &McpServerConfig) -> Value {
    match &config.source {
        Source::Command { command, args } => {
            let mut obj = json!({
                "command": command,
                "args": args,
                "tools": ["*"],
            });
            add_env(&mut obj, &config.env);
            obj
        }
        Source::Url { url, transport } => {
            json!({
                "url": url,
                "transport": transport_str(*transport),
                "tools": ["*"],
            })
        }
        Source::Package { .. } => unreachable!("Package should be resolved before transform"),
    }
}

fn opencode_transform(config: &McpServerConfig) -> Value {
    match &config.source {
        Source::Command { command, args } => {
            let mut cmd_array = vec![json!(command)];
            cmd_array.extend(args.iter().map(|a| json!(a)));
            let mut obj = json!({
                "command": cmd_array,
                "type": "stdio",
            });
            add_env(&mut obj, &config.env);
            obj
        }
        Source::Url { url, .. } => {
            json!({
                "url": url,
                "type": "sse",
            })
        }
        Source::Package { .. } => unreachable!("Package should be resolved before transform"),
    }
}

fn vscode_transform(config: &McpServerConfig) -> Value {
    match &config.source {
        Source::Command { command, args } => {
            let mut obj = json!({
                "command": command,
                "args": args,
                "type": "stdio",
            });
            add_env(&mut obj, &config.env);
            obj
        }
        Source::Url { url, .. } => {
            json!({
                "url": url,
                "type": "sse",
            })
        }
        Source::Package { .. } => unreachable!("Package should be resolved before transform"),
    }
}

fn zed_transform(config: &McpServerConfig) -> Value {
    match &config.source {
        Source::Command { command, args } => {
            let mut cmd = json!({
                "path": command,
                "args": args,
            });
            if !config.env.is_empty() {
                let env_map: serde_json::Map<String, Value> = config
                    .env
                    .iter()
                    .map(|(k, v)| (k.clone(), json!(v)))
                    .collect();
                cmd.as_object_mut()
                    .unwrap()
                    .insert("env".to_string(), Value::Object(env_map));
            }
            json!({
                "source": "custom",
                "command": cmd,
            })
        }
        Source::Url { url, .. } => {
            json!({
                "source": "custom",
                "url": url,
            })
        }
        Source::Package { .. } => unreachable!("Package should be resolved before transform"),
    }
}

fn transport_str(t: Transport) -> &'static str {
    match t {
        Transport::Stdio => "stdio",
        Transport::Http => "http",
        Transport::Sse => "sse",
    }
}

fn add_env(obj: &mut Value, env: &[(String, String)]) {
    if !env.is_empty() {
        let env_map: serde_json::Map<String, Value> =
            env.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
        obj.as_object_mut()
            .unwrap()
            .insert("env".to_string(), Value::Object(env_map));
    }
}

fn add_env_as_map(obj: &mut Value, env: &[(String, String)]) {
    if !env.is_empty() {
        let env_map: serde_json::Map<String, Value> =
            env.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
        obj.as_object_mut()
            .unwrap()
            .insert("env".to_string(), Value::Object(env_map));
    }
}

fn add_headers(obj: &mut Value, headers: &[(String, String)]) {
    if !headers.is_empty() {
        let headers_map: serde_json::Map<String, Value> =
            headers.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
        obj.as_object_mut()
            .unwrap()
            .insert("headers".to_string(), Value::Object(headers_map));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PackageManager;

    fn test_config() -> McpServerConfig {
        McpServerConfig {
            name: "test".to_string(),
            source: Source::Command {
                command: "/usr/bin/test-mcp".to_string(),
                args: vec!["--flag".to_string()],
            },
            env: vec![],
            headers: vec![],
        }
    }

    #[test]
    fn standard_command() {
        let v = transform(Agent::ClaudeCode, &test_config());
        assert_eq!(v["command"], "/usr/bin/test-mcp");
        assert_eq!(v["args"][0], "--flag");
    }

    #[test]
    fn goose_shape() {
        let v = transform(Agent::Goose, &test_config());
        assert_eq!(v["type"], "stdio");
        assert_eq!(v["cmd"], "/usr/bin/test-mcp");
        assert_eq!(v["enabled"], true);
    }

    #[test]
    fn zed_shape() {
        let v = transform(Agent::Zed, &test_config());
        assert_eq!(v["source"], "custom");
        assert_eq!(v["command"]["path"], "/usr/bin/test-mcp");
    }

    #[test]
    fn copilot_has_tools() {
        let v = transform(Agent::GithubCopilot, &test_config());
        assert_eq!(v["tools"][0], "*");
    }

    #[test]
    fn opencode_command_array() {
        let v = transform(Agent::OpenCode, &test_config());
        assert_eq!(v["command"][0], "/usr/bin/test-mcp");
        assert_eq!(v["command"][1], "--flag");
        assert_eq!(v["type"], "stdio");
    }

    #[test]
    fn vscode_has_type() {
        let v = transform(Agent::VsCode, &test_config());
        assert_eq!(v["type"], "stdio");
        assert_eq!(v["command"], "/usr/bin/test-mcp");
    }

    #[test]
    fn url_source_sse() {
        let config = McpServerConfig {
            name: "remote".to_string(),
            source: Source::Url {
                url: "https://example.com/mcp".to_string(),
                transport: Transport::Sse,
            },
            env: vec![],
            headers: vec![("Authorization".into(), "Bearer token".into())],
        };
        let v = transform(Agent::ClaudeCode, &config);
        assert_eq!(v["url"], "https://example.com/mcp");
        assert_eq!(v["transport"], "sse");
        assert_eq!(v["headers"]["Authorization"], "Bearer token");
    }

    #[test]
    fn npm_package_resolved() {
        let config = McpServerConfig {
            name: "test-npm".to_string(),
            source: Source::Package {
                manager: PackageManager::Npm,
                package: "@org/mcp-server".to_string(),
            },
            env: vec![],
            headers: vec![],
        };
        let v = transform(Agent::ClaudeCode, &config);
        assert_eq!(v["command"], "npx");
        assert_eq!(v["args"][0], "-y");
        assert_eq!(v["args"][1], "@org/mcp-server");
    }

    #[test]
    fn pip_package_resolved() {
        let config = McpServerConfig {
            name: "test-pip".to_string(),
            source: Source::Package {
                manager: PackageManager::Pip,
                package: "mcp-server-fetch".to_string(),
            },
            env: vec![],
            headers: vec![],
        };
        let v = transform(Agent::ClaudeCode, &config);
        assert_eq!(v["command"], "uvx");
        assert_eq!(v["args"][0], "mcp-server-fetch");
    }

    #[test]
    fn go_package_resolved() {
        let config = McpServerConfig {
            name: "test-go".to_string(),
            source: Source::Package {
                manager: PackageManager::Go,
                package: "github.com/user/mcp".to_string(),
            },
            env: vec![],
            headers: vec![],
        };
        let v = transform(Agent::Goose, &config);
        assert_eq!(v["cmd"], "go");
        assert_eq!(v["args"][0], "run");
        assert_eq!(v["args"][1], "github.com/user/mcp@latest");
        assert_eq!(v["type"], "stdio");
    }
}
