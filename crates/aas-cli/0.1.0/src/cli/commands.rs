use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aas", version, about = "Autonomous Agent System")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interactive setup wizard
    Init {
        /// Path to config file
        #[arg(long)]
        config: Option<String>,
        /// Use default configuration
        #[arg(long)]
        defaults: bool,
        /// Auto-start swarm after init
        #[arg(long)]
        auto_start: bool,
    },

    /// Start the agent swarm (foreground, single run)
    Run {
        /// Config file path
        #[arg(long)]
        config: Option<String>,
        /// Run for N seconds then exit
        #[arg(long)]
        duration: Option<String>,
        /// Detect issues but don't execute
        #[arg(long)]
        dry_run: bool,
    },

    /// Daemon mode (background service)
    Daemon {
        #[command(subcommand)]
        action: DaemonCommands,
    },

    /// Connect to external integrations
    Connect {
        /// Integration name (claude-code, openclaw, etc.)
        integration: String,
        /// Integration endpoint (for REST APIs)
        #[arg(long)]
        endpoint: Option<String>,
        /// API key or auth token
        #[arg(long)]
        token: Option<String>,
    },

    /// Disconnect from integration
    Disconnect {
        /// Integration name to disable
        integration: String,
    },

    /// List available integrations
    Integrations {
        /// Show only connected integrations
        #[arg(long)]
        connected_only: bool,
    },

    /// Stop all agents
    Stop,

    /// Restart specific agent or all
    Restart {
        /// Agent name (optional, restarts all if omitted)
        agent: Option<String>,
    },

    /// Show agent swarm status
    Status {
        /// Watch mode (continuous refresh)
        #[arg(long)]
        watch: bool,
    },

    /// Open web dashboard
    Dashboard {
        /// Port to listen on
        #[arg(long, default_value = "3000")]
        port: u16,
    },

    /// Interactive TUI monitor (macOS fm-style)
    Monitor,

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },

    /// View decision history
    History {
        /// Filter by agent name
        #[arg(long)]
        agent: Option<String>,
        /// Number of entries
        #[arg(long, default_value = "20")]
        limit: usize,
        /// Filter by problem type
        #[arg(long)]
        problem_type: Option<String>,
        /// Export format
        #[arg(long)]
        export: Option<String>,
    },

    /// View learning database stats
    Memory {
        #[command(subcommand)]
        action: MemoryCommands,
    },

    /// Manually trigger an agent
    Trigger {
        /// Agent name to trigger
        agent: String,
        /// Force immediate check
        #[arg(long)]
        force: bool,
    },

    /// Approve a pending decision
    Approve {
        /// Decision ID
        decision_id: String,
    },

    /// Reject a pending decision
    Reject {
        /// Decision ID
        decision_id: String,
    },

    /// Rollback a decision
    Rollback {
        /// Decision ID
        decision_id: String,
        /// Rollback all decisions from last N minutes
        #[arg(long)]
        minutes: Option<u64>,
    },

    /// Show reasoning for a decision
    Explain {
        /// Decision ID
        decision_id: String,
    },

    /// View agent logs
    Logs {
        /// Agent name (optional, shows all if omitted)
        agent: Option<String>,
        /// Follow log output
        #[arg(long, short)]
        follow: bool,
        /// Log level filter
        #[arg(long)]
        level: Option<String>,
    },

    /// Show agent errors
    Errors,

    /// Show active alerts
    Alerts,

    /// Show agent performance metrics
    Performance,

    /// Export current configuration
    ExportConfig,

    /// Backup database and config
    Backup,

    /// Restore from backup
    Restore {
        /// Backup file path
        backup_file: String,
    },

    /// Validate config file
    ValidateConfig {
        /// Config file path
        file: Option<String>,
    },

    /// Show version
    Version,

    /// Check for updates
    Update,

    /// Interactive REPL mode
    Interactive,

    /// Show help
    #[command(alias = "help", alias = "h")]
    Help,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// View current configuration
    Show,
    /// Edit config interactively
    Edit,
    /// Configure LLM provider
    Llm {
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        endpoint: Option<String>,
    },
    /// Enable/disable an agent
    Agent {
        #[arg(long)]
        enable: Option<String>,
        #[arg(long)]
        disable: Option<String>,
    },
    /// Reset to defaults
    Reset,
}

#[derive(Subcommand)]
pub enum MemoryCommands {
    /// Show memory stats
    Stats,
    /// List learned patterns
    Patterns {
        /// Search patterns
        #[arg(long)]
        search: Option<String>,
    },
    /// Show active predictions
    Predictions {
        /// Filter by agent
        #[arg(long)]
        agent: Option<String>,
    },
    /// Top 10 most common decisions
    TopDecisions,
    /// Clear memory
    Clear,
    /// Export memory to JSON
    Export,
}

#[derive(Subcommand)]
pub enum DaemonCommands {
    /// Start daemon (background service)
    Start {
        /// Config file path
        #[arg(long)]
        config: Option<String>,
        /// Run in foreground (for debugging)
        #[arg(long)]
        foreground: bool,
    },
    /// Stop daemon
    Stop,
    /// Restart daemon
    Restart,
    /// Check daemon status
    Status,
    /// View daemon logs
    Logs {
        /// Follow log output
        #[arg(long, short)]
        follow: bool,
        /// Log level filter
        #[arg(long)]
        level: Option<String>,
    },
}
