//! `aaai snap` — Phase 3: --template flag, ignore file support.

use std::path::PathBuf;
use clap::Args;
use colored::Colorize;

use aaai_core::{
    AuditDefinition, DiffEngine, DiffType, IgnoreRules,
    config::{definition::{AuditEntry, AuditStrategy}, io as config_io},
    templates::library as tmpl,
};

#[derive(Args)]
pub struct SnapArgs {
    #[arg(short = 'l', long, value_name = "PATH")]
    pub left: PathBuf,
    #[arg(short = 'r', long, value_name = "PATH")]
    pub right: PathBuf,
    #[arg(short = 'o', long, value_name = "FILE")]
    pub out: PathBuf,
    /// Merge new entries into an existing definition file.
    #[arg(long)]
    pub merge: bool,
    /// Apply a named rule template to all generated entries.
    /// Use `aaai snap --list-templates` to see available templates.
    #[arg(long, value_name = "TEMPLATE_ID")]
    pub template: Option<String>,
    /// List available rule templates and exit.
    #[arg(long)]
    pub list_templates: bool,
    /// Path to .aaaiignore file.
    #[arg(long, value_name = "FILE")]
    pub ignore: Option<PathBuf>,
    /// Preview what would be generated without writing the file.
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: SnapArgs) -> anyhow::Result<()> {
    if args.list_templates {
        println!("{}", "Available rule templates:".bold());
        println!();
        for t in tmpl::TEMPLATES {
            println!("  {:20}  {}  —  {}", t.id, t.name_ja, t.name);
            println!("  {:20}  {}", "", t.description);
            println!();
        }
        return Ok(());
    }

    println!("{}", "aaai snap".bold());

    // Resolve template strategy
    let template_strategy: Option<AuditStrategy> = match &args.template {
        Some(id) => {
            let t = tmpl::find(id)
                .ok_or_else(|| anyhow::anyhow!(
                    "Unknown template {:?}. Run `aaai snap --list-templates` to list available.", id
                ))?;
            Some((t.strategy)())
        }
        None => None,
    };

    // Load ignore rules
    let ignore_path = args.ignore.clone()
        .unwrap_or_else(|| args.left.join(".aaaiignore"));
    let ignore = IgnoreRules::load(&ignore_path)?;

    // Load or create definition
    let mut definition = if args.merge && args.out.exists() {
        println!("Merging into: {}", args.out.display());
        config_io::load(&args.out)?
    } else {
        AuditDefinition::new_empty()
    };

    let diffs = DiffEngine::compare_with_ignore(&args.left, &args.right, &ignore)?;
    let mut added = 0usize;
    let mut skipped = 0usize;

    for diff in &diffs {
        if diff.diff_type == DiffType::Unchanged
            || diff.diff_type == DiffType::Incomparable
            || diff.is_dir
        {
            continue;
        }

        if args.merge {
            if let Some(existing) = definition.find_entry(&diff.path) {
                if !existing.reason.trim().is_empty() {
                    skipped += 1;
                    continue;
                }
            }
        }

        let strategy = template_strategy.clone().unwrap_or(AuditStrategy::None);
        let entry = AuditEntry {
            path: diff.path.clone(),
            diff_type: diff.diff_type,
            reason: String::new(),
            strategy,
            enabled: true,
            ticket: None,
            approved_by: None,
            approved_at: None,
            expires_at: None,
            note: None,
            created_at: None,
            updated_at: None,
        };
        definition.upsert_entry(entry);
        added += 1;
    }

    if args.dry_run {
        println!("{}", "--- DRY RUN (not written) ---".cyan().bold());
        println!("Would write to: {}", args.out.display());
        for e in &definition.entries {
            println!("  {} {}  ({})", "entry:".dimmed(), e.path, e.diff_type);
        }
        println!("{}", format!("--- {} entries would be added, {} skipped ---", added, skipped).dimmed());
        return Ok(());
    }

    config_io::save(&definition, &args.out, false)?;

    println!("{} snapshot generated: {}", "✓".green(), args.out.display());
    println!("  {} entries added, {} skipped (already have a reason)", added.to_string().yellow(), skipped);
    if let Some(id) = &args.template {
        println!("  Template applied: {}", id.cyan());
    }
    println!();
    println!("{}", "Next: fill in the 'reason' field for each entry, then run `aaai audit`.".dimmed());
    Ok(())
}
