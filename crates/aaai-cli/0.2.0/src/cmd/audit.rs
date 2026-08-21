//! `aaai audit` subcommand (Phase 2: --verbose, --json-output, glob rules).

use std::path::PathBuf;
use std::process;

use clap::Args;
use colored::Colorize;

use aaai_core::{
    AuditEngine, AuditStatus, DiffEngine, DiffType,
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

    /// Show OK entries in addition to non-OK.
    #[arg(long)]
    pub verbose: bool,

    /// Show only the summary line (suppress per-file lines).
    #[arg(long)]
    pub quiet: bool,

    /// Output results as JSON to stdout.
    #[arg(long = "json-output")]
    pub json_output: bool,

    /// Allow Pending entries without failing (draft mode).
    #[arg(long)]
    pub allow_pending: bool,
}

pub fn run(args: AuditArgs) -> anyhow::Result<()> {
    let definition = config_io::load(&args.config)?;
    let diffs = DiffEngine::compare(&args.left, &args.right)?;
    let result = AuditEngine::evaluate(&diffs, &definition);
    let s = &result.summary;

    // ── JSON output ───────────────────────────────────────────────────────
    if args.json_output {
        let out = serde_json::json!({
            "result": if s.is_passing() { "PASSED" } else { "FAILED" },
            "summary": {
                "total":   s.total,
                "ok":      s.ok,
                "pending": s.pending,
                "failed":  s.failed,
                "ignored": s.ignored,
                "error":   s.error,
            },
            "entries": result.results.iter().map(|r| serde_json::json!({
                "path":      r.diff.path,
                "diff_type": r.diff.diff_type.to_string(),
                "status":    r.status.to_string(),
                "detail":    r.detail,
                "reason":    r.entry.as_ref().map(|e| &e.reason),
                "strategy":  r.entry.as_ref().map(|e| e.strategy.label()),
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        let ok = s.failed == 0 && s.error == 0
            && (args.allow_pending || s.pending == 0);
        if !ok { process::exit(1); }
        return Ok(());
    }

    // ── Human output ──────────────────────────────────────────────────────
    if !args.quiet {
        println!("{}", "aaai audit".bold());
        println!("Before : {}", args.left.display());
        println!("After  : {}", args.right.display());
        println!("Config : {}", args.config.display());
        println!();
    }

    if !args.quiet {
        for r in &result.results {
            // In default mode skip Unchanged; in verbose mode show OK too.
            let show = match r.status {
                AuditStatus::Ok =>
                    args.verbose && r.diff.diff_type != DiffType::Unchanged,
                AuditStatus::Ignored => args.verbose,
                _ => r.diff.diff_type != DiffType::Unchanged,
            };
            if !show { continue; }

            let status_str = match r.status {
                AuditStatus::Ok      => "OK     ".green().to_string(),
                AuditStatus::Pending => "PENDING".yellow().to_string(),
                AuditStatus::Failed  => "FAILED ".red().bold().to_string(),
                AuditStatus::Ignored => "IGNORED".dimmed().to_string(),
                AuditStatus::Error   => "ERROR  ".red().to_string(),
            };
            println!("{status_str}  {}  ({})", r.diff.path, r.diff.diff_type);
            if let Some(detail) = &r.detail {
                if r.status != AuditStatus::Ok {
                    println!("         {}", detail.dimmed());
                }
            }
            if let Some(entry) = &r.entry {
                if args.verbose && !entry.reason.is_empty() {
                    println!("         {}", format!("Reason: {}", entry.reason).dimmed());
                }
            }
        }
        println!();
    }

    let verdict_str = if s.is_passing() {
        "Result: PASSED".green().bold()
    } else {
        "Result: FAILED".red().bold()
    };
    println!("{verdict_str}");
    println!(
        "  Total: {}  OK: {}  Pending: {}  Failed: {}  Error: {}  Ignored: {}",
        s.total,
        s.ok.to_string().green(),
        s.pending.to_string().yellow(),
        s.failed.to_string().red(),
        s.error.to_string().red(),
        s.ignored,
    );

    let ok = s.failed == 0 && s.error == 0
        && (args.allow_pending || s.pending == 0);
    if !ok { process::exit(1); }
    Ok(())
}
