//! `aaai version` — detailed version and build information.

use clap::Args;
use colored::Colorize;

#[derive(Args)]
pub struct VersionArgs {
    /// Output as JSON.
    #[arg(long = "json-output")]
    pub json_output: bool,
}

pub fn run(args: VersionArgs) -> anyhow::Result<()> {
    let version   = env!("CARGO_PKG_VERSION");
    let pkg_name  = env!("CARGO_PKG_NAME");
    let authors   = env!("CARGO_PKG_AUTHORS");
    let license   = env!("CARGO_PKG_LICENSE");
    let repo      = env!("CARGO_PKG_REPOSITORY");
    // Rust version is baked in at compile time

    if args.json_output {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "name":    pkg_name,
            "version": version,
            "authors": authors,
            "license": license,
            "repository": repo,
        }))?);
        return Ok(());
    }

    println!("{} {}", "aaai".bold().cyan(), format!("v{version}").bold());
    println!();
    println!("  Authors    : {authors}");
    println!("  License    : {license}");
    println!("  Repository : {repo}");
    println!();
    println!("  Build profile : {}", if cfg!(debug_assertions) { "dev" } else { "release" });
    println!("  Target OS     : {}", std::env::consts::OS);
    println!("  Target arch   : {}", std::env::consts::ARCH);
    Ok(())
}
