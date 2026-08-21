//! Aang - Kora Rent Reclaim TUI
//!
//! A terminal user interface for monitoring and reclaiming rent-locked SOL
//! from accounts sponsored by a Kora node.

mod app;
mod config;
mod solana;
mod types;
mod ui;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use app::{App, InputMode, Tab};
use config::Config;

#[derive(Parser)]
#[command(name = "aang")]
#[command(author = "Aang Contributors")]
#[command(version = "0.1.0")]
#[command(about = "Kora Rent Reclaim TUI - Monitor and reclaim rent-locked SOL", long_about = None)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// RPC URL (overrides config)
    #[arg(long, value_name = "URL")]
    rpc_url: Option<String>,

    /// Run in verbose mode
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the TUI (default)
    Tui,

    /// Scan accounts and show status
    Scan {
        /// Account public keys to scan (comma-separated)
        #[arg(short, long)]
        accounts: Option<String>,

        /// Path to file with account pubkeys (one per line)
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Reclaim rent from eligible accounts
    Reclaim {
        /// Account public keys to reclaim (comma-separated)
        #[arg(short, long)]
        accounts: Option<String>,

        /// Reclaim all eligible accounts
        #[arg(long)]
        all: bool,

        /// Dry run - don't actually reclaim
        #[arg(long)]
        dry_run: bool,
    },

    /// Show rent statistics
    Stats {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate example configuration file
    Init {
        /// Output path for config file
        #[arg(short, long, default_value = "aang.toml")]
        output: PathBuf,
    },

    /// Add accounts to track
    Add {
        /// Account public keys (comma-separated)
        accounts: String,
    },

    /// Export tracked accounts
    Export {
        /// Output file path
        #[arg(short, long, default_value = "accounts.json")]
        output: PathBuf,
    },

    /// Import accounts from file
    Import {
        /// Input file path
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .finish();

    // Only set subscriber for non-TUI modes
    let is_tui = cli.command.is_none() || matches!(cli.command, Some(Commands::Tui));
    if !is_tui {
        tracing::subscriber::set_global_default(subscriber)
            .context("Failed to set tracing subscriber")?;
    }

    // Load or create config
    let mut config = if let Some(path) = &cli.config {
        Config::load(path).unwrap_or_else(|_| {
            eprintln!(
                "Warning: Could not load config from {}, using defaults",
                path.display()
            );
            Config::default()
        })
    } else {
        let default_path = Config::default_path();
        if default_path.exists() {
            Config::load(&default_path).unwrap_or_default()
        } else {
            Config::default()
        }
    };

    // Override RPC URL if provided
    if let Some(rpc_url) = cli.rpc_url {
        config.network.rpc_url = rpc_url;
    }

    // Handle commands
    match cli.command {
        None | Some(Commands::Tui) => run_tui(config).await,
        Some(Commands::Scan {
            accounts,
            file,
            json,
        }) => run_scan(config, accounts, file, json).await,
        Some(Commands::Reclaim {
            accounts,
            all,
            dry_run,
        }) => run_reclaim(config, accounts, all, dry_run).await,
        Some(Commands::Stats { json }) => run_stats(config, json).await,
        Some(Commands::Init { output }) => run_init(output),
        Some(Commands::Add { accounts }) => run_add(config, accounts).await,
        Some(Commands::Export { output }) => run_export(config, output).await,
        Some(Commands::Import { file }) => run_import(config, file).await,
    }
}

/// Run the TUI
async fn run_tui(config: Config) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(config);

    // Try to initialize Solana connection
    if let Err(e) = app.init_solana() {
        app.log(
            types::LogLevel::Warning,
            format!("Solana connection failed: {}", e),
            Some("Check your RPC URL in settings".to_string()),
        );
    }

    // Run event loop
    let result = run_event_loop(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

/// Main event loop
async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::render(f, app))?;

        // Handle events with timeout (for background tasks)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle input based on mode
                match app.input_mode {
                    InputMode::Normal => {
                        match key.code {
                            // Quit
                            KeyCode::Char('q') | KeyCode::Esc => {
                                return Ok(());
                            }
                            // Tab navigation
                            KeyCode::Tab => {
                                app.tab = app.tab.next();
                            }
                            KeyCode::BackTab => {
                                app.tab = app.tab.prev();
                            }
                            // Number keys for tabs
                            KeyCode::Char('1') => app.tab = Tab::Dashboard,
                            KeyCode::Char('2') => app.tab = Tab::Accounts,
                            KeyCode::Char('3') => app.tab = Tab::Logs,
                            KeyCode::Char('4') => app.tab = Tab::Alerts,
                            KeyCode::Char('5') => app.tab = Tab::Settings,

                            // Global actions
                            KeyCode::Char('s') | KeyCode::Char('S') => {
                                // Scan accounts
                                app.status_message = Some("Scanning...".to_string());
                                if let Err(e) = app.scan_accounts().await {
                                    app.log(
                                        types::LogLevel::Error,
                                        format!("Scan failed: {}", e),
                                        None,
                                    );
                                }
                                app.status_message = None;
                            }

                            // Command mode
                            KeyCode::Char(':') => {
                                app.input_mode = InputMode::Command;
                                app.input_buffer.clear();
                            }

                            // Tab-specific keys
                            _ => handle_tab_key(app, key.code).await,
                        }
                    }
                    InputMode::AddAccount => match key.code {
                        KeyCode::Enter => {
                            if !app.input_buffer.is_empty() {
                                if let Err(e) = app.add_account(&app.input_buffer.clone()) {
                                    app.log(
                                        types::LogLevel::Error,
                                        format!("Failed to add account: {}", e),
                                        None,
                                    );
                                }
                            }
                            app.input_mode = InputMode::Normal;
                            app.input_buffer.clear();
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.input_buffer.clear();
                        }
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        KeyCode::Char(c) => {
                            app.input_buffer.push(c);
                        }
                        _ => {}
                    },
                    InputMode::ConfirmReclaim => match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            if let Err(e) = app.reclaim_selected().await {
                                app.log(
                                    types::LogLevel::Error,
                                    format!("Reclaim failed: {}", e),
                                    None,
                                );
                            }
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                    InputMode::Command => match key.code {
                        KeyCode::Enter => {
                            let cmd = app.input_buffer.clone();
                            app.input_mode = InputMode::Normal;
                            app.input_buffer.clear();
                            handle_command(app, &cmd).await;
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.input_buffer.clear();
                        }
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        KeyCode::Char(c) => {
                            app.input_buffer.push(c);
                        }
                        _ => {}
                    },
                    _ => {
                        if key.code == KeyCode::Esc {
                            app.input_mode = InputMode::Normal;
                        }
                    }
                }
            }
        }

        // Background tasks would run here if auto_scan is enabled
    }
}

/// Handle tab-specific key presses
async fn handle_tab_key(app: &mut App, key: KeyCode) {
    match app.tab {
        Tab::Dashboard => match key {
            KeyCode::Char('r') | KeyCode::Char('R') => {
                // Reclaim all
                if let Err(e) = app.reclaim_all().await {
                    app.log(
                        types::LogLevel::Error,
                        format!("Reclaim all failed: {}", e),
                        None,
                    );
                }
            }
            _ => {}
        },
        Tab::Accounts => match key {
            KeyCode::Char('j') | KeyCode::Down => app.next_account(),
            KeyCode::Char('k') | KeyCode::Up => app.prev_account(),
            KeyCode::Char('a') | KeyCode::Char('A') => {
                app.input_mode = InputMode::AddAccount;
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                app.remove_account(app.account_selected);
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                app.input_mode = InputMode::ConfirmReclaim;
            }
            KeyCode::Enter => {
                // Show account details or reclaim
                if app.selected_account().is_some() {
                    app.input_mode = InputMode::ConfirmReclaim;
                }
            }
            _ => {}
        },
        Tab::Logs => match key {
            KeyCode::Char('j') | KeyCode::Down => app.scroll_logs_down(),
            KeyCode::Char('k') | KeyCode::Up => app.scroll_logs_up(),
            KeyCode::Char('c') | KeyCode::Char('C') => {
                app.logs.clear();
                app.log(types::LogLevel::Info, "Logs cleared", None);
            }
            _ => {}
        },
        Tab::Alerts => match key {
            KeyCode::Char('j') | KeyCode::Down => app.next_alert(),
            KeyCode::Char('k') | KeyCode::Up => app.prev_alert(),
            KeyCode::Enter => {
                app.acknowledge_alert(app.alert_selected);
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                app.clear_acknowledged_alerts();
            }
            _ => {}
        },
        Tab::Settings => {
            // Settings editing would go here
        }
    }
}

/// Handle command input
async fn handle_command(app: &mut App, cmd: &str) {
    let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "scan" => {
            if let Err(e) = app.scan_accounts().await {
                app.log(types::LogLevel::Error, format!("Scan failed: {}", e), None);
            }
        }
        "reclaim-all" | "reclaimall" => {
            if let Err(e) = app.reclaim_all().await {
                app.log(
                    types::LogLevel::Error,
                    format!("Reclaim all failed: {}", e),
                    None,
                );
            }
        }
        "export" => match app.export_accounts() {
            Ok(json) => {
                app.log(types::LogLevel::Success, "Accounts exported", Some(json));
            }
            Err(e) => {
                app.log(
                    types::LogLevel::Error,
                    format!("Export failed: {}", e),
                    None,
                );
            }
        },
        "quit" | "q" => {
            // This won't actually quit, would need different handling
            app.log(types::LogLevel::Info, "Press 'q' to quit", None);
        }
        "help" | "?" => {
            app.log(
                types::LogLevel::Info,
                "Commands: scan, reclaim-all, export, quit",
                None,
            );
        }
        _ => {
            app.log(
                types::LogLevel::Warning,
                format!("Unknown command: {}", parts[0]),
                Some("Type :help for available commands".to_string()),
            );
        }
    }
}

// CLI command implementations

async fn run_scan(
    config: Config,
    accounts: Option<String>,
    file: Option<PathBuf>,
    json: bool,
) -> Result<()> {
    info!("Running scan command");

    let mut app = App::new(config);
    app.init_solana()?;

    // Parse accounts from arguments or file
    if let Some(accounts_str) = accounts {
        for pubkey in accounts_str.split(',') {
            if let Err(e) = app.add_account(pubkey.trim()) {
                eprintln!("Warning: Could not add account {}: {}", pubkey, e);
            }
        }
    }

    if let Some(path) = file {
        let content = std::fs::read_to_string(&path)?;
        for line in content.lines() {
            let pubkey = line.trim();
            if !pubkey.is_empty() && !pubkey.starts_with('#') {
                if let Err(e) = app.add_account(pubkey) {
                    eprintln!("Warning: Could not add account {}: {}", pubkey, e);
                }
            }
        }
    }

    // Run scan
    app.scan_accounts().await?;

    if json {
        let output = serde_json::json!({
            "stats": app.stats,
            "accounts": app.accounts,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Scan Results");
        println!("============");
        println!("Total accounts:    {}", app.stats.total_accounts);
        println!("Active accounts:   {}", app.stats.active_accounts);
        println!("Reclaimable:       {}", app.stats.reclaimable_accounts);
        println!("Closed:            {}", app.stats.closed_accounts);
        println!("Rent locked:       {:.6} SOL", app.stats.locked_sol());
        println!(
            "Potential reclaim: {:.6} SOL",
            app.stats.potential_reclaim_sol()
        );
    }

    Ok(())
}

async fn run_reclaim(
    config: Config,
    accounts: Option<String>,
    all: bool,
    dry_run: bool,
) -> Result<()> {
    let mut config = config;
    if dry_run {
        config.operator.dry_run = true;
    }

    let mut app = App::new(config);
    app.init_solana()?;

    if let Some(accounts_str) = accounts {
        for pubkey in accounts_str.split(',') {
            if let Err(e) = app.add_account(pubkey.trim()) {
                eprintln!("Warning: Could not add account {}: {}", pubkey, e);
            }
        }
    }

    // Scan first
    app.scan_accounts().await?;

    if all {
        let count = app.reclaim_all().await?;
        println!("Reclaimed {} accounts", count);
    } else if !app.accounts.is_empty() {
        for i in 0..app.accounts.len() {
            app.account_selected = i;
            if let Err(e) = app.reclaim_selected().await {
                eprintln!("Failed to reclaim account {}: {}", i, e);
            }
        }
    }

    println!("Total reclaimed: {:.6} SOL", app.stats.reclaimed_sol());

    Ok(())
}

async fn run_stats(config: Config, json: bool) -> Result<()> {
    let mut app = App::new(config);
    app.init_solana()?;
    app.scan_accounts().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&app.stats)?);
    } else {
        println!("Rent Statistics");
        println!("===============");
        println!("Total accounts:     {}", app.stats.total_accounts);
        println!("Active accounts:    {}", app.stats.active_accounts);
        println!("Reclaimable:        {}", app.stats.reclaimable_accounts);
        println!("Closed:             {}", app.stats.closed_accounts);
        println!();
        println!(
            "Rent locked:        {:.6} SOL ({} lamports)",
            app.stats.locked_sol(),
            app.stats.total_rent_locked
        );
        println!(
            "Rent reclaimed:     {:.6} SOL ({} lamports)",
            app.stats.reclaimed_sol(),
            app.stats.total_rent_reclaimed
        );
        println!(
            "Potential reclaim:  {:.6} SOL",
            app.stats.potential_reclaim_sol()
        );
    }

    Ok(())
}

fn run_init(output: PathBuf) -> Result<()> {
    let content = Config::example_toml();
    std::fs::write(&output, content)?;
    println!("Created configuration file: {}", output.display());
    println!("\nEdit this file to configure your Kora operator settings.");
    Ok(())
}

async fn run_add(config: Config, accounts: String) -> Result<()> {
    let mut app = App::new(config);

    for pubkey in accounts.split(',') {
        match app.add_account(pubkey.trim()) {
            Ok(_) => println!("Added: {}", pubkey.trim()),
            Err(e) => eprintln!("Failed to add {}: {}", pubkey.trim(), e),
        }
    }

    Ok(())
}

async fn run_export(config: Config, output: PathBuf) -> Result<()> {
    let app = App::new(config);
    let json = app.export_accounts()?;
    std::fs::write(&output, json)?;
    println!(
        "Exported {} accounts to {}",
        app.accounts.len(),
        output.display()
    );
    Ok(())
}

async fn run_import(config: Config, file: PathBuf) -> Result<()> {
    let mut app = App::new(config);
    let content = std::fs::read_to_string(&file)?;
    let count = app.import_accounts(&content)?;
    println!("Imported {} accounts from {}", count, file.display());
    Ok(())
}
