//! `aaai check` — validate an audit definition file.

use std::path::PathBuf;
use clap::Args;
use colored::Colorize;

use aaai_core::config::io as config_io;

#[derive(Args)]
pub struct CheckArgs {
    /// Audit definition file to validate.
    #[arg(value_name = "FILE")]
    pub file: PathBuf,
    /// Show all entries (default: show only invalid ones).
    #[arg(long)]
    pub all: bool,
}

pub fn run(args: CheckArgs) -> anyhow::Result<()> {
    println!("{}", "aaai check".bold());
    println!("File: {}", args.file.display());
    println!();

    let def = config_io::load(&args.file).map_err(|e| {
        eprintln!("{}", format!("INVALID: {e}").red().bold());
        std::process::exit(4);
    })?;

    println!("Version : {}", def.version);
    println!("Entries : {}", def.entries.len());

    let expired       = def.expired_entries();
    let expiring_soon = def.expiring_soon(30);
    if !expired.is_empty() {
        println!("{}", format!("⚠  {} expired entries", expired.len()).yellow().bold());
    }
    if !expiring_soon.is_empty() {
        println!("{}", format!("⏰ {} entries expiring within 30 days", expiring_soon.len()).yellow());
    }
    println!();

    let mut issues = 0usize;
    for entry in &def.entries {
        let result = entry.is_approvable();
        let show = args.all || result.is_err();
        if !show { continue; }

        let icon = if result.is_ok() { "✓".green() } else { "✗".red().bold() };
        let reason_tag = if entry.reason.trim().is_empty() { " (no reason)".yellow().to_string() } else { String::new() };
        let ticket_tag = entry.ticket.as_deref().map(|t| format!(" [{}]", t)).unwrap_or_default();
        println!("{icon}  {}{ticket_tag}{reason_tag}", entry.path);

        if let Err(msg) = &result {
            println!("   {}", msg.red());
            issues += 1;
        }
        if entry.is_expired() {
            println!("   {}", format!("⚠ expired: {}", entry.expires_at.unwrap()).yellow());
        }
    }

    if issues == 0 {
        println!("{}", "All entries are valid.".green());
    } else {
        println!("{}", format!("{issues} entry/entries have validation errors.").red().bold());
        std::process::exit(1);
    }
    Ok(())
}
