//! aaai — audit for asset integrity
//!
//! Command-line interface entry point.

use clap::{Parser, Subcommand};

mod cmd;

#[derive(Parser)]
#[command(
    name  = "aaai",
    about = "audit for asset integrity — folder diff auditor",
    long_about = "aaai compares two folder trees and audits the differences \
                  against a YAML definition of expected changes.\n\n\
                  Each expected change requires a human-readable reason, \
                  making audit decisions traceable and explainable.",
    version,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run an audit: compare folders against the audit definition.
    Audit(cmd::audit::AuditArgs),

    /// Generate an audit-definition template from the current diff.
    Snap(cmd::snap::SnapArgs),

    /// Output an audit report (Markdown or JSON).
    Report(cmd::report::ReportArgs),
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Audit(args) => cmd::audit::run(args),
        Commands::Snap(args) => cmd::snap::run(args),
        Commands::Report(args) => cmd::report::run(args),
    }
}
