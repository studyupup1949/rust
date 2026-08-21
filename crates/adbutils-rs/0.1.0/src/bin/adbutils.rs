//! adbutils command-line interface. A subset of `adbutils/__main__.py`.

use std::path::PathBuf;

use adbutils::pidcat::{PidcatOptions, Priority};
use adbutils::{AdbClient, AdbDevice, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "adbutils", about = "Async adb utilities (Rust port of adbutils)")]
struct Cli {
    /// Target device serial (defaults to the sole connected device).
    #[arg(short, long, global = true)]
    serial: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the adb server version.
    Version,
    /// List devices (`-l` for extended info).
    Devices {
        #[arg(short = 'l', long)]
        extended: bool,
    },
    /// Run a shell command on the device.
    Shell {
        /// Command and arguments.
        #[arg(trailing_var_arg = true, required = true)]
        args: Vec<String>,
    },
    /// Capture a screenshot to a PNG file.
    Screenshot { output: PathBuf },
    /// Push a local file to the device.
    Push { local: PathBuf, remote: String },
    /// Pull a file from the device.
    Pull { remote: String, local: PathBuf },
    /// Forward a local socket to a device socket.
    Forward { local: String, remote: String },
    /// List active forwards.
    ForwardList,
    /// Install an APK from a path or URL.
    Install { path_or_url: String },
    /// Record the screen until Ctrl-C.
    Screenrecord { output: PathBuf },
    /// Read a system property.
    Getprop { name: String },
    /// Print the foreground app.
    Current,
    /// List installed packages (`-3` for third-party only).
    Packages {
        #[arg(short = '3', long)]
        third_party: bool,
    },
    /// Colorized logcat stream.
    Pidcat {
        /// Only show lines whose tag contains one of these substrings.
        filters: Vec<String>,
        /// Minimum level: V, D, I, W, E, F.
        #[arg(short = 'l', long, default_value = "V")]
        level: char,
        /// Clear the log buffer first.
        #[arg(short, long)]
        clear: bool,
    },
}

async fn resolve_device(client: &AdbClient, serial: &Option<String>) -> Result<AdbDevice> {
    match serial {
        Some(s) => Ok(client.device(s.clone())),
        None => client.any_device().await,
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let adb = AdbClient::default();

    match cli.command {
        Command::Version => {
            println!("{}", adb.server_version().await?);
        }
        Command::Devices { extended } => {
            for info in adb.list(extended).await? {
                if extended {
                    println!("{}\t{}\t{:?}", info.serial, info.state, info.tags);
                } else {
                    println!("{}\t{}", info.serial, info.state);
                }
            }
        }
        Command::Shell { args } => {
            let d = resolve_device(&adb, &cli.serial).await?;
            print!("{}", d.shell(args).await?);
            println!();
        }
        Command::Screenshot { output } => {
            let d = resolve_device(&adb, &cli.serial).await?;
            let img = d.screenshot(None, true).await?;
            img.save(&output)?;
            println!("saved {}", output.display());
        }
        Command::Push { local, remote } => {
            let d = resolve_device(&adb, &cli.serial).await?;
            let n = d.sync().push(&local, &remote, 0o644, false).await?;
            println!("pushed {n} bytes to {remote}");
        }
        Command::Pull { remote, local } => {
            let d = resolve_device(&adb, &cli.serial).await?;
            let n = d.sync().pull(&remote, &local, false).await?;
            println!("pulled {n} bytes to {}", local.display());
        }
        Command::Forward { local, remote } => {
            let d = resolve_device(&adb, &cli.serial).await?;
            d.forward(&local, &remote, false).await?;
            println!("{local} -> {remote}");
        }
        Command::ForwardList => {
            let d = resolve_device(&adb, &cli.serial).await?;
            for f in d.forward_list().await? {
                println!("{}\t{}\t{}", f.serial, f.local, f.remote);
            }
        }
        Command::Install { path_or_url } => {
            let d = resolve_device(&adb, &cli.serial).await?;
            d.install(&path_or_url, false, false, &["-r", "-t"]).await?;
            println!("installed");
        }
        Command::Screenrecord { output } => {
            let d = resolve_device(&adb, &cli.serial).await?;
            let rec = d.start_recording(&output).await?;
            println!("recording... press Ctrl-C to stop");
            tokio::signal::ctrl_c().await?;
            rec.stop().await?;
            println!("saved {}", output.display());
        }
        Command::Getprop { name } => {
            let d = resolve_device(&adb, &cli.serial).await?;
            println!("{}", d.getprop(&name).await?);
        }
        Command::Current => {
            let d = resolve_device(&adb, &cli.serial).await?;
            let info = d.app_current().await?;
            println!("{}/{} pid={}", info.package, info.activity, info.pid);
        }
        Command::Packages { third_party } => {
            let d = resolve_device(&adb, &cli.serial).await?;
            let filters: &[&str] = if third_party { &["-3"] } else { &[] };
            for p in d.list_packages(filters).await? {
                println!("{p}");
            }
        }
        Command::Pidcat { filters, level, clear } => {
            let d = resolve_device(&adb, &cli.serial).await?;
            let min_priority = Priority::from_letter(level.to_ascii_uppercase());
            let opts = PidcatOptions { min_priority, tag_filters: filters, clear };
            d.pidcat(opts, |line| println!("{line}")).await?;
        }
    }
    Ok(())
}
