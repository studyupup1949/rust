//! `aaai export` — export audit entries to CSV or TSV.
//!
//! Produces a tabular snapshot of the audit definition suitable for
//! review in spreadsheet applications.  Fields exported:
//!
//! path, diff_type, status, reason, strategy, ticket,
//! approved_by, approved_at, expires_at, enabled, note, created_at, updated_at

use std::path::PathBuf;
use clap::Args;
use colored::Colorize;

use aaai_core::{
    AuditEngine, DiffEngine,
    config::io as config_io,
};

#[derive(Args)]
pub struct ExportArgs {
    #[arg(short = 'l', long, value_name = "PATH")]
    pub left: PathBuf,
    #[arg(short = 'r', long, value_name = "PATH")]
    pub right: PathBuf,
    #[arg(short = 'c', long, value_name = "FILE")]
    pub config: PathBuf,
    /// Output file (default: stdout).
    #[arg(short = 'o', long, value_name = "FILE")]
    pub out: Option<PathBuf>,
    /// Field separator: "csv" (comma) or "tsv" (tab).
    #[arg(short = 'f', long, default_value = "csv",
          value_parser = ["csv", "tsv"])]
    pub format: String,
    /// Include Unchanged (OK without entry) entries.
    #[arg(long)]
    pub all: bool,
}

pub fn run(args: ExportArgs) -> anyhow::Result<()> {
    let sep: char = if args.format == "tsv" { '\t' } else { ',' };

    let definition = config_io::load(&args.config)?;
    let diffs      = DiffEngine::compare(&args.left, &args.right)?;
    let result     = AuditEngine::evaluate(&diffs, &definition);

    let mut lines: Vec<String> = Vec::new();

    // Header
    lines.push(join(&[
        "path", "diff_type", "status", "reason", "strategy",
        "ticket", "approved_by", "approved_at", "expires_at",
        "enabled", "note", "created_at", "updated_at",
    ], sep));

    // Rows
    for r in &result.results {
        use aaai_core::DiffType;
        if !args.all && r.diff.diff_type == DiffType::Unchanged { continue; }

        let entry = r.entry.as_ref();
        let row = join(&[
            &r.diff.path,
            &r.diff.diff_type.to_string(),
            &r.status.to_string(),
            entry.map(|e| e.reason.as_str()).unwrap_or(""),
            entry.map(|e| e.strategy.label()).unwrap_or(""),
            entry.and_then(|e| e.ticket.as_deref()).unwrap_or(""),
            entry.and_then(|e| e.approved_by.as_deref()).unwrap_or(""),
            &entry.and_then(|e| e.approved_at)
                .map(|t| t.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default(),
            &entry.and_then(|e| e.expires_at)
                .map(|d| d.to_string())
                .unwrap_or_default(),
            &entry.map(|e| if e.enabled { "true" } else { "false" })
                .unwrap_or(""),
            entry.and_then(|e| e.note.as_deref()).unwrap_or(""),
            &entry.and_then(|e| e.created_at)
                .map(|t| t.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default(),
            &entry.and_then(|e| e.updated_at)
                .map(|t| t.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default(),
        ], sep);
        lines.push(row);
    }

    let output = lines.join("\n") + "\n";

    match &args.out {
        Some(path) => {
            std::fs::write(path, output.as_bytes())?;
            println!("{} Exported {} rows to {}",
                "✓".green(), result.results.len(), path.display());
        }
        None => print!("{output}"),
    }
    Ok(())
}

fn join(fields: &[&str], sep: char) -> String {
    fields.iter()
        .map(|f| csv_escape(f, sep))
        .collect::<Vec<_>>()
        .join(&sep.to_string())
}

/// Wrap field in quotes if it contains the separator, a quote, or a newline.
fn csv_escape(s: &str, sep: char) -> String {
    if s.contains(sep) || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
