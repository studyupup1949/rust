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

    /// Regenerate gm.rs from server
    Codegen,

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

    /// Compare and sync local vs server project.json
    Sync,

    /// Create a new AutoCore project from template
    New {
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

async fn cmd_push_www(config: &Config, source: bool) -> Result<()> {
    let www_dir = if source {
        PathBuf::from("www")
    } else {
        PathBuf::from("www/dist")
    };

    if !www_dir.exists() {
        return Err(anyhow!(
            "{} not found. {}",
            www_dir.display(),
            if !source {
                "Use --source to push full www/"
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

    let binary_path = control_dir
        .join("target")
        .join(target_dir)
        .join(package_name);

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

async fn cmd_codegen(config: &Config) -> Result<()> {
    println!("Requesting gm.rs regeneration from server...");

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

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

        // Extract just gm.rs
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().ends_with("gm.rs") {
                let mut content = String::new();
                file.read_to_string(&mut content)?;

                let gm_path = PathBuf::from("control/src/gm.rs");
                if let Some(parent) = gm_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&gm_path, &content)?;
                println!("  Updated: {}", gm_path.display());
                return Ok(());
            }
        }

        println!("  Warning: gm.rs not found in download");
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
        if let Some(running) = data.get("Running") {
            let pid = running["pid"].as_u64().unwrap_or(0);
            println!("  Status: {} (PID: {})", "Running".green(), pid);
        } else if let Some(failed) = data.get("Failed") {
            let error = failed["error"].as_str().unwrap_or("unknown");
            println!("  Status: {} ({})", "Failed".red(), error);
        } else if data.as_str() == Some("Stopped") {
            println!("  Status: {}", "Stopped".yellow());
        } else {
            println!("  Status: {:?}", data);
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

async fn cmd_sync(config: &Config) -> Result<()> {
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
            cmd_codegen(config).await?;
            return Ok(());
        }
        "u" | "push" => {
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
            cmd_codegen(config).await?;
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

const TMPL_PROJECT_JSON: &str = r#"{
  "name": "__PROJECT_NAME__",
  "version": "0.1.0",
  "description": "AutoCore project: __PROJECT_NAME__",
  "modules": {},
  "control": {
    "enable": true,
    "source_directory": "./control",
    "entry_point": "main.rs",
    "signals": {
      "tick": {
        "description": "System Tick (10ms)",
        "source": "internal",
        "scan_rate_us": 10000
      }
    }
  },
  "variables": {}
}
"#;

const TMPL_GITIGNORE: &str = r#"target/
node_modules/
dist/
*.log
.DS_Store
Thumbs.db
.env
"#;

const TMPL_GNV_INI: &str = r#"[app]
description="AutoCore Application"
company="AutoCore"
name="__PROJECT_NAME__"
version_minor=0
version_build=0
version_major=0
"#;

const TMPL_CARGO_TOML: &str = r#"[package]
name = "__PROJECT_NAME__"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
autocore-std = "3.3"
log = "0.4"
"#;

const TMPL_CONTROL_MAIN_RS: &str = r#"//
// AUTO-GENERATED BY AUTOCORE-SERVER.
// DO NOT EDIT THIS FILE DIRECTLY.
//
// This file handles the system boilerplate and hands over control to
// the MyControlProgram defined in src/program.rs.
//

mod gm;
mod program;

use autocore_std::autocore_main;
use program::MyControlProgram;

autocore_main!(MyControlProgram, "autocore_cyclic", "tick");
"#;

const TMPL_PROGRAM_RS: &str = r#"use autocore_std::ControlProgram;
use crate::gm::GlobalMemory;

/// Your control program. Add fields to hold internal state.
pub struct MyControlProgram;

impl MyControlProgram {
    pub fn new() -> Self {
        Self
    }
}

impl ControlProgram for MyControlProgram {
    type Memory = GlobalMemory;

    fn initialize(&mut self, _mem: &mut Self::Memory) {
        log::info!("MyControlProgram initialized!");
    }

    fn process_tick(&mut self, _gm: &mut Self::Memory, _cycle: u64) {
        // Called every tick (10ms by default).
        // Access and modify shared variables through gm.
    }
}
"#;

const TMPL_GM_RS: &str = r#"use autocore_std::ChangeTracker;
use serde::{Deserialize, Serialize};

/// Global Memory — shared variables between control program and server.
///
/// This struct is auto-generated by `acctl codegen` once the project
/// is connected to a server. Add variables in project.json, push to
/// the server, then run `acctl codegen` to regenerate this file.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GlobalMemory {}

impl ChangeTracker for GlobalMemory {
    fn get_changes(&mut self) -> Vec<(String, serde_json::Value)> {
        vec![]
    }

    fn apply_update(&mut self, _name: &str, _value: &serde_json::Value) {}
}
"#;

const TMPL_WWW_PACKAGE_JSON: &str = r#"{
  "name": "__PROJECT_NAME__-webui",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "@adcops/autocore-react": "^3.3.2",
    "primeflex": "^3.3.1",
    "primeicons": "^6.0.1",
    "primereact": "^10.6.3",
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.15",
    "@types/react-dom": "^18.2.7",
    "@vitejs/plugin-react": "^4.2.1",
    "typescript": "^5.0.2",
    "vite": "^5.0.0"
  }
}
"#;

const TMPL_VITE_CONFIG_TS: &str = r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    proxy: {
      '/ws': {
        target: 'ws://localhost:8080',
        ws: true
      }
    }
  },
  build: {
    outDir: 'dist'
  }
})
"#;

const TMPL_TSCONFIG_JSON: &str = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
"#;

const TMPL_TSCONFIG_NODE_JSON: &str = r#"{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true,
    "strict": true
  },
  "include": ["vite.config.ts"]
}
"#;

const TMPL_INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>__PROJECT_NAME__ - AutoCore</title>
    <style>
      * {
        box-sizing: border-box;
        margin: 0;
        padding: 0;
      }
      body {
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
        background-color: #1a1a2e;
        color: #eee;
      }
    </style>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#;

const TMPL_MAIN_TSX: &str = r#"import React from 'react'
import ReactDOM from 'react-dom/client'
import { EventEmitterProvider } from '@adcops/autocore-react/core/EventEmitterContext'
import { AutoCoreTagProvider } from '@adcops/autocore-react/core/AutoCoreTagContext'
import { acTagSpec } from './AutoCoreTags'
import App from './App'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <EventEmitterProvider>
      <AutoCoreTagProvider tags={acTagSpec}>
        <App />
      </AutoCoreTagProvider>
    </EventEmitterProvider>
  </React.StrictMode>,
)
"#;

const TMPL_APP_TSX: &str = r#"import { useContext } from 'react';
import { EventEmitterContext } from '@adcops/autocore-react/core/EventEmitterContext';
import { AutoCoreHooks } from './AutoCore';

import './styles.css';

function App() {
    const { isConnected } = useContext(EventEmitterContext);
    const { isLoading } = AutoCoreHooks.useAutoCoreContext();

    const connected = isConnected();

    return (
        <div className="app">
            <header className="app-header">
                <h1>__PROJECT_NAME__</h1>
                <div className={`connection-status ${connected ? 'connected' : 'disconnected'}`}>
                    {isLoading ? 'Loading...' : connected ? 'Connected' : 'Disconnected'}
                </div>
            </header>

            <main className="app-main">
                <div className="getting-started">
                    <h2>Getting Started</h2>
                    <p>Your AutoCore project is ready. Add variables in <code>project.json</code>,
                       push to the server, then run <code>acctl codegen</code> to generate
                       the <code>gm.rs</code> bindings.</p>
                    <p>Define your tags in <code>src/AutoCoreTags.ts</code> and use
                       the hooks from <code>src/AutoCore.ts</code> to read and write values.</p>
                </div>
            </main>

            <footer className="app-footer">
                <p>__PROJECT_NAME__ - AutoCore</p>
            </footer>
        </div>
    );
}

export default App;
"#;

const TMPL_STYLES_CSS: &str = r#"/* Main app container */
.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

/* Header */
.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 2rem;
  background-color: #16213e;
  border-bottom: 2px solid #0f3460;
}

.app-header h1 {
  font-size: 1.5rem;
  font-weight: 600;
  color: #e94560;
}

.connection-status {
  padding: 0.5rem 1rem;
  border-radius: 4px;
  font-weight: 500;
  font-size: 0.875rem;
}

.connection-status.connected {
  background-color: #1b4332;
  color: #52b788;
}

.connection-status.disconnected {
  background-color: #641220;
  color: #e5383b;
}

/* Main content */
.app-main {
  flex: 1;
  padding: 2rem;
  overflow: auto;
}

.getting-started {
  max-width: 640px;
  margin: 2rem auto;
  background-color: #16213e;
  border-radius: 8px;
  padding: 2rem;
  border: 1px solid #0f3460;
}

.getting-started h2 {
  font-size: 1.25rem;
  font-weight: 600;
  color: #e94560;
  margin-bottom: 1rem;
}

.getting-started p {
  color: #a0aec0;
  line-height: 1.6;
  margin-bottom: 0.75rem;
}

.getting-started code {
  background-color: #1a1a2e;
  padding: 0.15rem 0.4rem;
  border-radius: 3px;
  font-size: 0.875rem;
  color: #4cc9f0;
}

/* Footer */
.app-footer {
  padding: 0.5rem 2rem;
  background-color: #16213e;
  border-top: 1px solid #0f3460;
  text-align: center;
  font-size: 0.75rem;
  color: #4a5568;
}
"#;

const TMPL_VITE_ENV_DTS: &str = "/// <reference types=\"vite/client\" />\n";

const TMPL_AUTOCORE_TS: &str = r#"/**
 * AutoCore Typed Hooks
 *
 * Usage:
 *   import { AutoCoreHooks } from './AutoCore';
 *   const { value, write } = AutoCoreHooks.useAutoCoreTag("myTag");
 */

import { useContext, useCallback } from "react";
import { AutoCoreTagContext } from "@adcops/autocore-react/core/AutoCoreTagContext";
import type { TagName } from "./AutoCoreTags";

/**
 * Hook to access a single tag value with type safety.
 */
function useAutoCoreTag<T = unknown>(tagName: TagName): {
    value: T | undefined;
    rawValue: unknown;
    isLoading: boolean;
    write: (value: T) => Promise<void>;
    tap: () => Promise<void>;
} {
    const ctx = useContext(AutoCoreTagContext);

    const write = useCallback(
        async (value: T) => {
            await ctx.write(tagName, value);
        },
        [ctx, tagName]
    );

    const tap = useCallback(async () => {
        await ctx.tap(tagName);
    }, [ctx, tagName]);

    return {
        value: ctx.values[tagName] as T | undefined,
        rawValue: ctx.rawValues[tagName],
        isLoading: ctx.isLoading,
        write,
        tap,
    };
}

/**
 * Hook to access multiple tag values at once.
 */
function useAutoCoreTags(
    tagNames: TagName[]
): {
    values: Record<string, unknown>;
    isLoading: boolean;
    write: (tagName: string, value: unknown) => Promise<void>;
} {
    const ctx = useContext(AutoCoreTagContext);

    const values: Record<string, unknown> = {};
    for (const name of tagNames) {
        values[name] = ctx.values[name];
    }

    return {
        values,
        isLoading: ctx.isLoading,
        write: ctx.write,
    };
}

/**
 * Hook to get the write function for a specific tag.
 */
function useAutoCoreWrite(tagName: TagName) {
    const ctx = useContext(AutoCoreTagContext);

    return useCallback(
        async (value: unknown) => {
            await ctx.write(tagName, value);
        },
        [ctx, tagName]
    );
}

/**
 * Hook to get the tap function for boolean tags.
 */
function useAutoCoreTap(tagName: TagName) {
    const ctx = useContext(AutoCoreTagContext);

    return useCallback(async () => {
        await ctx.tap(tagName);
    }, [ctx, tagName]);
}

/**
 * Hook to access the full context value.
 */
function useAutoCoreContext() {
    return useContext(AutoCoreTagContext);
}

/**
 * AutoCoreHooks namespace - provides all hooks in a single import.
 */
export const AutoCoreHooks = {
    useAutoCoreTag,
    useAutoCoreTags,
    useAutoCoreWrite,
    useAutoCoreTap,
    useAutoCoreContext,
};

export {
    useAutoCoreTag,
    useAutoCoreTags,
    useAutoCoreWrite,
    useAutoCoreTap,
    useAutoCoreContext,
};
"#;

const TMPL_AUTOCORE_TAGS_TS: &str = r#"/**
 * AutoCore Tag Definitions
 *
 * Define your tags here. Each tag maps a display name to a
 * server variable (module.variable_name).
 *
 * Example:
 *   {
 *     fqdn: "modbus.holding_0",
 *     tagName: "holding0",
 *     valueType: "number",
 *     subscriptionOptions: { sampling_interval_ms: 250 }
 *   }
 */

import type { TagConfig } from "@adcops/autocore-react/core/AutoCoreTagTypes";

export const acTagSpec: TagConfig[] = [];

// Export tag names as a type for IntelliSense support
export type TagName = string;
"#;

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

    let sub = |content: &str| content.replace("__PROJECT_NAME__", &name);

    // Root files
    write_template(&project_dir, "project.json", &sub(TMPL_PROJECT_JSON))?;
    write_template(&project_dir, ".gitignore", TMPL_GITIGNORE)?;
    write_template(&project_dir, "datastore/autocore_gnv.ini", &sub(TMPL_GNV_INI))?;
    println!("  Created project.json");

    // control/
    write_template(&project_dir, "control/Cargo.toml", &sub(TMPL_CARGO_TOML))?;
    write_template(&project_dir, "control/src/main.rs", TMPL_CONTROL_MAIN_RS)?;
    write_template(&project_dir, "control/src/program.rs", TMPL_PROGRAM_RS)?;
    write_template(&project_dir, "control/src/gm.rs", TMPL_GM_RS)?;
    println!("  Created control/ (Rust control program)");

    // www/
    write_template(&project_dir, "www/package.json", &sub(TMPL_WWW_PACKAGE_JSON))?;
    write_template(&project_dir, "www/vite.config.ts", TMPL_VITE_CONFIG_TS)?;
    write_template(&project_dir, "www/tsconfig.json", TMPL_TSCONFIG_JSON)?;
    write_template(&project_dir, "www/tsconfig.node.json", TMPL_TSCONFIG_NODE_JSON)?;
    write_template(&project_dir, "www/index.html", &sub(TMPL_INDEX_HTML))?;
    write_template(&project_dir, "www/src/main.tsx", TMPL_MAIN_TSX)?;
    write_template(&project_dir, "www/src/App.tsx", &sub(TMPL_APP_TSX))?;
    write_template(&project_dir, "www/src/styles.css", TMPL_STYLES_CSS)?;
    write_template(&project_dir, "www/src/vite-env.d.ts", TMPL_VITE_ENV_DTS)?;
    write_template(&project_dir, "www/src/AutoCore.ts", TMPL_AUTOCORE_TS)?;
    write_template(&project_dir, "www/src/AutoCoreTags.ts", TMPL_AUTOCORE_TAGS_TS)?;
    println!("  Created www/ (React web UI)");

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
        Commands::Pull { extract } => cmd_pull(&config, extract).await,
        Commands::Push { what } => match what {
            PushCommands::Project { restart } => cmd_push_project(&config, restart).await,
            PushCommands::Www { source } => cmd_push_www(&config, source).await,
            PushCommands::Control {
                source,
                no_build,
                start,
                force,
            } => cmd_push_control(&config, source, no_build, start, force).await,
        },
        Commands::Codegen => cmd_codegen(&config).await,
        Commands::Switch {
            project_name,
            restart,
        } => cmd_switch(&config, &project_name, restart).await,
        Commands::Status => cmd_status(&config).await,
        Commands::Logs { follow } => cmd_logs(&config, follow).await,
        Commands::Control { action } => cmd_control(&config, &action).await,
        Commands::Sync => cmd_sync(&config).await,
        Commands::Cmd { topic, args } => cmd_cmd(&config, &topic, args).await,
    }
}
