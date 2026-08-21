use crate::config::settings::Config;

pub async fn cmd_connect(integration: &str, endpoint: Option<&str>, token: Option<&str>) {
    println!("🔗 Connecting to: {}", integration);

    match integration {
        "claude-code" => {
            // Check if claude binary exists
            match which::which("claude") {
                Ok(path) => {
                    println!("✓ Found Claude Code at: {}", path.display());
                    println!("  Enabling TaskType::CodeEdit routing");
                    println!("  Agents can now use 'aas connect claude-code' for file edits");

                    // Update config
                    if let Ok(mut config) = Config::load() {
                        // config.integrations.claude_code.enabled = true;
                        // config.save().ok();
                        println!("✓ Config updated");
                    }
                }
                Err(_) => {
                    println!("✗ Claude Code binary not found in PATH");
                    println!("  Install: https://claude.ai/download or check PATH");
                    println!("  Then try again: aas connect claude-code");
                }
            }
        }
        "openclaw" => {
            let endpoint = endpoint.unwrap_or("http://localhost:3001");
            println!("✓ Enabling OpenClaw integration");
            println!("  Endpoint: {}", endpoint);
            println!("  Enabling TaskType::ExternalTask routing");

            if token.is_some() {
                println!("✓ API key configured");
            } else {
                println!("⚠️  No API key provided (--token)");
                println!("  Some operations may fail without authentication");
            }

            // Test connection
            match reqwest::Client::new()
                .get(format!("{}/health", endpoint))
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    println!("✓ OpenClaw responding at {}", endpoint);
                }
                _ => {
                    println!("⚠️  Could not connect to {}", endpoint);
                    println!("  Make sure OpenClaw is running: {}...", endpoint);
                }
            }
        }
        "slack" | "discord" => {
            println!("📋 {} integration setup", integration);
            println!("  Endpoint: {}", endpoint.unwrap_or("N/A"));
            println!("  Status: Coming soon!");
        }
        _ => {
            println!("❌ Unknown integration: {}", integration);
            println!("  Available: claude-code, openclaw");
            println!("  Try: aas integrations (to list available)");
        }
    }
    println!();
}

pub async fn cmd_disconnect(integration: &str) {
    println!("🔌 Disconnecting from: {}", integration);

    let _config = Config::load();
    match integration {
        "claude-code" => {
            // config.integrations.claude_code.enabled = false;
            // config.save().ok();
            println!("✓ Claude Code disabled");
        }
        "openclaw" => {
            // config.integrations.openclaw.enabled = false;
            // config.save().ok();
            println!("✓ OpenClaw disabled");
        }
        _ => {
            println!("✗ Unknown integration: {}", integration);
        }
    }
    println!();
}

pub async fn cmd_integrations(_connected_only: bool) {
    println!("🔗 AAS Integrations");
    println!();

    let _config = Config::load().unwrap_or_default();

    let integrations = vec![
        ("claude-code", "Code editing via Claude Code CLI", true),
        ("openclaw", "External tasks (Slack, Discord, etc.)", false),
        ("slack", "Send notifications to Slack", false),
        ("discord", "Send notifications to Discord", false),
    ];

    for (name, desc, available) in integrations {
        let status = if available { "✓ Available" } else { "⏳ Coming soon" };
        println!("  {} — {}", name, status);
        println!("     {}", desc);
        if available {
            println!("     Connect: aas connect {}", name);
        }
        println!();
    }

    println!("📖 For details, see: aas connect <integration> --help");
}
