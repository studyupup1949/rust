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
    let (config, config_dir) = Config::load(cli.config.as_ref())?;
    let profile = cli::resolve_profile(cli.profile.as_deref())?;

    match &cli.command {
        Command::Install => {
            let installer = Installer::new(config, profile, config_dir, cli.silent);
            installer.install()?;
        }
    }

    Ok(())
}
