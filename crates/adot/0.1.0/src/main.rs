use std::process;

use clap::Parser;

use adot::cli::{self, Cli, Command};
use adot::config::Config;
use adot::installer::Installer;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(&cli) {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<(), String> {
    let config = Config::load(cli.config.as_ref())?;
    let profile = cli::resolve_profile(cli.profile.as_deref())?;

    let config_dir = match cli.config.as_ref() {
        Some(path) => path
            .parent()
            .ok_or_else(|| "config path has no parent".to_string())?
            .canonicalize()
            .map_err(|e| format!("failed to resolve config dir: {e}"))?,
        None => config
            .dotpath
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf(),
    };

    match &cli.command {
        Command::Install => {
            let installer = Installer::new(config, profile, config_dir, cli.silent);
            installer.install()?;
        }
    }

    Ok(())
}
