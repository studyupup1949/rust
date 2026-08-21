//! aaai — audit for asset integrity
//! CLI entry point (Phase 3: check, history, granular exit codes).

use clap::{Parser, Subcommand};

mod cmd;

#[derive(Parser)]
#[command(
    name    = "aaai",
    about   = "audit for asset integrity — folder diff auditor",
    version,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compare two folders and audit differences against the definition.
    Audit(cmd::audit::AuditArgs),
    /// Generate an audit-definition template from the current diff.
    Snap(cmd::snap::SnapArgs),
    /// Output an audit report (Markdown or JSON).
    Report(cmd::report::ReportArgs),
    /// Validate an audit definition file without running a diff.
    Check(cmd::check::CheckArgs),
    /// Show recent audit runs from the history file.
    History(cmd::history::HistoryArgs),
    /// Initialise or display the project .aaai.yaml config.
    Config(cmd::config::ConfigArgs),
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn")
    ).init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Audit(args)   => cmd::audit::run(args),
        Commands::Snap(args)    => cmd::snap::run(args),
        Commands::Report(args)  => cmd::report::run(args),
        Commands::Check(args)   => cmd::check::run(args),
        Commands::History(args) => cmd::history::run(args),
        Commands::Config(args)   => cmd::config::run(args),
    }
}
