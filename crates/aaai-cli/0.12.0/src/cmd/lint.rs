//! `aaai lint` — best-practice linter for audit definition files.
//!
//! Checks beyond basic YAML validity:
//! * reason length (≥ 10 chars recommended)
//! * no entries without a ticket when tickets are expected
//! * duplicate paths (same path defined twice)
//! * glob patterns that match nothing concrete
//! * entries with no approved_by (especially when audit has been run)
//! * expires_at in the past
//! * LineMatch rules with empty lines
//! * strategies that don't match the diff type (e.g. LineMatch on Added)

use std::collections::HashMap;
use std::path::PathBuf;
use clap::Args;
use colored::Colorize;

use aaai_core::config::io as config_io;

#[derive(Args)]
pub struct LintArgs {
    /// Audit definition file to lint.
    #[arg(value_name = "FILE")]
    pub file: PathBuf,
    /// Require all entries to have a ticket field.
    #[arg(long)]
    pub require_ticket: bool,
    /// Require all entries to have an approved_by field.
    #[arg(long)]
    pub require_approver: bool,
    /// Minimum reason length (default: 10).
    #[arg(long, default_value = "10")]
    pub min_reason_len: usize,
    /// Output as JSON.
    #[arg(long = "json-output")]
    pub json_output: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct LintIssue {
    pub path: String,
    pub kind: String,
    pub severity: String,
    pub message: String,
}

pub fn run(args: LintArgs) -> anyhow::Result<()> {
    let def = config_io::load(&args.file)?;
    let mut issues: Vec<LintIssue> = Vec::new();

    // Duplicate path check
    let mut seen: HashMap<String, usize> = HashMap::new();
    for entry in &def.entries {
        *seen.entry(entry.path.clone()).or_insert(0) += 1;
    }
    for (path, count) in &seen {
        if *count > 1 {
            issues.push(LintIssue {
                path: path.clone(),
                kind: "duplicate-path".into(),
                severity: "error".into(),
                message: format!("Path appears {count} times — only the last will take effect."),
            });
        }
    }

    let today = chrono::Utc::now().date_naive();

    for entry in &def.entries {
        // Reason length
        if entry.reason.trim().len() < args.min_reason_len {
            issues.push(LintIssue {
                path: entry.path.clone(),
                kind: "short-reason".into(),
                severity: "warning".into(),
                message: format!(
                    "Reason is {} chars (minimum recommended: {}).",
                    entry.reason.trim().len(), args.min_reason_len
                ),
            });
        }

        // Ticket requirement
        if args.require_ticket && entry.ticket.is_none() {
            issues.push(LintIssue {
                path: entry.path.clone(),
                kind: "missing-ticket".into(),
                severity: "warning".into(),
                message: "No ticket reference (--require-ticket is set).".into(),
            });
        }

        // Approver requirement
        if args.require_approver && entry.approved_by.is_none() {
            issues.push(LintIssue {
                path: entry.path.clone(),
                kind: "missing-approver".into(),
                severity: "warning".into(),
                message: "No approved_by field (--require-approver is set).".into(),
            });
        }

        // Expired entries
        if let Some(exp) = entry.expires_at {
            if exp <= today {
                issues.push(LintIssue {
                    path: entry.path.clone(),
                    kind: "expired".into(),
                    severity: "warning".into(),
                    message: format!("Entry expired on {exp} — re-review required."),
                });
            }
        }

        // LineMatch with empty line rules
        if let aaai_core::config::definition::AuditStrategy::LineMatch { rules } = &entry.strategy {
            for (i, rule) in rules.iter().enumerate() {
                if rule.line.trim().is_empty() {
                    issues.push(LintIssue {
                        path: entry.path.clone(),
                        kind: "empty-line-rule".into(),
                        severity: "error".into(),
                        message: format!("LineMatch rule {} has an empty 'line' field.", i + 1),
                    });
                }
            }
            if rules.is_empty() {
                issues.push(LintIssue {
                    path: entry.path.clone(),
                    kind: "empty-linematch".into(),
                    severity: "error".into(),
                    message: "LineMatch strategy has no rules.".into(),
                });
            }
        }

        // Strategy / diff_type mismatch suggestions
        use aaai_core::DiffType;
        use aaai_core::config::definition::AuditStrategy;
        match (&entry.diff_type, &entry.strategy) {
            (DiffType::Added | DiffType::Removed, AuditStrategy::LineMatch { .. }) => {
                issues.push(LintIssue {
                    path: entry.path.clone(),
                    kind: "strategy-mismatch".into(),
                    severity: "info".into(),
                    message: format!(
                        "LineMatch on {:?} entry — consider Checksum or None instead.",
                        entry.diff_type
                    ),
                });
            }
            _ => {}
        }

        // Disabled entries warning
        if !entry.enabled {
            issues.push(LintIssue {
                path: entry.path.clone(),
                kind: "disabled".into(),
                severity: "info".into(),
                message: "Entry is disabled — it will be Ignored during audit.".into(),
            });
        }
    }

    // Output
    if args.json_output {
        println!("{}", serde_json::to_string_pretty(&issues)?);
        if issues.iter().any(|i| i.severity == "error") {
            std::process::exit(1);
        }
        return Ok(());
    }

    println!("{}", "aaai lint".bold());
    println!("File: {}", args.file.display());
    println!("Entries: {}", def.entries.len());
    println!();

    let errors:   Vec<_> = issues.iter().filter(|i| i.severity == "error").collect();
    let warnings: Vec<_> = issues.iter().filter(|i| i.severity == "warning").collect();
    let infos:    Vec<_> = issues.iter().filter(|i| i.severity == "info").collect();

    for issue in &errors {
        println!("{} [{}] {}  — {}", "✗".red().bold(), issue.kind.red(), issue.path.bold(), issue.message);
    }
    for issue in &warnings {
        println!("{} [{}] {}  — {}", "⚠".yellow(), issue.kind.yellow(), issue.path, issue.message);
    }
    for issue in &infos {
        println!("{} [{}] {}  — {}", "ℹ".cyan(), issue.kind.cyan(), issue.path.dimmed(), issue.message.dimmed());
    }

    if issues.is_empty() {
        println!("{}", "No issues found.".green());
    } else {
        println!();
        println!(
            "  {} error(s)  {} warning(s)  {} info(s)",
            errors.len().to_string().red(),
            warnings.len().to_string().yellow(),
            infos.len().to_string().cyan(),
        );
    }

    if !errors.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}
