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
pub(crate) struct Cli {
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
    /// Show a colour-coded audit statistics dashboard.
    Dashboard(cmd::dashboard::DashboardArgs),
    /// Re-run the audit whenever source files change.
    Watch(cmd::watch::WatchArgs),
    /// Generate shell completions and print them to stdout.
    Completions(cmd::completions::CompletionsArgs),
    /// Show raw folder diff without an audit definition.
    Diff(cmd::diff_cmd::DiffArgs),
    /// Merge two audit definition files.
    Merge(cmd::merge::MergeArgs),
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
        Commands::Config(args)      => cmd::config::run(args),
        Commands::Dashboard(args)   => cmd::dashboard::run(args),
        Commands::Watch(args)       => cmd::watch::run(args),
        Commands::Completions(args) => cmd::completions::run(args),
        Commands::Diff(args)        => cmd::diff_cmd::run(args),
        Commands::Merge(args)       => cmd::merge::run(args),
    }
}

#[cfg(test)]
mod tests;
