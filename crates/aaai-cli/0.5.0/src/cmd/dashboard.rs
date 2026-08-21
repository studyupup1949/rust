//! `aaai dashboard` — colour-coded audit statistics dashboard.

use std::path::PathBuf;
use clap::Args;
use colored::Colorize;

use aaai_core::{AuditEngine, AuditStatus, DiffEngine, DiffType, IgnoreRules,
                config::io as config_io};

#[derive(Args)]
pub struct DashboardArgs {
    #[arg(short = 'l', long, value_name = "PATH")]
    pub left: PathBuf,
    #[arg(short = 'r', long, value_name = "PATH")]
    pub right: PathBuf,
    #[arg(short = 'c', long, value_name = "FILE")]
    pub config: PathBuf,
    /// Show per-entry detail beneath the summary cards.
    #[arg(long)]
    pub detail: bool,
}

pub fn run(args: DashboardArgs) -> anyhow::Result<()> {
    let definition = config_io::load(&args.config)?;
    let diffs      = DiffEngine::compare(&args.left, &args.right)?;
    let result     = AuditEngine::evaluate(&diffs, &definition);
    let s          = &result.summary;

    // Expiry info
    let expired      = definition.expired_entries();
    let expiring_soon = definition.expiring_soon(30);

    // ── Banner ────────────────────────────────────────────────────────
    println!();
    println!("  {}",
        if s.is_passing() {
            "╔══════════════════════════════╗".green().bold()
        } else {
            "╔══════════════════════════════╗".red().bold()
        }
    );
    let verdict_line = if s.is_passing() {
        format!("  ║  aaai  ──  {}              ║", "PASSED".green().bold())
    } else {
        format!("  ║  aaai  ──  {}              ║", "FAILED".red().bold())
    };
    println!("{verdict_line}");
    println!("  {}",
        if s.is_passing() {
            "╚══════════════════════════════╝".green().bold()
        } else {
            "╚══════════════════════════════╝".red().bold()
        }
    );
    println!();

    // ── Stat cards ────────────────────────────────────────────────────
    print_stat_card("OK",      s.ok,      "green");
    print_stat_card("Pending", s.pending, "yellow");
    print_stat_card("Failed",  s.failed,  "red");
    print_stat_card("Error",   s.error,   "magenta");
    print_stat_card("Ignored", s.ignored, "white");
    println!();
    println!("  Total files: {}", s.total);

    // ── Expiry warnings ───────────────────────────────────────────────
    if !expired.is_empty() {
        println!();
        println!("  {} {} expired entries:", "⚠ ".yellow().bold(), expired.len());
        for e in &expired {
            println!("    {} — expired: {}", e.path.red(),
                e.expires_at.map(|d| d.to_string()).unwrap_or_default());
        }
    }
    if !expiring_soon.is_empty() {
        println!("  {} {} entries expiring within 30 days:", "⏰".yellow(), expiring_soon.len());
        for e in &expiring_soon {
            println!("    {} — {}", e.path,
                e.expires_at.map(|d| d.to_string()).unwrap_or_default());
        }
    }

    // ── Attention list ────────────────────────────────────────────────
    let attention: Vec<_> = result.results.iter()
        .filter(|r| matches!(r.status, AuditStatus::Failed | AuditStatus::Pending | AuditStatus::Error)
                 && r.diff.diff_type != DiffType::Unchanged)
        .collect();

    if !attention.is_empty() {
        println!();
        println!("  {} Needs attention:", "▸".bold());
        for r in &attention {
            let icon = match r.status {
                AuditStatus::Failed  => "✗".red().bold().to_string(),
                AuditStatus::Pending => "?".yellow().to_string(),
                AuditStatus::Error   => "!".magenta().to_string(),
                _                    => " ".to_string(),
            };
            println!("    {icon} {} ({})", r.diff.path, r.status);
            if let Some(d) = &r.detail {
                println!("      {}", d.dimmed());
            }
        }
    }

    if args.detail {
        println!();
        println!("  {} All changed entries:", "▸".bold());
        for r in &result.results {
            if r.diff.diff_type == DiffType::Unchanged { continue; }
            let icon = match r.status {
                AuditStatus::Ok      => "✓".green().to_string(),
                AuditStatus::Pending => "?".yellow().to_string(),
                AuditStatus::Failed  => "✗".red().bold().to_string(),
                AuditStatus::Error   => "!".magenta().to_string(),
                AuditStatus::Ignored => "–".dimmed().to_string(),
            };
            let stats = r.diff.stats.as_ref()
                .map(|st| format!(" +{} −{}", st.lines_added, st.lines_removed))
                .unwrap_or_default();
            let binary = if r.diff.is_binary { " [bin]" } else { "" };
            println!("    {icon} {}{stats}{binary}", r.diff.path);
        }
    }

    println!();
    Ok(())
}

fn print_stat_card(label: &str, count: usize, color: &str) {
    let count_str = match color {
        "green"   => count.to_string().green().bold().to_string(),
        "yellow"  => count.to_string().yellow().bold().to_string(),
        "red"     => count.to_string().red().bold().to_string(),
        "magenta" => count.to_string().magenta().bold().to_string(),
        _         => count.to_string().dimmed().to_string(),
    };
    println!("  {:8} {count_str}", format!("{label}:"));
}
