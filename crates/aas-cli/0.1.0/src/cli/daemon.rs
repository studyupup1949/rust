use crate::cli::commands::DaemonCommands;
use crate::config::settings::Config;
use std::path::PathBuf;
use std::process::Command;
use std::fs;

const DAEMON_PID_FILE: &str = ".aas.pid";

fn get_pid_file() -> PathBuf {
    if let Ok(data_dir) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(data_dir).join("aas").join(DAEMON_PID_FILE)
    } else {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("aas")
            .join(DAEMON_PID_FILE)
    }
}

pub async fn cmd_daemon(action: &DaemonCommands) {
    match action {
        DaemonCommands::Start { config, foreground } => cmd_daemon_start(config.as_deref(), *foreground).await,
        DaemonCommands::Stop => cmd_daemon_stop().await,
        DaemonCommands::Restart => {
            cmd_daemon_stop().await;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            cmd_daemon_start(None, false).await;
        }
        DaemonCommands::Status => cmd_daemon_status().await,
        DaemonCommands::Logs { follow, level } => cmd_daemon_logs(*follow, level.as_deref()).await,
    }
}

async fn cmd_daemon_start(config_path: Option<&str>, foreground: bool) {
    let config = if let Some(path) = config_path {
        match std::fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
                eprintln!("Invalid config: {}", e);
                Config::default()
            }),
            Err(e) => {
                eprintln!("Cannot read config: {}", e);
                Config::default()
            }
        }
    } else {
        Config::load().unwrap_or_default()
    };

    if foreground {
        // Run in foreground (useful for debugging)
        println!("🚀 Starting AAS daemon (foreground)");
        println!("   Config: {}", Config::config_path().display());
        println!("   Press Ctrl+C to stop");
        println!();

        // Would call cmd_run here, but that's in main.rs
        // For now, just show instructions
        println!("⚠️  Use 'aas run' for foreground mode instead");
    } else {
        // Spawn background daemon
        let pid_file = get_pid_file();
        let _ = fs::create_dir_all(pid_file.parent().unwrap());

        if pid_file.exists() {
            if let Ok(pid_str) = fs::read_to_string(&pid_file) {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    if is_process_running(pid) {
                        println!("⚠️  Daemon already running (PID: {})", pid);
                        return;
                    }
                }
            }
        }

        println!("🚀 Starting AAS daemon...");

        // Spawn: aas run --in-daemon-mode
        let child = Command::new(std::env::current_exe().unwrap_or_else(|_| "aas".into()))
            .arg("run")
            .arg("--daemon-mode")
            .spawn();

        match child {
            Ok(child) => {
                let pid = child.id();
                match fs::write(&pid_file, pid.to_string()) {
                    Ok(()) => {
                        println!("✓ Daemon started (PID: {})", pid);
                        println!("  View logs: aas daemon logs --follow");
                    }
                    Err(_) => {
                        eprintln!("Warning: could not write PID file");
                    }
                }
            }
            Err(e) => {
                eprintln!("✗ Failed to start daemon: {}", e);
            }
        }
    }
}

async fn cmd_daemon_stop() {
    let pid_file = get_pid_file();

    if !pid_file.exists() {
        println!("⚠️  Daemon not running");
        return;
    }

    if let Ok(pid_str) = fs::read_to_string(&pid_file) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            // Kill the process
            #[cfg(unix)]
            {
                let _ = Command::new("kill")
                    .arg(pid.to_string())
                    .spawn();
            }

            #[cfg(windows)]
            {
                let _ = Command::new("taskkill")
                    .arg("/PID")
                    .arg(pid.to_string())
                    .spawn();
            }

            let _ = fs::remove_file(&pid_file);
            println!("✓ Daemon stopped");
        }
    }
}

async fn cmd_daemon_status() {
    let pid_file = get_pid_file();

    if !pid_file.exists() {
        println!("⚠️  Daemon not running");
        return;
    }

    if let Ok(pid_str) = fs::read_to_string(&pid_file) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            if is_process_running(pid) {
                println!("✓ Daemon running (PID: {})", pid);
                println!("  Logs: aas daemon logs --follow");
            } else {
                println!("⚠️  PID file found but process not running");
                let _ = fs::remove_file(&pid_file);
            }
        }
    }
}

async fn cmd_daemon_logs(follow: bool, _level: Option<&str>) {
    let log_dir = if let Ok(data_dir) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(data_dir).join("aas").join("logs")
    } else {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("aas")
            .join("logs")
    };

    if !log_dir.exists() {
        println!("📝 No logs yet");
        return;
    }

    if follow {
        println!("📝 Tailing daemon logs (Ctrl+C to stop):");
        println!("   {}", log_dir.display());
        println!();

        // Would tail -f the log file here
        println!("⚠️  Use system tools: tail -f {}/aas.log", log_dir.display());
    } else {
        println!("📝 Daemon logs at: {}", log_dir.display());
        println!("   View: tail -f {}/aas.log", log_dir.display());
    }
}

#[cfg(unix)]
fn is_process_running(pid: u32) -> bool {
    match std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

#[cfg(windows)]
fn is_process_running(pid: u32) -> bool {
    match std::process::Command::new("tasklist")
        .arg("/FI")
        .arg(format!("PID eq {}", pid))
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains(&pid.to_string())
        }
        Err(_) => false,
    }
}
