// SPDX-License-Identifier: Apache-2.0

//! `achat` — minimal LAN agent-to-agent messaging CLI.
//!
//! Provides subcommands for starting/stopping daemons, sending messages,
//! browsing inboxes, and managing group membership.

use achat::{daemon, ipc, protocol, storage, util};
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use protocol::IpcResponse;
use std::io::{IsTerminal, Read};
use std::process::ExitCode;

fn sock_path(name: &str) -> std::path::PathBuf {
    storage::agent_dir(name).join("daemon.sock")
}

/// Persist the current agent identity to `~/.achat/current`.
fn set_current_identity(name: &str) {
    let _ = std::fs::create_dir_all(storage::base_dir());
    let _ = std::fs::write(storage::base_dir().join("current"), name);
}

/// Send IPC request, returning the response or an error.
fn ipc_call(name: &str, req: &protocol::IpcRequest) -> Result<IpcResponse> {
    let resp = ipc::send_request(&sock_path(name), req)?;
    if let IpcResponse::Error(e) = &resp {
        bail!("{e}");
    }
    Ok(resp)
}

/// Resolve identity: `--as` > `ACHAT_NAME` env > `~/.achat/current` > auto-detect.
fn resolve_identity(explicit: Option<&str>) -> Result<String> {
    if let Some(name) = explicit {
        return Ok(name.to_string());
    }
    if let Ok(name) = std::env::var("ACHAT_NAME") {
        if !name.is_empty() {
            return Ok(name);
        }
    }
    let current_file = storage::base_dir().join("current");
    if let Ok(name) = std::fs::read_to_string(&current_file) {
        let name = name.trim().to_string();
        if !name.is_empty() && daemon_alive(&name) {
            return Ok(name);
        }
    }
    let alive = list_alive_daemons();
    match alive.len() {
        1 => Ok(alive.into_iter().next().unwrap()),
        n if n > 1 => bail!(
            "multiple daemons running ({})\n  use: achat attach <name>",
            alive.join(", ")
        ),
        _ => bail!("no daemon running. use: achat up <name>"),
    }
}

fn daemon_alive(name: &str) -> bool {
    let pid_file = storage::agent_dir(name).join("daemon.pid");
    std::fs::read_to_string(&pid_file)
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .is_some_and(util::is_process_alive)
}

fn list_alive_daemons() -> Vec<String> {
    let agents_dir = storage::base_dir().join("agents");
    let Ok(entries) = std::fs::read_dir(&agents_dir) else {
        return vec![];
    };
    entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_string();
            daemon_alive(&name).then_some(name)
        })
        .collect()
}

fn read_content(message: &[String]) -> Result<String> {
    if !message.is_empty() {
        return Ok(message.join(" "));
    }
    if std::io::stdin().is_terminal() {
        bail!("no message provided. pass as argument or pipe via stdin");
    }
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .context("read stdin")?;
    if buf.is_empty() {
        bail!("empty message from stdin");
    }
    Ok(buf)
}

// --- CLI definition ---

#[derive(Parser)]
#[command(name = "achat", about = "LAN agent-to-agent messaging", version)]
struct Cli {
    #[arg(long = "as", global = true)]
    identity: Option<String>,
    #[arg(long, global = true)]
    pretty: bool,
    #[arg(long, global = true)]
    hint: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start daemon and register on LAN
    Up { name: String },
    /// Stop daemon
    Down,
    /// Rebind identity to a running daemon
    Attach { name: String },
    /// List online agents
    Ls,
    /// Send message to @agent or group
    Send {
        target: String,
        message: Vec<String>,
    },
    /// Broadcast message to all agents
    Cast { message: Vec<String> },
    /// Show direct messages
    Inbox {
        #[arg(short = 'n', long, default_value = "20")]
        limit: usize,
    },
    /// Show broadcast messages
    Feed {
        #[arg(short = 'n', long, default_value = "20")]
        limit: usize,
    },
    /// Join a named group
    Join { group: String },
    /// Leave a group
    Leave { group: String },
    /// Show message history
    Log {
        target: Option<String>,
        #[arg(short = 'n', long, default_value = "50")]
        limit: usize,
    },
    /// Show daemon status
    Status,
    /// Show commands as JSON for agents
    HelpJson,
    #[command(hide = true)]
    Daemon {
        #[arg(long)]
        name: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    if let Err(err) = run(&cli) {
        eprintln!("error: {err:#}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run(cli: &Cli) -> Result<()> {
    // Commands that don't require identity resolution.
    match &cli.command {
        Commands::Up { name } => return cmd_up(name),
        Commands::Attach { name } => return cmd_attach(name),
        Commands::HelpJson => {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({"commands": protocol::command_list()}))
                    .unwrap()
            );
            return Ok(());
        }
        Commands::Daemon { name } => {
            daemon::run(name);
            return Ok(());
        }
        Commands::Down
        | Commands::Ls
        | Commands::Send { .. }
        | Commands::Cast { .. }
        | Commands::Inbox { .. }
        | Commands::Feed { .. }
        | Commands::Log { .. }
        | Commands::Join { .. }
        | Commands::Leave { .. }
        | Commands::Status => {}
    }
    // Everything else requires a resolved identity.
    let name = resolve_identity(cli.identity.as_deref())?;
    match &cli.command {
        Commands::Down => cmd_down(&name),
        Commands::Ls => cmd_ls(&name, cli),
        Commands::Send { target, message } => cmd_send(&name, cli, target, message),
        Commands::Cast { message } => cmd_send_or_cast(&name, cli, None, message),
        Commands::Inbox { limit } => {
            cmd_messages(&name, cli, &protocol::IpcRequest::Inbox { limit: *limit })
        }
        Commands::Feed { limit } => {
            cmd_messages(&name, cli, &protocol::IpcRequest::Feed { limit: *limit })
        }
        Commands::Log { target, limit } => cmd_messages(
            &name,
            cli,
            &protocol::IpcRequest::Log {
                target: target.clone(),
                limit: *limit,
            },
        ),
        Commands::Join { group } => cmd_group(&name, cli, group, GroupAction::Join),
        Commands::Leave { group } => cmd_group(&name, cli, group, GroupAction::Leave),
        Commands::Status => cmd_status(&name, cli),
        Commands::Up { .. }
        | Commands::Attach { .. }
        | Commands::HelpJson
        | Commands::Daemon { .. } => unreachable!(),
    }
}

// --- Commands ---

fn cmd_up(name: &str) -> Result<()> {
    if daemon_alive(name) {
        bail!("{name} is already running");
    }
    let pid_file = storage::agent_dir(name).join("daemon.pid");
    let _ = std::fs::remove_file(&pid_file);

    let exe = std::env::current_exe().context("current exe")?;
    std::process::Command::new(exe)
        .args(["daemon", "--name", name])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .with_context(|| format!("spawn daemon for {name}"))?;

    std::thread::sleep(std::time::Duration::from_millis(500));
    if !sock_path(name).exists() {
        bail!("daemon failed to start. check ~/.achat/agents/{name}/daemon.log");
    }
    set_current_identity(name);
    println!("{}", serde_json::json!({"ok": true, "name": name}));
    Ok(())
}

fn cmd_down(name: &str) -> Result<()> {
    if !sock_path(name).exists() {
        bail!("{name} is not running");
    }
    let _ = ipc_call(name, &protocol::IpcRequest::Shutdown)?;
    let current_file = storage::base_dir().join("current");
    if std::fs::read_to_string(&current_file)
        .map(|s| s.trim() == name)
        .unwrap_or(false)
    {
        let _ = std::fs::remove_file(&current_file);
    }
    println!("{}", serde_json::json!({"ok": true}));
    Ok(())
}

fn cmd_attach(name: &str) -> Result<()> {
    if !daemon_alive(name) {
        bail!("no daemon running for {name}");
    }
    set_current_identity(name);
    println!("{}", serde_json::json!({"ok": true, "name": name}));
    Ok(())
}

fn cmd_ls(name: &str, cli: &Cli) -> Result<()> {
    let IpcResponse::Agents(agents) = ipc_call(name, &protocol::IpcRequest::ListAgents)? else {
        bail!("unexpected response");
    };
    if cli.pretty {
        for a in &agents {
            let g = if a.groups.is_empty() {
                String::new()
            } else {
                format!("  [{}]", a.groups.join(","))
            };
            println!("  {:<12} {}:{}{}", a.name, a.addr, a.port, g);
        }
    } else {
        for a in &agents {
            println!("{}", serde_json::to_string(a).unwrap());
        }
    }
    Ok(())
}

fn cmd_send(name: &str, cli: &Cli, target: &str, message: &[String]) -> Result<()> {
    cmd_send_or_cast(name, cli, Some(target), message)
}

fn cmd_send_or_cast(name: &str, cli: &Cli, target: Option<&str>, message: &[String]) -> Result<()> {
    let content = read_content(message)?;
    let to = match target {
        Some(t) if t.starts_with('@') => protocol::Target::Direct(t[1..].to_string()),
        Some(t) => protocol::Target::Group(t.to_string()),
        None => protocol::Target::Broadcast,
    };
    let is_broadcast = matches!(to, protocol::Target::Broadcast);
    let req = protocol::IpcRequest::Send { to, content };
    let IpcResponse::Ok { id } = ipc_call(name, &req)? else {
        bail!("unexpected response");
    };
    if cli.pretty {
        println!(
            "{}",
            if is_broadcast {
                "broadcast sent."
            } else {
                "sent."
            }
        );
    } else {
        let mut resp = serde_json::json!({"ok": true});
        if let Some(id) = id {
            resp["id"] = id.into();
        }
        if cli.hint {
            let hint = if is_broadcast {
                "achat feed"
            } else {
                "achat inbox"
            };
            resp["hint"] = serde_json::json!([hint]);
        }
        println!("{}", serde_json::to_string(&resp).unwrap());
    }
    Ok(())
}

fn cmd_messages(name: &str, cli: &Cli, req: &protocol::IpcRequest) -> Result<()> {
    let IpcResponse::Messages {
        msgs,
        total,
        truncated,
    } = ipc_call(name, req)?
    else {
        bail!("unexpected response");
    };
    if cli.pretty {
        for m in &msgs {
            let ts = &m.ts[11..16];
            let who = match &m.to {
                protocol::Target::Direct(_) => format!("{} → you", m.from),
                protocol::Target::Group(g) => format!("{} → #{}", m.from, g),
                protocol::Target::Broadcast => format!("{} → all", m.from),
            };
            println!("[{ts}] {who}: {}", m.content);
        }
        if truncated {
            eprintln!("... showing {}/{total}, use -n {total} for all", msgs.len());
        }
    } else {
        for m in &msgs {
            println!("{}", serde_json::to_string(m).unwrap());
        }
        if truncated {
            println!(
                "{}",
                serde_json::json!({"_truncated": true, "showing": msgs.len(), "total": total, "hint": format!("use -n {} for more", total)})
            );
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum GroupAction {
    Join,
    Leave,
}

fn cmd_group(name: &str, cli: &Cli, group: &str, action: GroupAction) -> Result<()> {
    let req = match action {
        GroupAction::Join => protocol::IpcRequest::JoinGroup {
            group: group.to_string(),
        },
        GroupAction::Leave => protocol::IpcRequest::LeaveGroup {
            group: group.to_string(),
        },
    };
    let _ = ipc_call(name, &req)?;
    if cli.pretty {
        let verb = match action {
            GroupAction::Join => "joined",
            GroupAction::Leave => "left",
        };
        println!("{verb} {group}");
    } else {
        println!("{}", serde_json::json!({"ok": true}));
    }
    Ok(())
}

fn cmd_status(name: &str, cli: &Cli) -> Result<()> {
    let IpcResponse::Status {
        name: n,
        uptime_secs,
        groups,
        peers,
    } = ipc_call(name, &protocol::IpcRequest::Status)?
    else {
        bail!("unexpected response");
    };
    if cli.pretty {
        let g = if groups.is_empty() {
            "none".into()
        } else {
            groups.join(", ")
        };
        println!("agent:   {n}\nuptime:  {uptime_secs}s\npeers:   {peers}\ngroups:  {g}");
    } else {
        println!(
            "{}",
            serde_json::json!({"name": n, "uptime_secs": uptime_secs, "groups": groups, "peers": peers})
        );
    }
    Ok(())
}
