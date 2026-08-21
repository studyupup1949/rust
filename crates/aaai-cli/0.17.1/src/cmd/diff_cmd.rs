//! `aaai diff` — raw folder diff without an audit definition.
//!
//! Useful for quick inspection before creating or updating a definition file.

use std::path::PathBuf;
use clap::Args;
use colored::Colorize;
use similar::{ChangeTag, TextDiff};

use aaai_core::{DiffEngine, DiffType, IgnoreRules};

#[derive(Args)]
pub struct DiffArgs {
    /// Before (source) folder.
    #[arg(short = 'l', long, value_name = "PATH")]
    pub left: PathBuf,
    /// After (target) folder.
    #[arg(short = 'r', long, value_name = "PATH")]
    pub right: PathBuf,
    /// Path to .aaaiignore file.
    #[arg(long, value_name = "FILE")]
    pub ignore: Option<PathBuf>,
    /// Show actual diff content for Modified text files.
    #[arg(long)]
    pub content: bool,
    /// Show Unchanged files too.
    #[arg(long)]
    pub all: bool,
    /// Output as JSON.
    #[arg(long = "json-output")]
    pub json_output: bool,
}

pub fn run(args: DiffArgs) -> anyhow::Result<()> {
    let ignore_path = args.ignore.clone()
        .unwrap_or_else(|| args.left.join(".aaaiignore"));
    let ignore = IgnoreRules::load(&ignore_path)?;
    let diffs = DiffEngine::compare_with_ignore(&args.left, &args.right, &ignore)?;

    if args.json_output {
        let out: Vec<_> = diffs.iter()
            .filter(|d| args.all || d.diff_type != DiffType::Unchanged)
            .map(|d| serde_json::json!({
                "path":      d.path,
                "diff_type": d.diff_type.to_string(),
                "is_binary": d.is_binary,
                "before_size": d.before_size,
                "after_size":  d.after_size,
                "lines_added":   d.stats.as_ref().map(|s| s.lines_added),
                "lines_removed": d.stats.as_ref().map(|s| s.lines_removed),
            }))
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("{}", "aaai diff".bold());
    println!("Before : {}", args.left.display());
    println!("After  : {}", args.right.display());
    println!();

    let mut added = 0usize; let mut removed = 0usize;
    let mut modified = 0usize; let mut unchanged = 0usize;

    for d in &diffs {
        match d.diff_type {
            DiffType::Added    => added    += 1,
            DiffType::Removed  => removed  += 1,
            DiffType::Modified => modified += 1,
            DiffType::Unchanged => { unchanged += 1; if !args.all { continue; } }
            _ => {}
        }

        let (icon, color_fn): (&str, fn(&str) -> colored::ColoredString) = match d.diff_type {
            DiffType::Added        => ("+", |s| s.green()),
            DiffType::Removed      => ("-", |s| s.red()),
            DiffType::Modified     => ("~", |s| s.yellow()),
            DiffType::Unchanged    => (" ", |s| s.dimmed()),
            DiffType::TypeChanged  => ("T", |s| s.magenta()),
            DiffType::Unreadable   => ("!", |s| s.red()),
            DiffType::Incomparable => ("?", |s| s.magenta()),
        };

        let stats_str = d.stats.as_ref()
            .map(|s| format!(" +{} -{}", s.lines_added, s.lines_removed))
            .unwrap_or_default();
        let size_str = d.size_change_label()
            .map(|l| format!("  [{l}]"))
            .unwrap_or_default();
        let binary_str = if d.is_binary { "  [binary]" } else { "" };

        println!("{} {}{stats_str}{size_str}{binary_str}",
            color_fn(icon),
            color_fn(&d.path));

        // Optionally show diff content
        if args.content && d.diff_type == DiffType::Modified && !d.is_binary {
            let before = d.before_text.as_deref().unwrap_or("");
            let after  = d.after_text.as_deref().unwrap_or("");
            let td = TextDiff::from_lines(before, after);
            for change in td.iter_all_changes() {
                let line = change.value().trim_end_matches('\n');
                match change.tag() {
                    ChangeTag::Insert => println!("  {}", format!("+{line}").green()),
                    ChangeTag::Delete => println!("  {}", format!("-{line}").red()),
                    ChangeTag::Equal  => {}
                }
            }
        }
    }

    println!();
    println!("  {} added  {} removed  {} modified  {} unchanged",
        added.to_string().green(),
        removed.to_string().red(),
        modified.to_string().yellow(),
        unchanged.to_string().dimmed());

    Ok(())
}
