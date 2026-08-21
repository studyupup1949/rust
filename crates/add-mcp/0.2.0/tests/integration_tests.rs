use add_mcp::types::{McpServerConfig, PackageManager, Source, Transport};
use add_mcp::{Agent, Scope};
use std::fs;
use tempfile::TempDir;

fn make_command_config(name: &str, command: &str, args: &[&str]) -> McpServerConfig {
    McpServerConfig {
        name: name.to_string(),
        source: Source::Command {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        },
        env: vec![],
        headers: vec![],
    }
}

fn make_url_config(name: &str, url: &str, transport: Transport) -> McpServerConfig {
    McpServerConfig {
        name: name.to_string(),
        source: Source::Url {
            url: url.to_string(),
            transport,
        },
        env: vec![],
        headers: vec![],
    }
}

#[test]
fn install_command_claude_code_creates_config() {
    let tmp = TempDir::new().unwrap();
    let config = make_command_config("test-server", "/usr/bin/test-server", &["--flag"]);
    let results =
        add_mcp::install_with_home(&config, &[Agent::ClaudeCode], Scope::Global, tmp.path());

    assert_eq!(results.len(), 1);
    let r = results[0].as_ref().unwrap();
    assert_eq!(r.agent, Agent::ClaudeCode);
    assert!(r.created);
    assert!(!r.already_existed);

    let config_path = tmp.path().join(".claude.json");
    assert!(config_path.exists());

    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(
        content["mcpServers"]["test-server"]["command"],
        "/usr/bin/test-server"
    );
    assert_eq!(content["mcpServers"]["test-server"]["args"][0], "--flag");
}

#[test]
fn install_command_cursor_creates_config() {
    let tmp = TempDir::new().unwrap();
    let config = make_command_config("my-mcp", "/path/to/my-mcp", &[]);
    let results = add_mcp::install_with_home(&config, &[Agent::Cursor], Scope::Global, tmp.path());

    assert_eq!(results.len(), 1);
    let r = results[0].as_ref().unwrap();
    assert!(r.created);

    let config_path = tmp.path().join(".cursor/mcp.json");
    assert!(config_path.exists());

    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(
        content["mcpServers"]["my-mcp"]["command"],
        "/path/to/my-mcp"
    );
}

#[test]
fn install_url_vscode() {
    let tmp = TempDir::new().unwrap();
    let config = make_url_config("remote-server", "https://example.com/mcp", Transport::Sse);
    let results = add_mcp::install_with_home(&config, &[Agent::VsCode], Scope::Global, tmp.path());

    assert_eq!(results.len(), 1);
    let r = results[0].as_ref().unwrap();
    assert!(r.created);

    let config_path = tmp.path().join(".config/Code/User/mcp.json");
    assert!(config_path.exists());

    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(
        content["servers"]["remote-server"]["url"],
        "https://example.com/mcp"
    );
    assert_eq!(content["servers"]["remote-server"]["type"], "sse");
}

#[test]
fn install_preserves_existing_config() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join(".claude.json");
    fs::write(
        &config_path,
        r#"{"mcpServers":{"existing":{"command":"old"}},"otherKey":"preserved"}"#,
    )
    .unwrap();

    let config = make_command_config("new-server", "/usr/bin/new", &[]);
    let results =
        add_mcp::install_with_home(&config, &[Agent::ClaudeCode], Scope::Global, tmp.path());

    let r = results[0].as_ref().unwrap();
    assert!(!r.created);
    assert!(!r.already_existed);

    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    // Existing server preserved
    assert_eq!(content["mcpServers"]["existing"]["command"], "old");
    // New server added
    assert_eq!(
        content["mcpServers"]["new-server"]["command"],
        "/usr/bin/new"
    );
    // Other keys preserved
    assert_eq!(content["otherKey"], "preserved");
}

#[test]
fn install_overwrites_existing_server() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join(".claude.json");
    fs::write(
        &config_path,
        r#"{"mcpServers":{"my-server":{"command":"old-binary"}}}"#,
    )
    .unwrap();

    let config = make_command_config("my-server", "/usr/bin/new-binary", &[]);
    let results =
        add_mcp::install_with_home(&config, &[Agent::ClaudeCode], Scope::Global, tmp.path());

    let r = results[0].as_ref().unwrap();
    assert!(r.already_existed);

    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(
        content["mcpServers"]["my-server"]["command"],
        "/usr/bin/new-binary"
    );
}

#[test]
fn install_goose_yaml() {
    let tmp = TempDir::new().unwrap();
    let config = make_command_config("goose-mcp", "/usr/bin/goose-mcp", &[]);
    let results = add_mcp::install_with_home(&config, &[Agent::Goose], Scope::Global, tmp.path());

    assert_eq!(results.len(), 1);
    let r = results[0].as_ref().unwrap();
    assert!(r.created);

    let config_path = tmp.path().join(".config/goose/config.yaml");
    assert!(config_path.exists());

    let content: serde_yaml::Value =
        serde_yaml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();

    let server = &content["extensions"]["goose-mcp"];
    assert_eq!(server["cmd"].as_str().unwrap(), "/usr/bin/goose-mcp");
    assert_eq!(server["type"].as_str().unwrap(), "stdio");
    assert!(server["enabled"].as_bool().unwrap());
}

#[test]
fn install_zed_json() {
    let tmp = TempDir::new().unwrap();
    let config = make_command_config("zed-mcp", "/usr/bin/zed-mcp", &["serve"]);
    let results = add_mcp::install_with_home(&config, &[Agent::Zed], Scope::Global, tmp.path());

    assert_eq!(results.len(), 1);
    let r = results[0].as_ref().unwrap();
    assert!(r.created);

    let config_path = tmp.path().join(".config/zed/settings.json");
    assert!(config_path.exists());

    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(content["context_servers"]["zed-mcp"]["source"], "custom");
    assert_eq!(
        content["context_servers"]["zed-mcp"]["command"]["path"],
        "/usr/bin/zed-mcp"
    );
    assert_eq!(
        content["context_servers"]["zed-mcp"]["command"]["args"][0],
        "serve"
    );
}

#[test]
fn install_codex_toml() {
    let tmp = TempDir::new().unwrap();
    let config = make_command_config("codex-mcp", "/usr/bin/codex-mcp", &[]);
    let results = add_mcp::install_with_home(&config, &[Agent::Codex], Scope::Global, tmp.path());

    assert_eq!(results.len(), 1);
    let r = results[0].as_ref().unwrap();
    assert!(r.created);

    let config_path = tmp.path().join(".codex/config.toml");
    assert!(config_path.exists());

    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: toml::Value = toml::from_str(&content).unwrap();
    assert_eq!(
        parsed["mcp_servers"]["codex-mcp"]["command"]
            .as_str()
            .unwrap(),
        "/usr/bin/codex-mcp"
    );
}

#[test]
fn install_multiple_agents() {
    let tmp = TempDir::new().unwrap();
    let config = make_command_config("multi", "/usr/bin/multi", &[]);
    let results = add_mcp::install_with_home(
        &config,
        &[Agent::ClaudeCode, Agent::Cursor, Agent::VsCode],
        Scope::Global,
        tmp.path(),
    );

    assert_eq!(results.len(), 3);
    for r in &results {
        assert!(r.is_ok());
    }
}

#[test]
fn detect_finds_installed_agents() {
    let tmp = TempDir::new().unwrap();

    // Create a Claude Code config
    let config_path = tmp.path().join(".claude.json");
    fs::write(
        &config_path,
        r#"{"mcpServers":{"test":{"command":"test"}}}"#,
    )
    .unwrap();

    let detected = add_mcp::detect_agents_with_home(false, tmp.path());
    let claude_code = detected.iter().find(|d| d.agent == Agent::ClaudeCode);
    assert!(claude_code.is_some());
    let cc = claude_code.unwrap();
    assert!(cc.has_servers);
    assert_eq!(cc.scope, Scope::Global);
}

#[test]
fn install_copilot_has_tools() {
    let tmp = TempDir::new().unwrap();
    let config = make_command_config("copilot-mcp", "/usr/bin/copilot-mcp", &[]);
    let results =
        add_mcp::install_with_home(&config, &[Agent::GithubCopilot], Scope::Global, tmp.path());

    assert_eq!(results.len(), 1);
    let r = results[0].as_ref().unwrap();
    assert!(r.created);

    let config_path = tmp.path().join(".copilot/mcp-config.json");
    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(content["mcpServers"]["copilot-mcp"]["tools"][0], "*");
}

#[test]
fn install_opencode_command_array() {
    let tmp = TempDir::new().unwrap();
    let config = make_command_config("oc-mcp", "/usr/bin/oc-mcp", &["--verbose"]);
    let results =
        add_mcp::install_with_home(&config, &[Agent::OpenCode], Scope::Global, tmp.path());

    assert_eq!(results.len(), 1);
    let config_path = tmp.path().join(".config/opencode/opencode.json");
    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(content["mcp"]["oc-mcp"]["command"][0], "/usr/bin/oc-mcp");
    assert_eq!(content["mcp"]["oc-mcp"]["command"][1], "--verbose");
    assert_eq!(content["mcp"]["oc-mcp"]["type"], "stdio");
}

#[test]
fn install_with_env_vars() {
    let tmp = TempDir::new().unwrap();
    let config = McpServerConfig {
        name: "env-test".to_string(),
        source: Source::Command {
            command: "/usr/bin/env-test".to_string(),
            args: vec![],
        },
        env: vec![("API_KEY".to_string(), "secret123".to_string())],
        headers: vec![],
    };
    let results =
        add_mcp::install_with_home(&config, &[Agent::ClaudeCode], Scope::Global, tmp.path());

    let config_path = tmp.path().join(".claude.json");
    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(
        content["mcpServers"]["env-test"]["env"]["API_KEY"],
        "secret123"
    );
    assert!(results[0].is_ok());
}

#[test]
fn install_npm_package() {
    let tmp = TempDir::new().unwrap();
    let config = McpServerConfig {
        name: "my-npm-mcp".to_string(),
        source: Source::Package {
            manager: PackageManager::Npm,
            package: "@org/my-npm-mcp".to_string(),
        },
        env: vec![],
        headers: vec![],
    };
    let results =
        add_mcp::install_with_home(&config, &[Agent::ClaudeCode], Scope::Global, tmp.path());

    let config_path = tmp.path().join(".claude.json");
    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(content["mcpServers"]["my-npm-mcp"]["command"], "npx");
    assert_eq!(content["mcpServers"]["my-npm-mcp"]["args"][0], "-y");
    assert_eq!(
        content["mcpServers"]["my-npm-mcp"]["args"][1],
        "@org/my-npm-mcp"
    );
    assert!(results[0].is_ok());
}

#[test]
fn install_pip_package() {
    let tmp = TempDir::new().unwrap();
    let config = McpServerConfig {
        name: "mcp-server-fetch".to_string(),
        source: Source::Package {
            manager: PackageManager::Pip,
            package: "mcp-server-fetch".to_string(),
        },
        env: vec![],
        headers: vec![],
    };
    let results =
        add_mcp::install_with_home(&config, &[Agent::ClaudeCode], Scope::Global, tmp.path());

    let config_path = tmp.path().join(".claude.json");
    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(content["mcpServers"]["mcp-server-fetch"]["command"], "uvx");
    assert_eq!(
        content["mcpServers"]["mcp-server-fetch"]["args"][0],
        "mcp-server-fetch"
    );
    assert!(results[0].is_ok());
}

#[test]
fn install_go_package() {
    let tmp = TempDir::new().unwrap();
    let config = McpServerConfig {
        name: "mcp-server".to_string(),
        source: Source::Package {
            manager: PackageManager::Go,
            package: "github.com/user/mcp-server".to_string(),
        },
        env: vec![],
        headers: vec![],
    };
    let results =
        add_mcp::install_with_home(&config, &[Agent::ClaudeCode], Scope::Global, tmp.path());

    let config_path = tmp.path().join(".claude.json");
    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(content["mcpServers"]["mcp-server"]["command"], "go");
    assert_eq!(content["mcpServers"]["mcp-server"]["args"][0], "run");
    assert_eq!(
        content["mcpServers"]["mcp-server"]["args"][1],
        "github.com/user/mcp-server@latest"
    );
    assert!(results[0].is_ok());
}

#[test]
fn install_pip_package_zed() {
    let tmp = TempDir::new().unwrap();
    let config = McpServerConfig {
        name: "pip-zed".to_string(),
        source: Source::Package {
            manager: PackageManager::Pip,
            package: "mcp-server-fetch".to_string(),
        },
        env: vec![],
        headers: vec![],
    };
    let results = add_mcp::install_with_home(&config, &[Agent::Zed], Scope::Global, tmp.path());

    let config_path = tmp.path().join(".config/zed/settings.json");
    let content: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(content["context_servers"]["pip-zed"]["source"], "custom");
    assert_eq!(
        content["context_servers"]["pip-zed"]["command"]["path"],
        "uvx"
    );
    assert_eq!(
        content["context_servers"]["pip-zed"]["command"]["args"][0],
        "mcp-server-fetch"
    );
    assert!(results[0].is_ok());
}

#[test]
fn install_go_package_goose() {
    let tmp = TempDir::new().unwrap();
    let config = McpServerConfig {
        name: "go-goose".to_string(),
        source: Source::Package {
            manager: PackageManager::Go,
            package: "github.com/user/mcp".to_string(),
        },
        env: vec![],
        headers: vec![],
    };
    let results = add_mcp::install_with_home(&config, &[Agent::Goose], Scope::Global, tmp.path());

    let config_path = tmp.path().join(".config/goose/config.yaml");
    let content: serde_yaml::Value =
        serde_yaml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    let server = &content["extensions"]["go-goose"];
    assert_eq!(server["cmd"].as_str().unwrap(), "go");
    assert_eq!(server["args"][0].as_str().unwrap(), "run");
    assert_eq!(
        server["args"][1].as_str().unwrap(),
        "github.com/user/mcp@latest"
    );
    assert_eq!(server["type"].as_str().unwrap(), "stdio");
    assert!(results[0].is_ok());
}
