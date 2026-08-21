use add_mcp::source::{infer_name, parse_source};
use add_mcp::types::{Agent, McpServerConfig, PackageManager, Scope, Transport};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "add-mcp",
    version,
    about = "Install MCP servers into AI client configurations"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install an MCP server into agent configs
    Install {
        /// Source: command path, URL, or package name
        source: String,

        /// Package manager (npm, pip, go, cargo)
        #[arg(long = "from", value_name = "MANAGER")]
        from: Option<String>,

        /// Server name (inferred from source if omitted)
        #[arg(short, long)]
        name: Option<String>,

        /// Target agents (can be repeated)
        #[arg(short, long = "agent", value_name = "AGENT")]
        agents: Vec<String>,

        /// Install to all supported agents
        #[arg(long)]
        all: bool,

        /// Install to global (user-level) config
        #[arg(short, long)]
        global: bool,

        /// Transport type for URL sources
        #[arg(short, long, value_name = "TYPE")]
        transport: Option<String>,

        /// HTTP headers (KEY:VALUE, can be repeated)
        #[arg(long = "header", value_name = "KEY:VALUE")]
        headers: Vec<String>,

        /// Environment variables (KEY=VALUE, can be repeated)
        #[arg(short, long = "env", value_name = "KEY=VALUE")]
        env: Vec<String>,

        /// Extra arguments to pass to the command
        #[arg(last = true)]
        extra_args: Vec<String>,

        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// List all supported agents
    ListAgents,

    /// Detect installed agents
    Detect {
        /// Also check local (project-level) configs
        #[arg(long)]
        local: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install {
            source,
            from,
            name,
            agents,
            all,
            global,
            transport,
            headers,
            env,
            extra_args,
            yes: _,
        } => {
            let transport = transport.and_then(|t| match t.to_lowercase().as_str() {
                "http" => Some(Transport::Http),
                "sse" => Some(Transport::Sse),
                _ => None,
            });

            let package_manager = from.and_then(|f| {
                let pm = PackageManager::from_str_loose(&f);
                if pm.is_none() {
                    eprintln!("Error: unknown package manager '{f}'");
                    eprintln!("Available: npm, pip, go, cargo");
                    std::process::exit(2);
                }
                pm
            });

            let parsed = match parse_source(&source, transport, package_manager) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(2);
                }
            };

            // If extra args provided and source is a command, append them
            let parsed = if !extra_args.is_empty() {
                match parsed {
                    add_mcp::Source::Command { command, mut args } => {
                        args.extend(extra_args);
                        add_mcp::Source::Command { command, args }
                    }
                    other => other,
                }
            } else {
                parsed
            };

            let server_name = match name {
                Some(n) => n,
                None => match infer_name(&parsed) {
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(2);
                    }
                },
            };

            let target_agents = if all {
                Agent::ALL.to_vec()
            } else if agents.is_empty() {
                eprintln!("Error: specify at least one --agent or use --all");
                std::process::exit(2);
            } else {
                let mut parsed_agents = Vec::new();
                for a in &agents {
                    match Agent::from_str_loose(a) {
                        Some(agent) => parsed_agents.push(agent),
                        None => {
                            eprintln!("Error: unknown agent '{a}'");
                            eprintln!("Available agents:");
                            for agent in Agent::ALL {
                                eprintln!("  {agent}");
                            }
                            std::process::exit(2);
                        }
                    }
                }
                parsed_agents
            };

            let scope = if global { Scope::Global } else { Scope::Local };

            let parsed_headers: Vec<(String, String)> = headers
                .iter()
                .filter_map(|h| {
                    let (k, v) = h.split_once(':')?;
                    Some((k.trim().to_string(), v.trim().to_string()))
                })
                .collect();

            let parsed_env: Vec<(String, String)> = env
                .iter()
                .filter_map(|e| {
                    let (k, v) = e.split_once('=')?;
                    Some((k.trim().to_string(), v.trim().to_string()))
                })
                .collect();

            let config = McpServerConfig {
                name: server_name,
                source: parsed,
                env: parsed_env,
                headers: parsed_headers,
            };

            let results = add_mcp::install(&config, &target_agents, scope);

            let mut any_error = false;
            for result in results {
                match result {
                    Ok(r) => {
                        let action = if r.already_existed {
                            "updated"
                        } else if r.created {
                            "created"
                        } else {
                            "added"
                        };
                        println!(
                            "{}: {} '{}' in {} ({})",
                            r.agent, action, config.name, r.path, r.scope
                        );
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        any_error = true;
                    }
                }
            }

            if any_error {
                std::process::exit(1);
            }
        }

        Commands::ListAgents => {
            for agent in Agent::ALL {
                let def = add_mcp::agent::agent_def(*agent);
                let local = if def.has_local {
                    " (local+global)"
                } else {
                    " (global only)"
                };
                println!("{agent}{local}");
            }
        }

        Commands::Detect { local } => {
            let detected = add_mcp::detect_agents(local);
            if detected.is_empty() {
                println!("No AI client configs found.");
            } else {
                for d in detected {
                    let servers = if d.has_servers { " [has servers]" } else { "" };
                    println!("{} ({}) — {}{servers}", d.agent, d.scope, d.path);
                }
            }
        }
    }
}
