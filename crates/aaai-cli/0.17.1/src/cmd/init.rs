//! `aaai init` — interactive project setup wizard.
//!
//! Guides the user through:
//! 1. Before / after folder paths
//! 2. Audit definition file location
//! 3. Optional .aaaiignore patterns
//! 4. Saving a .aaai.yaml project config
//! 5. Optionally running `snap` to generate the initial definition

use std::io::{self, Write};
use std::path::PathBuf;
use clap::Args;
use colored::Colorize;

use aaai_core::{
    AuditDefinition, DiffEngine, DiffType,
    config::{definition::{AuditEntry, AuditStrategy}, io as config_io},
    project::config::ProjectConfig,
};

#[derive(Args)]
pub struct InitArgs {
    /// Directory to initialise (default: current directory).
    #[arg(long, default_value = ".")]
    pub dir: PathBuf,
    /// Skip interactive prompts and accept defaults.
    #[arg(long)]
    pub non_interactive: bool,
}

pub fn run(args: InitArgs) -> anyhow::Result<()> {
    println!("{}", "aaai init".bold());
    println!("Setting up an audit project in: {}", args.dir.display());
    println!();

    // Check if .aaai.yaml already exists.
    let config_path = args.dir.join(".aaai.yaml");
    if config_path.exists() {
        println!("{}", "⚠  .aaai.yaml already exists.".yellow());
        println!("   Delete it first or use `aaai config` to view/edit.");
        return Ok(());
    }

    if args.non_interactive {
        return non_interactive_init(&args.dir, &config_path);
    }

    // Interactive prompts.
    let before = prompt("Before (source) folder path", "before")?;
    let after  = prompt("After (target) folder path",  "after")?;
    let def    = prompt("Audit definition file",        "audit/audit.yaml")?;
    let approver = prompt("Your name (approver_name, optional)", "")?;
    let mask   = prompt_bool("Enable secret masking by default?", false)?;

    // Build project config.
    let cfg = ProjectConfig {
        version: "1".into(),
        default_definition: Some(def.clone()),
        default_ignore: None,
        approver_name: if approver.is_empty() { None } else { Some(approver) },
        mask_secrets: mask,
        custom_mask_patterns: Vec::new(),
        suppress_warnings: Vec::new(),
    };
    cfg.save(&config_path)?;
    println!("{} .aaai.yaml written.", "✓".green());

    // Create definition parent dir.
    let def_path = args.dir.join(&def);
    if let Some(parent) = def_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Optionally run snap.
    if !def_path.exists() {
        let run_snap = prompt_bool(
            "Before and After paths given. Run `aaai snap` now to generate an initial definition?",
            true,
        )?;
        if run_snap {
            let before_p = PathBuf::from(&before);
            let after_p  = PathBuf::from(&after);
            if before_p.is_dir() && after_p.is_dir() {
                let diffs = DiffEngine::compare(&before_p, &after_p)?;
                let mut definition = AuditDefinition::new_empty();
                for diff in &diffs {
                    if diff.diff_type == DiffType::Unchanged || diff.is_dir { continue; }
                    definition.upsert_entry(AuditEntry {
                        path: diff.path.clone(),
                        diff_type: diff.diff_type,
                        reason: String::new(),
                        strategy: AuditStrategy::None,
                        enabled: true,
                        ticket: None,
                        approved_by: None,
                        approved_at: None,
                        expires_at: None,
                        note: None,
                        created_at: None,
                        updated_at: None,
                    });
                }
                config_io::save(&definition, &def_path, false)?;
                println!("{} Definition template written to {}",
                    "✓".green(), def_path.display());
                println!("  {} entries generated — fill in 'reason' fields before auditing.",
                    definition.entries.len().to_string().yellow());
            } else {
                println!("{}", "Skipped snap — folder paths not found.".yellow());
            }
        }
    }

    println!();
    println!("{}", "Setup complete! Next steps:".bold());
    println!("  1. Fill in 'reason' for each entry in {def}");
    println!("  2. Run:  aaai audit --left {before} --right {after} --config {def}");

    Ok(())
}

fn non_interactive_init(_dir_unused: &PathBuf, config_path: &PathBuf) -> anyhow::Result<()> {
    let cfg = ProjectConfig::default();
    cfg.save(config_path)?;
    println!("{} .aaai.yaml written to {}", "✓".green(), config_path.display());
    println!("Edit the file to configure your project defaults.");
    Ok(())
}

fn prompt(label: &str, default: &str) -> anyhow::Result<String> {
    let default_hint = if default.is_empty() { String::new() }
                       else { format!(" [{default}]") };
    print!("{}{}: ", label.bold(), default_hint);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();
    if trimmed.is_empty() && !default.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed)
    }
}

fn prompt_bool(label: &str, default: bool) -> anyhow::Result<bool> {
    let hint = if default { "[Y/n]" } else { "[y/N]" };
    print!("{} {}: ", label.bold(), hint);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_lowercase();
    Ok(match trimmed.as_str() {
        "y" | "yes" => true,
        "n" | "no"  => false,
        _            => default,
    })
}
