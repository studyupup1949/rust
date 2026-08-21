//! AutoCore Control Tool (acctl)
//!
//! CLI for managing AutoCore projects, control programs, and deployments.
//!
//! # Installation
//!
//! ```bash
//! cargo install acctl
//! ```
//!
//! # Usage
//!
//! ```bash
//! acctl set-target 192.168.1.100
//! acctl push control --start
//! acctl status
//! acctl logs --follow
//! ```

use anyhow::{anyhow, Context, Result};
use base64::Engine;
use chrono::{DateTime, Local, TimeZone};
use clap::{Parser, Subcommand};
use colored::*;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use mechutil::ipc::{CommandMessage, MessageType};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};
use autocore_util::templates::*;

mod doc;
mod tags;

const UPLOAD_CHUNK_SIZE: usize = 256 * 1024;

// ============================================================================
// CLI Argument Structures
// ============================================================================

#[derive(Parser)]
#[command(name = "acctl")]
#[command(author = "ADC <support@automateddesign.com>")]
#[command(version)]
#[command(about = "AutoCore Control Tool - CLI for managing AutoCore projects", long_about = None)]
#[command(after_help = "Examples:
  acctl clone 192.168.1.100 --list       List available projects on server
  acctl clone 192.168.1.100              Clone active project from server
  acctl clone 192.168.1.100 my_project   Clone specific project from server
  acctl push control --start             Build, deploy, and start control program
  acctl status                           Show server and control status
  acctl logs --follow                    Stream logs from control program
")]
struct Cli {
    /// Override server host
    #[arg(long, global = true)]
    host: Option<String>,

    /// Override server port
    #[arg(long, global = true)]
    port: Option<u16>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clone project from server into a new directory
    Clone {
        /// Server IP address or hostname
        host: String,

        /// Project name to clone (defaults to currently active project)
        project: Option<String>,

        /// Server port (default: 11969)
        #[arg(short = 'P', long, default_value = "11969")]
        port: u16,

        /// Directory name (defaults to project name)
        #[arg(short, long)]
        directory: Option<String>,

        /// List available projects instead of cloning
        #[arg(short, long)]
        list: bool,
    },

    /// Set target server IP address
    SetTarget {
        /// Server IP address or hostname
        ip: String,

        /// Server port (default: 11969)
        #[arg(short, long)]
        port: Option<u16>,
    },

    /// Pull project from server
    Pull {
        /// Extract zip after download
        #[arg(short = 'x', long)]
        extract: bool,
    },

    /// Push files to server
    Push {
        #[command(subcommand)]
        what: PushCommands,
    },

    /// Pull the server's results/ directory into the local datastore.
    PullResults,

    /// Regenerate gm.rs from server
    Codegen {
        /// Skip project.json sync check
        #[arg(short, long)]
        force: bool,
    },

    /// Regenerate www/src/AutoCoreTags.ts from project.json (local, no server needed)
    CodegenTags {
        /// Force full regeneration — overwrite acTagSpecCustom with the empty template
        /// (a .bak of the old file is always written alongside when overwriting)
        #[arg(short, long)]
        force: bool,
    },

    /// Switch to different project on server
    Switch {
        /// Project name to switch to
        project_name: String,

        /// Restart server after switch
        #[arg(short, long)]
        restart: bool,
    },

    /// Get server and control program status
    Status,

    /// View control program logs
    Logs {
        /// Stream logs continuously
        #[arg(short, long)]
        follow: bool,
    },

    /// Control program management
    Control {
        /// Action to perform
        #[arg(value_parser = ["start", "stop", "restart", "status"])]
        action: String,
    },

    /// Bidirectional sync. First reconciles project.json (interactive on
    /// diff), then mtime-wins syncs the datastore tree (excluding results/).
    Sync {
        /// Show what would change without applying anything.
        #[arg(long)]
        dry_run: bool,
    },

    /// Create a new AutoCore project from template
    New {
        /// Project name (alphanumeric, underscores, hyphens)
        name: String,
    },

    /// Create a new AutoCore project preconfigured with a Test
    /// Information System scaffold (control wires `tick_with_autostart`,
    /// HMI wraps the four TIS components in a `<TisProvider>`).
    NewTisProject {
        /// Project name (alphanumeric, underscores, hyphens)
        name: String,
    },

    /// Send a command to the server (like the AutoCore console)
    #[command(
        after_help = "Examples:\n  acctl cmd system.get_domains\n  acctl cmd ethercat.configure --device RC8_0 ListProfiles\n  acctl cmd system.control --action status\n  acctl cmd modbus.get_status"
    )]
    Cmd {
        /// Command topic (domain.command, e.g. ethercat.configure)
        topic: String,

        /// Arguments passed to the command (flags and positional args)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Export project variables to CSV file
    ExportVars {
        /// Output CSV file path
        #[arg(short, long, default_value = "variables.csv")]
        output: String,
    },

    /// Import variables from CSV file into project.json
    ImportVars {
        /// Input CSV file path
        #[arg(short, long, default_value = "variables.csv")]
        input: String,
    },

    /// Find and resolve variables with duplicate hardware links
    DedupVars,

    /// Validate project.json for errors
    Validate,

    /// Show project summary
    Info,

    /// Upload a file to the project directory on the server
    Upload {
        /// Local file path to upload
        source: String,

        /// Destination path relative to project directory (default: lib/<filename>)
        #[arg(short, long)]
        dest: Option<String>,
    },

    /// Documentation management
    Doc {
        #[command(subcommand)]
        cmd: DocCommand,
    },

    /// Retrofit AMS (Asset Management System) into the current project.
    /// Adds an `asset_types: {}` block to project.json (idempotent) so
    /// `Project::normalize()` injects the baseline `ams_*` GM scalars
    /// next time codegen runs. See doc/ams_product_plan.md.
    AddAms,

    /// Retrofit TIS (Test Information System) into the current project
    /// without scaffolding a fresh one. Adds an empty `test_methods: {}`
    /// block to project.json so the `tis_*` GM scalars and codegen
    /// kick in next time you run `acctl codegen`.
    AddTis,

    /// Asset Management System export/import.
    Ams {
        #[command(subcommand)]
        cmd: AmsCommand,
    },
}

#[derive(Subcommand)]
pub(crate) enum AmsCommand {
    /// Pull the full AMS dataset (registry + per-asset history + usage)
    /// from the server into a single JSON document. Suitable as a
    /// pre-deploy backup or for cloning to a test environment.
    Export {
        /// Output JSON file (default: ams_export.json).
        #[arg(short, long, default_value = "ams_export.json")]
        output: String,
    },
    /// Apply an exported AMS document to the current server. Default
    /// behaviour is merge: existing assets get any new calibration
    /// records appended; missing assets are created.
    Import {
        /// Input JSON file produced by `acctl ams export`.
        #[arg(short, long)]
        input: String,
        /// Show what would change but don't actually modify the server.
        #[arg(long)]
        dry_run: bool,
    },
    /// Walk every `asset_ref` declared in this project's
    /// `test_methods` block and create one stub asset in the AMS
    /// registry per `(asset_type, location)` pair under
    /// `select: by_location`. Lets a project that just ran
    /// `acctl add-ams` skip past the "every test errors with `no
    /// matching asset in registry`" stage. After running, the
    /// stubs need their serial number and current calibration
    /// filled in via the HMI's <AssetRegistryTable> / <CalibrationEntryDialog>.
    Backfill {
        /// Show what would be created but don't actually modify the server.
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum DocCommand {
    /// Scaffold doc/ in an existing project (for projects created before `acctl doc` support)
    Init {
        /// Overwrite existing doc/ files instead of skipping them
        #[arg(short, long)]
        force: bool,
    },
    /// Build the project documentation (HTML output at doc/book/)
    Build,
    /// Serve the project documentation locally with live-reload
    Serve {
        #[arg(short, long, default_value = "4444")]
        port: u16,
    },
    /// Auto-generate variables.md from project.json
    GenerateVars,
    /// Remove generated documentation output
    Clean,
}

#[derive(Subcommand)]
enum PushCommands {
    /// Push project.json
    Project {
        /// Restart server after push
        #[arg(short, long)]
        restart: bool,
    },

    /// Push www files
    Www {
        /// Push full www/ instead of just dist/
        #[arg(short, long)]
        source: bool,

        /// Skip npm run build before pushing
        #[arg(long)]
        no_build: bool,
    },

    /// Push control binary
    Control {
        /// Push full source instead of binary
        #[arg(short, long)]
        source: bool,

        /// Skip building
        #[arg(long)]
        no_build: bool,

        /// Start after upload
        #[arg(long)]
        start: bool,

        /// Skip project.json sync check
        #[arg(short, long)]
        force: bool,
    },

    /// Push generated documentation (zipped doc/book/)
    Doc {
        /// Skip the local build before pushing. Fails if doc/book/ is missing.
        #[arg(long)]
        no_build: bool,
    },

    /// Push the local datastore/scripts/ directory to the server.
    Scripts,
}

// ============================================================================
// Configuration
// ============================================================================

#[derive(Debug, Deserialize, Serialize, Default)]
struct Config {
    server: Option<ServerConfig>,
    build: Option<BuildConfig>,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
struct ServerConfig {
    host: Option<String>,
    port: Option<u16>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct BuildConfig {
    release: Option<bool>,
}

impl Config {
    fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            toml::from_str(&content).context("Failed to parse config file")
        } else {
            Ok(Config::default())
        }
    }

    fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&config_path, content)
            .context("Failed to write config file")?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        // First check current directory
        let local_config = PathBuf::from("acctl.toml");
        if local_config.exists() {
            return Ok(local_config);
        }

        // Then check home directory
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not determine home directory"))?;
        Ok(home.join(".acctl.toml"))
    }

    fn get_host(&self) -> String {
        self.server
            .as_ref()
            .and_then(|s| s.host.clone())
            .unwrap_or_else(|| "127.0.0.1".to_string())
    }

    fn get_port(&self) -> u16 {
        self.server
            .as_ref()
            .and_then(|s| s.port)
            .unwrap_or(11969)
    }

    fn is_release(&self) -> bool {
        self.build
            .as_ref()
            .and_then(|b| b.release)
            .unwrap_or(true)
    }
}

// ============================================================================
// WebSocket Communication
// ============================================================================

// CommandMessage and MessageType imported from mechutil::ipc

struct WsClient {
    write: futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    read: futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
}

/// Simple response wrapper for easier handling in commands
struct CommandResponse {
    success: bool,
    error_message: String,
    data: serde_json::Value,
}

impl WsClient {
    async fn connect(host: &str, port: u16) -> Result<Self> {
        let url = format!("ws://{}:{}/ws/", host, port);
        let (ws_stream, _) = connect_async(&url)
            .await
            .context(format!("Failed to connect to {}", url))?;

        let (write, read) = ws_stream.split();
        Ok(WsClient { write, read })
    }

    /// Send a command and wait for response
    /// Topic format: "domain.fname" (e.g., "system.download_project")
    async fn send_command(
        &mut self,
        topic: &str,
        data: serde_json::Value,
    ) -> Result<CommandResponse> {
        // Use mechutil's CommandMessage::request constructor
        let msg = CommandMessage::request(topic, data);
        let transaction_id = msg.transaction_id;

        let json = serde_json::to_string(&msg)?;
        self.write.send(Message::Text(json)).await?;

        // Wait for response
        let timeout = Duration::from_secs(30);
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            match tokio::time::timeout(Duration::from_secs(1), self.read.next()).await {
                Ok(Some(Ok(Message::Text(text)))) => {
                    let response: CommandMessage = match serde_json::from_str(&text) {
                        Ok(r) => r,
                        Err(_) => continue, // Skip malformed messages
                    };

                    // Match by transaction_id (response to our request)
                    if response.transaction_id == transaction_id {
                        return Ok(CommandResponse {
                            success: response.success,
                            error_message: response.error_message,
                            data: response.data,
                        });
                    }

                    // Skip broadcast messages
                    if response.message_type == MessageType::Broadcast {
                        continue;
                    }
                }
                Ok(Some(Ok(_))) => continue,
                Ok(Some(Err(e))) => return Err(anyhow!("WebSocket error: {}", e)),
                Ok(None) => return Err(anyhow!("Connection closed")),
                Err(_) => continue, // Timeout on single read, keep trying
            }
        }

        Err(anyhow!("Timeout waiting for response"))
    }

    async fn close(mut self) -> Result<()> {
        self.write.close().await?;
        Ok(())
    }
}

// ============================================================================
// Log Entry
// ============================================================================

#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp_ms: u64,
    level: String,
    source: String,
    message: String,
}

fn print_log_entry(entry: &LogEntry) {
    let dt: DateTime<Local> = Local
        .timestamp_millis_opt(entry.timestamp_ms as i64)
        .single()
        .unwrap_or_else(Local::now);

    let time_str = dt.format("%H:%M:%S%.3f").to_string();

    let level_colored = match entry.level.as_str() {
        "ERROR" => entry.level.red().bold(),
        "WARN" => entry.level.yellow(),
        "INFO" => entry.level.green(),
        "DEBUG" => entry.level.blue(),
        "TRACE" => entry.level.dimmed(),
        _ => entry.level.normal(),
    };

    println!(
        "{} [{}] {}: {}",
        time_str.dimmed(),
        level_colored,
        entry.source.cyan(),
        entry.message
    );
}

// ============================================================================
// Command Implementations
// ============================================================================

async fn cmd_clone(
    host: String,
    port: u16,
    project: Option<String>,
    directory: Option<String>,
    list: bool,
) -> Result<()> {
    println!("Connecting to {}:{}...", host, port);

    let mut client = WsClient::connect(&host, port).await?;

    // If --list flag, just show available projects and exit
    if list {
        let response = client
            .send_command("system.list_projects", serde_json::json!({}))
            .await?;

        client.close().await?;

        if !response.success {
            return Err(anyhow!("Error: {}", response.error_message));
        }

        let projects_dir = response.data["projects_directory"]
            .as_str()
            .unwrap_or("unknown");
        println!("\n{} {}", "Projects Directory:".bold(), projects_dir);
        println!("{}", "Available Projects:".bold());

        if let Some(projects) = response.data["projects"].as_array() {
            for proj in projects {
                let name = proj["name"].as_str().unwrap_or("?");
                let valid = proj["valid"].as_bool().unwrap_or(false);
                let status = if valid {
                    "valid".green()
                } else {
                    "invalid".red()
                };
                println!("  - {} ({})", name, status);
            }
        }

        println!("\nTo clone a project:");
        println!("  acctl clone {} <project_name>", host);
        return Ok(());
    }

    // If project name specified, activate it first
    if let Some(ref proj_name) = project {
        println!("Activating project '{}'...", proj_name);
        let response = client
            .send_command(
                "system.activate",
                serde_json::json!({"project_name": proj_name}),
            )
            .await?;

        if !response.success {
            client.close().await?;
            return Err(anyhow!(
                "Failed to activate project '{}': {}",
                proj_name,
                response.error_message
            ));
        }
    }

    // Download the project (inline mode for CLI to get base64 data)
    let response = client
        .send_command("system.download_project", serde_json::json!({"inline": true}))
        .await?;

    if !response.success {
        client.close().await?;
        return Err(anyhow!("Error: {}", response.error_message));
    }

    let data = &response.data;
    let filename = data["filename"].as_str().unwrap_or("project.zip");
    let project_name = data["project_name"]
        .as_str()
        .map(|s| s.to_lowercase().replace(' ', "_"))
        .unwrap_or_else(|| {
            // Extract from filename (e.g., "my_project_project.zip" -> "my_project")
            filename
                .trim_end_matches("_project.zip")
                .trim_end_matches(".zip")
                .to_string()
        });

    let data_b64 = data["data"]
        .as_str()
        .ok_or_else(|| anyhow!("No data in response"))?;
    let size = data["size"].as_u64().unwrap_or(0);

    println!("  Project: {}", project_name);
    println!("  Size: {} bytes", size);

    // Determine target directory
    let target_dir = directory.unwrap_or_else(|| project_name.clone());
    let target_path = PathBuf::from(&target_dir);

    if target_path.exists() {
        return Err(anyhow!(
            "Directory '{}' already exists. Use a different name with --directory",
            target_dir
        ));
    }

    // Decode and extract
    let zip_data = base64::engine::general_purpose::STANDARD.decode(data_b64)?;

    println!("Extracting to {}...", target_dir);
    fs::create_dir_all(&target_path)?;

    let cursor = std::io::Cursor::new(&zip_data);
    let mut archive = ZipArchive::new(cursor)?;

    // Extract, stripping the first directory component if present
    // (zip contains "project_name/..." structure)
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let raw_name = file.name().to_string();

        // Strip the first path component (the project name in the zip)
        let stripped_name = raw_name
            .split('/')
            .skip(1)
            .collect::<Vec<_>>()
            .join("/");

        if stripped_name.is_empty() {
            continue;
        }

        let outpath = target_path.join(&stripped_name);

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    // Create local acctl.toml in the project directory
    let config_content = format!(
        r#"# AutoCore Control Tool Configuration
# Generated by: acctl clone {}

[server]
host = "{}"
port = {}

[build]
release = true
"#,
        host, host, port
    );

    let config_path = target_path.join("acctl.toml");
    fs::write(&config_path, config_content)?;

    client.close().await?;

    println!("{}", "Clone complete!".green());
    println!();
    println!("Next steps:");
    println!("  cd {}", target_dir);
    println!("  acctl status              # Check connection");
    println!("  acctl push control --start  # Build and deploy");

    Ok(())
}

async fn cmd_set_target(ip: String, port: Option<u16>) -> Result<()> {
    let mut config = Config::load().unwrap_or_default();

    let server = config.server.get_or_insert(ServerConfig::default());
    server.host = Some(ip.clone());
    if let Some(p) = port {
        server.port = Some(p);
    }

    config.save()?;

    let config_path = Config::config_path()?;
    println!("Updated {}", config_path.display());
    println!("  Host: {}", ip);
    if let Some(p) = port {
        println!("  Port: {}", p);
    }

    Ok(())
}

async fn cmd_pull(config: &Config, extract: bool) -> Result<()> {
    println!("Pulling project from server...");

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    // Use inline mode for CLI to get base64 data directly
    let response = client
        .send_command("system.download_project", serde_json::json!({"inline": true}))
        .await?;

    client.close().await?;

    if !response.success {
        return Err(anyhow!("Error: {}", response.error_message));
    }

    let filename = response.data["filename"]
        .as_str()
        .unwrap_or("project.zip");
    let data_b64 = response.data["data"]
        .as_str()
        .ok_or_else(|| anyhow!("No data in response"))?;
    let size = response.data["size"].as_u64().unwrap_or(0);

    println!("  Received: {} ({} bytes)", filename, size);

    let zip_data = base64::engine::general_purpose::STANDARD.decode(data_b64)?;
    fs::write(filename, &zip_data)?;
    println!("  Saved to: {}", filename);

    if extract {
        let extract_dir = "pulled_project";
        if Path::new(extract_dir).exists() {
            fs::remove_dir_all(extract_dir)?;
        }

        let cursor = std::io::Cursor::new(&zip_data);
        let mut archive = ZipArchive::new(cursor)?;
        archive.extract(extract_dir)?;

        println!("  Extracted to: {}", extract_dir);
    }

    Ok(())
}

async fn cmd_push_project(config: &Config, restart: bool) -> Result<()> {
    // Find project.json
    let project_path = if Path::new("project.json").exists() {
        PathBuf::from("project.json")
    } else if Path::new("../project.json").exists() {
        PathBuf::from("../project.json")
    } else {
        return Err(anyhow!("project.json not found"));
    };

    let content = fs::read_to_string(&project_path)?;
    let project_json: serde_json::Value = serde_json::from_str(&content)?;

    println!("Pushing project.json to server...");

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    let response = client
        .send_command(
            "system.upload_project",
            serde_json::json!({
                "project_json": project_json,
                "restart": restart
            }),
        )
        .await?;

    client.close().await?;

    if !response.success {
        return Err(anyhow!("Error: {}", response.error_message));
    }

    let status = response.data["status"].as_str().unwrap_or("unknown");
    println!("  Status: {}", status);

    if response.data["restarting"].as_bool().unwrap_or(false) {
        println!("  Server is restarting...");
    }

    Ok(())
}

async fn cmd_push_www(config: &Config, source: bool, no_build: bool) -> Result<()> {
    let www_root = PathBuf::from("www");

    // Build before pushing dist (skip if --source or --no-build)
    if !source && !no_build && www_root.exists() {
        println!("Building www...");
        let status = std::process::Command::new("npm")
            .arg("run")
            .arg("build")
            .current_dir(&www_root)
            .status()?;
        if !status.success() {
            return Err(anyhow!("npm run build failed"));
        }
        println!("Build successful!");
    }

    let www_dir = if source {
        www_root
    } else {
        PathBuf::from("www/dist")
    };

    if !www_dir.exists() {
        return Err(anyhow!(
            "{} not found. {}",
            www_dir.display(),
            if !source {
                "Run npm run build in www/ first, or use --source to push full www/"
            } else {
                ""
            }
        ));
    }

    println!("Creating zip of {}...", www_dir.display());

    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        add_dir_to_zip(&mut zip, &www_dir, "", options)?;
        zip.finish()?;
    }

    let zip_data = buffer.into_inner();
    let zip_b64 = base64::engine::general_purpose::STANDARD.encode(&zip_data);

    println!("Pushing www files ({} bytes)...", zip_data.len());

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    let response = client
        .send_command(
            "system.upload_www",
            serde_json::json!({
                "data": zip_b64,
                "source": source
            }),
        )
        .await?;

    client.close().await?;

    if !response.success {
        return Err(anyhow!("Error: {}", response.error_message));
    }

    let path = response.data["path"].as_str().unwrap_or("unknown");
    let files = response.data["files_extracted"].as_u64().unwrap_or(0);
    println!("  Uploaded to: {}", path);
    println!("  Files extracted: {}", files);

    Ok(())
}

async fn cmd_push_doc(config: &Config, no_build: bool) -> Result<()> {
    let book_dir = PathBuf::from("doc/book");

    if no_build {
        if !book_dir.join("index.html").exists() {
            return Err(anyhow!(
                "doc/book/index.html not found. Run `acctl doc build` first or omit --no-build."
            ));
        }
    } else {
        // Fresh build: generate-vars → cargo doc → mdbook build
        doc::cmd_doc(&DocCommand::Build).await?;
    }

    if !book_dir.exists() {
        return Err(anyhow!("doc/book/ not found after build"));
    }

    println!("Creating zip of doc/book/...");
    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        add_dir_to_zip(&mut zip, &book_dir, "", options)?;
        zip.finish()?;
    }

    let zip_data = buffer.into_inner();
    let zip_b64 = base64::engine::general_purpose::STANDARD.encode(&zip_data);

    println!(
        "Pushing documentation ({:.1} KB compressed)...",
        zip_data.len() as f64 / 1024.0
    );

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    let response = client
        .send_command(
            "system.upload_doc",
            serde_json::json!({ "data": zip_b64 }),
        )
        .await?;

    client.close().await?;

    if !response.success {
        return Err(anyhow!("Error: {}", response.error_message));
    }

    let path = response.data["path"].as_str().unwrap_or("unknown");
    let files = response.data["files_extracted"].as_u64().unwrap_or(0);
    println!("  Uploaded to: {}", path);
    println!("  Files extracted: {}", files);
    println!("  Documentation is now live on the server's doc port (default 4444).");

    Ok(())
}

async fn cmd_push_control(config: &Config, source: bool, no_build: bool, start: bool, force: bool) -> Result<()> {
    let control_dir = PathBuf::from("control");
    if !control_dir.exists() {
        return Err(anyhow!("control/ directory not found"));
    }

    // Pre-push project.json sync check
    if !force {
        if let Err(e) = check_project_sync(config).await {
            return Err(e);
        }
    }

    // If --source flag, upload the entire control source directory
    if source {
        return cmd_push_control_source(config).await;
    }

    let release = config.is_release();

    // Build if not skipped
    if !no_build {
        println!("Building control program...");

        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("build");
        if release {
            cmd.arg("--release");
        }
        cmd.current_dir(&control_dir);

        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Build failed"));
        }
        println!("Build successful!");
    }

    // Find binary
    let target_dir = if release { "release" } else { "debug" };

    // Read package name from Cargo.toml
    let cargo_toml_path = control_dir.join("Cargo.toml");
    let cargo_content = fs::read_to_string(&cargo_toml_path)?;
    let cargo: toml::Value = toml::from_str(&cargo_content)?;
    let package_name = cargo["package"]["name"]
        .as_str()
        .ok_or_else(|| anyhow!("Could not find package name in Cargo.toml"))?;

    let binary_name = format!("{}{}", package_name, std::env::consts::EXE_SUFFIX);
    let binary_path = control_dir
        .join("target")
        .join(target_dir)
        .join(&binary_name);

    if !binary_path.exists() {
        return Err(anyhow!("Binary not found: {}", binary_path.display()));
    }

    // Connect and deploy
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    // Stop if running
    println!("Stopping control program...");
    let _ = client
        .send_command("system.control", serde_json::json!({"action": "stop"}))
        .await;

    // Upload binary using chunked protocol with fallback to single message
    let binary_data = fs::read(&binary_path)?;
    let total_size = binary_data.len();
    let total_chunks = (total_size + UPLOAD_CHUNK_SIZE - 1) / UPLOAD_CHUNK_SIZE;

    println!("Uploading binary ({} bytes, {} chunks)...", total_size, total_chunks);

    // Try chunked upload first
    let init_response = client
        .send_command(
            "system.control",
            serde_json::json!({
                "action": "upload_init",
                "total_size": total_size,
                "chunk_size": UPLOAD_CHUNK_SIZE,
                "total_chunks": total_chunks,
                "release": release,
                "package_name": package_name
            }),
        )
        .await?;

    let upload_path;

    if !init_response.success && init_response.error_message.contains("Unknown control action") {
        // Old server: fall back to single-message upload
        println!("  Server does not support chunked upload, falling back to single message...");
        let binary_b64 = base64::engine::general_purpose::STANDARD.encode(&binary_data);
        let response = client
            .send_command(
                "system.control",
                serde_json::json!({
                    "action": "upload",
                    "binary": binary_b64,
                    "release": release,
                    "package_name": package_name
                }),
            )
            .await?;

        if !response.success {
            client.close().await?;
            return Err(anyhow!("Error: {}", response.error_message));
        }
        upload_path = response.data["path"].as_str().unwrap_or("unknown").to_string();
    } else if !init_response.success {
        client.close().await?;
        return Err(anyhow!("Error: {}", init_response.error_message));
    } else {
        // Chunked upload path
        let upload_id = init_response.data["upload_id"]
            .as_u64()
            .ok_or_else(|| anyhow!("Server did not return upload_id"))?;

        for (i, chunk) in binary_data.chunks(UPLOAD_CHUNK_SIZE).enumerate() {
            let chunk_b64 = base64::engine::general_purpose::STANDARD.encode(chunk);
            println!("  Chunk {}/{}", i + 1, total_chunks);

            let chunk_response = client
                .send_command(
                    "system.control",
                    serde_json::json!({
                        "action": "upload_chunk",
                        "upload_id": upload_id,
                        "chunk_index": i,
                        "data": chunk_b64
                    }),
                )
                .await?;

            if !chunk_response.success {
                client.close().await?;
                return Err(anyhow!("Chunk {} failed: {}", i, chunk_response.error_message));
            }
        }

        let complete_response = client
            .send_command(
                "system.control",
                serde_json::json!({
                    "action": "upload_complete",
                    "upload_id": upload_id
                }),
            )
            .await?;

        if !complete_response.success {
            client.close().await?;
            return Err(anyhow!("Error: {}", complete_response.error_message));
        }
        upload_path = complete_response.data["path"].as_str().unwrap_or("unknown").to_string();
    }

    println!("  Uploaded to: {}", upload_path);

    // Start if requested
    if start {
        println!("Starting control program...");
        let response = client
            .send_command(
                "system.control",
                serde_json::json!({
                    "action": "start",
                    "no_build": true
                }),
            )
            .await?;

        if response.success {
            let pid = response.data["pid"].as_u64().unwrap_or(0);
            println!("  PID: {}", pid);
        } else {
            println!("  Warning: {}", response.error_message);
        }
    }

    client.close().await?;
    Ok(())
}

/// Push the entire control source directory to the server
async fn cmd_push_control_source(config: &Config) -> Result<()> {
    let control_dir = PathBuf::from("control");

    println!("Creating control source archive...");

    // Create zip in memory
    let mut zip_data = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut zip_data);
        let mut zip = ZipWriter::new(cursor);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // Walk the control directory, excluding target/
        fn add_dir_to_zip<W: Write + std::io::Seek>(
            zip: &mut ZipWriter<W>,
            dir: &Path,
            base: &Path,
            options: SimpleFileOptions,
        ) -> Result<()> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                let name = path.strip_prefix(base)?.to_string_lossy().to_string();

                // Skip target directory and hidden files
                if name.starts_with("target") || name.starts_with('.') {
                    continue;
                }

                if path.is_dir() {
                    zip.add_directory(&name, options)?;
                    add_dir_to_zip(zip, &path, base, options)?;
                } else {
                    zip.start_file(&name, options)?;
                    let data = fs::read(&path)?;
                    zip.write_all(&data)?;
                }
            }
            Ok(())
        }

        add_dir_to_zip(&mut zip, &control_dir, &control_dir, options)?;
        zip.finish()?;
    }

    println!("  Archive size: {} bytes", zip_data.len());

    // Connect and upload
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    println!("Uploading control source...");
    let zip_b64 = base64::engine::general_purpose::STANDARD.encode(&zip_data);

    let response = client
        .send_command(
            "system.upload_control_project",
            serde_json::json!({
                "data": zip_b64
            }),
        )
        .await?;

    client.close().await?;

    if !response.success {
        return Err(anyhow!("Upload failed: {}", response.error_message));
    }

    let files_count = response.data["files_extracted"].as_u64().unwrap_or(0);
    println!("  Uploaded {} files to server", files_count);
    println!("Control source push complete!");

    Ok(())
}

async fn cmd_codegen(config: &Config, force: bool) -> Result<()> {
    if !force {
        check_project_sync(config).await?;
    }

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    // Validate the server's currently-loaded project before generating
    // any code. Catches AMS placeholder typos, broken module configs,
    // and bad variable links — the things that would otherwise fail at
    // module-spawn time, well after codegen has already run.
    if let Err(e) = validate_project_remote(&mut client, None).await {
        client.close().await?;
        return Err(e);
    }

    println!("Requesting gm.rs regeneration from server...");

    let response = client
        .send_command("system.update_control", serde_json::json!({}))
        .await?;

    if !response.success {
        client.close().await?;
        return Err(anyhow!("Error: {}", response.error_message));
    }

    println!("  gm.rs updated on server");

    // Download updated control project (inline mode for CLI to get base64 data)
    println!("Downloading updated gm.rs...");
    let response = client
        .send_command("system.download_control_project", serde_json::json!({"inline": true}))
        .await?;

    client.close().await?;

    if response.success {
        let data_b64 = response.data["data"]
            .as_str()
            .ok_or_else(|| anyhow!("No data in response"))?;
        let zip_data = base64::engine::general_purpose::STANDARD.decode(data_b64)?;

        let cursor = std::io::Cursor::new(&zip_data);
        let mut archive = ZipArchive::new(cursor)?;

        // Extract gm.rs and (if present) www/src/autocore/tis.ts. The
        // server bundles both so a single `acctl codegen` keeps the Rust
        // mapping and the TS test-method schema in sync.
        let mut wrote_gm = false;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            let dest: Option<PathBuf> = if name.ends_with("control/src/gm.rs") || name == "control/src/gm.rs" {
                Some(PathBuf::from("control/src/gm.rs"))
            } else if name.ends_with("www/src/autocore/tis.ts") || name == "www/src/autocore/tis.ts" {
                Some(PathBuf::from("www/src/autocore/tis.ts"))
            } else {
                None
            };

            if let Some(dest) = dest {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&dest, &content)?;
                println!("  Updated: {}", dest.display());
                if dest.ends_with("gm.rs") {
                    wrote_gm = true;
                }
            }
        }

        if !wrote_gm {
            println!("  Warning: gm.rs not found in download");
        }
        return Ok(());
    } else {
        println!(
            "  Warning: Could not download updated gm.rs: {}",
            response.error_message
        );
    }

    Ok(())
}

async fn cmd_switch(config: &Config, project_name: &str, restart: bool) -> Result<()> {
    println!("Switching to project: {}", project_name);

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    let response = client
        .send_command(
            "system.activate",
            serde_json::json!({
                "project_name": project_name
            }),
        )
        .await?;

    if !response.success {
        client.close().await?;
        return Err(anyhow!("Error: {}", response.error_message));
    }

    println!("  Project '{}' activated", project_name);

    if restart {
        println!("Restarting server...");
        let _ = client
            .send_command("system.restart", serde_json::json!({}))
            .await;
        println!("  Restart initiated");
    }

    client.close().await?;
    Ok(())
}

async fn cmd_status(config: &Config) -> Result<()> {
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    // Get control status
    let response = client
        .send_command("system.control", serde_json::json!({"action": "status"}))
        .await?;

    println!("{}", "Control Program Status:".bold());
    if response.success {
        let data = &response.data;
        // Status may be wrapped in { "status": ..., "control_stale": bool }
        let status = data.get("status").unwrap_or(data);
        if let Some(running) = status.get("Running") {
            let pid = running["pid"].as_u64().unwrap_or(0);
            println!("  Status: {} (PID: {})", "Running".green(), pid);
        } else if let Some(failed) = status.get("Failed") {
            let error = failed["error"].as_str().unwrap_or("unknown");
            println!("  Status: {} ({})", "Failed".red(), error);
        } else if status.as_str() == Some("Stopped") {
            println!("  Status: {}", "Stopped".yellow());
        } else {
            println!("  Status: {:?}", status);
        }
        if data.get("control_stale").and_then(|v| v.as_bool()).unwrap_or(false) {
            println!("  {}", "Warning: Running with outdated project configuration. Run 'acctl push control --start' to rebuild.".yellow());
        }
    } else {
        println!("  Error: {}", response.error_message);
    }

    // List projects
    let response = client
        .send_command("system.list_projects", serde_json::json!({}))
        .await?;

    if response.success {
        let projects_dir = response.data["projects_directory"]
            .as_str()
            .unwrap_or("unknown");
        println!("\n{} {}", "Projects Directory:".bold(), projects_dir);
        println!("{}", "Available Projects:".bold());

        if let Some(projects) = response.data["projects"].as_array() {
            for proj in projects {
                let name = proj["name"].as_str().unwrap_or("?");
                let valid = proj["valid"].as_bool().unwrap_or(false);
                let status = if valid {
                    "valid".green()
                } else {
                    "invalid".red()
                };
                println!("  - {} ({})", name, status);
            }
        }
    }

    client.close().await?;
    Ok(())
}

async fn cmd_logs(config: &Config, follow: bool) -> Result<()> {
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    // Get buffer first
    let response = client
        .send_command("log.get_buffer", serde_json::json!({}))
        .await?;

    if response.success {
        if let Some(entries) = response.data.as_array() {
            if entries.is_empty() {
                println!("No log entries");
            }
            for entry in entries {
                if let Ok(log_entry) = serde_json::from_value::<LogEntry>(entry.clone()) {
                    print_log_entry(&log_entry);
                }
            }
        }
    }

    if follow {
        println!("{}", "Streaming logs (Ctrl+C to stop)...".dimmed());

        loop {
            match tokio::time::timeout(Duration::from_secs(60), client.read.next()).await {
                Ok(Some(Ok(Message::Text(text)))) => {
                    if let Ok(msg) = serde_json::from_str::<CommandMessage>(&text) {
                        // Check for broadcast from log domain (topic starts with "log.")
                        if msg.message_type == MessageType::Broadcast && msg.topic.starts_with("log.") {
                            // Broadcast data may have a "value" wrapper or be direct
                            let entry_value = msg.data.get("value").cloned().unwrap_or(msg.data.clone());
                            if let Ok(entry) = serde_json::from_value::<LogEntry>(entry_value) {
                                print_log_entry(&entry);
                            }
                        }
                    }
                }
                Ok(Some(Ok(_))) => continue,
                Ok(Some(Err(e))) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
                Ok(None) => {
                    eprintln!("Connection closed");
                    break;
                }
                Err(_) => continue, // Timeout, keep going
            }
        }
    }

    client.close().await?;
    Ok(())
}

async fn cmd_control(config: &Config, action: &str) -> Result<()> {
    println!("Control program: {}...", action);

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    let response = client
        .send_command(
            "system.control",
            serde_json::json!({"action": action}),
        )
        .await?;

    client.close().await?;

    if response.success {
        match action {
            "start" => {
                let pid = response.data["pid"].as_u64().unwrap_or(0);
                println!("  Started (PID: {})", pid);
            }
            "stop" => {
                let status = response.data["status"].as_str().unwrap_or("stopped");
                println!("  Status: {}", status);
            }
            "restart" => {
                let pid = response.data["pid"].as_u64().unwrap_or(0);
                println!("  Restarted (PID: {})", pid);
            }
            "status" => {
                println!("  Status: {:?}", response.data);
            }
            _ => {}
        }
    } else {
        return Err(anyhow!("Error: {}", response.error_message));
    }

    Ok(())
}

// ============================================================================
// Sync
// ============================================================================

/// Show top-level key differences between two JSON values.
fn show_json_diff(local: &serde_json::Value, server: &serde_json::Value) {
    let local_obj = local.as_object();
    let server_obj = server.as_object();

    let (Some(local_map), Some(server_map)) = (local_obj, server_obj) else {
        println!("  Values differ (not both objects)");
        return;
    };

    // Keys only in local
    for key in local_map.keys() {
        if !server_map.contains_key(key) {
            println!("  {} key '{}' (not on server)", "+".green(), key);
        }
    }

    // Keys only in server
    for key in server_map.keys() {
        if !local_map.contains_key(key) {
            println!("  {} key '{}' (not in local)", "-".red(), key);
        }
    }

    // Keys in both but different
    for key in local_map.keys() {
        if let Some(server_val) = server_map.get(key) {
            if local_map[key] != *server_val {
                println!("  {} key '{}' differs", "~".yellow(), key);
            }
        }
    }
}

/// Send a project (or the currently-loaded one) to `system.validate_project`
/// and bail with a formatted error report when anything came back. Used as
/// a pre-flight on `sync` push and `codegen`.
///
/// Pass `Some(project_json)` to validate a specific blob (push path);
/// pass `None` to validate the server's currently-loaded project (codegen path).
async fn validate_project_remote(
    client: &mut WsClient,
    project_json: Option<&serde_json::Value>,
) -> Result<()> {
    let mut payload = serde_json::Map::new();
    if let Some(pj) = project_json {
        payload.insert("project_json".to_string(), pj.clone());
    }
    let response = client
        .send_command("system.validate_project", serde_json::Value::Object(payload))
        .await?;

    if !response.success {
        // The server didn't even run the validator (e.g., older build
        // without the command). Surface that as a soft warning rather
        // than a hard fail so the user can still proceed.
        eprintln!(
            "{} {}",
            "Warning: server-side validation unavailable:".yellow(),
            response.error_message,
        );
        return Ok(());
    }

    let ok = response.data.get("ok").and_then(|v| v.as_bool()).unwrap_or(true);
    if ok {
        println!("{}", "Project validation: OK".green());
        return Ok(());
    }

    let empty: Vec<serde_json::Value> = Vec::new();
    let errs = response.data.get("errors").and_then(|v| v.as_array()).unwrap_or(&empty);

    // Group by category for readable output.
    let mut by_category: std::collections::BTreeMap<&str, Vec<&serde_json::Value>> =
        std::collections::BTreeMap::new();
    for e in errs {
        let cat = e.get("category").and_then(|v| v.as_str()).unwrap_or("(unknown)");
        by_category.entry(cat).or_default().push(e);
    }

    eprintln!("{}", "Project validation failed:".red().bold());
    for (cat, entries) in &by_category {
        eprintln!();
        eprintln!("  {} ({})", cat.red(), entries.len());
        for e in entries {
            let path = e.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let message = e.get("message").and_then(|v| v.as_str()).unwrap_or("");
            eprintln!("    {} {}", path.dimmed(), message);
        }
    }

    Err(anyhow!(
        "{} validation error(s) across {} categor{}. Fix and retry.",
        errs.len(),
        by_category.len(),
        if by_category.len() == 1 { "y" } else { "ies" },
    ))
}

/// Check if local project.json matches the server's version.
/// Returns Ok(()) if in sync or if check cannot be performed (missing file, old server).
/// Returns Err if files differ.
async fn check_project_sync(config: &Config) -> Result<()> {
    // Find local project.json
    let project_path = if Path::new("project.json").exists() {
        PathBuf::from("project.json")
    } else if Path::new("../project.json").exists() {
        PathBuf::from("../project.json")
    } else {
        eprintln!("{}", "Warning: project.json not found locally, skipping sync check.".yellow());
        return Ok(());
    };

    let local_content = match fs::read_to_string(&project_path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("{}", "Warning: Could not read local project.json, skipping sync check.".yellow());
            return Ok(());
        }
    };

    let local_json: serde_json::Value = match serde_json::from_str(&local_content) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("{}", "Warning: Could not parse local project.json, skipping sync check.".yellow());
            return Ok(());
        }
    };

    // Fetch server's project.json
    let mut client = match WsClient::connect(&config.get_host(), config.get_port()).await {
        Ok(c) => c,
        Err(_) => {
            eprintln!("{}", "Warning: Could not connect to server for sync check, skipping.".yellow());
            return Ok(());
        }
    };

    let response = client
        .send_command("system.get_project", serde_json::json!({}))
        .await;

    let response = match response {
        Ok(r) => r,
        Err(_) => {
            let _ = client.close().await;
            eprintln!("{}", "Warning: Could not fetch server project, skipping sync check.".yellow());
            return Ok(());
        }
    };

    let _ = client.close().await;

    if !response.success {
        // Server may not support get_project (old version)
        eprintln!("{}", "Warning: Server does not support get_project, skipping sync check.".yellow());
        return Ok(());
    }

    let server_json = response.data;

    if local_json != server_json {
        return Err(anyhow!(
            "Project files differ. Run 'acctl sync' first, or use '--force' to skip."
        ));
    }

    Ok(())
}

/// Check if the control program is stale and print a warning if so.
async fn warn_if_control_stale(config: &Config) {
    let Ok(mut client) = WsClient::connect(&config.get_host(), config.get_port()).await else {
        return;
    };
    let Ok(response) = client
        .send_command("system.control", serde_json::json!({"action": "status"}))
        .await
    else {
        let _ = client.close().await;
        return;
    };
    let _ = client.close().await;

    if response.success {
        let is_stale = response.data.get("control_stale")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if is_stale {
            println!("\n{}", "Warning: Control program is running with outdated code.".yellow().bold());
            println!("  Run '{}' to rebuild.", "acctl push control --start".bold());
        }
    }
}

async fn cmd_sync(config: &Config, dry_run: bool) -> Result<()> {
    sync_project_json(config, dry_run).await?;
    println!();
    sync_datastore(config, dry_run).await?;
    Ok(())
}

async fn sync_project_json(config: &Config, dry_run: bool) -> Result<()> {
    // Find local project.json
    let project_path = if Path::new("project.json").exists() {
        PathBuf::from("project.json")
    } else if Path::new("../project.json").exists() {
        PathBuf::from("../project.json")
    } else {
        return Err(anyhow!("project.json not found in current or parent directory"));
    };

    let local_content = fs::read_to_string(&project_path)?;
    let local_json: serde_json::Value = serde_json::from_str(&local_content)
        .context("Failed to parse local project.json")?;

    // Fetch server's project.json
    println!("Fetching project.json from server...");
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    let response = client
        .send_command("system.get_project", serde_json::json!({}))
        .await?;

    if !response.success {
        client.close().await?;
        return Err(anyhow!("Failed to get server project: {}", response.error_message));
    }

    let server_json = response.data;

    // Semantic comparison
    if local_json == server_json {
        println!("{}", "Project files are in sync.".green());
        client.close().await?;
        return Ok(());
    }

    println!("{}", "Project files differ:".yellow());
    show_json_diff(&local_json, &server_json);

    if dry_run {
        println!("{}", "(dry-run) Skipping interactive resolution.".dimmed());
        client.close().await?;
        return Ok(());
    }

    // Prompt user
    println!();
    println!("  [p]ull  - overwrite local with server version");
    println!("  [u]sh   - push local to server");
    println!("  [s]kip  - do nothing");
    print!("Choice: ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let choice = input.trim().to_lowercase();

    match choice.as_str() {
        "p" | "pull" => {
            let pretty = serde_json::to_string_pretty(&server_json)?;
            fs::write(&project_path, pretty)?;
            println!("{}", "Local project.json updated from server.".green());
            client.close().await?;
            // Regenerate gm.rs so control code stays in sync with the new project.json
            println!("Regenerating codegen...");
            cmd_codegen(config, true).await?;
            warn_if_control_stale(config).await;
            return Ok(());
        }
        "u" | "push" => {
            // Validate locally-edited project against the server's AMS
            // registry BEFORE pushing. A bad file should never reach the
            // server — that's what produced the "no --config passed"
            // silent failure before this command existed.
            if let Err(e) = validate_project_remote(&mut client, Some(&local_json)).await {
                client.close().await?;
                return Err(e);
            }
            let response = client
                .send_command(
                    "system.upload_project",
                    serde_json::json!({
                        "project_json": local_json,
                        "restart": false
                    }),
                )
                .await?;

            if !response.success {
                client.close().await?;
                return Err(anyhow!("Push failed: {}", response.error_message));
            }
            println!("{}", "Server project.json updated from local.".green());
            client.close().await?;
            // Regenerate gm.rs so control code stays in sync with the new project.json
            println!("Regenerating codegen...");
            cmd_codegen(config, true).await?;
            warn_if_control_stale(config).await;
            return Ok(());
        }
        "s" | "skip" => {
            println!("Skipped.");
        }
        _ => {
            println!("Unknown choice, skipping.");
        }
    }

    client.close().await?;
    Ok(())
}

// ============================================================================
// Datastore sync / pull-results / push-scripts
// ============================================================================

/// Locate the local project root (directory containing project.json).
fn find_project_root() -> Result<PathBuf> {
    if Path::new("project.json").exists() {
        return Ok(PathBuf::from("."));
    }
    if Path::new("../project.json").exists() {
        return Ok(PathBuf::from(".."));
    }
    Err(anyhow!("project.json not found in current or parent directory"))
}

/// Walk a directory and return [(rel_path, mtime_ms, size)] for every file.
/// Empty list if the dir doesn't exist. `excludes` filters by relative-path prefix.
fn walk_local_files(root: &Path, excludes: &[&str]) -> Result<Vec<(String, i64, u64)>> {
    use walkdir::WalkDir;
    if !root.exists() { return Ok(Vec::new()); }
    let mut out = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() { continue; }
        let rel = match entry.path().strip_prefix(root) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };
        if excludes.iter().any(|p| rel.starts_with(p)) { continue; }
        let meta = entry.metadata()?;
        let mtime_ms = meta.modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        out.push((rel, mtime_ms, meta.len()));
    }
    Ok(out)
}

/// Mtime-wins bidirectional sync of <project>/datastore (excluding results/).
async fn sync_datastore(config: &Config, dry_run: bool) -> Result<()> {
    let project_root = find_project_root()?;
    let local_datastore = project_root.join("datastore");
    let exclude_results = ["results/"];

    println!("Syncing datastore (excluding results/)...");

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let server_resp = client
        .send_command(
            "system.list_datastore",
            serde_json::json!({ "exclude_prefixes": exclude_results }),
        )
        .await?;
    if !server_resp.success {
        client.close().await?;
        return Err(anyhow!("list_datastore failed: {}", server_resp.error_message));
    }

    let server_files: Vec<(String, i64)> = server_resp.data["files"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| {
            Some((
                v.get("path")?.as_str()?.to_string(),
                v.get("mtime_ms")?.as_i64()?,
            ))
        }).collect())
        .unwrap_or_default();

    let local_files = walk_local_files(&local_datastore, &exclude_results)?;

    use std::collections::HashMap;
    let server_map: HashMap<&str, i64> = server_files.iter()
        .map(|(p, m)| (p.as_str(), *m)).collect();
    let local_map:  HashMap<&str, i64> = local_files.iter()
        .map(|(p, m, _)| (p.as_str(), *m)).collect();

    // Tolerate small clock skew / FS resolution differences.
    const SKEW_MS: i64 = 2000;

    let mut to_pull: Vec<String> = Vec::new();   // server → local
    let mut to_push: Vec<String> = Vec::new();   // local  → server

    for (path, server_mt) in &server_map {
        match local_map.get(path) {
            Some(local_mt) => {
                if *server_mt > *local_mt + SKEW_MS { to_pull.push(path.to_string()); }
                else if *local_mt > *server_mt + SKEW_MS { to_push.push(path.to_string()); }
            }
            None => to_pull.push(path.to_string()),
        }
    }
    for (path, _, _) in &local_files {
        if !server_map.contains_key(path.as_str()) { to_push.push(path.clone()); }
    }

    if to_pull.is_empty() && to_push.is_empty() {
        println!("  {}", "datastore in sync".green());
        client.close().await?;
        return Ok(());
    }

    if !to_pull.is_empty() {
        println!("  {} {} file(s) to pull from server:", "↓".cyan(), to_pull.len());
        for p in &to_pull { println!("    {}", p); }
    }
    if !to_push.is_empty() {
        println!("  {} {} file(s) to push to server:", "↑".cyan(), to_push.len());
        for p in &to_push { println!("    {}", p); }
    }

    if dry_run {
        println!("  {}", "(dry-run) no changes applied".dimmed());
        client.close().await?;
        return Ok(());
    }

    // Pull: ask server for a zip of just these paths, extract on top of local.
    if !to_pull.is_empty() {
        let resp = client.send_command(
            "system.download_datastore",
            serde_json::json!({ "paths": to_pull }),
        ).await?;
        if !resp.success {
            client.close().await?;
            return Err(anyhow!("download_datastore failed: {}", resp.error_message));
        }
        let b64 = resp.data["data"].as_str()
            .ok_or_else(|| anyhow!("download_datastore: missing 'data'"))?;
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64)
            .context("base64 decode")?;
        std::fs::create_dir_all(&local_datastore)?;
        let n = extract_zip_preserving_mtime(&bytes, &local_datastore)?;
        println!("  pulled {} file(s)", n);
    }

    // Push: zip local versions of `to_push` and send.
    if !to_push.is_empty() {
        let zip_bytes = build_zip_from_paths(&local_datastore, &to_push)?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&zip_bytes);
        let resp = client.send_command(
            "system.upload_datastore",
            serde_json::json!({ "data": b64, "preserve_mtime": true }),
        ).await?;
        if !resp.success {
            client.close().await?;
            return Err(anyhow!("upload_datastore failed: {}", resp.error_message));
        }
        let n = resp.data["files_extracted"].as_u64().unwrap_or(0);
        println!("  pushed {} file(s)", n);
    }

    client.close().await?;
    Ok(())
}

/// Build a zip of the given relative paths under `root`, preserving mtimes.
fn build_zip_from_paths(root: &Path, paths: &[String]) -> Result<Vec<u8>> {
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;
    let mut buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buffer);
    let base_options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for rel in paths {
        let full = root.join(rel);
        if !full.is_file() { continue; }
        let content = fs::read(&full)
            .with_context(|| format!("read {}", rel))?;
        let opts = full.metadata().ok()
            .and_then(|m| m.modified().ok())
            .and_then(systemtime_to_ziptime)
            .map(|dt| base_options.last_modified_time(dt))
            .unwrap_or(base_options);
        zip.start_file(rel, opts)?;
        zip.write_all(&content)?;
    }
    zip.finish()?;
    Ok(buffer.into_inner())
}

/// Extract a zip onto `target_dir`, preserving each entry's mtime so the
/// next sync sees the right relative ages.
fn extract_zip_preserving_mtime(zip_data: &[u8], target_dir: &Path) -> Result<usize> {
    use std::io::Cursor;
    use zip::ZipArchive;
    fs::create_dir_all(target_dir)?;
    let mut archive = ZipArchive::new(Cursor::new(zip_data))?;
    let mut count = 0usize;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(p) => target_dir.join(p),
            None => continue,
        };
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
            continue;
        }
        if let Some(parent) = outpath.parent() { fs::create_dir_all(parent)?; }
        let mut outfile = fs::File::create(&outpath)?;
        std::io::copy(&mut file, &mut outfile)?;
        if let Some(dt) = file.last_modified() {
            if let Some(t) = ziptime_to_systemtime(&dt) {
                let _ = filetime::set_file_mtime(&outpath, filetime::FileTime::from_system_time(t));
            }
        }
        count += 1;
    }
    Ok(count)
}

fn systemtime_to_ziptime(t: std::time::SystemTime) -> Option<zip::DateTime> {
    use chrono::{Datelike, Timelike, TimeZone, Utc};
    let secs = t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs() as i64;
    let dt = Utc.timestamp_opt(secs, 0).single()?;
    zip::DateTime::from_date_and_time(
        dt.year() as u16, dt.month() as u8, dt.day() as u8,
        dt.hour() as u8,  dt.minute() as u8, dt.second() as u8,
    ).ok()
}

fn ziptime_to_systemtime(dt: &zip::DateTime) -> Option<std::time::SystemTime> {
    use chrono::{NaiveDate, NaiveTime, NaiveDateTime, TimeZone, Utc};
    let date = NaiveDate::from_ymd_opt(dt.year() as i32, dt.month() as u32, dt.day() as u32)?;
    let time = NaiveTime::from_hms_opt(dt.hour() as u32, dt.minute() as u32, dt.second() as u32)?;
    let naive = NaiveDateTime::new(date, time);
    let utc = Utc.from_utc_datetime(&naive);
    Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(utc.timestamp() as u64))
}

/// `acctl pull results` — download server's results/ tree into local datastore/results/.
async fn cmd_pull_results(config: &Config) -> Result<()> {
    let project_root = find_project_root()?;
    let local_datastore = project_root.join("datastore");

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let list = client.send_command(
        "system.list_datastore",
        serde_json::json!({ "prefix": "results/" }),
    ).await?;
    if !list.success {
        client.close().await?;
        return Err(anyhow!("list_datastore: {}", list.error_message));
    }
    let paths: Vec<String> = list.data["files"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.get("path").and_then(|s| s.as_str()).map(String::from)).collect())
        .unwrap_or_default();
    if paths.is_empty() {
        println!("{}", "Server has no results to pull.".dimmed());
        client.close().await?;
        return Ok(());
    }

    println!("Pulling {} results file(s)...", paths.len());
    let resp = client.send_command(
        "system.download_datastore",
        serde_json::json!({ "paths": paths }),
    ).await?;
    if !resp.success {
        client.close().await?;
        return Err(anyhow!("download_datastore: {}", resp.error_message));
    }
    let bytes = base64::engine::general_purpose::STANDARD.decode(
        resp.data["data"].as_str().ok_or_else(|| anyhow!("missing 'data'"))?,
    )?;
    fs::create_dir_all(&local_datastore)?;
    let n = extract_zip_preserving_mtime(&bytes, &local_datastore)?;
    println!("{} pulled {} file(s) into {:?}", "✓".green(), n, local_datastore.join("results"));
    client.close().await?;
    Ok(())
}

/// `acctl push scripts` — upload local datastore/scripts/ to the server.
async fn cmd_push_scripts(config: &Config) -> Result<()> {
    let project_root = find_project_root()?;
    let local_scripts = project_root.join("datastore").join("scripts");
    if !local_scripts.is_dir() {
        return Err(anyhow!("No local datastore/scripts/ directory at {:?}", local_scripts));
    }

    // Build relative paths under datastore/ (so server-side extraction
    // lands them at <datastore>/scripts/...).
    let local_datastore = project_root.join("datastore");
    let entries = walk_local_files(&local_datastore, &[])?;
    let paths: Vec<String> = entries.into_iter()
        .map(|(p, _, _)| p)
        .filter(|p| p.starts_with("scripts/"))
        .collect();
    if paths.is_empty() {
        println!("{}", "datastore/scripts/ is empty; nothing to push.".dimmed());
        return Ok(());
    }

    println!("Pushing {} script file(s)...", paths.len());
    let zip_bytes = build_zip_from_paths(&local_datastore, &paths)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&zip_bytes);

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let resp = client.send_command(
        "system.upload_datastore",
        serde_json::json!({ "data": b64, "preserve_mtime": true }),
    ).await?;
    if !resp.success {
        client.close().await?;
        return Err(anyhow!("upload_datastore: {}", resp.error_message));
    }
    let n = resp.data["files_extracted"].as_u64().unwrap_or(0);
    println!("{} pushed {} file(s)", "✓".green(), n);
    client.close().await?;
    Ok(())
}

// ============================================================================
// Generic Command Execution
// ============================================================================

/// Parse a string value into a serde_json::Value, attempting number/bool/JSON
/// before falling back to a plain string. Matches the autocore console behavior.
fn parse_arg_value(val: &str) -> serde_json::Value {
    if val == "true" {
        return serde_json::Value::Bool(true);
    }
    if val == "false" {
        return serde_json::Value::Bool(false);
    }
    if let Ok(n) = val.parse::<i64>() {
        return serde_json::json!(n);
    }
    if let Ok(n) = val.parse::<f64>() {
        return serde_json::json!(n);
    }
    if val.starts_with('{') || val.starts_with('[') {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(val) {
            return v;
        }
    }
    serde_json::Value::String(val.to_string())
}

/// Convert a list of CLI arguments into a JSON data object, using the same
/// conventions as the autocore web console:
///   --name value   → { "name": value }
///   -f value       → { "f": value }
///   --flag         → { "flag": true }
///   positional     → collected into "_args" array; if exactly one, also set as "action"
fn args_to_data(args: Vec<String>) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    let mut positional: Vec<serde_json::Value> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if let Some(flag_name) = arg.strip_prefix("--") {
            // Long flag: --name value or --flag
            let next = args.get(i + 1);
            if let Some(next_val) = next {
                if !next_val.starts_with('-') || next_val.parse::<f64>().is_ok() {
                    map.insert(flag_name.to_string(), parse_arg_value(next_val));
                    i += 2;
                    continue;
                }
            }
            map.insert(flag_name.to_string(), serde_json::Value::Bool(true));
            i += 1;
        } else if arg.starts_with('-') && arg.len() == 2 {
            // Short flag: -f value or -f
            let flag_name = &arg[1..];
            let next = args.get(i + 1);
            if let Some(next_val) = next {
                if !next_val.starts_with('-') || next_val.parse::<f64>().is_ok() {
                    map.insert(flag_name.to_string(), parse_arg_value(next_val));
                    i += 2;
                    continue;
                }
            }
            map.insert(flag_name.to_string(), serde_json::Value::Bool(true));
            i += 1;
        } else {
            // Positional argument
            positional.push(parse_arg_value(arg));
            i += 1;
        }
    }

    if !positional.is_empty() {
        if positional.len() == 1 {
            if let Some(s) = positional[0].as_str() {
                map.insert("action".to_string(), serde_json::Value::String(s.to_string()));
            }
        }
        map.insert("_args".to_string(), serde_json::Value::Array(positional));
    }

    serde_json::Value::Object(map)
}

async fn cmd_cmd(config: &Config, topic: &str, args: Vec<String>) -> Result<()> {
    // Validate topic format (must contain a dot)
    if !topic.contains('.') {
        return Err(anyhow!(
            "Invalid topic format '{}'. Expected domain.command (e.g. ethercat.configure)",
            topic
        ));
    }

    let data = args_to_data(args);

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    let response = client.send_command(topic, data).await?;

    client.close().await?;

    if response.success {
        // Print response data
        if response.data.is_null() {
            println!("{}", "OK".green());
        } else {
            let pretty = serde_json::to_string_pretty(&response.data)?;
            println!("{}", pretty);
        }
    } else {
        return Err(anyhow!("Error: {}", response.error_message));
    }

    Ok(())
}

// ============================================================================
// New Project Scaffolding
// ============================================================================

/// Write a template file, creating parent directories as needed.
fn write_template(base: &Path, rel_path: &str, content: &str) -> Result<()> {
    let full_path = base.join(rel_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&full_path, content)?;
    Ok(())
}

async fn cmd_new(name: String) -> Result<()> {
    // Validate project name
    if name.is_empty() {
        return Err(anyhow!("Project name cannot be empty"));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(anyhow!(
            "Project name must contain only alphanumeric characters, underscores, and hyphens"
        ));
    }

    let project_dir = PathBuf::from(&name);
    if project_dir.exists() {
        return Err(anyhow!("Directory '{}' already exists", name));
    }

    println!("Creating project '{}'...", name);

    let sub = |content: &str| content.replace("{name}", &name);

    // Root files
    write_template(&project_dir, "project.json", &sub(PROJECT_JSON))?;
    write_template(&project_dir, ".gitignore", GITIGNORE)?;
    write_template(&project_dir, "datastore/autocore_gnv.ini", &sub(GNV_INI))?;
    println!("  Created project.json");

    // control/
    write_template(&project_dir, "control/Cargo.toml", &sub(CONTROL_CARGO_TOML))?;
    write_template(&project_dir, "control/src/main.rs", CONTROL_MAIN_RS)?;
    write_template(&project_dir, "control/src/program.rs", CONTROL_PROGRAM_RS)?;
    write_template(&project_dir, "control/src/gm.rs", CONTROL_GM_RS)?;
    println!("  Created control/ (Rust control program)");

    // www/
    write_template(&project_dir, "www/package.json", &sub(WWW_PACKAGE_JSON))?;
    write_template(&project_dir, "www/vite.config.ts", WWW_VITE_CONFIG_TS)?;
    write_template(&project_dir, "www/tsconfig.json", WWW_TSCONFIG_JSON)?;
    write_template(&project_dir, "www/tsconfig.node.json", WWW_TSCONFIG_NODE_JSON)?;
    write_template(&project_dir, "www/index.html", &sub(WWW_INDEX_HTML))?;
    write_template(&project_dir, "www/src/main.tsx", WWW_MAIN_TSX)?;
    write_template(&project_dir, "www/src/App.tsx", &sub(WWW_APP_TSX))?;
    write_template(&project_dir, "www/src/styles.css", WWW_STYLES_CSS)?;
    write_template(&project_dir, "www/src/vite-env.d.ts", WWW_VITE_ENV_DTS)?;
    write_template(&project_dir, "www/src/AutoCore.ts", WWW_AUTOCORE_TS)?;
    write_template(&project_dir, "www/src/AutoCoreTags.ts", WWW_AUTOCORE_TAGS_TS)?;
    println!("  Created www/ (React web UI)");

    // doc/
    write_template(&project_dir, "doc/book.toml", &sub(DOC_BOOK_TOML))?;
    write_template(&project_dir, "doc/src/SUMMARY.md", DOC_SUMMARY_MD)?;
    write_template(&project_dir, "doc/src/introduction.md", &sub(DOC_INTRO_MD))?;
    write_template(&project_dir, "doc/src/control_api.md", DOC_CONTROL_API_MD)?;
    write_template(&project_dir, "doc/src/variables.md", DOC_VARIABLES_MD)?;
    println!("  Created doc/ (mdBook user manual)");

    println!("  Created datastore/");

    // git init
    let git_status = std::process::Command::new("git")
        .arg("init")
        .current_dir(&project_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match git_status {
        Ok(s) if s.success() => println!("  Initialized git repository"),
        _ => println!("  Warning: git init failed (git may not be installed)"),
    }

    println!();
    println!("{}", format!("Project '{}' created!", name).green());
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  acctl set-target <server-ip>");
    println!("  acctl push project --restart    # Upload project.json to server");
    println!("  acctl push control --start      # Build, deploy, and start control program");
    println!("  cd www && npm install && npm run dev   # Start web UI dev server");

    Ok(())
}

// ============================================================================
// `acctl new-tis-project` — Phase 6 of the TIS plan
// ============================================================================

/// Project.json for `acctl new-tis-project`. Carries one fully-wired
/// `test_methods` entry (`translational_traction`) plus a couple of
/// trivial GM variables so the gm.rs codegen has something to chew on
/// the first time the user runs `acctl codegen`. The TIS-specific
/// readiness scalars (`tis_staged*`, `tis_active*`) are auto-injected
/// by `Project::normalize()` on server load and don't need to live
/// here.
const TIS_PROJECT_JSON: &str = r#"{
  "name": "{name}",
  "version": "0.1.0",
  "description": "AutoCore project — Test Information System scaffold.",
  "control": {
    "enable": false,
    "release": false,
    "source_directory": "control",
    "entry_point": "main.rs"
  },
  "modules": {},
  "variables": {
    "sample_id": {
      "type": "string",
      "max_length": 64,
      "description": "Operator-supplied sample identifier for the current run.",
      "ux": true
    }
  },
  "test_methods": {
    "translational_traction": {
      "project_fields": [
        { "name": "customer", "type": "string", "required": true },
        { "name": "operator", "type": "string" }
      ],
      "config_fields": [
        { "name": "specimen_notes", "type": "string" },
        { "name": "control_load",   "type": "f32", "units": "N" }
      ],
      "cycle_fields": [
        { "name": "cycle_index", "type": "u32" },
        { "name": "actual_load", "type": "f32", "units": "N" }
      ],
      "results_fields": [
        { "name": "avg_load", "type": "f32", "units": "N" }
      ],
      "views": {
        "load_per_cycle": {
          "type": "cycle_scatter",
          "x": { "field": "cycle_index", "label": "Cycle" },
          "y": [ { "field": "actual_load", "label": "Load (N)" } ]
        }
      }
    }
  }
}
"#;

/// Minimal control program that drives one TIS lifecycle per scan via
/// `tick_with_autostart`, records cycles, and finishes when `req_stop`
/// is set. The user replaces the body of the running state with their
/// real machine logic.
const TIS_CONTROL_PROGRAM_RS: &str = r#"use autocore_std::{ControlProgram, TickContext};
use crate::gm::{GlobalMemory, TestInformationSystem};

pub struct MyProgram {
    tis: TestInformationSystem,
}

impl MyProgram {
    pub fn new() -> Self {
        Self { tis: TestInformationSystem::new() }
    }
}

impl ControlProgram for MyProgram {
    type Memory = GlobalMemory;

    fn process_tick(&mut self, ctx: &mut TickContext<Self::Memory>) {
        // Drain pending IPC + try to start a staged test on this tick.
        // Returns Some(test_type) only on the tick a new run actually
        // begins; use it to gate "first cycle" logic if you need to.
        if let Some(_test_type) = self.tis.tick_with_autostart(ctx) {
            log::info!("[ctrl] new test started — initialising cycle state");
        }

        // Record one cycle per tick when active. record_cycle is a
        // no-op while no test is active.
        self.tis.record_cycle(ctx);

        // End the test when the operator clears the run from the HMI.
        // The standard pattern: HMI calls tis.clear_staged when Cancel
        // is pressed; the control program calls end_active when its
        // own machine cycle naturally completes.
        // (Replace this stub with your real done condition.)
        // self.tis.end_active(ctx);
    }
}
"#;

/// HMI App.tsx wrapping the TIS components in a `<TisProvider>` and
/// a three-tab layout: Project (select/create + history), Test
/// (sample/method/config), Data (live view). The components
/// self-drive from context — no prop threading needed.
const TIS_WWW_APP_TSX: &str = r#"import { EventEmitterProvider } from '@adcops/autocore-react/core/EventEmitterContext';
import { AutoCoreTagProvider } from '@adcops/autocore-react/core/AutoCoreTagContext';
import { PrimeReactProvider } from 'primereact/api';
import { TabView, TabPanel } from 'primereact/tabview';

import {
    TisProvider,
    ProjectSelector,
    TestSetupForm,
    TestDataView,
    ResultHistoryTable,
} from '@adcops/autocore-react/components';

import { acTagSpec } from './AutoCoreTags';

import 'primereact/resources/primereact.min.css';
import 'primeicons/primeicons.css';

export default function App() {
    return (
        <EventEmitterProvider>
            <PrimeReactProvider>
                <AutoCoreTagProvider tags={acTagSpec} eagerRead>
                    <TisProvider>
                        <TabView>
                            {/* Project tab: pick or create a project, then
                                browse its run history. ResultHistoryTable
                                is project-scoped across methods. */}
                            <TabPanel header="Project">
                                <ProjectSelector />
                                <ResultHistoryTable />
                            </TabPanel>
                            {/* Test tab: per-run setup. Sample ID, Test
                                Method, Test Configuration. Renders an
                                empty state if no project is selected. */}
                            <TabPanel header="Test">
                                <TestSetupForm />
                            </TabPanel>
                            {/* Data tab: live view of the active or
                                selected run. */}
                            <TabPanel header="Data">
                                <TestDataView />
                            </TabPanel>
                        </TabView>
                    </TisProvider>
                </AutoCoreTagProvider>
            </PrimeReactProvider>
        </EventEmitterProvider>
    );
}
"#;

async fn cmd_new_tis_project(name: String) -> Result<()> {
    use autocore_util::templates::{
        GITIGNORE, GNV_INI, CONTROL_CARGO_TOML, CONTROL_MAIN_RS, CONTROL_GM_RS,
        WWW_PACKAGE_JSON, WWW_VITE_CONFIG_TS, WWW_TSCONFIG_JSON, WWW_TSCONFIG_NODE_JSON,
        WWW_INDEX_HTML, WWW_MAIN_TSX, WWW_STYLES_CSS, WWW_VITE_ENV_DTS,
        WWW_AUTOCORE_TS, WWW_AUTOCORE_TAGS_TS,
        DOC_BOOK_TOML, DOC_SUMMARY_MD, DOC_INTRO_MD, DOC_CONTROL_API_MD, DOC_VARIABLES_MD,
    };

    if name.is_empty() {
        return Err(anyhow!("Project name cannot be empty"));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(anyhow!(
            "Project name must contain only alphanumeric characters, underscores, and hyphens"
        ));
    }

    let project_dir = PathBuf::from(&name);
    if project_dir.exists() {
        return Err(anyhow!("Directory '{}' already exists", name));
    }

    println!("Creating TIS project '{}'...", name);

    let sub = |content: &str| content.replace("{name}", &name);

    // Root files — TIS-flavored project.json with one test_methods entry.
    write_template(&project_dir, "project.json", &sub(TIS_PROJECT_JSON))?;
    write_template(&project_dir, ".gitignore", GITIGNORE)?;
    write_template(&project_dir, "datastore/autocore_gnv.ini", &sub(GNV_INI))?;
    println!("  Created project.json (with translational_traction test method)");

    // control/ — main.rs + a TIS-shaped program.rs. gm.rs is the
    // generic stub; running `acctl codegen` against a server with this
    // project.json loaded fills in TestInformationSystem and the
    // per-method TestManagers for real.
    write_template(&project_dir, "control/Cargo.toml", &sub(CONTROL_CARGO_TOML))?;
    write_template(&project_dir, "control/src/main.rs", CONTROL_MAIN_RS)?;
    write_template(&project_dir, "control/src/program.rs", TIS_CONTROL_PROGRAM_RS)?;
    write_template(&project_dir, "control/src/gm.rs", CONTROL_GM_RS)?;
    println!("  Created control/ with TestInformationSystem + tick_with_autostart wiring");

    // www/ — App.tsx wraps everything in <TisProvider> + 3 tabs.
    write_template(&project_dir, "www/package.json", &sub(WWW_PACKAGE_JSON))?;
    write_template(&project_dir, "www/vite.config.ts", WWW_VITE_CONFIG_TS)?;
    write_template(&project_dir, "www/tsconfig.json", WWW_TSCONFIG_JSON)?;
    write_template(&project_dir, "www/tsconfig.node.json", WWW_TSCONFIG_NODE_JSON)?;
    write_template(&project_dir, "www/index.html", &sub(WWW_INDEX_HTML))?;
    write_template(&project_dir, "www/src/main.tsx", WWW_MAIN_TSX)?;
    write_template(&project_dir, "www/src/App.tsx", &sub(TIS_WWW_APP_TSX))?;
    write_template(&project_dir, "www/src/styles.css", WWW_STYLES_CSS)?;
    write_template(&project_dir, "www/src/vite-env.d.ts", WWW_VITE_ENV_DTS)?;
    write_template(&project_dir, "www/src/AutoCore.ts", WWW_AUTOCORE_TS)?;
    write_template(&project_dir, "www/src/AutoCoreTags.ts", WWW_AUTOCORE_TAGS_TS)?;
    println!("  Created www/ with <TisProvider> + 3-tab layout (Setup/Data/History)");

    // doc/
    write_template(&project_dir, "doc/book.toml", &sub(DOC_BOOK_TOML))?;
    write_template(&project_dir, "doc/src/SUMMARY.md", DOC_SUMMARY_MD)?;
    write_template(&project_dir, "doc/src/introduction.md", &sub(DOC_INTRO_MD))?;
    write_template(&project_dir, "doc/src/control_api.md", DOC_CONTROL_API_MD)?;
    write_template(&project_dir, "doc/src/variables.md", DOC_VARIABLES_MD)?;
    println!("  Created doc/ (mdBook user manual)");

    println!("  Created datastore/");

    // git init
    let git_status = std::process::Command::new("git")
        .arg("init")
        .current_dir(&project_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    match git_status {
        Ok(s) if s.success() => println!("  Initialized git repository"),
        _ => println!("  Warning: git init failed (git may not be installed)"),
    }

    println!();
    println!("{}", format!("TIS project '{}' created!", name).green());
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  acctl set-target <server-ip>");
    println!("  acctl push project --restart            # Upload project.json to server");
    println!("  acctl codegen-tags                      # Regenerate gm.rs + tis.ts");
    println!("  acctl push control --start              # Build, deploy, start control program");
    println!("  cd www && npm install && npm run dev    # Start the HMI dev server");
    println!();
    println!("From here, edit the test_methods block in project.json to add your");
    println!("real schema; the Project, Test, and Data tabs all pick it up");
    println!("automatically the next time the page reloads.");

    Ok(())
}

// ============================================================================
// Helpers
// ============================================================================

fn add_dir_to_zip<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    src_dir: &Path,
    prefix: &str,
    options: SimpleFileOptions,
) -> Result<()> {
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip node_modules and hidden files
        if name_str == "node_modules" || name_str.starts_with('.') {
            continue;
        }

        let zip_path = if prefix.is_empty() {
            name_str.to_string()
        } else {
            format!("{}/{}", prefix, name_str)
        };

        if path.is_dir() {
            add_dir_to_zip(zip, &path, &zip_path, options)?;
        } else {
            zip.start_file(&zip_path, options)?;
            let mut file = fs::File::open(&path)?;
            std::io::copy(&mut file, zip)?;
        }
    }

    Ok(())
}

// ============================================================================
// CSV Helpers
// ============================================================================

/// Locate project.json in current or parent directory.
fn find_project_path() -> Result<PathBuf> {
    if Path::new("project.json").exists() {
        Ok(PathBuf::from("project.json"))
    } else if Path::new("../project.json").exists() {
        Ok(PathBuf::from("../project.json"))
    } else {
        Err(anyhow!("project.json not found in current or parent directory"))
    }
}

/// Escape a field for CSV output per RFC 4180.
fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        let escaped = field.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        field.to_string()
    }
}

/// Parse a single CSV row, handling quoted fields with escaped quotes.
fn parse_csv_row(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    // Escaped quote
                    chars.next();
                    current.push('"');
                } else {
                    // End of quoted field
                    in_quotes = false;
                }
            } else {
                current.push(ch);
            }
        } else if ch == '"' {
            in_quotes = true;
        } else if ch == ',' {
            fields.push(current.clone());
            current.clear();
        } else {
            current.push(ch);
        }
    }
    fields.push(current);
    fields
}

// ============================================================================
// Export / Import Variables
// ============================================================================

async fn cmd_export_vars(output: &str) -> Result<()> {
    let project_path = find_project_path()?;
    let content = fs::read_to_string(&project_path)
        .context("Failed to read project.json")?;
    let project: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse project.json")?;

    let variables = match project.get("variables").and_then(|v| v.as_object()) {
        Some(vars) if !vars.is_empty() => vars,
        _ => {
            println!("No variables found in project.json");
            return Ok(());
        }
    };

    let mut names: Vec<&String> = variables.keys().collect();
    names.sort();

    let mut out = String::new();
    out.push_str("name,type,direction,link,description,initial\n");

    for name in &names {
        let var = &variables[*name];
        let var_type = var.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let direction = var.get("direction").and_then(|v| v.as_str()).unwrap_or("");
        let link = var.get("link").and_then(|v| v.as_str()).unwrap_or("");
        let description = var.get("description").and_then(|v| v.as_str()).unwrap_or("");
        let initial = match var.get("initial") {
            Some(v) if !v.is_null() => v.to_string(),
            _ => String::new(),
        };

        out.push_str(&format!(
            "{},{},{},{},{},{}\n",
            csv_escape(name),
            csv_escape(var_type),
            csv_escape(direction),
            csv_escape(link),
            csv_escape(description),
            csv_escape(&initial),
        ));
    }

    fs::write(output, &out).context("Failed to write CSV file")?;
    println!("Exported {} variables to {}", names.len(), output);
    Ok(())
}

async fn cmd_import_vars(input: &str) -> Result<()> {
    let csv_content = fs::read_to_string(input)
        .context(format!("Failed to read CSV file: {}", input))?;

    let mut lines = csv_content.lines();

    // Parse header
    let header_line = lines.next().ok_or_else(|| anyhow!("CSV file is empty"))?;
    let headers = parse_csv_row(header_line);
    let col = |name: &str| -> Option<usize> {
        headers.iter().position(|h| h.trim() == name)
    };
    let col_name = col("name").ok_or_else(|| anyhow!("CSV missing 'name' column"))?;
    let col_type = col("type").ok_or_else(|| anyhow!("CSV missing 'type' column"))?;
    let col_direction = col("direction").ok_or_else(|| anyhow!("CSV missing 'direction' column"))?;
    let col_link = col("link");
    let col_description = col("description");
    let col_initial = col("initial");

    let valid_directions = ["input", "output", "command", "status", "internal"];
    let valid_types = [
        "bool", "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "f32", "f64",
    ];

    // Load project.json
    let project_path = find_project_path()?;
    let content = fs::read_to_string(&project_path)
        .context("Failed to read project.json")?;
    let mut project: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse project.json")?;

    // Ensure variables object exists
    if project.get("variables").is_none() {
        project["variables"] = serde_json::json!({});
    }

    // Build a map of existing links (lowercase) -> variable name for duplicate detection
    let mut existing_links: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    if let Some(vars) = project.get("variables").and_then(|v| v.as_object()) {
        for (var_name, var_val) in vars {
            if let Some(link) = var_val.get("link").and_then(|l| l.as_str()) {
                existing_links.insert(link.to_lowercase(), var_name.clone());
            }
        }
    }

    let mut added = 0usize;
    let mut updated = 0usize;
    let mut skipped = 0usize;

    for (line_num, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let row = parse_csv_row(line);
        let get = |idx: usize| -> String {
            row.get(idx).map(|s| s.trim().to_string()).unwrap_or_default()
        };

        let name = get(col_name);
        if name.is_empty() {
            eprintln!("Warning: row {} has empty name, skipping", line_num + 2);
            skipped += 1;
            continue;
        }

        let var_type = get(col_type);
        if !valid_types.contains(&var_type.as_str()) {
            eprintln!(
                "Warning: row {} ('{}') has invalid type '{}', skipping",
                line_num + 2,
                name,
                var_type
            );
            skipped += 1;
            continue;
        }

        let direction = get(col_direction);
        if !valid_directions.contains(&direction.as_str()) {
            eprintln!(
                "Warning: row {} ('{}') has invalid direction '{}', skipping",
                line_num + 2,
                name,
                direction
            );
            skipped += 1;
            continue;
        }

        let link = col_link.map(|i| get(i)).unwrap_or_default();
        let description = col_description.map(|i| get(i)).unwrap_or_default();
        let initial_str = col_initial.map(|i| get(i)).unwrap_or_default();

        // Check for duplicate link: skip if another variable already uses this link
        if !link.is_empty() {
            let link_lower = link.to_lowercase();
            if let Some(existing_var) = existing_links.get(&link_lower) {
                if existing_var != &name {
                    eprintln!(
                        "Warning: row {} ('{}') has link '{}' already used by '{}', skipping",
                        line_num + 2,
                        name,
                        link,
                        existing_var
                    );
                    skipped += 1;
                    continue;
                }
            }
        }

        let initial: serde_json::Value = if initial_str.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_str(&initial_str).unwrap_or(serde_json::Value::String(initial_str))
        };

        let mut var_obj = serde_json::Map::new();
        var_obj.insert("type".to_string(), serde_json::json!(var_type));
        var_obj.insert("direction".to_string(), serde_json::json!(direction));
        if !link.is_empty() {
            var_obj.insert("link".to_string(), serde_json::json!(link));
        }
        if !description.is_empty() {
            var_obj.insert("description".to_string(), serde_json::json!(description));
        }
        if !initial.is_null() {
            var_obj.insert("initial".to_string(), initial);
        }

        let is_update = project["variables"].get(&name).is_some();
        project["variables"][&name] = serde_json::Value::Object(var_obj);

        // Track the link for duplicate detection within the same import
        if !link.is_empty() {
            existing_links.insert(link.to_lowercase(), name.clone());
        }

        if is_update {
            updated += 1;
        } else {
            added += 1;
        }
    }

    // Write back project.json
    let pretty = serde_json::to_string_pretty(&project)
        .context("Failed to serialize project.json")?;
    fs::write(&project_path, pretty)
        .context("Failed to write project.json")?;

    println!(
        "Imported: {} added, {} updated, {} skipped",
        added, updated, skipped
    );
    Ok(())
}

// ============================================================================
// Dedup Vars
// ============================================================================

async fn cmd_dedup_vars() -> Result<()> {
    let project_path = find_project_path()?;
    let content = fs::read_to_string(&project_path)
        .context("Failed to read project.json")?;
    let mut project: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse project.json")?;

    let variables = match project.get("variables").and_then(|v| v.as_object()) {
        Some(vars) if !vars.is_empty() => vars,
        _ => {
            println!("No variables found in project.json");
            return Ok(());
        }
    };

    // Build link (lowercase) -> Vec<variable_name>
    let mut link_to_vars: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for (var_name, var_val) in variables {
        if let Some(link) = var_val.get("link").and_then(|l| l.as_str()) {
            link_to_vars
                .entry(link.to_lowercase())
                .or_default()
                .push(var_name.clone());
        }
    }

    // Filter to only duplicate groups
    let mut duplicates: Vec<(String, Vec<String>)> = link_to_vars
        .into_iter()
        .filter(|(_, vars)| vars.len() > 1)
        .collect();
    duplicates.sort_by(|a, b| a.0.cmp(&b.0));

    if duplicates.is_empty() {
        println!("{}", "No duplicate links found.".green());
        return Ok(());
    }

    println!(
        "{}",
        format!("Found {} duplicate link(s):", duplicates.len()).yellow()
    );
    println!();

    let mut to_remove: Vec<String> = Vec::new();

    for (link, var_names) in &duplicates {
        println!("Duplicate link: {}", link);
        for (i, var_name) in var_names.iter().enumerate() {
            let var = &variables[var_name];
            let var_type = var.get("type").and_then(|v| v.as_str()).unwrap_or("?");
            let direction = var.get("direction").and_then(|v| v.as_str()).unwrap_or("?");
            println!(
                "  [{}] {}  (type: {}, direction: {})",
                i + 1,
                var_name,
                var_type,
                direction
            );
        }

        // Prompt user
        let options: String = (1..=var_names.len())
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join("/");
        print!("Keep which? [{}]: ", options);
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice.parse::<usize>() {
            Ok(n) if n >= 1 && n <= var_names.len() => {
                // Remove all except the chosen one
                for (i, var_name) in var_names.iter().enumerate() {
                    if i != n - 1 {
                        to_remove.push(var_name.clone());
                    }
                }
                println!(
                    "  Keeping '{}', removing {}",
                    var_names[n - 1],
                    var_names
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != n - 1)
                        .map(|(_, name)| format!("'{}'", name))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            _ => {
                println!("  Invalid choice, skipping this group.");
            }
        }
        println!();
    }

    if to_remove.is_empty() {
        println!("No variables removed.");
        return Ok(());
    }

    // Remove chosen duplicates
    if let Some(vars) = project.get_mut("variables").and_then(|v| v.as_object_mut()) {
        for name in &to_remove {
            vars.remove(name);
        }
    }

    // Write back project.json
    let pretty = serde_json::to_string_pretty(&project)
        .context("Failed to serialize project.json")?;
    fs::write(&project_path, pretty)
        .context("Failed to write project.json")?;

    println!(
        "{}",
        format!("Removed {} duplicate variable(s).", to_remove.len()).green()
    );
    Ok(())
}

// ============================================================================
// Upload File
// ============================================================================

async fn cmd_upload(config: &Config, source: &str, dest: Option<String>) -> Result<()> {
    let source_path = PathBuf::from(source);

    if !source_path.exists() {
        return Err(anyhow!("Source file not found: {}", source));
    }

    if !source_path.is_file() {
        return Err(anyhow!("Source is not a file: {}", source));
    }

    // Determine destination path
    let dest_path = match dest {
        Some(d) => d,
        None => {
            // Default to lib/<filename>
            let filename = source_path
                .file_name()
                .ok_or_else(|| anyhow!("Could not determine filename"))?
                .to_string_lossy();
            format!("lib/{}", filename)
        }
    };

    // Read and encode the file
    let file_data = fs::read(&source_path)
        .context(format!("Failed to read file: {}", source))?;
    let file_size = file_data.len();
    let data_b64 = base64::engine::general_purpose::STANDARD.encode(&file_data);

    println!("Uploading {} ({} bytes) to {}...", source, file_size, dest_path);

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    let response = client
        .send_command(
            "system.upload_file",
            serde_json::json!({
                "path": dest_path,
                "data": data_b64
            }),
        )
        .await?;

    client.close().await?;

    if !response.success {
        return Err(anyhow!("Upload failed: {}", response.error_message));
    }

    let server_path = response.data["path"].as_str().unwrap_or(&dest_path);
    let bytes_written = response.data["size"].as_u64().unwrap_or(file_size as u64);

    println!("{}", "Upload complete!".green());
    println!("  Server path: {}", server_path);
    println!("  Bytes written: {}", bytes_written);

    Ok(())
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle commands that don't need config
    match &cli.command {
        Commands::Clone {
            host,
            project,
            port,
            directory,
            list,
        } => {
            return cmd_clone(host.clone(), *port, project.clone(), directory.clone(), *list).await;
        }
        Commands::SetTarget { ip, port } => {
            return cmd_set_target(ip.clone(), *port).await;
        }
        Commands::New { name } => {
            return cmd_new(name.clone()).await;
        }
        Commands::NewTisProject { name } => {
            return cmd_new_tis_project(name.clone()).await;
        }
        Commands::ExportVars { output } => {
            return cmd_export_vars(&output).await;
        }
        Commands::ImportVars { input } => {
            return cmd_import_vars(&input).await;
        }
        Commands::DedupVars => {
            return cmd_dedup_vars().await;
        }
        Commands::Validate => {
            return cmd_validate().await;
        }
        Commands::Info => {
            return cmd_info().await;
        }
        Commands::Doc { cmd } => {
            return doc::cmd_doc(cmd).await;
        }
        Commands::CodegenTags { force } => {
            return tags::cmd_codegen_tags(*force).await;
        }
        Commands::AddAms => {
            return cmd_add_ams().await;
        }
        Commands::AddTis => {
            return cmd_add_tis().await;
        }
        _ => {}
    }

    // Load config for other commands
    let mut config = Config::load().unwrap_or_default();

    // Apply CLI overrides
    if let Some(host) = cli.host {
        config.server.get_or_insert(ServerConfig::default()).host = Some(host);
    }
    if let Some(port) = cli.port {
        config.server.get_or_insert(ServerConfig::default()).port = Some(port);
    }

    // Dispatch commands
    match cli.command {
        Commands::Clone { .. } => unreachable!(),
        Commands::SetTarget { .. } => unreachable!(),
        Commands::New { .. } => unreachable!(),
        Commands::NewTisProject { .. } => unreachable!(),
        Commands::ExportVars { .. } => unreachable!(),
        Commands::ImportVars { .. } => unreachable!(),
        Commands::DedupVars => unreachable!(),
        Commands::Validate => unreachable!(),
        Commands::Info => unreachable!(),
        Commands::Doc { .. } => unreachable!(),
        Commands::CodegenTags { .. } => unreachable!(),
        Commands::AddAms => unreachable!(),
        Commands::AddTis => unreachable!(),
        Commands::Ams { cmd } => match cmd {
            AmsCommand::Export { output } => cmd_ams_export(&config, &output).await,
            AmsCommand::Import { input, dry_run } => cmd_ams_import(&config, &input, dry_run).await,
            AmsCommand::Backfill { dry_run } => cmd_ams_backfill(&config, dry_run).await,
        },
        Commands::Pull { extract } => cmd_pull(&config, extract).await,
        Commands::Push { what } => match what {
            PushCommands::Project { restart } => cmd_push_project(&config, restart).await,
            PushCommands::Www { source, no_build } => cmd_push_www(&config, source, no_build).await,
            PushCommands::Control {
                source,
                no_build,
                start,
                force,
            } => cmd_push_control(&config, source, no_build, start, force).await,
            PushCommands::Doc { no_build } => cmd_push_doc(&config, no_build).await,
            PushCommands::Scripts => cmd_push_scripts(&config).await,
        },
        Commands::Codegen { force } => cmd_codegen(&config, force).await,
        Commands::Switch {
            project_name,
            restart,
        } => cmd_switch(&config, &project_name, restart).await,
        Commands::Status => cmd_status(&config).await,
        Commands::Logs { follow } => cmd_logs(&config, follow).await,
        Commands::Control { action } => cmd_control(&config, &action).await,
        Commands::Sync { dry_run } => cmd_sync(&config, dry_run).await,
        Commands::PullResults => cmd_pull_results(&config).await,
        Commands::Cmd { topic, args } => cmd_cmd(&config, &topic, args).await,
        Commands::Upload { source, dest } => cmd_upload(&config, &source, dest).await,
    }
}

// ============================================================================
// Validate
// ============================================================================

async fn cmd_validate() -> Result<()> {
    let path = PathBuf::from("project.json");
    if !path.exists() {
        return Err(anyhow!("project.json not found in current directory"));
    }

    let content = fs::read_to_string(&path)?;
    let project: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow!("JSON syntax error: {}", e))?;

    let mut warnings: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    // Check modules
    let module_domains: Vec<String> = if let Some(modules) = project.get("modules").and_then(|m| m.as_object()) {
        for (domain, module) in modules {
            if module.get("config").is_none() {
                warnings.push(format!("Module '{}' has no 'config' field", domain));
            }
        }
        modules.keys().cloned().collect()
    } else {
        warnings.push("No 'modules' section found".to_string());
        Vec::new()
    };

    // Check variables
    let valid_types = ["bool", "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "f32", "f64"];
    let mut var_count = 0;
    let mut link_count = 0;
    let mut var_names = std::collections::HashSet::new();

    if let Some(variables) = project.get("variables").and_then(|v| v.as_object()) {
        for (name, var) in variables {
            var_count += 1;

            // Duplicate check
            let lower = name.to_lowercase();
            if !var_names.insert(lower.clone()) {
                errors.push(format!("Duplicate variable name: '{}'", name));
            }

            // Type check
            match var.get("type").and_then(|t| t.as_str()) {
                None => errors.push(format!("Variable '{}' missing 'type' field", name)),
                Some(t) if !valid_types.contains(&t) => {
                    errors.push(format!("Variable '{}' has invalid type '{}'", name, t));
                }
                _ => {}
            }

            // Link check
            if let Some(link) = var.get("link").and_then(|l| l.as_str()) {
                link_count += 1;
                if let Some((domain, _)) = link.split_once('.') {
                    if !module_domains.iter().any(|d| d == domain) {
                        warnings.push(format!(
                            "Variable '{}' links to '{}' but module '{}' is not configured",
                            name, link, domain
                        ));
                    }
                } else {
                    warnings.push(format!("Variable '{}' link '{}' has no domain prefix", name, link));
                }
            }
        }
    }

    // Print results
    if errors.is_empty() && warnings.is_empty() {
        println!("{}", colored::Colorize::green("✓ project.json is valid"));
    } else {
        for e in &errors {
            println!("{} {}", colored::Colorize::red("ERROR:"), e);
        }
        for w in &warnings {
            println!("{}  {}", colored::Colorize::yellow("WARN:"), w);
        }
    }

    println!("  {} modules, {} variables ({} linked)", module_domains.len(), var_count, link_count);

    if !errors.is_empty() {
        return Err(anyhow!("{} error(s) found", errors.len()));
    }

    Ok(())
}

// ============================================================================
// Info
// ============================================================================

async fn cmd_info() -> Result<()> {
    let path = PathBuf::from("project.json");
    if !path.exists() {
        return Err(anyhow!("project.json not found in current directory"));
    }

    let content = fs::read_to_string(&path)?;
    let project: serde_json::Value = serde_json::from_str(&content)?;

    // Project name
    let name = project.get("name")
        .and_then(|n| n.as_str())
        .or_else(|| {
            std::env::current_dir().ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .as_deref()
                .map(|_| "")  // fallback handled below
        })
        .unwrap_or("unknown");
    let dir_name = std::env::current_dir().ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "unknown".to_string());
    let display_name = if name.is_empty() { &dir_name } else { name };
    println!("Project: {}", colored::Colorize::bold(display_name));

    // Target
    if let Ok(config) = Config::load() {
        println!("Target:  {}:{}", config.get_host(), config.get_port());
    } else if PathBuf::from("acctl.toml").exists() {
        println!("Target:  (configured in acctl.toml)");
    } else {
        println!("Target:  (not set — run acctl set-target)");
    }

    // Modules
    if let Some(modules) = project.get("modules").and_then(|m| m.as_object()) {
        println!("Modules:");
        for (domain, module) in modules {
            let mut details = Vec::new();
            if let Some(config) = module.get("config") {
                if let Some(tasks) = config.get("tasks").and_then(|t| t.as_array()) {
                    let ch_count: usize = tasks.iter()
                        .filter_map(|t| t.get("channels").and_then(|c| c.as_array()))
                        .map(|c| c.len())
                        .sum();
                    details.push(format!("{} tasks, {} channels", tasks.len(), ch_count));
                }
                if let Some(daq) = config.get("daq").and_then(|d| d.as_array()) {
                    if !daq.is_empty() {
                        details.push(format!("{} DAQ", daq.len()));
                    }
                }
            }
            let detail_str = if details.is_empty() { String::new() } else { format!(" ({})", details.join(", ")) };
            println!("  {}{}", domain, detail_str);
        }
    }

    // Variables
    if let Some(variables) = project.get("variables").and_then(|v| v.as_object()) {
        let linked = variables.values().filter(|v| v.get("link").is_some()).count();
        println!("Variables: {} total, {} linked", variables.len(), linked);
    }

    // Control
    let control_dir = PathBuf::from("control");
    if control_dir.exists() {
        let cargo_path = control_dir.join("Cargo.toml");
        if let Ok(cargo_content) = fs::read_to_string(&cargo_path) {
            if let Ok(cargo) = cargo_content.parse::<toml::Value>() {
                let pkg = cargo.get("package").and_then(|p| p.get("name")).and_then(|n| n.as_str()).unwrap_or("unknown");
                println!("Control: {}", pkg);
            }
        }
    }

    // WWW
    let www_dist = PathBuf::from("www/dist");
    if www_dist.exists() {
        if let Ok(meta) = fs::metadata(&www_dist) {
            if let Ok(modified) = meta.modified() {
                let dt: chrono::DateTime<chrono::Local> = modified.into();
                println!("WWW:     www/dist (last modified: {})", dt.format("%Y-%m-%d %H:%M"));
            } else {
                println!("WWW:     www/dist");
            }
        }
    } else if PathBuf::from("www").exists() {
        println!("WWW:     www/ (not built — run npm run build in www/)");
    }

    Ok(())
}

// ============================================================================
// AMS Retrofit + Export/Import (Phase 7 of doc/ams_product_plan.md)
// ============================================================================

/// Locate the project.json in the current directory or the nearest
/// ancestor. Returns an `(path, parsed_json)` tuple. We parse as a raw
/// `serde_json::Value` rather than `Project` so we preserve any keys
/// the Rust parser doesn't know about — `acctl add-*` mustn't drop
/// future fields.
fn load_project_json_relaxed() -> Result<(PathBuf, serde_json::Value)> {
    let mut dir = std::env::current_dir()?;
    loop {
        let candidate = dir.join("project.json");
        if candidate.is_file() {
            let bytes = fs::read(&candidate)?;
            let value: serde_json::Value = serde_json::from_slice(&bytes)
                .with_context(|| format!("parsing {}", candidate.display()))?;
            return Ok((candidate, value));
        }
        if !dir.pop() {
            return Err(anyhow!(
                "project.json not found in current directory or any parent"
            ));
        }
    }
}

fn save_project_json_relaxed(path: &Path, value: &serde_json::Value) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

async fn cmd_add_ams() -> Result<()> {
    let (path, mut value) = load_project_json_relaxed()?;
    let obj = value
        .as_object_mut()
        .ok_or_else(|| anyhow!("project.json is not a JSON object"))?;
    if obj.contains_key("asset_types") {
        println!("{}", "AMS already enabled (asset_types block present).".yellow());
        return Ok(());
    }
    obj.insert(
        "asset_types".to_string(),
        serde_json::Value::Object(Default::default()),
    );
    save_project_json_relaxed(&path, &value)?;
    println!("{}", format!("Wrote asset_types block to {}", path.display()).green());
    println!();
    println!("Next steps:");
    println!("  1. Add custom asset types under `asset_types` in project.json (optional —");
    println!("     load_cell / linear_encoder / spring are built-in).");
    println!("  2. Run `acctl push project --restart` to upload the change.");
    println!("  3. Run `acctl codegen` to refresh control/src/gm.rs and www/src/autocore/ams.ts");
    println!("     with the AMS types and the three baseline ams_* GM scalars.");
    println!("  4. Add `<AmsProvider>` and the AMS components to your HMI:");
    println!("       import {{ AmsProvider, AssetRegistryTable, AssetDetailView }}");
    println!("         from '@adcops/autocore-react';");
    Ok(())
}

async fn cmd_add_tis() -> Result<()> {
    let (path, mut value) = load_project_json_relaxed()?;
    let obj = value
        .as_object_mut()
        .ok_or_else(|| anyhow!("project.json is not a JSON object"))?;
    if obj.contains_key("test_methods") {
        println!("{}", "TIS already enabled (test_methods block present).".yellow());
        return Ok(());
    }
    obj.insert(
        "test_methods".to_string(),
        serde_json::Value::Object(Default::default()),
    );
    save_project_json_relaxed(&path, &value)?;
    println!("{}", format!("Wrote test_methods block to {}", path.display()).green());
    println!();
    println!("Next steps:");
    println!("  1. Declare at least one method under `test_methods` in project.json.");
    println!("     See doc/ch15-test-information-system.md for the schema.");
    println!("  2. Run `acctl push project --restart` then `acctl codegen`.");
    println!("  3. Wrap your HMI tabs in `<TisProvider>` and add <TestSetupForm/>,");
    println!("     <TestDataView/>, <ResultHistoryTable/> from `@adcops/autocore-react`.");
    Ok(())
}

async fn cmd_ams_export(config: &Config, output: &str) -> Result<()> {
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    // 1. List every asset (include retired so the export is complete).
    let list = client
        .send_command("ams.list_assets", serde_json::json!({ "include_retired": true }))
        .await?;
    if !list.success {
        return Err(anyhow!("ams.list_assets failed: {}", list.error_message));
    }
    let assets_index = list
        .data
        .get("assets")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // 2. For each asset: full asset.json, every calibration, usage.
    let mut assets_out = Vec::new();
    for entry in &assets_index {
        let asset_id = entry
            .get("asset_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("registry entry missing asset_id"))?;
        let asset_resp = client
            .send_command("ams.read_asset", serde_json::json!({ "asset_id": asset_id }))
            .await?;
        if !asset_resp.success {
            eprintln!("  warning: read_asset({}) failed: {}", asset_id, asset_resp.error_message);
            continue;
        }
        let cal_list = client
            .send_command("ams.list_calibrations", serde_json::json!({ "asset_id": asset_id }))
            .await?;
        let cal_ids = cal_list
            .data
            .get("cal_ids")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let mut cals = Vec::new();
        for cid in &cal_ids {
            if let Some(s) = cid.as_str() {
                let c = client
                    .send_command(
                        "ams.read_calibration",
                        serde_json::json!({ "asset_id": asset_id, "cal_id": s }),
                    )
                    .await?;
                if c.success {
                    cals.push(c.data);
                }
            }
        }
        let usage = client
            .send_command("ams.read_usage", serde_json::json!({ "asset_id": asset_id }))
            .await?;

        assets_out.push(serde_json::json!({
            "asset":        asset_resp.data,
            "calibrations": cals,
            "usage":        usage.data,
        }));
    }

    let document = serde_json::json!({
        "version":     1,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "registry":    { "assets": assets_index },
        "assets":      assets_out,
    });

    let bytes = serde_json::to_vec_pretty(&document)?;
    fs::write(output, &bytes)?;
    client.close().await?;

    println!(
        "{}",
        format!(
            "Exported {} asset(s) → {} ({} bytes)",
            assets_out.len(),
            output,
            bytes.len()
        )
        .green()
    );
    Ok(())
}

async fn cmd_ams_backfill(config: &Config, dry_run: bool) -> Result<()> {
    let (path, value) = load_project_json_relaxed()?;
    println!("Reading asset_refs from {}", path.display());

    // Collect every (asset_type, location) pair under by_location refs
    // across all test_methods. Dedupe — the same physical fixture may
    // be referenced from multiple methods, but we only want one stub.
    let mut pairs: std::collections::BTreeSet<(String, String)> = Default::default();
    let mut by_id_field_refs = Vec::<(String, String, String)>::new();

    if let Some(methods) = value.get("test_methods").and_then(|v| v.as_object()) {
        for (method_id, method) in methods {
            let Some(refs) = method.get("asset_refs").and_then(|v| v.as_array()) else { continue };
            for r in refs {
                let asset_type = r.get("asset_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let select = r.get("select").and_then(|v| v.as_str()).unwrap_or("");
                if asset_type.is_empty() {
                    continue;
                }
                if select == "by_location" {
                    if let Some(loc) = r.get("location").and_then(|v| v.as_str()) {
                        pairs.insert((asset_type, loc.to_string()));
                    }
                } else if select == "by_id_field" {
                    let field = r.get("field").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    by_id_field_refs.push((method_id.clone(), field, asset_type));
                }
            }
        }
    }

    if pairs.is_empty() && by_id_field_refs.is_empty() {
        println!("No asset_refs declared in any test_method. Nothing to backfill.");
        return Ok(());
    }

    if dry_run {
        println!("{}", "Dry-run: showing what would be created, no changes will be applied.".yellow());
    }

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    // Skip pairs that already have an active asset at that location.
    let existing = client
        .send_command("ams.list_assets", serde_json::json!({ "include_retired": false }))
        .await?;
    let existing_pairs: std::collections::HashSet<(String, String)> = existing
        .data
        .get("assets")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| {
                    let t = e.get("asset_type").and_then(|v| v.as_str())?.to_string();
                    let l = e.get("location").and_then(|v| v.as_str())?.to_string();
                    Some((t, l))
                })
                .collect()
        })
        .unwrap_or_default();

    let mut created = 0usize;
    let mut skipped = 0usize;

    for (asset_type, location) in &pairs {
        if existing_pairs.contains(&(asset_type.clone(), location.clone())) {
            println!("  · {} @ {} — already in registry", asset_type, location);
            skipped += 1;
            continue;
        }
        println!("  + create stub {} @ {}", asset_type, location);
        if !dry_run {
            let resp = client
                .send_command("ams.create_asset", serde_json::json!({
                    "asset_type": asset_type,
                    "location":   location,
                    "custom":     { "_backfilled": true },
                }))
                .await?;
            if !resp.success {
                eprintln!("    create_asset failed: {}", resp.error_message);
                continue;
            }
        }
        created += 1;
    }

    client.close().await?;

    println!();
    println!("{}", format!(
        "Backfill {}: created={} skipped={}",
        if dry_run { "dry-run summary" } else { "complete" }, created, skipped,
    ).green());

    if !by_id_field_refs.is_empty() {
        println!();
        println!("{}", "by_id_field refs need manual setup:".yellow());
        for (method, field, asset_type) in &by_id_field_refs {
            println!("  · {}.asset_refs.{} (asset_type={}) — operator selects this asset_id at stage time", method, field, asset_type);
        }
        println!();
        println!("Use the <AssetRegistryTable>'s 'Add Asset' button (or `acctl cmd ams.create_asset asset_type=…`) to register one,");
        println!("then enter the resulting asset_id into the corresponding config field at test stage time.");
    }

    if !dry_run && created > 0 {
        println!();
        println!("Next step: open the AMS HMI tab and fill in serial numbers + current calibrations on the new stubs.");
    }
    Ok(())
}

async fn cmd_ams_import(config: &Config, input: &str, dry_run: bool) -> Result<()> {
    let bytes = fs::read(input).with_context(|| format!("reading {}", input))?;
    let document: serde_json::Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing {}", input))?;
    let assets = document
        .get("assets")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("input document has no `assets` array"))?
        .clone();

    if dry_run {
        println!("{}", "Dry-run: showing what would be imported, no changes will be applied.".yellow());
    }

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let existing = client
        .send_command("ams.list_assets", serde_json::json!({ "include_retired": true }))
        .await?;
    let existing_ids: std::collections::HashSet<String> = existing
        .data
        .get("assets")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("asset_id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let mut created = 0usize;
    let mut skipped = 0usize;
    let mut cal_added = 0usize;
    let mut usage_merged = 0usize;

    for record in &assets {
        let asset = record.get("asset").cloned().unwrap_or(serde_json::Value::Null);
        let asset_id = asset.get("asset_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let asset_type = asset.get("asset_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if asset_id.is_empty() || asset_type.is_empty() {
            eprintln!("  skipping malformed record (no asset_id / asset_type)");
            continue;
        }

        if existing_ids.contains(&asset_id) {
            println!("  asset {} already exists — leaving in place", asset_id);
            skipped += 1;
        } else {
            println!("  + create {} ({})", asset_id, asset_type);
            if !dry_run {
                let resp = client
                    .send_command("ams.create_asset", serde_json::json!({
                        "asset_id":      asset_id,
                        "asset_type":    asset_type,
                        "serial":        asset.get("serial").cloned().unwrap_or_default(),
                        "location":      asset.get("location").cloned().unwrap_or_default(),
                        "custom":        asset.get("custom").cloned().unwrap_or(serde_json::json!({})),
                        "sub_locations": asset.get("sub_locations").cloned().unwrap_or(serde_json::Value::Null),
                    }))
                    .await?;
                if !resp.success {
                    eprintln!("    create_asset failed: {}", resp.error_message);
                    continue;
                }
            }
            created += 1;
        }

        // Calibrations — append any cal_id not already on disk. Server
        // honours the `cal_id` override added in Phase 7.
        let cals = record
            .get("calibrations")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for cal in &cals {
            let cal_id = cal.get("cal_id").and_then(|v| v.as_str()).unwrap_or("");
            if cal_id.is_empty() {
                continue;
            }
            println!("    + cal {}", cal_id);
            if !dry_run {
                let resp = client
                    .send_command("ams.add_calibration", serde_json::json!({
                        "asset_id":     asset_id,
                        "cal_id":       cal_id,
                        "performed_at": cal.get("performed_at").cloned().unwrap_or_default(),
                        "performed_by": cal.get("performed_by").cloned().unwrap_or_default(),
                        "expires_at":   cal.get("expires_at").cloned().unwrap_or_default(),
                        "values":       cal.get("values").cloned().unwrap_or(serde_json::json!({})),
                        "cert_ref":     cal.get("cert_ref").cloned().unwrap_or_default(),
                        "notes":        cal.get("notes").cloned().unwrap_or_default(),
                    }))
                    .await?;
                if !resp.success && !resp.error_message.contains("already exists") {
                    eprintln!("      add_calibration failed: {}", resp.error_message);
                }
            }
            cal_added += 1;
        }

        // Usage merge — additive, taking max of existing/imported so a
        // stale backup never decreases counts.
        if let Some(usage) = record.get("usage") {
            let imported_cycles = usage.get("cycles").and_then(|v| v.as_u64()).unwrap_or(0);
            let imported_hours  = usage.get("hours_run").and_then(|v| v.as_f64()).unwrap_or(0.0);
            if !dry_run && (imported_cycles > 0 || imported_hours > 0.0) {
                let cur = client
                    .send_command("ams.read_usage", serde_json::json!({ "asset_id": asset_id }))
                    .await?;
                let cur_cycles = cur.data.get("cycles").and_then(|v| v.as_u64()).unwrap_or(0);
                let cur_hours  = cur.data.get("hours_run").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let dc = imported_cycles.saturating_sub(cur_cycles);
                let dh = (imported_hours - cur_hours).max(0.0);
                if dc > 0 || dh > 0.0 {
                    let _ = client
                        .send_command("ams.tick_usage", serde_json::json!({
                            "asset_id":     asset_id,
                            "delta_cycles": dc,
                            "delta_hours":  dh,
                        }))
                        .await?;
                    usage_merged += 1;
                }
            }
        }
    }

    client.close().await?;

    println!();
    println!(
        "{}",
        format!(
            "Import {}: created={} skipped={} calibrations={} usage_merged={}",
            if dry_run { "dry-run summary" } else { "complete" },
            created,
            skipped,
            cal_added,
            usage_merged,
        )
        .green()
    );
    Ok(())
}
