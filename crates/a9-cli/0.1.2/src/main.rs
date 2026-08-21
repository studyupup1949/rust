use std::{collections::HashMap, env, fs, path::PathBuf, process};

use clap::Parser;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

const ORG: &str = "ars-vivendi";

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
    /// Tool name, optionally with semver requirement: `lint`, `a9-lint`, `lint@0.1.23`, `lint@^0.1`
    tool: String,
    /// Semver requirement; overrides @version in tool name (e.g. `^0.1`, `>=0.1.20`, `0.1.23`)
    #[arg(long, value_name = "REQ")]
    version: Option<String>,
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

fn fetch_tags(repo: &str, token: &str) -> Result<Vec<String>, String> {
    let url = format!("https://api.github.com/repos/{ORG}/{repo}/tags?per_page=100");
    let tags: Vec<GhTag> = ureq::get(&url)
        .set("Authorization", &format!("Bearer {token}"))
        .set("User-Agent", "a9-cli")
        .call()
        .map_err(|e| format!("GitHub tags API error: {e}"))?
        .into_json()
        .map_err(|e| format!("failed to parse tags response: {e}"))?;

    Ok(tags.into_iter().map(|t| t.name).collect())
}

fn resolve_tag(repo: &str, token: &str, req: Option<&str>) -> Result<String, String> {
    let tags = fetch_tags(repo, token)?;

    if tags.is_empty() {
        return Err(format!("no tags found for {repo}"));
    }

    let Some(req_str) = req else {
        return Ok(tags.into_iter().next().unwrap());
    };

    let vreq = VersionReq::parse(req_str)
        .map_err(|e| format!("invalid version requirement '{req_str}': {e}"))?;

    let mut candidates: Vec<Version> = tags
        .iter()
        .filter_map(|t| Version::parse(t.trim_start_matches('v')).ok())
        .filter(|v| vreq.matches(v))
        .collect();

    candidates.sort();

    candidates
        .into_iter()
        .next_back()
        .map(|v| format!("v{v}"))
        .ok_or_else(|| format!("no tag matching '{req_str}' found for {repo}"))
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
    let (tool_name, inline_req) = args
        .tool
        .split_once('@')
        .map_or((args.tool.as_str(), None), |(n, r)| (n, Some(r)));
    let repo = crate_name(tool_name);
    let req = args.version.as_deref().or(inline_req);
    let tag = resolve_tag(&repo, &token, req)?;

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
        match resolve_tag(repo, &token, None) {
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
