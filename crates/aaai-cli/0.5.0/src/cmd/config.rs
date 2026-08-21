//! `aaai config` — show or initialise the project `.aaai.yaml`.

use std::path::PathBuf;
use clap::Args;
use colored::Colorize;

use aaai_core::project::config::{ProjectConfig, CONFIG_FILENAME};

#[derive(Args)]
pub struct ConfigArgs {
    /// Initialise a new .aaai.yaml in the current directory.
    #[arg(long)]
    pub init: bool,
    /// Directory for init (default: current directory).
    #[arg(long, value_name = "DIR")]
    pub dir: Option<PathBuf>,
    /// Show the discovered config (default: search from current directory).
    #[arg(long)]
    pub show: bool,
}

pub fn run(args: ConfigArgs) -> anyhow::Result<()> {
    println!("{}", "aaai config".bold());

    let target_dir = args.dir.clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    if args.init {
        let out = target_dir.join(CONFIG_FILENAME);
        if out.exists() && !args.dir.is_some() {
            // Ask for confirmation in practice; for CI safety just report.
            println!("{}", format!("⚠  {} already exists: {}", CONFIG_FILENAME, out.display()).yellow());
            println!("   Delete it first if you want to re-initialise.");
            return Ok(());
        }
        std::fs::create_dir_all(&target_dir)?;
        std::fs::write(&out, ProjectConfig::starter_yaml())?;
        println!("{} Created: {}", "✓".green(), out.display());
        println!();
        println!("Edit the file to configure your project defaults.");
        return Ok(());
    }

    // Discover and display.
    match ProjectConfig::discover(&target_dir)? {
        Some((cfg, dir)) => {
            println!("Found: {}", dir.join(CONFIG_FILENAME).display());
            println!();
            println!("  default_definition : {}",
                cfg.default_definition.as_deref().unwrap_or("(not set)"));
            println!("  default_ignore     : {}",
                cfg.default_ignore.as_deref().unwrap_or("(not set)"));
            println!("  approver_name      : {}",
                cfg.approver_name.as_deref().unwrap_or("(not set)"));
            println!("  mask_secrets       : {}", cfg.mask_secrets);
            if !cfg.custom_mask_patterns.is_empty() {
                println!("  custom_mask_patterns:");
                for p in &cfg.custom_mask_patterns {
                    println!("    - {p}");
                }
            }
        }
        None => {
            println!("No {} found (searched from {}).", CONFIG_FILENAME, target_dir.display());
            println!("Run `aaai config --init` to create one.");
        }
    }
    Ok(())
}
