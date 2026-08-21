//! aaai — audit for asset integrity
//! CLI entry point.

use clap::{Parser, Subcommand};

mod cmd;

/// Top-level `--help` footer. Surfaced when the user invokes `aaai --help`
/// with no subcommand. Keeps the on-ramp visible: init → snap → audit.
const TOP_LEVEL_AFTER_HELP: &str = "\
Getting started:
  aaai init                                 # interactive setup wizard
  aaai snap -l ./before -r ./after \\
            -o audit.yaml                   # create an audit definition
  aaai audit -l ./before -r ./after \\
             -c audit.yaml                  # run the audit
  aaai report -o report.md                  # write a Markdown report

See `aaai <subcommand> --help` for details on each command,
or `aaai exit-codes` for the exit-code reference.

For the desktop UI, run `aaai-gui`.\
";

#[derive(Parser)]
#[command(
    name    = "aaai",
    about   = "audit for asset integrity — folder diff auditor",
    version,
    after_help = TOP_LEVEL_AFTER_HELP,
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
    /// Interactive project setup wizard.
    Init(cmd::init::InitArgs),
    /// Export audit entries to CSV or TSV.
    Export(cmd::export::ExportArgs),
    /// Show detailed version and build information.
    Version(cmd::version_cmd::VersionArgs),
    /// Lint an audit definition file for best-practice issues.
    Lint(cmd::lint::LintArgs),
    /// Print the exit-code reference table.
    #[command(name = "exit-codes")]
    ExitCodes(cmd::exit_codes::ExitCodesArgs),
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn")
    ).init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Audit(args)       => cmd::audit::run(args),
        Commands::Snap(args)        => cmd::snap::run(args),
        Commands::Report(args)      => cmd::report::run(args),
        Commands::Check(args)       => cmd::check::run(args),
        Commands::History(args)     => cmd::history::run(args),
        Commands::Config(args)      => cmd::config::run(args),
        Commands::Dashboard(args)   => cmd::dashboard::run(args),
        Commands::Watch(args)       => cmd::watch::run(args),
        Commands::Completions(args) => cmd::completions::run(args),
        Commands::Diff(args)        => cmd::diff_cmd::run(args),
        Commands::Merge(args)       => cmd::merge::run(args),
        Commands::Init(args)        => cmd::init::run(args),
        Commands::Export(args)      => cmd::export::run(args),
        Commands::Version(args)     => cmd::version_cmd::run(args),
        Commands::Lint(args)        => cmd::lint::run(args),
        Commands::ExitCodes(args)   => cmd::exit_codes::run(args),
    }
}

#[cfg(test)]
mod tests;
