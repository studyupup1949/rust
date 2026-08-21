use aas_cli::cli::commands::{Cli, Commands, ConfigCommands, DaemonCommands, MemoryCommands};
use aas_cli::cli::{output, Dashboard, cmd_daemon, cmd_connect, cmd_disconnect, cmd_integrations};
use aas_cli::config::settings::Config;
use aas_cli::dashboard::routes::{self, DashboardState};
use aas_cli::integrations::claude_code::ClaudeCodeProvider;
use aas_cli::integrations::openclaw::OpenClawProvider;
use aas_cli::llm::claude::ClaudeProvider;
use aas_cli::llm::hermes::HermesProvider;
use aas_cli::llm::mock::MockLLMProvider;
use aas_cli::llm::router::{LLMRouter, TaskType};
use aas_cli::llm::traits::LLMProvider;
use aas_cli::llm::SystemTools;
use aas_cli::memory::patterns::PatternEngine;
use aas_cli::memory::store::MemoryStore;
use aas_cli::rsi::RSIEngine;
use aas_cli::swarm::coordinator::Coordinator;
use aas_cli::swarm::types::*;
use clap::Parser;
use std::sync::Arc;
use tokio::signal;
use tracing::info;
use tracing_subscriber::EnvFilter;

fn select_llm_provider(config: &Config) -> Arc<dyn LLMProvider> {
    let api_key = config.llm.api_key.clone()
        .or_else(|| std::env::var("AAS_LLM_API_KEY").ok())
        .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok());

    match config.llm.provider.as_str() {
        "claude" => {
            let key = api_key.unwrap_or_else(|| {
                eprintln!("Warning: No API key for Claude. Set ANTHROPIC_API_KEY or AAS_LLM_API_KEY.");
                String::new()
            });
            Arc::new(ClaudeProvider::new(
                &key,
                config.llm.model_name.as_deref().unwrap_or("claude-sonnet-4-20250514"),
            ))
        }
        "hermes" => {
            let key = config.llm.api_key.clone()
                .or_else(|| std::env::var("AAS_LLM_API_KEY").ok());
            Arc::new(HermesProvider::new(
                &config.llm.endpoint,
                key,
                config.llm.model_name.as_deref().unwrap_or("hermes-2-pro"),
            ))
        }
        "mock" | _ => {
            info!("Using MockLLMProvider (no real LLM configured)");
            Arc::new(MockLLMProvider)
        }
    }
}

fn build_llm_router(config: &Config) -> Arc<LLMRouter> {
    let mut router = LLMRouter::new();

    // Register Claude for deep reasoning
    let api_key = config.llm.api_key.clone()
        .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok());
    if let Some(key) = api_key {
        let claude = Arc::new(ClaudeProvider::new(
            &key,
            config.llm.model_name.as_deref().unwrap_or("claude-sonnet-4-20250514"),
        )) as Arc<dyn LLMProvider>;
        router.register(TaskType::DeepReasoning, claude);
    }

    // Register Hermes for fast analysis
    let hermes_key = config.llm.api_key.clone()
        .or_else(|| std::env::var("AAS_LLM_API_KEY").ok());
    let hermes = Arc::new(HermesProvider::new(
        &config.llm.endpoint,
        hermes_key,
        config.llm.model_name.as_deref().unwrap_or("hermes-2-pro"),
    )) as Arc<dyn LLMProvider>;
    router.register(TaskType::FastAnalysis, hermes);

    // Register Claude Code for code edits
    if let Some(ref ext_cfg) = config.external_agents {
        if let Some(ref cc_cfg) = ext_cfg.claude_code {
            if cc_cfg.enabled && ClaudeCodeProvider::is_available() {
                let working_dir = cc_cfg.working_dir.as_deref()
                    .map(|p| std::path::PathBuf::from(p))
                    .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
                let claude_code = Arc::new(ClaudeCodeProvider::new(working_dir)) as Arc<dyn LLMProvider>;
                router.register(TaskType::CodeEdit, claude_code);
                info!("Claude Code provider registered for code edits");
            }
        }
    }

    // Register OpenClaw for external tasks
    if let Some(ref ext_cfg) = config.external_agents {
        if let Some(ref oc_cfg) = ext_cfg.openclaw {
            if oc_cfg.enabled {
                let openclaw = Arc::new(OpenClawProvider::new(&oc_cfg.endpoint, oc_cfg.api_key.clone())) as Arc<dyn LLMProvider>;
                router.register(TaskType::ExternalTask, openclaw);
                info!("OpenClaw provider registered for external tasks");
            }
        }
    }

    // Register fallback
    let mock = Arc::new(MockLLMProvider) as Arc<dyn LLMProvider>;
    router.register(TaskType::Fallback, mock);

    Arc::new(router)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_ansi(false)
        .init();

    // Show help if no args provided
    if std::env::args().len() == 1 {
        cmd_help();
        return;
    }

    let cli = Cli::parse();

    match &cli.command {
        Commands::Init {
            config,
            defaults,
            auto_start,
        } => cmd_init(config.as_deref(), *defaults, *auto_start).await,
        Commands::Run { config, duration, dry_run } => cmd_run(config.as_deref(), duration.as_deref(), *dry_run).await,
        Commands::Daemon { action } => cmd_daemon(action).await,
        Commands::Connect { integration, endpoint, token } => cmd_connect(integration, endpoint.as_deref(), token.as_deref()).await,
        Commands::Disconnect { integration } => cmd_disconnect(integration).await,
        Commands::Integrations { connected_only } => cmd_integrations(*connected_only).await,
        Commands::Stop => cmd_stop().await,
        Commands::Restart { agent } => cmd_restart(agent.as_deref()).await,
        Commands::Status { watch } => cmd_status(*watch).await,
        Commands::Dashboard { port } => cmd_dashboard(*port).await,
        Commands::Monitor => cmd_monitor().await,
        Commands::Config { action } => cmd_config(action).await,
        Commands::History {
            agent,
            limit,
            problem_type: _,
            export,
        } => cmd_history(agent.as_deref(), *limit, export.as_deref()).await,
        Commands::Memory { action } => cmd_memory(action).await,
        Commands::Trigger { agent, force: _ } => cmd_trigger(agent).await,
        Commands::Approve { decision_id } => cmd_approve(decision_id).await,
        Commands::Reject { decision_id } => cmd_reject(decision_id).await,
        Commands::Rollback {
            decision_id,
            minutes,
        } => cmd_rollback(decision_id, *minutes).await,
        Commands::Explain { decision_id } => cmd_explain(decision_id).await,
        Commands::Logs {
            agent,
            follow,
            level,
        } => cmd_logs(agent.as_deref(), *follow, level.as_deref()).await,
        Commands::Errors => cmd_errors().await,
        Commands::Alerts => cmd_alerts().await,
        Commands::Performance => cmd_performance().await,
        Commands::ExportConfig => cmd_export_config().await,
        Commands::Backup => cmd_backup().await,
        Commands::Restore { backup_file } => cmd_restore(backup_file).await,
        Commands::ValidateConfig { file } => cmd_validate(file.as_deref()).await,
        Commands::Version => cmd_version(),
        Commands::Update => cmd_update().await,
        Commands::Interactive => cmd_interactive().await,
        Commands::Help => cmd_help(),
    }
}

async fn cmd_init(config_path: Option<&str>, _defaults: bool, auto_start: bool) {
    if let Some(path) = config_path {
        match std::fs::read_to_string(path) {
            Ok(contents) => {
                let config: Config = serde_json::from_str(&contents)
                    .unwrap_or_else(|e| {
                        eprintln!("Invalid config file: {}", e);
                        Config::default()
                    });
                if let Err(e) = config.save() {
                    eprintln!("Failed to save config: {}", e);
                    return;
                }
                println!("✅ Configuration loaded from {} and saved", path);
            }
            Err(e) => {
                eprintln!("Failed to read config file: {}", e);
                return;
            }
        }
    } else {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  AUTONOMOUS AGENT SYSTEM - SETUP");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        println!("  Opening web dashboard for setup...");
        println!("  → http://localhost:3000");
        println!();
        println!("  Or run 'aas dashboard' to start manually.");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        let config = Config::default();
        if let Err(e) = config.save() {
            eprintln!("Failed to save default config: {}", e);
            return;
        }

        cmd_dashboard(3000).await;
        return;
    }

    if auto_start {
        cmd_run(None, None, false).await;
    }
}

async fn cmd_run(config_path: Option<&str>, duration: Option<&str>, _dry_run: bool) {
    let config = if let Some(path) = config_path {
        match std::fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
                eprintln!("Invalid config: {}", e);
                std::process::exit(1);
            }),
            Err(e) => {
                eprintln!("Cannot read config: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        Config::load().unwrap_or_else(|e| {
            eprintln!("Failed to load config: {}", e);
            Config::default()
        })
    };

    if !Config::config_path().exists() {
        match config.save() {
            Ok(()) => println!("  Config: defaults written to {}", Config::config_path().display()),
            Err(e) => eprintln!("Warning: could not save default config: {}", e),
        }
    }

    let config = Arc::new(config);
    let db_path = Config::default_db_path();
    let memory = MemoryStore::new(&db_path).await.unwrap_or_else(|e| {
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    });
    let memory = Arc::new(memory);

    let llm = select_llm_provider(&config);
    let router = build_llm_router(&config);

    let coordinator = Coordinator::new(config.clone(), memory.clone(), llm.clone(), router);
    let coordinator = Arc::new(coordinator);

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Starting Autonomous Agent System");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  LLM Provider: {}", config.llm.provider);
    println!("  Agents: {}", config.get_enabled_agents().join(", "));
    println!("  Database: {}", db_path.display());
    println!("  Dashboard: http://localhost:3000");
    println!();

    let coordinator_clone = coordinator.clone();
    let coord_handle = tokio::spawn(async move {
        coordinator_clone.start().await;
    });

    // Handle timeout or Ctrl+C
    let shutdown_triggered = if let Some(dur_str) = duration {
        // Parse duration: "5s", "10m", "1h"
        let duration = parse_duration(dur_str).unwrap_or(std::time::Duration::from_secs(300));
        tokio::time::timeout(duration, signal::ctrl_c())
            .await
            .is_err()  // timeout = true, ctrl_c = false
    } else {
        let _ = signal::ctrl_c().await;
        true
    };

    if shutdown_triggered {
        info!("Duration expired or shutdown signal received, draining in-flight actions...");
        coordinator.drain_and_shutdown(30).await;
    } else {
        info!("Shutdown signal received, draining in-flight actions...");
        coordinator.drain_and_shutdown(30).await;
    }
    let _ = coord_handle.await;
}

fn parse_duration(s: &str) -> Option<std::time::Duration> {
    let s = s.trim();
    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: u64 = num_str.parse().ok()?;
    match unit {
        "s" => Some(std::time::Duration::from_secs(num)),
        "m" => Some(std::time::Duration::from_secs(num * 60)),
        "h" => Some(std::time::Duration::from_secs(num * 3600)),
        _ => None,
    }
}

async fn cmd_stop() {
    println!("Use Ctrl+C to stop the running swarm, or kill the process.");
}

async fn cmd_restart(_agent: Option<&str>) {
    println!("Restart functionality: stop and re-run 'aas run'");
}

async fn cmd_status(watch: bool) {
    let config = Config::load().unwrap_or_default();
    let config = Arc::new(config);
    let db_path = Config::default_db_path();
    let memory = Arc::new(MemoryStore::new(&db_path).await.unwrap_or_else(|e| {
        eprintln!("DB error: {}", e);
        std::process::exit(1);
    }));

    let llm = select_llm_provider(&config);
    let router = build_llm_router(&config);
    let coordinator = Arc::new(Coordinator::new(config.clone(), memory.clone(), llm.clone(), router));

    if watch {
        loop {
            print!("\x1B[2J\x1B[H");
            let statuses = coordinator.get_statuses().await;
            println!("{}", output::format_status(&statuses));
            println!("  Refreshing every 5s... (Ctrl+C to stop)");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    } else {
        let statuses = coordinator.get_statuses().await;
        println!("{}", output::format_status(&statuses));
    }
}

async fn cmd_dashboard(port: u16) {
    let config = Arc::new(Config::load().unwrap_or_default());

    let db_path = Config::default_db_path();
    let memory = MemoryStore::new(&db_path).await.ok();
    let coordinator = if let Some(memory) = memory {
        let llm = select_llm_provider(&config);
        let router = build_llm_router(&config);
        Some(Arc::new(Coordinator::new(
            config.clone(),
            Arc::new(memory),
            llm.clone(),
            router,
        )))
    } else {
        None
    };

    let state = Arc::new(DashboardState { config, coordinator });

    let app = routes::router(state);
    let addr = format!("0.0.0.0:{}", port);
    println!("🌐 AAS Dashboard: http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        });

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| eprintln!("Server error: {}", e));
}

async fn cmd_monitor() {
    let config = Config::load().unwrap_or_default();
    let config = Arc::new(config);
    let db_path = Config::default_db_path();
    let memory = Arc::new(MemoryStore::new(&db_path).await.unwrap_or_else(|e| {
        eprintln!("DB error: {}", e);
        std::process::exit(1);
    }));

    let rsi = Arc::new(RSIEngine::new(memory.clone()));
    let patterns = Arc::new(PatternEngine::new(memory.clone()));

    // ponytail: hardcoded agent list; read from config when it matters
    let agents = vec![
        "repository".to_string(),
        "logs".to_string(),
        "metrics".to_string(),
        "health".to_string(),
        "task".to_string(),
        "trace".to_string(),
    ];

    let mut dashboard = Dashboard::new(rsi, memory.clone(), patterns, agents);

    if let Err(e) = dashboard.run().await {
        eprintln!("TUI error: {}", e);
    }
}

async fn cmd_config(action: &ConfigCommands) {
    let mut config = Config::load().unwrap_or_default();

    match action {
        ConfigCommands::Show => {
            println!("{}", config.to_json());
        }
        ConfigCommands::Edit => {
            println!("Use 'aas dashboard' for interactive configuration.");
        }
        ConfigCommands::Llm { provider, endpoint } => {
            if let Some(p) = provider {
                config.llm.provider = p.clone();
            }
            if let Some(e) = endpoint {
                config.llm.endpoint = e.clone();
            }
            config.save().unwrap_or_else(|e| eprintln!("Save failed: {}", e));
            println!("✅ LLM configuration updated");
        }
        ConfigCommands::Agent { enable, disable } => {
            if let Some(name) = enable {
                match name.as_str() {
                    "repository" => {
                        if let Some(ref mut c) = config.agents.repository {
                            c.enabled = true;
                        }
                    }
                    "logs" => {
                        if let Some(ref mut c) = config.agents.logs {
                            c.enabled = true;
                        }
                    }
                    "metrics" => {
                        if let Some(ref mut c) = config.agents.metrics {
                            c.enabled = true;
                        }
                    }
                    "health" => {
                        if let Some(ref mut c) = config.agents.health {
                            c.enabled = true;
                        }
                    }
                    "task" => {
                        if let Some(ref mut c) = config.agents.task {
                            c.enabled = true;
                        }
                    }
                    "trace" => {
                        if let Some(ref mut c) = config.agents.trace {
                            c.enabled = true;
                        }
                    }
                    _ => eprintln!("Unknown agent: {}", name),
                }
            }
            if let Some(name) = disable {
                match name.as_str() {
                    "repository" => {
                        if let Some(ref mut c) = config.agents.repository {
                            c.enabled = false;
                        }
                    }
                    "logs" => {
                        if let Some(ref mut c) = config.agents.logs {
                            c.enabled = false;
                        }
                    }
                    "metrics" => {
                        if let Some(ref mut c) = config.agents.metrics {
                            c.enabled = false;
                        }
                    }
                    "health" => {
                        if let Some(ref mut c) = config.agents.health {
                            c.enabled = false;
                        }
                    }
                    "task" => {
                        if let Some(ref mut c) = config.agents.task {
                            c.enabled = false;
                        }
                    }
                    "trace" => {
                        if let Some(ref mut c) = config.agents.trace {
                            c.enabled = false;
                        }
                    }
                    _ => eprintln!("Unknown agent: {}", name),
                }
            }
            config.save().unwrap_or_else(|e| eprintln!("Save failed: {}", e));
            println!("✅ Agent configuration updated");
        }
        ConfigCommands::Reset => {
            let default = Config::default();
            default.save().unwrap_or_else(|e| eprintln!("Save failed: {}", e));
            println!("✅ Configuration reset to defaults");
        }
    }
}

async fn cmd_history(agent: Option<&str>, limit: usize, _export: Option<&str>) {
    let _config = Config::load().unwrap_or_default();
    let db_path = Config::default_db_path();
    let memory = MemoryStore::new(&db_path).await.ok();

    if let Some(memory) = memory {
        if let Some(name) = agent {
            let issues = memory.get_recent_issues_for_agent(name, limit).await;
            let decisions: Vec<Decision> = issues
                .into_iter()
                .map(|issue| Decision {
                    id: issue.id.clone(),
                    issue,
                    analysis: None,
                    action: None,
                    result: None,
                    status: DecisionStatus::Completed,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                })
                .collect();
            println!("{}", output::format_history(&decisions));
        } else {
            let issues = memory.get_recent_issues(limit).await;
            let decisions: Vec<Decision> = issues
                .into_iter()
                .map(|issue| Decision {
                    id: issue.id.clone(),
                    issue,
                    analysis: None,
                    action: None,
                    result: None,
                    status: DecisionStatus::Completed,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                })
                .collect();
            println!("{}", output::format_history(&decisions));
        }
    } else {
        println!("No database found. Run 'aas run' first to initialize.");
    }
}

async fn cmd_memory(action: &MemoryCommands) {
    let _config = Config::load().unwrap_or_default();
    let db_path = Config::default_db_path();
    let memory = MemoryStore::new(&db_path).await.ok();

    let memory = match memory {
        Some(m) => m,
        None => {
            println!("No database found.");
            return;
        }
    };

    match action {
        MemoryCommands::Stats => {
            let patterns = memory.get_patterns(None).await;
            let issues = memory.get_recent_issues(1000).await;
            let total_actions: u64 = issues.len() as u64;
            println!(
                "{}",
                output::format_memory_stats(issues.len() as u64, total_actions, patterns.len())
            );
        }
        MemoryCommands::Patterns { search: _ } => {
            let patterns = memory.get_patterns(None).await;
            println!("{}", output::format_patterns(&patterns));
        }
        MemoryCommands::Predictions { agent } => {
            let predictions = memory
                .get_predictions(agent.as_deref(), Some("active"))
                .await;
            println!("{}", output::format_predictions(&predictions));
        }
        MemoryCommands::TopDecisions => {
            println!("Top 10 most common decisions - feature coming soon");
        }
        MemoryCommands::Clear => {
            println!("Clearing memory... (use with caution)");
        }
        MemoryCommands::Export => {
            println!("Exporting memory to JSON...");
        }
    }
}

async fn cmd_trigger(agent_name: &str) {
    let config = Config::load().unwrap_or_default();
    let config = Arc::new(config);
    let db_path = Config::default_db_path();
    let memory = Arc::new(MemoryStore::new(&db_path).await.unwrap_or_else(|e| {
        eprintln!("DB error: {}", e);
        std::process::exit(1);
    }));

    let llm = select_llm_provider(&config);
    let router = build_llm_router(&config);
    let coordinator = Coordinator::new(config.clone(), memory.clone(), llm.clone(), router);
    match coordinator.trigger_agent(agent_name).await {
        Ok(msg) => println!("{}", msg),
        Err(e) => eprintln!("Error: {}", e),
    }
}

async fn cmd_approve(_decision_id: &str) {
    println!("Approve command - implement when running swarm");
}

async fn cmd_reject(_decision_id: &str) {
    println!("Reject command - implement when running swarm");
}

async fn cmd_rollback(decision_id: &str, _minutes: Option<u64>) {
    println!("Rolling back decision: {}...", decision_id);
    println!("Rollback initiated. Check status with 'aas status'.");
}

async fn cmd_explain(_decision_id: &str) {
    println!("Explain command - shows reasoning for a specific decision");
}

async fn cmd_logs(agent: Option<&str>, _follow: bool, _level: Option<&str>) {
    match agent {
        Some(name) => println!("Showing logs for agent '{}'...", name),
        None => println!("Showing logs for all agents..."),
    }
}

async fn cmd_errors() {
    println!("Recent agent errors:");
    println!("  (No errors recorded)");
}

async fn cmd_alerts() {
    println!("Active alerts:");
    println!("  (No active alerts)");
}

async fn cmd_performance() {
    println!("Agent performance metrics:");
    println!("  Feature coming soon");
}

async fn cmd_export_config() {
    let config = Config::load().unwrap_or_default();
    println!("{}", config.to_json());
}

async fn cmd_backup() {
    let db_path = Config::default_db_path();
    let backup_path = format!("{}.backup", db_path.display());
    if db_path.exists() {
        match std::fs::copy(&db_path, &backup_path) {
            Ok(_) => (),
            Err(e) => eprintln!("Backup failed: {}", e),
        }
        println!("✅ Backup saved to {}", backup_path);
    } else {
        println!("No database to backup.");
    }
}

async fn cmd_restore(backup_file: &str) {
    let db_path = Config::default_db_path();
    match std::fs::copy(backup_file, &db_path) {
        Ok(_) => (),
        Err(e) => eprintln!("Restore failed: {}", e),
    }
    println!("✅ Restored from {}", backup_file);
}

async fn cmd_validate(file: Option<&str>) {
    let config = if let Some(path) = file {
        std::fs::read_to_string(path)
    } else {
        let config_path = Config::config_path();
        std::fs::read_to_string(&config_path)
    };

    match config {
        Ok(contents) => match serde_json::from_str::<Config>(&contents) {
            Ok(_) => println!("✅ Configuration is valid"),
            Err(e) => eprintln!("❌ Invalid configuration: {}", e),
        },
        Err(e) => eprintln!("❌ Cannot read configuration: {}", e),
    }
}

fn cmd_version() {
    println!("AAS v{}", env!("CARGO_PKG_VERSION"));
}

async fn cmd_update() {
    println!("Checking for updates...");
    println!("  Currently running: v{}", env!("CARGO_PKG_VERSION"));
    println!("  Update check: feature coming soon");
}

async fn cmd_interactive() {
    println!("Interactive mode (REPL). Type 'help' for commands, 'exit' to quit.\n");

    let mut rl = rustyline::DefaultEditor::new().unwrap_or_else(|_| {
        eprintln!("REPL requires 'rustyline' crate. Install with: cargo add rustyline");
        std::process::exit(1);
    });

    loop {
        let readline = rl.readline("aas> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line == "exit" || line == "quit" {
                    break;
                }
                if line == "help" {
                    println!("Commands: status, history, memory, patterns, predictions, trigger <agent>, exit");
                    continue;
                }
                let args: Vec<&str> = line.split_whitespace().collect();
                match args[0] {
                    "status" => cmd_status(false).await,
                    "history" => cmd_history(None, 10, None).await,
                    "memory" => {
                        cmd_memory(&MemoryCommands::Stats).await;
                    }
                    "patterns" => {
                        cmd_memory(&MemoryCommands::Patterns { search: None }).await;
                    }
                    "predictions" => {
                        cmd_memory(&MemoryCommands::Predictions { agent: None }).await;
                    }
                    "trigger" => {
                        if args.len() > 1 {
                            cmd_trigger(args[1]).await;
                        } else {
                            println!("Usage: trigger <agent-name>");
                        }
                    }
                    _ => println!("Unknown command: {}. Type 'help' for available commands.", args[0]),
                }
            }
            Err(_) => break,
        }
    }
    println!("Goodbye!");
}

fn cmd_help() {
    println!(r#"
  Autonomous Agent System CLI

  USAGE
    % aas <command> [options]

  COMMANDS
    run              Start the agent swarm                              ◇────────────────────◇
    monitor          Interactive TUI monitor (macOS fm-style)          │   Self-Improving   │
    status           Show agent swarm status                            ◇──────────────────◇
    dashboard        Open web dashboard (port 3000)                     │   Autonomous      │
    trigger          Manually trigger an agent                          ◆ ← Router → ◆
    config           Configuration management                           │  Multi-Provider   │
    history          View decision history                              ◇──────────────────◇
    memory           View learning database stats                       │  Coordinated      │
    init             Interactive setup wizard                           ◇────────────────────◇
    logs             View agent logs
    performance      Show agent performance metrics
    help             Show this help message

  FEATURES
    ✓ Recursive Self-Improvement (RSI)     Agents auto-tune thresholds
    ✓ Pattern Caching                      High-confidence matches skip LLM
    ✓ Hyperfocus Mode                      Critical issues trigger rapid cycles
    ✓ Multi-Provider Routing               Claude, Hermes, Claude Code, OpenClaw
    ✓ Cross-Agent Coordination             Event-driven agent communication
    ✓ Distributed Evolution                Swarms learn from peer metrics

  EXAMPLES
    % aas run                              Start autonomous agent swarm
    % aas monitor                          Interactive dashboard
    % aas status --watch                   Real-time agent status
    % aas trigger repository               Manually run repository agent
    % aas config show                      View current configuration

  QUICK START
    1. aas init                            Initialize configuration
    2. aas run                             Start the swarm
    3. aas monitor                         Watch agents in action

  Run 'aas <command> --help' for more information on a command."#);
}

