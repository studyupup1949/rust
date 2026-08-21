//! `aaai snap` — generate an audit-definition template from the current diff.

use std::path::PathBuf;
use clap::Args;
use colored::Colorize;

use aaai_core::{
    AuditDefinition, DiffEngine, DiffType,
    config::{definition::{AuditEntry, AuditStrategy}, io as config_io},
};

#[derive(Args)]
pub struct SnapArgs {
    /// Before (source) folder.
    #[arg(short = 'l', long, value_name = "PATH")]
    pub left: PathBuf,

    /// After (target) folder.
    #[arg(short = 'r', long, value_name = "PATH")]
    pub right: PathBuf,

    /// Output audit definition file.
    #[arg(short = 'o', long, value_name = "FILE")]
    pub out: PathBuf,

    /// If the output file already exists, merge new entries into it
    /// rather than overwriting.
    #[arg(long)]
    pub merge: bool,
}

pub fn run(args: SnapArgs) -> anyhow::Result<()> {
    println!("{}", "aaai snap".bold());

    // Load or create definition
    let mut definition = if args.merge && args.out.exists() {
        println!("Merging into existing: {}", args.out.display());
        config_io::load(&args.out)?
    } else {
        AuditDefinition::new_empty()
    };

    let diffs = DiffEngine::compare(&args.left, &args.right)?;
    let mut added = 0usize;
    let mut skipped = 0usize;

    for diff in &diffs {
        // Skip Unchanged, Incomparable, and directory-only entries.
        if diff.diff_type == DiffType::Unchanged
            || diff.diff_type == DiffType::Incomparable
            || diff.is_dir
        {
            continue;
        }

        // In merge mode, skip entries that already have a non-empty reason.
        if args.merge {
            if let Some(existing) = definition.find_entry(&diff.path) {
                if !existing.reason.trim().is_empty() {
                    skipped += 1;
                    continue;
                }
            }
        }

        let entry = AuditEntry {
            path: diff.path.clone(),
            diff_type: diff.diff_type,
            reason: String::new(), // Intentionally blank — must be filled by human.
            strategy: AuditStrategy::None,
            enabled: true,
            note: None,
        };
        definition.upsert_entry(entry);
        added += 1;
    }

    config_io::save(&definition, &args.out, false)?;

    println!("{} snapshot generated: {}", "✓".green(), args.out.display());
    println!(
        "  {} entries added, {} skipped (already have a reason)",
        added.to_string().yellow(),
        skipped
    );
    println!();
    println!(
        "{}",
        "Next step: open the file and fill in the 'reason' field for each entry,\n\
         then run `aaai audit` to verify."
            .dimmed()
    );

    Ok(())
}
