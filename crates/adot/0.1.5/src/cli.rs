use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "adot")]
#[command(about = "A minimal dotfile manager")]
pub struct Cli {
    /// Path to config.yaml (overrides default lookup)
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Profile name (defaults to hostname)
    #[arg(short, long, global = true)]
    pub profile: Option<String>,

    /// Suppress output
    #[arg(short, long, global = true)]
    pub silent: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Install dotfiles for the current host's profile
    Install,
}

/// Resolve the profile name. Uses -p flag if given, otherwise hostname.
pub fn resolve_profile(overwrite: Option<&str>) -> Result<String, String> {
    if let Some(name) = overwrite {
        return Ok(name.to_string());
    }

    hostname().ok_or_else(|| "failed to get hostname".to_string())
}

fn hostname() -> Option<String> {
    let output = std::process::Command::new("hostname")
        .arg("-s")
        .output()
        .ok()?;

    let name = String::from_utf8(output.stdout).ok()?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }

    Some(name.to_string())
}
