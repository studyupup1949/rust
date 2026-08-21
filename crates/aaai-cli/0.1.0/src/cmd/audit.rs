//! `aaai audit` subcommand.

use std::path::PathBuf;
use std::process;

use clap::Args;
use colored::Colorize;

use aaai_core::{
    AuditEngine, AuditStatus, DiffEngine,
    config::io as config_io,
};

#[derive(Args)]
pub struct AuditArgs {
    /// Before (source) folder.
    #[arg(short = 'l', long, value_name = "PATH")]
    pub left: PathBuf,

    /// After (target) folder.
    #[arg(short = 'r', long, value_name = "PATH")]
    pub right: PathBuf,

    /// Audit definition file (YAML).
    #[arg(short = 'c', long, value_name = "FILE")]
    pub config: PathBuf,

    /// Show only summary (suppress per-file lines).
    #[arg(long)]
    pub summary_only: bool,

    /// Allow Pending entries without failing (useful in draft mode).
    #[arg(long)]
    pub allow_pending: bool,
}

pub fn run(args: AuditArgs) -> anyhow::Result<()> {
    // Load definition
    let definition = config_io::load(&args.config)?;

    // Diff
    println!("{}", "aaai audit".bold());
    println!("Before : {}", args.left.display());
    println!("After  : {}", args.right.display());
    println!("Config : {}", args.config.display());
    println!();

    let diffs = DiffEngine::compare(&args.left, &args.right)?;
    let result = AuditEngine::evaluate(&diffs, &definition);
    let s = &result.summary;

    // Per-file output
    if !args.summary_only {
        for r in &result.results {
            if r.diff.diff_type == aaai_core::DiffType::Unchanged {
                continue;
            }
            let status_str = match r.status {
                AuditStatus::Ok => "OK     ".green().to_string(),
                AuditStatus::Pending => "PENDING".yellow().to_string(),
                AuditStatus::Failed => "FAILED ".red().bold().to_string(),
                AuditStatus::Ignored => "IGNORED".dimmed().to_string(),
                AuditStatus::Error => "ERROR  ".red().to_string(),
            };
            println!("{status_str}  {}  ({})", r.diff.path, r.diff.diff_type);
            if let Some(detail) = &r.detail {
                if r.status != AuditStatus::Ok {
                    println!("         {}", detail.dimmed());
                }
            }
        }
        println!();
    }

    // Summary
    let verdict_str = if s.is_passing() {
        "Result: OK".green().bold()
    } else {
        "Result: FAILED".red().bold()
    };
    println!("{verdict_str}");
    println!(
        "  Total: {}  OK: {}  Pending: {}  Failed: {}  Error: {}",
        s.total,
        s.ok.to_string().green(),
        s.pending.to_string().yellow(),
        s.failed.to_string().red(),
        s.error.to_string().red(),
    );

    // Exit code
    let ok = s.failed == 0
        && s.error == 0
        && (args.allow_pending || s.pending == 0);

    if !ok {
        process::exit(1);
    }
    Ok(())
}
