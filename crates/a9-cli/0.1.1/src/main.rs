use std::{collections::HashMap, env, fs, path::PathBuf, process};

use clap::Parser;
use serde::{Deserialize, Serialize};

const ORG: &str = "ars-vivendi";

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
}

#[derive(Deserialize)]
struct GhTag {
    name: String,
}

#[derive(Deserialize)]
struct Crates2 {
    installs: HashMap<String, serde_json::Value>,
}

/// Arguments for the install subcommand
#[derive(Parser)]
struct InstallArgs {
    /// Tool short name (e.g. `lint` or `a9-lint` installs a9-lint from ars-vivendi)
    tool: String,
    /// Tag to install; defaults to latest release
    #[arg(long)]
    tag: Option<String>,
    /// Force reinstall even if already up to date
    #[arg(long)]
    force: bool,
    /// Use Cargo.lock from the repository
    #[arg(long)]
    locked: bool,
}

/// Arguments for the uninstall subcommand
#[derive(Parser)]
struct UninstallArgs {
    /// Tool short name (e.g. `lint` or `a9-lint`)
    tool: String,
}

/// Arguments for the update subcommand
#[derive(Parser)]
struct UpdateArgs {
    /// Tool short name; omit to update all installed a9-* tools
    tool: Option<String>,
    /// Use Cargo.lock from the repository
    #[arg(long)]
    locked: bool,
}

#[derive(Parser)]
enum Commands {
    /// Install an a9 tool from github.com/ars-vivendi
    Install(InstallArgs),
    /// Uninstall an a9 tool
    Uninstall(UninstallArgs),
    /// Update a9 tool(s) to latest release; omit tool to update all
    Update(UpdateArgs),
    /// List installed a9 tools
    List,
}

/// CLI for a9 tools
#[derive(Parser)]
struct Cli {
    /// Output result as JSON
    #[arg(global = true, long)]
    json: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Serialize)]
struct CommandResult {
    success: bool,
    message: String,
}

fn crate_name(tool: &str) -> String {
    let normalized = tool.replace('_', "-");
    let stripped = normalized.strip_prefix("a9-").unwrap_or(&normalized);

    format!("a9-{stripped}")
}

fn get_token() -> Result<String, String> {
    env::var("A9_GITHUB_TOKEN").map_err(|_| "A9_GITHUB_TOKEN is not set".to_string())
}

fn authed_url(repo: &str, token: &str) -> String {
    format!("https://x-access-token:{token}@github.com/{ORG}/{repo}")
}

fn latest_tag(repo: &str, token: &str) -> Result<String, String> {
    let releases_url = format!("https://api.github.com/repos/{ORG}/{repo}/releases/latest");
    let resp = ureq::get(&releases_url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("User-Agent", "a9-cli")
        .call();

    match resp {
        Ok(r) => {
            return r
                .into_json::<GhRelease>()
                .map(|r| r.tag_name)
                .map_err(|e| format!("failed to parse GitHub response: {e}"));
        }
        Err(ureq::Error::Status(404, _)) => {}
        Err(e) => return Err(format!("GitHub API error: {e}")),
    }

    let tags_url = format!("https://api.github.com/repos/{ORG}/{repo}/tags");
    let tags: Vec<GhTag> = ureq::get(&tags_url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("User-Agent", "a9-cli")
        .call()
        .map_err(|e| format!("GitHub tags API error: {e}"))?
        .into_json()
        .map_err(|e| format!("failed to parse tags response: {e}"))?;

    tags.into_iter()
        .next()
        .map(|t| t.name)
        .ok_or_else(|| format!("no tags found for {repo}"))
}

fn cargo_install(repo: &str, tag: &str, force: bool, locked: bool, token: &str) -> bool {
    let url = authed_url(repo, token);

    let mut args = vec!["install", "--git", url.as_str(), "--tag", tag];

    if force {
        args.push("--force");
    }

    if locked {
        args.push("--locked");
    }

    process::Command::new("cargo")
        .args(&args)
        .status()
        .is_ok_and(|s| s.success())
}

fn cargo_uninstall(repo: &str) -> bool {
    process::Command::new("cargo")
        .args(["uninstall", repo])
        .status()
        .is_ok_and(|s| s.success())
}

fn cargo_home() -> PathBuf {
    env::var("CARGO_HOME").map_or_else(
        |_| PathBuf::from(env::var("HOME").unwrap_or_default()).join(".cargo"),
        PathBuf::from,
    )
}

fn installed_a9_tools() -> Vec<(String, String)> {
    let content = fs::read_to_string(cargo_home().join(".crates2.json")).unwrap_or_default();

    let Ok(data) = serde_json::from_str::<Crates2>(&content) else {
        return vec![];
    };

    data.installs
        .keys()
        .filter_map(|key| {
            let mut parts = key.splitn(3, ' ');

            let name = parts.next()?;
            let version = parts.next()?;

            name.starts_with("a9-")
                .then(|| (name.to_string(), version.to_string()))
        })
        .collect()
}

fn handle_install(args: &InstallArgs) -> Result<String, String> {
    let token = get_token()?;
    let repo = crate_name(&args.tool);

    let tag = match &args.tag {
        Some(t) => t.clone(),
        None => latest_tag(&repo, &token)?,
    };

    if cargo_install(&repo, &tag, args.force, args.locked, &token) {
        Ok(format!("{repo} {tag} installed"))
    } else {
        Err(format!("cargo install {repo} failed"))
    }
}

fn handle_uninstall(args: &UninstallArgs) -> Result<String, String> {
    let repo = crate_name(&args.tool);

    if cargo_uninstall(&repo) {
        Ok(format!("{repo} uninstalled"))
    } else {
        Err(format!("cargo uninstall {repo} failed"))
    }
}

fn handle_update(args: &UpdateArgs) -> Result<String, String> {
    let token = get_token()?;

    let repos: Vec<String> = if let Some(t) = &args.tool {
        vec![crate_name(t)]
    } else {
        let installed = installed_a9_tools();

        if installed.is_empty() {
            return Ok("no a9 tools installed".to_string());
        }

        installed.into_iter().map(|(name, _)| name).collect()
    };

    let mut failures = vec![];

    for repo in &repos {
        match latest_tag(repo, &token) {
            Err(e) => failures.push(format!("{repo}: {e}")),
            Ok(tag) => {
                if !cargo_install(repo, &tag, false, args.locked, &token) {
                    failures.push(format!("{repo}: cargo install failed"));
                }
            }
        }
    }

    if failures.is_empty() {
        Ok(format!("updated {} tool(s)", repos.len()))
    } else {
        Err(failures.join("; "))
    }
}

fn handle_list() -> String {
    let tools = installed_a9_tools();

    if tools.is_empty() {
        "no a9 tools installed".to_string()
    } else {
        tools
            .into_iter()
            .map(|(name, version)| format!("{name} {version}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Install(args) => handle_install(args),
        Commands::Uninstall(args) => handle_uninstall(args),
        Commands::Update(args) => handle_update(args),
        Commands::List => Ok(handle_list()),
    };

    let success = result.is_ok();

    let message = match result {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {e}");

            e
        }
    };

    if cli.json {
        println!(
            "{}",
            serde_json::to_string(&CommandResult { success, message }).unwrap()
        );
    } else if success {
        println!("{message}");
    }

    if !success {
        process::exit(101);
    }
}
