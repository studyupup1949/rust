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

use aaai::{
    AuditEngine, DiffEngine, MaskingEngine,
    config::io as config_io,
    project::config::ProjectConfig,
};

const EXPORT_AFTER_HELP: &str = "\
Next steps:
  Open the resulting CSV/TSV in a spreadsheet for external review, then
  approve changes inside aaai-gui or by editing audit.yaml directly.\
";

#[derive(Args)]
#[command(after_help = EXPORT_AFTER_HELP)]
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

    // RFC 103 F4 — this file bypassed the report API entirely and never
    // masked anything. A CSV/TSV export is opened in a spreadsheet for
    // external review, the same untrusted-sink reasoning as `aaai report`
    // (§5.1a); masking is always enabled here too.
    let custom_patterns = ProjectConfig::discover(&args.left)
        .unwrap_or(None)
        .map(|(config, _)| config.custom_mask_patterns)
        .unwrap_or_default();
    let masker = MaskingEngine::with_custom(&custom_patterns);

    let mut lines: Vec<String> = Vec::new();

    // Header
    lines.push(join(&[
        "path", "diff_type", "status", "reason", "strategy",
        "ticket", "approved_by", "approved_at", "expires_at",
        "enabled", "note", "created_at", "updated_at",
    ], sep));

    // Rows
    for r in &result.results {
        use aaai::DiffType;
        if !args.all && r.diff.diff_type == DiffType::Unchanged { continue; }

        let entry = r.entry.as_ref();
        // Masked (free text, §4): reason, ticket, note. `path` is never
        // masked — it is the audit's identifier, not a secret carrier.
        let reason = entry.map(|e| masker.mask(&e.reason)).unwrap_or_default();
        let ticket = entry
            .and_then(|e| e.ticket.as_deref())
            .map(|t| masker.mask(t))
            .unwrap_or_default();
        let note = entry
            .and_then(|e| e.note.as_deref())
            .map(|n| masker.mask(n))
            .unwrap_or_default();
        let row = join(
            &[
                &r.diff.path,
                &r.diff.diff_type.to_string(),
                &r.status.to_string(),
                &reason,
                entry.map(|e| e.strategy.label()).unwrap_or(""),
                &ticket,
                entry.and_then(|e| e.approved_by.as_deref()).unwrap_or(""),
                &entry
                    .and_then(|e| e.approved_at)
                    .map(|t| t.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                    .unwrap_or_default(),
                &entry
                    .and_then(|e| e.expires_at)
                    .map(|d| d.to_string())
                    .unwrap_or_default(),
                &entry
                    .map(|e| if e.enabled { "true" } else { "false" })
                    .unwrap_or(""),
                &note,
                &entry
                    .and_then(|e| e.created_at)
                    .map(|t| t.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                    .unwrap_or_default(),
                &entry
                    .and_then(|e| e.updated_at)
                    .map(|t| t.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                    .unwrap_or_default(),
            ],
            sep,
        );
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

/// Wrap field in quotes if it contains the separator, a quote, a newline, or
/// a bare CR; neutralize a leading formula-trigger character.
///
/// RFC 103 F1 — RFC 4180 quoting alone does not stop spreadsheet formula
/// injection: the CSV parser consumes the quotes before the spreadsheet
/// evaluates the cell contents. A cell whose first character is `=`, `+`,
/// `-`, `@`, tab, or CR is prefixed with a single apostrophe, applied
/// *before* quoting so the apostrophe lands inside the quoted field.
fn csv_escape(s: &str, sep: char) -> String {
    let neutralized = if starts_with_formula_trigger(s) {
        format!("'{s}")
    } else {
        s.to_string()
    };
    if neutralized.contains(sep)
        || neutralized.contains('"')
        || neutralized.contains('\n')
        || neutralized.contains('\r')
    {
        format!("\"{}\"", neutralized.replace('"', "\"\""))
    } else {
        neutralized
    }
}

/// True if `s`'s first character would be interpreted by a spreadsheet as
/// the start of a formula: `=`, `+`, `-`, `@`, a tab, or a carriage return.
fn starts_with_formula_trigger(s: &str) -> bool {
    matches!(s.chars().next(), Some('=' | '+' | '-' | '@' | '\t' | '\r'))
}

#[cfg(test)]
mod tests;
