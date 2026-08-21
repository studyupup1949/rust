//! `aaai merge` — merge two audit definition files.
//!
//! Takes a "base" definition and an "overlay" definition and produces a merged
//! result.  The overlay's entries take precedence for any path that appears in
//! both files.  Entries unique to either file are included as-is.
//!
//! An optional `--detect-conflicts` flag reports paths present in both files
//! with differing diff_type values (a common sign of a stale definition).

use std::path::PathBuf;
use clap::Args;
use colored::Colorize;

use aaai_core::config::{definition::AuditEntry, io as config_io};

#[derive(Args)]
pub struct MergeArgs {
    /// Base definition file (kept when no conflict).
    #[arg(value_name = "BASE")]
    pub base: PathBuf,
    /// Overlay definition file (wins on conflict).
    #[arg(value_name = "OVERLAY")]
    pub overlay: PathBuf,
    /// Output file (default: overwrite BASE).
    #[arg(short = 'o', long, value_name = "FILE")]
    pub out: Option<PathBuf>,
    /// Report conflicts without merging.
    #[arg(long)]
    pub detect_conflicts: bool,
    /// Dry run — print what the merge would produce without writing.
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: MergeArgs) -> anyhow::Result<()> {
    println!("{}", "aaai merge".bold());

    let mut base_def    = config_io::load(&args.base)?;
    let overlay_def = config_io::load(&args.overlay)?;

    // Conflict detection
    let conflicts: Vec<(&AuditEntry, &AuditEntry)> = overlay_def.entries.iter()
        .filter_map(|oe| {
            base_def.find_entry(&oe.path).and_then(|be| {
                if be.diff_type != oe.diff_type { Some((be, oe)) } else { None }
            })
        })
        .collect();

    if !conflicts.is_empty() {
        println!("{}", format!("⚠  {} conflicting entries:", conflicts.len()).yellow().bold());
        for (base_e, over_e) in &conflicts {
            println!("  {} — base: {:?}  overlay: {:?}",
                base_e.path, base_e.diff_type, over_e.diff_type);
        }
        println!();
    }

    if args.detect_conflicts {
        if conflicts.is_empty() {
            println!("{}", "No conflicts found.".green());
        }
        return Ok(());
    }

    // Merge: overlay entries upserted into base.
    let overlay_count = overlay_def.entries.len();
    for entry in overlay_def.entries {
        base_def.upsert_entry(entry);
    }

    println!("  Base entries    : {}", base_def.entries.len() - overlay_count.min(base_def.entries.len()));
    println!("  Overlay entries : {overlay_count}");
    println!("  Result entries  : {}", base_def.entries.len());

    if args.dry_run {
        println!("{}", "--- DRY RUN (not written) ---".cyan().bold());
        return Ok(());
    }

    let out = args.out.unwrap_or(args.base.clone());
    config_io::save(&base_def, &out, true)?;

    println!("{} Merged definition written to: {}", "✓".green(), out.display());
    Ok(())
}
