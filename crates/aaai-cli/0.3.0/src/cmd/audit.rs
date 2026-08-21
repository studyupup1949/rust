//! `aaai audit` — Phase 3: ignore file, history, expiry warnings, granular exit codes.
//!
//! Exit codes:
//!   0  PASSED  — all entries OK or Ignored
//!   1  FAILED  — one or more audit failures
//!   2  PENDING — unresolved entries (and --allow-pending not set)
//!   3  ERROR   — file-level read / compare errors
//!   4  (reserved for config error, handled by anyhow before this point)

use std::path::PathBuf;
use std::process;

use clap::Args;
use colored::Colorize;

use aaai_core::{
    AuditEngine, AuditStatus, DiffEngine, DiffType, IgnoreRules,
    config::io as config_io,
    history::{record::HistoryRecord, store as history_store},
};

#[derive(Args)]
pub struct AuditArgs {
    #[arg(short = 'l', long, value_name = "PATH")]
    pub left: PathBuf,
    #[arg(short = 'r', long, value_name = "PATH")]
    pub right: PathBuf,
    #[arg(short = 'c', long, value_name = "FILE")]
    pub config: PathBuf,
    /// Path to .aaaiignore file (default: <left>/.aaaiignore).
    #[arg(long, value_name = "FILE")]
    pub ignore: Option<PathBuf>,
    /// Show all entries including OK and Ignored.
    #[arg(long)]
    pub verbose: bool,
    /// Print only the summary line.
    #[arg(long)]
    pub quiet: bool,
    /// Output results as JSON to stdout.
    #[arg(long = "json-output")]
    pub json_output: bool,
    /// Allow Pending entries without failing.
    #[arg(long)]
    pub allow_pending: bool,
    /// Do not record this run in the history file.
    #[arg(long)]
    pub no_history: bool,
}

pub fn run(args: AuditArgs) -> anyhow::Result<()> {
    // Load ignore rules
    let ignore_path = args.ignore.clone()
        .unwrap_or_else(|| args.left.join(".aaaiignore"));
    let ignore = IgnoreRules::load(&ignore_path)?;

    // Load definition
    let definition = config_io::load(&args.config)?;

    // Diff + audit
    let diffs  = DiffEngine::compare_with_ignore(&args.left, &args.right, &ignore)?;
    let result = AuditEngine::evaluate(&diffs, &definition);
    let s      = &result.summary;

    // Append history
    if !args.no_history {
        let record = HistoryRecord::new(&args.left, &args.right, Some(&args.config), s);
        if let Err(e) = history_store::append(&record) {
            log::warn!("Could not write history: {e}");
        }
    }

    // Expiry warnings
    let expired      = definition.expired_entries();
    let expiring_soon = definition.expiring_soon(30);

    // ── JSON output ────────────────────────────────────────────────────
    if args.json_output {
        let doc = serde_json::json!({
            "result": if s.is_passing() { "PASSED" } else { "FAILED" },
            "summary": { "total": s.total, "ok": s.ok, "pending": s.pending,
                         "failed": s.failed, "ignored": s.ignored, "error": s.error },
            "expired_count": expired.len(),
            "expiring_soon_count": expiring_soon.len(),
            "entries": result.results.iter().map(|r| serde_json::json!({
                "path":       r.diff.path,
                "diff_type":  r.diff.diff_type.to_string(),
                "status":     r.status.to_string(),
                "reason":     r.entry.as_ref().map(|e| &e.reason),
                "ticket":     r.entry.as_ref().and_then(|e| e.ticket.as_ref()),
                "approved_by":r.entry.as_ref().and_then(|e| e.approved_by.as_ref()),
                "approved_at":r.entry.as_ref().and_then(|e| e.approved_at),
                "expires_at": r.entry.as_ref().and_then(|e| e.expires_at),
                "strategy":   r.entry.as_ref().map(|e| e.strategy.label()),
                "detail":     r.detail,
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&doc)?);
        process::exit(exit_code(s, args.allow_pending));
    }

    // ── Human output ──────────────────────────────────────────────────
    if !args.quiet {
        println!("{}", "aaai audit".bold());
        println!("Before : {}", args.left.display());
        println!("After  : {}", args.right.display());
        println!("Config : {}", args.config.display());
        if ignore_path.exists() {
            println!("Ignore : {}", ignore_path.display());
        }
        println!();

        // Expiry warnings
        if !expired.is_empty() {
            println!("{}", format!("⚠  {} EXPIRED entries in definition:", expired.len()).yellow().bold());
            for e in &expired {
                println!("   {} (expired: {})",
                    e.path,
                    e.expires_at.map(|d| d.to_string()).unwrap_or_default());
            }
            println!();
        }
        if !expiring_soon.is_empty() {
            println!("{}", format!("⏰ {} entries expiring within 30 days:", expiring_soon.len()).yellow());
            for e in &expiring_soon {
                println!("   {} (expires: {})",
                    e.path,
                    e.expires_at.map(|d| d.to_string()).unwrap_or_default());
            }
            println!();
        }

        // Per-file lines
        for r in &result.results {
            let show = match r.status {
                AuditStatus::Ok      => args.verbose && r.diff.diff_type != DiffType::Unchanged,
                AuditStatus::Ignored => args.verbose,
                _                    => r.diff.diff_type != DiffType::Unchanged,
            };
            if !show { continue; }

            let status_str = match r.status {
                AuditStatus::Ok      => "OK     ".green().to_string(),
                AuditStatus::Pending => "PENDING".yellow().to_string(),
                AuditStatus::Failed  => "FAILED ".red().bold().to_string(),
                AuditStatus::Ignored => "IGNORED".dimmed().to_string(),
                AuditStatus::Error   => "ERROR  ".red().to_string(),
            };
            let ticket_tag = r.entry.as_ref()
                .and_then(|e| e.ticket.as_ref())
                .map(|t| format!(" [{}]", t))
                .unwrap_or_default();
            let expiry_tag = r.entry.as_ref()
                .and_then(|e| e.expires_at)
                .map(|d| {
                    let today = chrono::Utc::now().date_naive();
                    if d <= today { format!(" ⚠expired:{d}") }
                    else { format!(" ⏰:{d}") }
                })
                .unwrap_or_default();

            println!("{status_str}  {}{ticket_tag}{expiry_tag}  ({})",
                r.diff.path, r.diff.diff_type);

            if let Some(detail) = &r.detail {
                if r.status != AuditStatus::Ok {
                    println!("         {}", detail.dimmed());
                }
            }
            if args.verbose {
                if let Some(entry) = &r.entry {
                    if !entry.reason.is_empty() {
                        println!("         Reason: {}", entry.reason.dimmed());
                    }
                    if let Some(ab) = &entry.approved_by {
                        println!("         Approved by: {}", ab.dimmed());
                    }
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

    process::exit(exit_code(s, args.allow_pending));
}

fn exit_code(s: &aaai_core::AuditSummary, allow_pending: bool) -> i32 {
    if s.error > 0   { return 3; }
    if s.failed > 0  { return 1; }
    if !allow_pending && s.pending > 0 { return 2; }
    0
}
