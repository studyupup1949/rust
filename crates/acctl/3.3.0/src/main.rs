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
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

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

#[derive(Debug, Serialize, Deserialize)]
struct CommandMessage {
    topic: String,
    data: serde_json::Value,
    message_type: String,
    #[serde(default)]
    success: bool,
    #[serde(default)]
    error_message: String,
}

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

impl WsClient {
    async fn connect(host: &str, port: u16) -> Result<Self> {
        let url = format!("ws://{}:{}/ws", host, port);
        let (ws_stream, _) = connect_async(&url)
            .await
            .context(format!("Failed to connect to {}", url))?;

        let (write, read) = ws_stream.split();
        Ok(WsClient { write, read })
    }

    async fn send_command(
        &mut self,
        topic: &str,
        data: serde_json::Value,
    ) -> Result<CommandMessage> {
        let msg = CommandMessage {
            topic: topic.to_string(),
            data,
            message_type: "Request".to_string(),
            success: false,
            error_message: String::new(),
        };

        let json = serde_json::to_string(&msg)?;
        self.write.send(Message::Text(json)).await?;

        // Wait for response
        let timeout = Duration::from_secs(30);
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            match tokio::time::timeout(Duration::from_secs(1), self.read.next()).await {
                Ok(Some(Ok(Message::Text(text)))) => {
                    let response: CommandMessage = serde_json::from_str(&text)?;
                    if response.topic == topic {
                        return Ok(response);
                    }
                    // Skip broadcast messages
                    if response.message_type == "Broadcast" {
                        continue;
                    }
                }
                Ok(Some(Ok(_))) => continue,
                Ok(Some(Err(e))) => return Err(anyhow!("WebSocket error: {}", e)),
                Ok(None) => return Err(anyhow!("Connection closed")),
                Err(_) => continue, // Timeout, keep trying
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

async fn cmd_push_control(config: &Config, _source: bool, no_build: bool, start: bool) -> Result<()> {
    let control_dir = PathBuf::from("control");
    if !control_dir.exists() {
        return Err(anyhow!("control/ directory not found"));
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

    let binary_size = fs::metadata(&binary_path)?.len();

    // Connect and deploy
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    // Stop if running
    println!("Stopping control program...");
    let _ = client
        .send_command("system.control", serde_json::json!({"action": "stop"}))
        .await;

    // Upload binary
    println!("Uploading binary ({} bytes)...", binary_size);
    let binary_data = fs::read(&binary_path)?;
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

    let path = response.data["path"].as_str().unwrap_or("unknown");
    println!("  Uploaded to: {}", path);

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
                        if msg.topic == "log.stream" {
                            if let Ok(entry) = serde_json::from_value::<LogEntry>(msg.data) {
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
        Commands::Pull { extract } => cmd_pull(&config, extract).await,
        Commands::Push { what } => match what {
            PushCommands::Project { restart } => cmd_push_project(&config, restart).await,
            PushCommands::Www { source } => cmd_push_www(&config, source).await,
            PushCommands::Control {
                source,
                no_build,
                start,
            } => cmd_push_control(&config, source, no_build, start).await,
        },
        Commands::Codegen => cmd_codegen(&config).await,
        Commands::Switch {
            project_name,
            restart,
        } => cmd_switch(&config, &project_name, restart).await,
        Commands::Status => cmd_status(&config).await,
        Commands::Logs { follow } => cmd_logs(&config, follow).await,
        Commands::Control { action } => cmd_control(&config, &action).await,
    }
}
