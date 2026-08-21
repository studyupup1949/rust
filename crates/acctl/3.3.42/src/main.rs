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
use tokio_tungstenite::tungstenite::Message;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};
use autocore_util::templates::*;

mod asset_management;
mod doc;
mod tags;
mod test_methods;

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

    /// List GNV snapshots on the server. The server takes a snapshot of
    /// `autocore_gnv.ini` before every project push (and before each
    /// snapshot restore) so an unexpected reset of NV values can be
    /// rolled back. Newest first; pass the printed `name` to
    /// `restore-gnv-snapshot` to use one.
    ListGnvSnapshots,

    /// Restore a GNV snapshot. Overwrites the server's
    /// `<datastore>/autocore_gnv.ini` with the named snapshot and
    /// triggers `gm.reinitialize` so the values land in SHM. The
    /// pre-restore state is itself snapshotted first.
    RestoreGnvSnapshot {
        /// Snapshot filename as listed by `list-gnv-snapshots`
        /// (e.g. `autocore_gnv-20260601-143022.ini`).
        name: String,
    },

    /// Create a whole-system backup ON the target machine before an upgrade.
    /// Tar-balls the autocore binaries, server config, all projects, the web
    /// console and the systemd unit into `/srv/autocore/system_backup`, so a
    /// bad upgrade can be rolled back with `remote-restore`. Fast over the link
    /// (the archive stays on the box). Test results/captures are excluded by
    /// default. The newest 5 backups are kept.
    #[command(visible_alias = "remote_backup")]
    RemoteBackup {
        /// Include each project's datastore `results/` and `captures/` (can be
        /// large). Off by default.
        #[arg(long)]
        include_results: bool,

        /// Free-text note stored in the backup manifest (e.g. "pre 3.4 upgrade").
        #[arg(long)]
        note: Option<String>,

        /// Just list the backups already on the server and exit.
        #[arg(long)]
        list: bool,
    },

    /// Update this machine's installed autocore packages from its channel (apt).
    ///
    /// Runs apt on THIS machine — SSH to the target and run it there with acctl
    /// pointed at localhost. RT-safe: it refuses to run mid-test, then restarts
    /// and health-gates the server, auto-rolling-back if the update leaves it
    /// degraded. See UPDATE_SYSTEM_PLAN.md §4.
    Update {
        /// Show which autocore packages have updates available, then exit.
        #[arg(long)]
        list: bool,

        /// Dry run: show what an update would change, without installing.
        #[arg(long)]
        check: bool,

        /// Roll back to the version set recorded before the last update.
        #[arg(long)]
        rollback: bool,

        /// Install a specific previously-recorded version set (timestamp/name
        /// from the update history) instead of the latest.
        #[arg(long)]
        version: Option<String>,

        /// Switch the tracked channel (stable|beta|dev) and exit.
        #[arg(long)]
        channel: Option<String>,

        /// Skip the confirmation prompt.
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Reconcile installed module packages with the active project, or
    /// install/remove a module on demand. RT-safe like `update`. §4.
    Modules {
        #[command(subcommand)]
        cmd: ModulesCommand,
    },

    /// One-time migration of a pre-split machine — where the ethercat binary +
    /// device_definitions.json are still owned by the old autocore_server deb (or
    /// were orphaned/deleted) — onto the standalone autocore-ethercat +
    /// autocore-ethercat-esi packages. Safe to run repeatedly. §3.4.
    Migrate {
        /// Skip the confirmation prompt.
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Manage OFFLINE version snapshots (air-gapped machines). Import a signed
    /// bundle built on the online build host (`export-snapshot.sh`), then switch
    /// between imported snapshots like rustup toolchains — no internet needed.
    /// §11.
    Snapshot {
        #[command(subcommand)]
        cmd: SnapshotCommand,
    },

    /// Restore a whole-system backup on the target machine. With no NAME, lists
    /// the backups (date/time, version, size) and prompts for a selection.
    /// Overwrites binaries, config and projects, re-applies file capabilities,
    /// reloads the systemd unit and restarts the server (the connection drops).
    #[command(visible_alias = "remote_restore")]
    RemoteRestore {
        /// Backup filename as listed by `remote-backup --list`
        /// (e.g. `backup_20260601T143022_v3.3.135.tar.gz`). Omit to choose
        /// interactively.
        name: Option<String>,

        /// Skip the confirmation prompt.
        #[arg(long)]
        yes: bool,
    },

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

    /// Sync with the server. Reconciles project.json (interactive on
    /// diff), then pulls the critical datastore items (autocore_gnv.ini,
    /// assets/). Pass `all` to instead mtime-wins sync the entire
    /// datastore tree (excluding results/) — can be slow on large
    /// projects / remote links.
    Sync {
        /// Sync scope: omit for critical files only; `all` (alias
        /// `datastore`) for the full datastore tree; `backups` to pull
        /// system backups from the server into a local `backups/` dir.
        #[arg(value_parser = ["all", "datastore", "backups"])]
        scope: Option<String>,

        /// Show what would change without applying anything.
        #[arg(long)]
        dry_run: bool,
    },

    /// Deploy the current local project to the target server in one shot,
    /// creating it there if it does not exist. Pushes built artifacts only
    /// (project.json, datastore/gnv, control binary, www/dist) — never source.
    /// Stages everything into the project, activates it, and restarts the
    /// server ONCE so the HMI is live immediately — no manual "push www then
    /// restart again" dance. Ideal for standing a project up on a local dev box.
    #[command(
        after_help = "Examples:\n  acctl deploy                 # deploy the local project (name from project.json)\n  acctl deploy my_project      # deploy under an explicit name\n  acctl deploy --no-control    # skip the control program"
    )]
    Deploy {
        /// Project name on the server. Defaults to the local project.json `name`.
        project_name: Option<String>,

        /// Skip building and pushing the control program.
        #[arg(long)]
        no_control: bool,

        /// Skip building and pushing the web HMI.
        #[arg(long)]
        no_www: bool,

        /// Push already-built artifacts; skip the cargo/npm builds.
        #[arg(long)]
        no_build: bool,

        /// Stage the files but do not activate the project or restart the server.
        #[arg(long)]
        no_restart: bool,
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
    /// without scaffolding a fresh one. Creates an empty `test_methods.json`
    /// sidecar next to project.json so the `tis_*` GM scalars and codegen
    /// kick in next time you run `acctl codegen`. project.json is not
    /// touched.
    AddTis,

    /// Add a CiA-402 axis to the current project, then run `acctl codegen` to
    /// generate its drive handle in control/src/gm.rs. EtherCAT axes bind to a
    /// slave via `--link` and land in `modules.ethercat.config.axes`;
    /// `--backend virtual` creates a fieldbus-less simulated axis in the
    /// backend-neutral `modules.motion.config.axes`. Idempotent on `--name`.
    AddAxis {
        /// Axis name — used for the generated handle struct (e.g. "Press").
        #[arg(long)]
        name: String,
        /// Slave name to bind to (required for the EtherCAT backend; omit for virtual).
        #[arg(long, default_value = "")]
        link: String,
        /// CiA-402 profile type.
        #[arg(long = "type", default_value = "pp")]
        axis_type: String,
        /// Backend: "ethercat" (default) or "virtual".
        #[arg(long, default_value = "ethercat")]
        backend: String,
    },

    /// Asset Management System export/import.
    Ams {
        #[command(subcommand)]
        cmd: AmsCommand,
    },

    /// Registered tools/editors (e.g. labelit-studio). `list` shows what the
    /// server discovered in its registry; `rescan` makes it re-read the
    /// registry and (re)start/stop service tools without a restart — used by
    /// package install/uninstall scripts.
    Tools {
        #[command(subcommand)]
        cmd: ToolsCommand,
    },

    /// Machine-global active configuration (which hardware build this machine
    /// runs). Stored machine-locally in config.ini `[general]
    /// active_configuration`, overriding each module's `default_configuration`.
    /// Never carried in the portable project.json.
    Config {
        #[command(subcommand)]
        cmd: ConfigCommand,
    },
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// List the configuration names declared in the active project, with each
    /// module's declared default.
    List,
    /// Show the effective active configuration and where it comes from
    /// (config.ini override vs. project default).
    Show,
    /// Set the machine-global active configuration (writes config.ini). Requires
    /// a restart to take effect.
    Set {
        /// Configuration name (must exist in the project's `configurations`).
        name: String,
        /// Restart server after setting.
        #[arg(short, long)]
        restart: bool,
    },
    /// Clear the config.ini override so each module falls back to its declared
    /// default_configuration. Requires a restart to take effect.
    Clear {
        /// Restart server after clearing.
        #[arg(short, long)]
        restart: bool,
    },
    /// Validate every configuration in the project (resolves each and checks the
    /// EtherCAT network for unresolved links, PDO-name drift across variants,
    /// and duplicate FQDNs). Bus-free.
    Validate,
}

#[derive(Subcommand)]
enum ToolsCommand {
    /// List tools registered on the server (name, editors, running URL).
    List,
    /// Reconcile running tools with the registry (start new, stop removed).
    Rescan,
}

#[derive(Subcommand)]
enum ModulesCommand {
    /// Show declared-vs-installed module status for the active project
    /// (read-only). Inspects LOCAL installed packages.
    List,
    /// Install every module the active project declares but that isn't
    /// installed; warn about installed modules the project doesn't declare.
    /// This is the safety net that makes the ADC-SN-3833 missing-module state
    /// impossible.
    Sync {
        /// Also REMOVE installed module packages the active project doesn't
        /// declare (default: only warn about them).
        #[arg(long)]
        remove_extras: bool,
        /// Skip confirmation prompts.
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Install a module package on demand (e.g. `python`). Name may be the bare
    /// module (`python`) or the full package (`autocore-python`).
    Install {
        name: String,
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Remove a module package. Refuses (without --force) if the active project
    /// still declares it, since that would degrade the server.
    Remove {
        name: String,
        /// Remove even if the active project declares this module.
        #[arg(long)]
        force: bool,
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum SnapshotCommand {
    /// Import a signed snapshot bundle (from export-snapshot.sh) into the local
    /// offline repo. Verifies the signature, merges the shared pool, and makes
    /// the snapshot available to `use`. Does NOT change what's installed.
    Import {
        /// Path to the bundle tarball.
        bundle: String,
    },
    /// List imported snapshots (the offline "toolchain" library) and mark the
    /// active one.
    List,
    /// Switch to an imported snapshot — installs its exact package set, restarts,
    /// health-gates, and auto-rolls-back to the previously active snapshot if the
    /// server doesn't come back healthy.
    Use {
        name: String,
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Remove an imported snapshot and garbage-collect pool files no remaining
    /// snapshot references. Refuses to remove the active snapshot.
    Remove {
        name: String,
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Detach from offline mode when a machine moves to online updates: disables
    /// the local offline apt source so it can't interfere with (or block) an
    /// online `acctl update`. Keeps the imported snapshots by default so you can
    /// go back offline later; pass --purge to also delete the local repo and
    /// reclaim disk.
    Detach {
        /// Also delete the local offline repo (pool + snapshots) to free space.
        #[arg(long)]
        purge: bool,
        #[arg(short = 'y', long)]
        yes: bool,
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
    /// Walk every `asset_ref` declared in this project's test methods
    /// (test_methods.json sidecar, or a legacy embedded block in
    /// project.json) and create one stub asset in the AMS
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

    /// Publish the local AMS data (datastore/assets/) to the server.
    ///
    /// `acctl sync` pulls AMS asset/calibration records from the server but
    /// never pushes them: that data is machine-local (the transducer in
    /// THIS machine, its cert history, usage counters) while project.json is
    /// shared across machines, so an auto-push could clobber another
    /// machine's assets. Use this command to deliberately publish this
    /// machine's AMS state to the shared server — e.g. after registering or
    /// recalibrating an asset the other machines should see.
    ///
    /// Additive: it uploads local files but does not delete server-side
    /// assets that were removed locally. For full reconciliation use
    /// `acctl ams export` / `acctl ams import`. After upload it calls
    /// `ams.reinitialize` so the running server reloads from disk.
    Assets {
        /// Skip the post-upload `ams.reinitialize` call. Without a reinit
        /// or a restart, the server's next AMS write can clobber the pushed
        /// files from stale in-memory state. Only set this if you're about
        /// to restart the server.
        #[arg(long)]
        no_reinit: bool,
    },

    /// Push the local test_methods.json sidecar to the server (restore-from-backup).
    ///
    /// TIS methods are authored on the machine via the HMI's method editor,
    /// so the machine is the source of truth for test_methods.json:
    /// `acctl sync` pulls it down but never pushes it, and `acctl push
    /// project` doesn't carry it either. Use this command to deliberately
    /// OVERWRITE the machine's methods with the local sidecar — e.g.
    /// restoring a known-good backup or seeding a fresh machine. The
    /// server backs the previous file up to test_methods.json.bak first.
    TestMethods {
        /// Skip the confirmation prompt.
        #[arg(long)]
        yes: bool,
    },

    /// Push the local asset_management.json sidecar to the server (restore-from-backup).
    ///
    /// AMS *configuration* — custom asset types, the built-in allowlist, and
    /// project-level asset refs — lives in asset_management.json next to
    /// project.json. Like test methods, `acctl sync` pulls it down but never
    /// pushes it, and `acctl push project` doesn't carry it either. Use this
    /// command to deliberately OVERWRITE the machine's AMS configuration with
    /// the local sidecar — e.g. seeding a fresh machine or restoring a
    /// known-good backup. The server backs the previous file up to
    /// asset_management.json.bak first.
    ///
    /// Distinct from `acctl push assets`, which publishes this machine's asset
    /// *instances* (datastore/assets/); this pushes the AMS *config*.
    AssetConfig {
        /// Skip the confirmation prompt.
        #[arg(long)]
        yes: bool,
    },

    /// Push the local datastore/autocore_gnv.ini to the server (restore-from-backup).
    ///
    /// `acctl sync` pulls GNV from the server but does not push it — the
    /// server's GNV is live state (NV writes from the control program land
    /// there). Use this command only when you want to deliberately overwrite
    /// the server's GNV with the local copy, e.g. restoring a known-good
    /// snapshot. The command triggers `gm.reinitialize` after upload so the
    /// new values take effect in the running GM without a full server
    /// restart; pass --no-reinit to skip that step (use only if you plan to
    /// restart the server yourself).
    Gnv {
        /// Skip the post-upload `gm.reinitialize` call. Without a reinit
        /// or full restart, the next GM write will clobber the restored
        /// file with stale in-memory state. Only set this if you're about
        /// to restart the server.
        #[arg(long)]
        no_reinit: bool,
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
        // Default tungstenite limits (16 MiB frame / 64 MiB message) are too
        // small for project/datastore zips coming back from the server, which
        // sends large responses as a single frame.
        let mut ws_config = tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default();
        ws_config.max_message_size = Some(1 << 30); // 1 GiB
        ws_config.max_frame_size = Some(1 << 30);
        let (ws_stream, _) = tokio_tungstenite::connect_async_with_config(&url, Some(ws_config), false)
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
        self.send_command_timeout(topic, data, Duration::from_secs(30)).await
    }

    /// Like `send_command` but with a caller-chosen response timeout —
    /// large datastore transfers over slow links can exceed the 30s default.
    async fn send_command_timeout(
        &mut self,
        topic: &str,
        data: serde_json::Value,
        timeout: Duration,
    ) -> Result<CommandResponse> {
        // Use mechutil's CommandMessage::request constructor
        let msg = CommandMessage::request(topic, data);
        let transaction_id = msg.transaction_id;

        let json = serde_json::to_string(&msg)?;
        self.write.send(Message::Text(json)).await?;

        // Wait for response
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

/// Deploy the current local project to the target server in one shot.
///
/// Creates the project on the server if absent, stages project.json,
/// datastore/GNV, and the web HMI into it (even when it is NOT the active
/// project), then activates it and restarts the server ONCE — so by the time
/// the server comes back its www is in place and the HMI serves immediately,
/// avoiding the "pushed www after the restart → 404" ordering trap. The
/// control program is then built and started live against the now-active
/// project. Pushes built artifacts only; it never ships source.
async fn cmd_deploy(
    config: &Config,
    project_name: Option<String>,
    no_control: bool,
    no_www: bool,
    no_build: bool,
    no_restart: bool,
) -> Result<()> {
    let project_root = find_project_root()?;
    let project_json_path = project_root.join("project.json");
    let content = fs::read_to_string(&project_json_path)
        .map_err(|e| anyhow!("reading {}: {}", project_json_path.display(), e))?;
    let project_json: serde_json::Value = serde_json::from_str(&content)?;

    let name = match project_name {
        Some(n) => n,
        None => project_json
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!(
                "project.json has no \"name\" — pass it explicitly: acctl deploy <name>"
            ))?
            .to_string(),
    };
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(anyhow!(
            "invalid project name '{}': use only letters, digits, '_' or '-'",
            name
        ));
    }

    println!(
        "{} '{}' to {}:{}",
        "Deploying".bold(),
        name,
        config.get_host(),
        config.get_port()
    );

    // 1. Create the project on the server if it isn't there yet.
    {
        let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
        let list = client
            .send_command("system.list_projects", serde_json::json!({}))
            .await?;
        let exists = list.data["projects"]
            .as_array()
            .map(|ps| ps.iter().any(|p| p["name"].as_str() == Some(name.as_str())))
            .unwrap_or(false);
        if !exists {
            println!("  Creating project '{}' on the server...", name);
            let resp = client
                .send_command(
                    "system.new_project",
                    serde_json::json!({ "project_name": name }),
                )
                .await?;
            if !resp.success {
                client.close().await?;
                return Err(anyhow!("new_project: {}", resp.error_message));
            }
            println!("  {} created", "✓".green());
        } else {
            println!("  Project exists — updating in place.");
        }
        client.close().await?;
    }

    // 2. Stage project.json, GNV, and the HMI into the (possibly non-active)
    //    project. These uploads never touch the running server.
    println!("→ project.json");
    cmd_push_project(config, false, Some(&name)).await?;

    // test_methods.json is machine-owned (HMI-authored) and never rides
    // along automatically — remind the operator how to seed a fresh box.
    if test_methods::sidecar_path(&project_json_path).is_file() {
        println!("  (test_methods.json not auto-pushed — seed/restore with `acctl push test-methods`)");
    }
    // asset_management.json (AMS config) is likewise never auto-pushed.
    if asset_management::sidecar_path(&project_json_path).is_file() {
        println!("  (asset_management.json not auto-pushed — seed/restore with `acctl push asset-config`)");
    }

    let gnv_path = project_root.join("datastore").join("autocore_gnv.ini");
    if gnv_path.is_file() {
        println!("→ datastore/autocore_gnv.ini");
        cmd_push_gnv(config, true, Some(&name)).await?;
    } else {
        println!("  (no archived GNV — skipping)");
    }

    // Stage the committed test-method files so a fresh target can run before
    // its first `acctl sync`. No-op if there's no datastore/methods/ directory.
    if project_root.join("datastore").join("methods").is_dir() {
        println!("→ datastore/methods");
        cmd_push_methods(config, Some(&name)).await?;
    }

    if no_www {
        println!("  (--no-www — skipping HMI)");
    } else if project_root.join("www").exists() {
        println!("→ www");
        cmd_push_www(config, false, no_build, Some(&name)).await?;
    } else {
        println!("  (no www/ directory — skipping HMI)");
    }

    // 3. Activate + restart ONCE. After this the server runs `name` with its
    //    www already on disk, so :8080 serves the HMI right away.
    if no_restart {
        println!("{} staged (not activated; --no-restart given)", "✓".green());
        return Ok(());
    }
    println!("→ activating '{}' and restarting...", name);
    {
        let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
        let resp = client
            .send_command(
                "system.activate",
                serde_json::json!({ "project_name": name }),
            )
            .await?;
        if !resp.success {
            client.close().await?;
            return Err(anyhow!("activate: {}", resp.error_message));
        }
        let _ = client
            .send_command("system.restart", serde_json::json!({}))
            .await;
        client.close().await?;
    }
    wait_for_server(config).await?;

    // 4. Build + start the control program live against the now-active project.
    if no_control {
        println!("  (--no-control — skipping control program)");
    } else if project_root.join("control").exists() {
        println!("→ control program");
        cmd_push_control(config, false, no_build, true, false).await?;
    } else {
        println!("  (no control/ directory — skipping control program)");
    }

    println!(
        "{} deploy complete — open http://{}:8080",
        "✓".green(),
        config.get_host()
    );
    Ok(())
}

/// Poll the target server until it responds again after a restart (up to ~30s).
/// Sleeps first so we don't see the *old* process still up in its 0.5s
/// exit-delay window and mistake it for "already back".
async fn wait_for_server(config: &Config) -> Result<()> {
    use std::io::Write as _;
    tokio::time::sleep(Duration::from_secs(2)).await;
    print!("  waiting for server");
    let _ = std::io::stdout().flush();
    for _ in 0..60 {
        if let Ok(mut client) = WsClient::connect(&config.get_host(), config.get_port()).await {
            let r = client
                .send_command("system.list_projects", serde_json::json!({}))
                .await;
            let _ = client.close().await;
            if matches!(r, Ok(resp) if resp.success) {
                println!(" up");
                return Ok(());
            }
        }
        print!(".");
        let _ = std::io::stdout().flush();
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Err(anyhow!("server did not come back within 30s of restart"))
}

async fn cmd_push_project(config: &Config, restart: bool, target: Option<&str>) -> Result<()> {
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

    // test_methods.json is deliberately NOT pushed here: methods are
    // authored on the machine via the HMI, so the machine is the source
    // of truth for them. `acctl sync` pulls the sidecar down; the
    // explicit overwrite path is `acctl push test-methods`.
    //
    // asset_management.json (AMS config) is likewise not carried here — the
    // AMS config keys never appear in project.json after the sidecar split.
    // Seed/restore it deliberately with `acctl push asset-config`.
    println!("Pushing project.json to server...");

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    let mut payload = serde_json::json!({
        "project_json": project_json,
        "restart": restart
    });
    if let Some(t) = target {
        payload["project_name"] = serde_json::json!(t);
    }

    let response = client
        .send_command("system.upload_project", payload)
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

/// Increment the last dotted component of a version string — the build/patch
/// number. Returns `None` if that component isn't a plain integer, so a
/// hand-set pre-release version like `1.2.0-rc1` is left untouched rather
/// than guessed at.
fn increment_build_number(current: &str) -> Option<String> {
    let mut parts: Vec<String> = current.split('.').map(|s| s.to_string()).collect();
    let n = parts.last()?.parse::<u64>().ok()?;
    *parts.last_mut().unwrap() = (n + 1).to_string();
    Some(parts.join("."))
}

/// Bump the patch (build) number in `www/package.json` and return the new
/// version string. This runs immediately before `npm run build` so the new
/// version is baked into the bundle (via Vite `define` → `__APP_VERSION__`)
/// and displayed by the HMI. Keeping the bump in acctl means the version can
/// never drift from what is actually deployed.
///
/// The edit is surgical — only the `version` string value is rewritten — so
/// package.json key order and formatting are preserved.
fn bump_www_build_number(www_root: &Path) -> Result<String> {
    let pkg_path = www_root.join("package.json");
    let content = fs::read_to_string(&pkg_path)
        .with_context(|| format!("reading {}", pkg_path.display()))?;

    let parsed: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("parsing {}", pkg_path.display()))?;
    let current = parsed
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("{} has no string \"version\" field", pkg_path.display()))?;

    let new_version = match increment_build_number(current) {
        Some(v) => v,
        None => {
            println!(
                "  Note: www version \"{}\" has a non-numeric build component; leaving it unchanged.",
                current
            );
            return Ok(current.to_string());
        }
    };

    // Surgical replace of just the version value, tolerating the optional
    // space after the colon, to preserve the rest of package.json verbatim.
    let with_space = format!("\"version\": \"{}\"", current);
    let no_space = format!("\"version\":\"{}\"", current);
    let updated = if content.contains(&with_space) {
        content.replacen(&with_space, &format!("\"version\": \"{}\"", new_version), 1)
    } else if content.contains(&no_space) {
        content.replacen(&no_space, &format!("\"version\":\"{}\"", new_version), 1)
    } else {
        return Err(anyhow!(
            "could not locate the version string in {} to update it surgically",
            pkg_path.display()
        ));
    };

    fs::write(&pkg_path, updated)
        .with_context(|| format!("writing {}", pkg_path.display()))?;
    Ok(new_version)
}

/// Bump the patch (build) number in `control/Cargo.toml`'s `[package]` version
/// and return the new version string. Runs immediately before `cargo build` so
/// the compiled control binary's `CARGO_PKG_VERSION` reflects the deployed
/// build, mirroring the www bump.
///
/// The rewrite is scoped to the `[package]` table so a dependency's own
/// `version = "..."` can never be touched, and is otherwise byte-for-byte
/// preserving.
fn bump_control_build_number(control_dir: &Path) -> Result<String> {
    let cargo_path = control_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path)
        .with_context(|| format!("reading {}", cargo_path.display()))?;

    let parsed: toml::Value = toml::from_str(&content)
        .with_context(|| format!("parsing {}", cargo_path.display()))?;
    let current = parsed
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("{} has no [package] version", cargo_path.display()))?;

    let new_version = match increment_build_number(current) {
        Some(v) => v,
        None => {
            println!(
                "  Note: control version \"{}\" has a non-numeric build component; leaving it unchanged.",
                current
            );
            return Ok(current.to_string());
        }
    };

    // Restrict the replacement to the [package] table: from the `[package]`
    // header to the next table header (or EOF). This guarantees we never
    // rewrite a dependency's `version = "..."`.
    let header = "[package]";
    let pkg_idx = content
        .find(header)
        .ok_or_else(|| anyhow!("{} has no [package] section", cargo_path.display()))?;
    let body_start = pkg_idx + header.len();
    let section_len = content[body_start..]
        .find("\n[")
        .map(|i| i + 1)
        .unwrap_or(content.len() - body_start);
    let section = &content[body_start..body_start + section_len];

    let variants = [
        format!("version = \"{}\"", current),
        format!("version=\"{}\"", current),
    ];
    let (needle, rel_pos) = variants
        .iter()
        .find_map(|n| section.find(n.as_str()).map(|pos| (n.clone(), pos)))
        .ok_or_else(|| {
            anyhow!(
                "could not locate the [package] version in {} to update it surgically",
                cargo_path.display()
            )
        })?;

    let abs_start = body_start + rel_pos;
    let replacement = needle.replacen(current, &new_version, 1);
    let mut updated = String::with_capacity(content.len() + 4);
    updated.push_str(&content[..abs_start]);
    updated.push_str(&replacement);
    updated.push_str(&content[abs_start + needle.len()..]);

    fs::write(&cargo_path, updated)
        .with_context(|| format!("writing {}", cargo_path.display()))?;
    Ok(new_version)
}

async fn cmd_push_www(config: &Config, source: bool, no_build: bool, target: Option<&str>) -> Result<()> {
    let www_root = PathBuf::from("www");

    // Build before pushing dist (skip if --source or --no-build)
    if !source && !no_build && www_root.exists() {
        // Bump the build number BEFORE building so the new version is baked
        // into the bundle. (If the build subsequently fails, the number has
        // already advanced — a harmless gap, not a correctness problem.)
        match bump_www_build_number(&www_root) {
            Ok(v) => println!("Bumped www version to {}", v),
            Err(e) => println!("  Warning: could not bump www version ({}); building as-is.", e),
        }
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
    let total_size = zip_data.len();
    let total_chunks = (total_size + UPLOAD_CHUNK_SIZE - 1) / UPLOAD_CHUNK_SIZE;

    println!("Pushing www files ({} bytes, {} chunks)...", total_size, total_chunks);

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    // Try chunked upload first; fall back to single-message on older
    // servers that only know `system.upload_www`. Single-message
    // hangs on payloads of a few MB over local TCP/WS for reasons we
    // never fully isolated — chunking sidesteps it.
    let mut init_payload = serde_json::json!({
        "total_size": total_size,
        "chunk_size": UPLOAD_CHUNK_SIZE,
        "total_chunks": total_chunks,
        "source": source
    });
    if let Some(t) = target {
        init_payload["project_name"] = serde_json::json!(t);
    }
    let init_response = client
        .send_command("system.upload_www_init", init_payload)
        .await?;

    let upload_response_data;

    if !init_response.success && (init_response.error_message.contains("Unknown")
        || init_response.error_message.contains("upload_www_init"))
    {
        println!("  Server does not support chunked www upload, falling back to single message...");
        let zip_b64 = base64::engine::general_purpose::STANDARD.encode(&zip_data);
        let mut single_payload = serde_json::json!({
            "data": zip_b64,
            "source": source
        });
        if let Some(t) = target {
            single_payload["project_name"] = serde_json::json!(t);
        }
        let response = client
            .send_command("system.upload_www", single_payload)
            .await?;

        if !response.success {
            client.close().await?;
            return Err(anyhow!("Error: {}", response.error_message));
        }
        upload_response_data = response.data;
    } else if !init_response.success {
        client.close().await?;
        return Err(anyhow!("Error: {}", init_response.error_message));
    } else {
        let upload_id = init_response.data["upload_id"]
            .as_u64()
            .ok_or_else(|| anyhow!("Server did not return upload_id"))?;

        for (i, chunk) in zip_data.chunks(UPLOAD_CHUNK_SIZE).enumerate() {
            let chunk_b64 = base64::engine::general_purpose::STANDARD.encode(chunk);
            println!("  Chunk {}/{}", i + 1, total_chunks);

            let chunk_response = client
                .send_command(
                    "system.upload_www_chunk",
                    serde_json::json!({
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
                "system.upload_www_complete",
                serde_json::json!({
                    "upload_id": upload_id
                }),
            )
            .await?;

        if !complete_response.success {
            client.close().await?;
            return Err(anyhow!("Error: {}", complete_response.error_message));
        }
        upload_response_data = complete_response.data;
    }

    client.close().await?;

    let path = upload_response_data["path"].as_str().unwrap_or("unknown");
    let files = upload_response_data["files_extracted"].as_u64().unwrap_or(0);
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
        // Bump the build number BEFORE building so the compiled binary's
        // CARGO_PKG_VERSION reflects the deployed build. (A subsequent build
        // failure just leaves a harmless gap in the sequence.)
        match bump_control_build_number(&control_dir) {
            Ok(v) => println!("Bumped control version to {}", v),
            Err(e) => println!("  Warning: could not bump control version ({}); building as-is.", e),
        }
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

    // --- Compatibility gate -------------------------------------------------
    // Compare the freshly built control program against the RUNNING server so a
    // mismatch surfaces HERE (with a fix) instead of as a refused startup. The
    // control program's runtime header check is the hard backstop; this is the
    // friendly heads-up.
    if let Ok(resp) = client.send_command("gm.compat", serde_json::json!({})).await {
        if resp.success {
            let server = &resp.data;
            // Ask the binary what it was built against. Time-box + kill-on-drop:
            // an OLD control binary won't know --print-compat and would try to
            // run for real, so never let it hang the push.
            let probe = tokio::time::timeout(
                Duration::from_secs(5),
                tokio::process::Command::new(&binary_path)
                    .arg("--print-compat")
                    .kill_on_drop(true)
                    .output(),
            )
            .await;

            match probe {
                Ok(Ok(out)) if out.status.success() => {
                    let ctrl: serde_json::Value =
                        serde_json::from_slice(&out.stdout).unwrap_or(serde_json::Value::Null);

                    let s_abi = server.get("shm_abi_version").and_then(|v| v.as_u64());
                    let c_abi = ctrl.get("shm_abi_version").and_then(|v| v.as_u64());
                    let s_fmt = server.get("format_version").and_then(|v| v.as_u64());
                    let c_fmt = ctrl.get("format_version").and_then(|v| v.as_u64());
                    let s_hash = server.get("layout_hash").and_then(|v| v.as_str());
                    let c_hash = ctrl.get("layout_hash").and_then(|v| v.as_str());
                    let s_mech = server.get("mechutil_version").and_then(|v| v.as_str()).unwrap_or("?");
                    let c_mech = ctrl.get("mechutil_version").and_then(|v| v.as_str()).unwrap_or("?");

                    if (c_abi.is_some() && s_abi != c_abi) || (c_fmt.is_some() && s_fmt != c_fmt) {
                        // ABI/format skew would read mismapped memory — hard stop.
                        eprintln!(
                            "✗ Incompatible mechutil ABI: control built against {} (ABI {:?}/fmt {:?}), \
                             server is {} (ABI {:?}/fmt {:?}).",
                            c_mech, c_abi, c_fmt, s_mech, s_abi, s_fmt
                        );
                        eprintln!("  Pin the control program to the server's mechutil and rebuild:");
                        eprintln!("      # control/Cargo.toml");
                        eprintln!("      mechutil = \"={}\"", s_mech);
                        if !force {
                            return Err(anyhow!("Aborting push (use --force to override)."));
                        }
                        eprintln!("  --force given: pushing anyway.");
                    } else if c_hash.is_some() && s_hash != c_hash {
                        // Layout drift: control is fine; the running server serves an
                        // older layout. Push proceeds, but the server must restart.
                        println!(
                            "ℹ Layout changed vs the running server (server {} / control {}).",
                            s_hash.unwrap_or("?"),
                            c_hash.unwrap_or("?")
                        );
                        println!("  → autocore-server must be RESTARTED after this push to apply it");
                        println!("    (the control program will refuse to start otherwise).");
                    } else {
                        println!("✓ Control program matches the running server.");
                    }

                    if c_mech != s_mech && c_mech != "?" {
                        println!("  (note: control mechutil {} vs server {})", c_mech, s_mech);
                    }
                }
                _ => eprintln!(
                    "⚠ Could not read the control program's --print-compat; skipping compat gate \
                     (rebuild against current autocore-std to enable it)."
                ),
            }
        } else {
            eprintln!("⚠ Server did not answer gm.compat (older server?); skipping compat gate.");
        }
    }
    // -----------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// autocore-std codegen-compatibility floor
//
// The server's codegen emits autocore-std constructs that require a minimum
// std version. Unlike the mechutil SHM ABI (a *runtime* gate between two
// running processes, checked at push time), this is a *build-time* mismatch in
// a single artifact: too-old std → gm.rs won't compile, with a cryptic rustc
// error. We catch it at codegen time with an actionable message.
//
// These floors MUST track what src/codegen.rs actually emits:
//   - `impl autocore_std::GmCompat`  → always           → 3.3.52
//   - `autocore_std::motion::SimDrive` (virtual axes)    → 3.3.55 (when used)
const STD_MIN_GMCOMPAT: (u32, u32, u32) = (3, 3, 52);
const STD_MIN_SIMDRIVE: (u32, u32, u32) = (3, 3, 55);

/// Parse a semver `"a.b.c"` (ignoring any pre-release suffix on the patch).
fn parse_semver(s: &str) -> Option<(u32, u32, u32)> {
    let mut parts = s.split('.');
    let major = parts.next()?.trim().parse().ok()?;
    let minor = parts.next()?.trim().parse().ok()?;
    let patch_raw = parts.next()?;
    let patch_digits: String = patch_raw.chars().take_while(|c| c.is_ascii_digit()).collect();
    Some((major, minor, patch_digits.parse().ok()?))
}

/// The minimum autocore-std the server's codegen output will require for THIS
/// project, derived from the same rules codegen uses.
fn required_min_autocore_std(project: &serde_json::Value) -> (u32, u32, u32) {
    let mut min = STD_MIN_GMCOMPAT; // GmCompat is emitted unconditionally
    for home in ["ethercat", "motion"] {
        let Some(axes) = project
            .get("modules")
            .and_then(|m| m.get(home))
            .and_then(|m| m.get("config"))
            .and_then(|c| c.get("axes"))
            .and_then(|a| a.as_array())
        else {
            continue;
        };
        for axis in axes {
            let kind = axis.get("backend").and_then(|b| b.get("kind")).and_then(|k| k.as_str());
            if kind == Some("virtual") {
                min = min.max(STD_MIN_SIMDRIVE);
            }
        }
    }
    min
}

/// Read the autocore-std version the control program is pinned to from
/// `control/Cargo.lock`. `None` if absent/unparseable (then we skip the gate).
fn read_control_autocore_std_version() -> Option<(u32, u32, u32)> {
    let text = std::fs::read_to_string("control/Cargo.lock").ok()?;
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if line.trim() == "name = \"autocore-std\"" {
            // version line sits within this `[[package]]` block.
            for l in lines.by_ref().take(4) {
                if let Some(rest) = l.trim().strip_prefix("version = \"") {
                    return parse_semver(rest.trim_end_matches('"'));
                }
            }
        }
    }
    None
}

async fn cmd_codegen(config: &Config, force: bool) -> Result<()> {
    // Build-time autocore-std floor (see helpers above). Local check (reads
    // control/Cargo.lock + project.json), so it runs first — fail fast, before
    // touching the server. Catches a too-old std pin with an actionable message
    // instead of a cryptic rustc error after gm.rs regenerates. Skipped
    // silently if either version is unreadable.
    if let Some(ctrl_std) = read_control_autocore_std_version() {
        if let Ok((_, project_json)) = load_project_json_relaxed() {
            let req = required_min_autocore_std(&project_json);
            if ctrl_std < req {
                eprintln!(
                    "✗ autocore-std too old for this server's codegen output.\n  \
                     control/Cargo.lock pins {}.{}.{}, but the generated gm.rs needs ≥ {}.{}.{}.",
                    ctrl_std.0, ctrl_std.1, ctrl_std.2, req.0, req.1, req.2,
                );
                eprintln!("  Bump it, then re-run codegen:");
                eprintln!(
                    "      (cd control && cargo update -p autocore-std --precise {}.{}.{})",
                    req.0, req.1, req.2,
                );
                if !force {
                    return Err(anyhow!("Aborting codegen (use --force to override)."));
                }
                eprintln!("  --force given: continuing anyway.");
            }
        }
    }

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

/// `acctl config show` / `acctl config list`.
async fn cmd_config_show(config: &Config, list_only: bool) -> Result<()> {
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let response = client
        .send_command("system.active_configuration", serde_json::json!({}))
        .await?;
    client.close().await?;

    if !response.success {
        return Err(anyhow!("Error: {}", response.error_message));
    }

    let data = &response.data;
    let available = data["available"].as_array().cloned().unwrap_or_default();
    let defaults = data["module_defaults"].as_object().cloned().unwrap_or_default();

    if available.is_empty() {
        println!("No configurations are defined in the active project.");
        return Ok(());
    }

    println!("{}", "Configurations:".bold());
    for c in &available {
        println!("  {}", c.as_str().unwrap_or("?"));
    }

    if !defaults.is_empty() {
        println!("\n{}", "Module defaults:".bold());
        for (m, d) in &defaults {
            println!("  {} → {}", m, d.as_str().unwrap_or("?"));
        }
    }

    if !list_only {
        let source = data["source"].as_str().unwrap_or("none");
        match data["override"].as_str() {
            Some(name) => println!(
                "\n{} {} (source: {})",
                "Active:".bold(),
                name.green(),
                source
            ),
            None => println!(
                "\n{} {} (source: {})",
                "Active:".bold(),
                "per-module default".yellow(),
                source
            ),
        }
    }
    Ok(())
}

/// `acctl config set <name>` / `acctl config clear` (name = None).
async fn cmd_config_set(config: &Config, name: Option<&str>, restart: bool) -> Result<()> {
    match name {
        Some(n) => println!("Setting active configuration: {}", n),
        None => println!("Clearing active configuration override"),
    }

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let response = client
        .send_command(
            "system.set_active_configuration",
            serde_json::json!({ "name": name }),
        )
        .await?;

    if !response.success {
        client.close().await?;
        return Err(anyhow!("Error: {}", response.error_message));
    }

    if let Some(m) = response.data["message"].as_str() {
        println!("  {}", m);
    }

    if restart {
        println!("Restarting server...");
        let _ = client
            .send_command("system.restart", serde_json::json!({}))
            .await;
        println!("  Restart initiated");
    } else {
        println!("  {}", "Restart required to apply (acctl control restart).".yellow());
    }

    client.close().await?;
    Ok(())
}

/// `acctl config validate` — asks the ethercat module to statically validate
/// every configuration (bus-free).
async fn cmd_config_validate(config: &Config) -> Result<()> {
    println!("Validating configurations...");
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let response = client
        .send_command("ethercat.validate_configurations", serde_json::json!({}))
        .await?;
    client.close().await?;

    if !response.success {
        return Err(anyhow!("Validation failed: {}", response.error_message));
    }

    let issues = response.data["issues"].as_array().cloned().unwrap_or_default();
    if issues.is_empty() {
        println!("  {} all configurations valid", "✓".green());
        return Ok(());
    }
    println!("  {} {} issue(s):", "✗".red(), issues.len());
    for issue in &issues {
        let cfg = issue["configuration"].as_str().unwrap_or("?");
        let detail = issue["detail"].as_str().unwrap_or("?");
        println!("    [{}] {}", cfg.bold(), detail);
    }
    Err(anyhow!("{} configuration issue(s) found", issues.len()))
}

async fn cmd_tools_list(config: &Config) -> Result<()> {
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let response = client
        .send_command("system.list_tools", serde_json::json!({}))
        .await?;
    if !response.success {
        return Err(anyhow!("list_tools failed: {}", response.error_message));
    }
    let tools = response.data["tools"].as_array().cloned().unwrap_or_default();
    if tools.is_empty() {
        println!("No tools registered.");
        return Ok(());
    }
    println!("{}", "Registered tools:".bold());
    for t in &tools {
        let name = t["name"].as_str().unwrap_or("?");
        let running = t["running"].as_bool().unwrap_or(false);
        let state = if running { "running".green() } else { "stopped".yellow() };
        print!("  {} [{}]", name.bold(), state);
        if let Some(url) = t["url"].as_str() {
            print!("  {}", url);
        }
        println!();
        if let Some(editors) = t["editors"].as_array() {
            for e in editors {
                let domain = e["target_domain"].as_str().unwrap_or("?");
                let label = e["label"].as_str().unwrap_or("editor");
                println!("      edits {} — {}", domain.cyan(), label);
            }
        }
    }
    Ok(())
}

async fn cmd_tools_rescan(config: &Config) -> Result<()> {
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let response = client
        .send_command("system.rescan_tools", serde_json::json!({}))
        .await?;
    if !response.success {
        return Err(anyhow!("rescan_tools failed: {}", response.error_message));
    }
    let started = response.data["started"].as_array().cloned().unwrap_or_default();
    let stopped = response.data["stopped"].as_array().cloned().unwrap_or_default();
    let names = |v: &[serde_json::Value]| {
        v.iter().filter_map(|x| x.as_str().map(String::from)).collect::<Vec<_>>().join(", ")
    };
    println!("{}", "Rescanned tool registry.".bold());
    println!("  started: {}", if started.is_empty() { "(none)".into() } else { names(&started).green().to_string() });
    println!("  stopped: {}", if stopped.is_empty() { "(none)".into() } else { names(&stopped).yellow().to_string() });
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

// ============================================================================
// Update — apt-backed package updates with RT safety gates
// (UPDATE_SYSTEM_PLAN.md §4). Runs on the LOCAL machine; drives apt + the
// aptly-published private repo. Refuses mid-test, restarts + health-gates the
// server, and auto-rolls-back on persistent degradation.
// ============================================================================

struct UpdateArgs {
    list: bool,
    check: bool,
    rollback: bool,
    version: Option<String>,
    channel: Option<String>,
    yes: bool,
}

const APT_SOURCES: &str = "/etc/apt/sources.list.d/autocore.sources";
const UPDATE_HISTORY_DIR: &str = "/var/lib/autocore/update";
const HEALTH_TIMEOUT: Duration = Duration::from_secs(40);

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "::1")
}

/// Require EUID 0 (apt needs root). Shells `id -u` to avoid a libc dependency.
fn require_root() -> Result<()> {
    let out = std::process::Command::new("id")
        .arg("-u")
        .output()
        .context("failed to run `id -u`")?;
    if String::from_utf8_lossy(&out.stdout).trim() != "0" {
        return Err(anyhow!(
            "`acctl update` must run as root (it drives apt). Re-run with sudo."
        ));
    }
    Ok(())
}

/// Run apt-get with inherited stdio; error on non-zero exit.
fn run_apt(args: &[&str]) -> Result<()> {
    let status = std::process::Command::new("apt-get")
        .args(args)
        .env("DEBIAN_FRONTEND", "noninteractive")
        .status()
        .context("failed to spawn apt-get (is it installed?)")?;
    if !status.success() {
        return Err(anyhow!("apt-get {:?} failed ({})", args, status));
    }
    Ok(())
}

/// `apt-get update` that tolerates a single BROKEN OFFLINE source.
///
/// A machine updated offline leaves behind a local `file://` source
/// (OFFLINE_REPO). If that repo dir is later removed while the source file
/// lingers, a plain `apt-get update` returns non-zero and would abort an online
/// update that is otherwise fine. So: if the ONLY sources that failed are the
/// offline repo, warn and continue; if anything else failed, propagate the
/// error. (`acctl snapshot detach` removes that source cleanly — this is the
/// belt-and-suspenders for machines where it wasn't run.)
fn apt_update_tolerant() -> Result<()> {
    let out = std::process::Command::new("apt-get")
        .arg("update")
        .env("DEBIAN_FRONTEND", "noninteractive")
        .output()
        .context("failed to spawn apt-get (is it installed?)")?;
    if out.status.success() {
        return Ok(());
    }
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    // Lines apt uses to report a failed source.
    let err_lines: Vec<&str> = combined
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with("E:") || t.starts_with("Err") || t.starts_with("W:") || t.contains("Failed to fetch")
        })
        .collect();
    let only_offline = !err_lines.is_empty()
        && err_lines.iter().all(|l| l.contains(OFFLINE_REPO));
    if only_offline {
        eprintln!(
            "{}",
            "Warning: the local offline snapshot source is stale/broken; ignoring it and \
             continuing with the online source. Run `sudo acctl snapshot detach` to remove it."
                .yellow()
        );
        return Ok(());
    }
    // Something real failed — surface apt's own output and fail.
    eprint!("{}", combined);
    Err(anyhow!("apt-get update failed"))
}

/// Installed packages whose name starts with `autocore` → (name, version).
fn installed_autocore_packages() -> Result<Vec<(String, String)>> {
    let out = std::process::Command::new("dpkg-query")
        .args([
            "-W",
            "-f",
            "${db:Status-Abbrev} ${Package} ${Version}\n",
            "autocore*",
        ])
        .output()
        .context("failed to run dpkg-query")?;
    // dpkg-query exits non-zero when the glob matches nothing; that's fine — we
    // just parse whatever it printed to stdout.
    let text = String::from_utf8_lossy(&out.stdout);
    let mut pkgs = Vec::new();
    for line in text.lines() {
        let mut it = line.split_whitespace();
        let status = it.next().unwrap_or("");
        let name = it.next().unwrap_or("");
        let version = it.next().unwrap_or("");
        // "ii" == installed; skip half-installed / config-only / not-installed.
        if status.starts_with("ii") && !name.is_empty() && !version.is_empty() {
            pkgs.push((name.to_string(), version.to_string()));
        }
    }
    Ok(pkgs)
}

/// RT gate: refuse to touch packages while a test is running. Reads the
/// `tis_active` GM scalar via the local server. A non-TIS project has no such
/// scalar (gm.read fails) → treated as "no test".
async fn gate_not_mid_test(config: &Config) -> Result<()> {
    let mut client = match WsClient::connect(&config.get_host(), config.get_port()).await {
        Ok(c) => c,
        Err(_) => {
            // Server down ⇒ definitionally no test running. Don't hard-block an
            // update just because the server is off.
            println!(
                "  {}",
                "(server not reachable — assuming no test in progress)".yellow()
            );
            return Ok(());
        }
    };
    let resp = client
        .send_command("gm.read", serde_json::json!({ "name": "tis_active" }))
        .await;
    let _ = client.close().await;
    if let Ok(r) = resp {
        if r.success {
            let active = r
                .data
                .as_bool()
                .or_else(|| r.data.get("value").and_then(|v| v.as_bool()))
                .or_else(|| r.data.get("tis_active").and_then(|v| v.as_bool()))
                .unwrap_or(false);
            if active {
                return Err(anyhow!(
                    "A test is currently running (tis_active=true). Refusing to update \
                     mid-test — wait for it to finish (or stop it) and retry."
                ));
            }
        }
        // !success ⇒ scalar absent (non-TIS project) ⇒ not mid-test.
    }
    Ok(())
}

fn record_version_set(pkgs: &[(String, String)], label: &str) -> Result<PathBuf> {
    std::fs::create_dir_all(UPDATE_HISTORY_DIR)
        .with_context(|| format!("failed to create {}", UPDATE_HISTORY_DIR))?;
    let ts = Local::now().format("%Y%m%dT%H%M%S").to_string();
    let path = PathBuf::from(UPDATE_HISTORY_DIR).join(format!("{}.json", ts));
    let map: serde_json::Map<String, serde_json::Value> = pkgs
        .iter()
        .map(|(n, v)| (n.clone(), serde_json::Value::String(v.clone())))
        .collect();
    let doc = serde_json::json!({ "recorded_at": ts, "label": label, "packages": map });
    std::fs::write(&path, serde_json::to_string_pretty(&doc)?)?;
    Ok(path)
}

fn list_history() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(UPDATE_HISTORY_DIR)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "json"))
        .collect();
    v.sort(); // timestamped names sort chronologically
    v
}

fn load_version_set(path: &Path) -> Result<Vec<(String, String)>> {
    let doc: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let pkgs = doc
        .get("packages")
        .and_then(|p| p.as_object())
        .ok_or_else(|| anyhow!("malformed history file {}", path.display()))?;
    Ok(pkgs
        .iter()
        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
        .collect())
}

/// Install an exact set of package versions (allowing downgrades — that's the
/// point of a rollback/pin).
fn apply_version_set(pkgs: &[(String, String)]) -> Result<()> {
    let mut args: Vec<String> = vec!["install".into(), "--allow-downgrades".into(), "-y".into()];
    for (n, v) in pkgs {
        args.push(format!("{}={}", n, v));
    }
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    run_apt(&refs)
}

fn restart_server() -> Result<()> {
    if !Path::new("/run/systemd/system").exists() {
        println!(
            "  {}",
            "(no systemd — restart the server yourself to apply the change)".yellow()
        );
        return Ok(());
    }
    println!("Restarting autocore_server...");
    let status = std::process::Command::new("systemctl")
        .args(["restart", "autocore_server"])
        .status()
        .context("failed to run systemctl")?;
    if !status.success() {
        return Err(anyhow!("systemctl restart autocore_server failed"));
    }
    Ok(())
}

/// Poll `system.health` until healthy or timeout. Returns (healthy, reason).
/// Transient degradation while modules come up is tolerated — only a state that
/// persists to the timeout counts as unhealthy.
async fn wait_healthy(config: &Config, timeout: Duration) -> (bool, String) {
    let start = std::time::Instant::now();
    let mut last = String::from("server did not become reachable");
    while start.elapsed() < timeout {
        if let Ok(mut client) = WsClient::connect(&config.get_host(), config.get_port()).await {
            let resp = client
                .send_command("system.health", serde_json::json!({}))
                .await;
            let _ = client.close().await;
            if let Ok(r) = resp {
                if r.data.get("healthy").and_then(|v| v.as_bool()).unwrap_or(false) {
                    return (true, "healthy".into());
                }
                let degr = r
                    .data
                    .get("degradations")
                    .map(|d| d.to_string())
                    .unwrap_or_default();
                last = format!("degraded: {}", degr);
            } else {
                last = "connected but system.health failed".into();
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    (false, last)
}

fn switch_channel(channel: &str) -> Result<()> {
    if !matches!(channel, "stable" | "beta" | "dev") {
        return Err(anyhow!("channel must be one of: stable, beta, dev"));
    }
    let path = Path::new(APT_SOURCES);
    if !path.exists() {
        return Err(anyhow!(
            "{} not found — is the autocore-server package installed?",
            APT_SOURCES
        ));
    }
    let content = std::fs::read_to_string(path)?;
    let mut saw_suite = false;
    let mut new: String = content
        .lines()
        .map(|l| {
            let t = l.trim_start();
            if t.starts_with("Suites:") {
                saw_suite = true;
                format!("Suites: {}", channel)
            } else if t.starts_with("Enabled:") {
                // Switching channel implies opting in.
                "Enabled: yes".to_string()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if !saw_suite {
        new.push_str(&format!("\nSuites: {}", channel));
    }
    new.push('\n');
    std::fs::write(path, new)?;
    println!("Now tracking channel '{}'. Refreshing package lists...", channel.green());
    apt_update_tolerant()?;
    println!("Done. Run `acctl update` to move to this channel's current version set.");
    Ok(())
}

fn show_upgradable() -> Result<()> {
    let out = std::process::Command::new("apt")
        .args(["list", "--upgradable"])
        .output()
        .context("failed to run apt list")?;
    let text = String::from_utf8_lossy(&out.stdout);
    let autocore: Vec<&str> = text.lines().filter(|l| l.contains("autocore")).collect();
    if autocore.is_empty() {
        println!("{}", "All autocore packages are up to date.".green());
    } else {
        println!("{}", "Autocore packages with updates available:".bold());
        for l in autocore {
            println!("  {}", l);
        }
    }
    Ok(())
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{} [y/N] ", prompt);
    std::io::stdout().flush().ok();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

async fn do_rollback_or_pin(
    config: &Config,
    version: Option<&str>,
    yes: bool,
) -> Result<()> {
    let history = list_history();
    if history.is_empty() {
        return Err(anyhow!(
            "No update history recorded yet — nothing to roll back to. \
             (A rollback point is written on each `acctl update`.)"
        ));
    }
    let target = match version {
        Some(name) => history
            .iter()
            .find(|p| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .map_or(false, |s| s == name || s.contains(name))
            })
            .cloned()
            .ok_or_else(|| {
                let avail: Vec<String> = history
                    .iter()
                    .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(String::from))
                    .collect();
                anyhow!("No recorded version set '{}'. Available: {}", name, avail.join(", "))
            })?,
        // Most recent recorded pre-update set = "the previous known-good".
        None => history.last().cloned().unwrap(),
    };
    let set = load_version_set(&target)?;
    println!("{}", "Rolling back to version set:".bold());
    for (n, v) in &set {
        println!("  {} = {}", n, v);
    }
    if !yes && !confirm("Apply this version set?")? {
        println!("Aborted.");
        return Ok(());
    }
    apply_version_set(&set)?;
    restart_server()?;
    let (ok, reason) = wait_healthy(config, HEALTH_TIMEOUT).await;
    if ok {
        println!("{}", "Rollback complete — server healthy.".green());
        Ok(())
    } else {
        Err(anyhow!("Rollback applied but the server is {}.", reason))
    }
}

async fn cmd_update(config: &Config, args: UpdateArgs) -> Result<()> {
    // `update` drives apt on THIS machine; the safety gates query the LOCAL
    // server, so the target must be loopback (run it on the box over SSH).
    let host = config.get_host();
    if !is_loopback_host(&host) {
        return Err(anyhow!(
            "`acctl update` operates on the LOCAL machine's packages, but the target is '{}'. \
             SSH to the target and run it there with acctl pointed at localhost \
             (`acctl set-target 127.0.0.1`).",
            host
        ));
    }
    require_root()?;

    // Channel switch is a standalone action.
    if let Some(ch) = args.channel.as_deref() {
        return switch_channel(ch);
    }

    // Every path below wants fresh apt metadata.
    println!("{}", "Refreshing package lists...".bold());
    apt_update_tolerant()?;

    // Read-only queries: no mutation, no mid-test gate needed.
    if args.list {
        return show_upgradable();
    }
    if args.check {
        let pkgs = installed_autocore_packages()?;
        if pkgs.is_empty() {
            println!("No autocore packages installed.");
            return Ok(());
        }
        println!("{}", "Dry run (no changes will be made):".bold());
        let mut a = vec!["install", "--only-upgrade", "-s"];
        a.extend(pkgs.iter().map(|(n, _)| n.as_str()));
        return run_apt(&a);
    }

    // Mutating paths: refuse mid-test first.
    println!("{}", "Checking test state...".bold());
    gate_not_mid_test(config).await?;

    if args.rollback || args.version.is_some() {
        return do_rollback_or_pin(config, args.version.as_deref(), args.yes).await;
    }

    // ---- the update itself ----
    let before = installed_autocore_packages()?;
    if before.is_empty() {
        println!("No autocore packages installed — nothing to update.");
        return Ok(());
    }

    // Show the plan (simulate), then confirm.
    println!("{}", "Planned changes:".bold());
    {
        let mut a = vec!["install", "--only-upgrade", "-s"];
        a.extend(before.iter().map(|(n, _)| n.as_str()));
        run_apt(&a)?;
    }
    if !args.yes && !confirm("Proceed with update?")? {
        println!("Aborted.");
        return Ok(());
    }

    // Record the CURRENT set as the rollback target BEFORE upgrading.
    let hist = record_version_set(&before, "pre-update")?;
    println!("Recorded rollback point: {}", hist.display());

    // Apply.
    {
        let mut a = vec!["install", "--only-upgrade", "-y"];
        a.extend(before.iter().map(|(n, _)| n.as_str()));
        run_apt(&a)?;
    }

    // Restart + health gate; auto-rollback on persistent degradation. This is
    // the core lesson of ADC-SN-3833: an update that leaves the machine dark is
    // worse than no update. Shares `restart_and_wait_healthy` with `snapshot use`
    // so the two paths behave identically.
    println!("{}", "Waiting for the server to come back healthy...".bold());
    match restart_and_wait_healthy(config).await {
        Ok(()) => {
            println!("{}", "Update complete — server healthy.".green());
            Ok(())
        }
        Err(reason) => {
            eprintln!("{} {}", "Server did not come back healthy:".red(), reason);
            eprintln!("{}", "Auto-rolling back to the pre-update version set...".yellow());
            let set = load_version_set(&hist)?;
            apply_version_set(&set)?;
            match restart_and_wait_healthy(config).await {
                Ok(()) => Err(anyhow!(
                    "Update left the server degraded ({}); rolled back to the previous version set (now healthy).",
                    reason
                )),
                Err(reason2) => Err(anyhow!(
                    "Update degraded the server ({}) AND rollback did not restore health ({}). Manual intervention needed.",
                    reason, reason2
                )),
            }
        }
    }
}

// ============================================================================
// Modules — reconcile installed module packages with the active project, plus
// on-demand install/remove and the pre-split migration (UPDATE_SYSTEM_PLAN.md
// §3.4 / §4). These make the ADC-SN-3833 missing-module state structurally
// impossible: the project declares the set, apt guarantees the packages.
// ============================================================================

/// A module declared (and enabled) in the active project.json.
struct DeclaredModule {
    #[allow(dead_code)]
    domain: String,
    /// The deb package that provides it.
    package: String,
    /// Whether such a package actually exists in apt (installed or in the repo).
    /// False for in-process modules like `control` that have no deb.
    has_deb: bool,
}

/// Map a project module (domain + optional executable) to its deb package name.
/// Prefers the executable's basename when it looks like an autocore package
/// (stripping a `_dev` mock suffix); otherwise the `autocore-<domain>` convention.
fn module_package_name(domain: &str, executable: Option<&str>) -> String {
    if let Some(exe) = executable {
        let base = exe.rsplit(['/', '\\']).next().unwrap_or(exe);
        if base.starts_with("autocore-") {
            return base.trim_end_matches("_dev").to_string();
        }
    }
    format!("autocore-{}", domain)
}

/// Normalize a user-supplied module name to a package name.
fn to_module_package(name: &str) -> String {
    if name.starts_with("autocore-") {
        name.to_string()
    } else {
        format!("autocore-{}", name)
    }
}

/// Installed autocore packages that count as MODULES (excludes the server and the
/// ethercat-esi data package, which rides along with autocore-ethercat).
fn is_module_package(name: &str) -> bool {
    // `autocore_server` has an underscore, so it's already excluded here.
    name.starts_with("autocore-") && name != "autocore-ethercat-esi"
}

/// True if apt knows this package (installed or available in a configured repo).
fn apt_cache_has(pkg: &str) -> bool {
    std::process::Command::new("apt-cache")
        .args(["show", pkg])
        .output()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false)
}

/// The package that owns a file, per dpkg (first package if several), or None.
fn dpkg_owner(path: &str) -> Option<String> {
    let out = std::process::Command::new("dpkg")
        .args(["-S", path])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines()
        .next()
        .and_then(|l| l.split(':').next())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
}

/// Guard shared by all mutating module ops: must be local + root.
fn local_root_guard(config: &Config) -> Result<()> {
    let host = config.get_host();
    if !is_loopback_host(&host) {
        return Err(anyhow!(
            "This operates on the LOCAL machine's packages, but the target is '{}'. \
             SSH to the target and run it there with acctl pointed at localhost.",
            host
        ));
    }
    require_root()
}

/// The active project's declared+enabled modules, mapped to packages.
async fn declared_modules(config: &Config) -> Result<Vec<DeclaredModule>> {
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let r = client
        .send_command("system.get_project", serde_json::json!({}))
        .await?;
    let _ = client.close().await;
    if !r.success {
        return Err(anyhow!("could not read active project: {}", r.error_message));
    }
    let mut out = Vec::new();
    if let Some(map) = r.data.get("modules").and_then(|m| m.as_object()) {
        for (domain, cfg) in map {
            // Absent `enabled` ⇒ treat as enabled (it's in the modules map).
            let enabled = cfg.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
            if !enabled {
                continue;
            }
            let exe = cfg.get("executable").and_then(|v| v.as_str());
            let package = module_package_name(domain, exe);
            let has_deb = apt_cache_has(&package);
            out.push(DeclaredModule {
                domain: domain.clone(),
                package,
                has_deb,
            });
        }
    }
    Ok(out)
}

/// Installed module package names (sorted, deduped).
fn installed_module_packages() -> Result<Vec<String>> {
    let mut v: Vec<String> = installed_autocore_packages()?
        .into_iter()
        .map(|(n, _)| n)
        .filter(|n| is_module_package(n))
        .collect();
    v.sort();
    v.dedup();
    Ok(v)
}

async fn cmd_modules_list(config: &Config) -> Result<()> {
    let declared = declared_modules(config).await?;
    let installed = installed_module_packages()?;

    println!("{}", "Active project declares these modules:".bold());
    if declared.is_empty() {
        println!("  (none)");
    }
    for m in &declared {
        if !m.has_deb {
            println!("  {} [{}]", m.package, "no deb — in-process/bundled".dimmed());
        } else if installed.contains(&m.package) {
            println!("  {} [{}]", m.package, "installed".green());
        } else {
            println!("  {} [{}]", m.package, "MISSING".red());
        }
    }

    let declared_pkgs: Vec<&String> = declared
        .iter()
        .filter(|m| m.has_deb)
        .map(|m| &m.package)
        .collect();
    let extras: Vec<&String> = installed
        .iter()
        .filter(|p| !declared_pkgs.contains(p))
        .collect();
    if !extras.is_empty() {
        println!("{}", "Installed but NOT declared by the project (extras):".bold());
        for p in extras {
            println!("  {} [{}]", p, "extra".yellow());
        }
    }
    Ok(())
}

async fn cmd_modules_sync(config: &Config, remove_extras: bool, yes: bool) -> Result<()> {
    local_root_guard(config)?;
    println!("{}", "Refreshing package lists...".bold());
    apt_update_tolerant()?;
    gate_not_mid_test(config).await?;

    let declared = declared_modules(config).await?;
    let installed = installed_module_packages()?;

    let declared_pkgs: Vec<String> = declared
        .iter()
        .filter(|m| m.has_deb)
        .map(|m| m.package.clone())
        .collect();

    // In-process / bundled modules with no deb — reported, never acted on.
    for m in declared.iter().filter(|m| !m.has_deb) {
        println!(
            "  {} — no deb (in-process/bundled), skipping",
            m.package.dimmed()
        );
    }

    let missing: Vec<String> = declared_pkgs
        .iter()
        .filter(|p| !installed.contains(p))
        .cloned()
        .collect();
    let extras: Vec<String> = installed
        .iter()
        .filter(|p| !declared_pkgs.contains(p))
        .cloned()
        .collect();

    if missing.is_empty() {
        println!("{}", "All declared modules are installed.".green());
    } else {
        println!(
            "{} {}",
            "Missing modules to install:".bold(),
            missing.join(" ")
        );
    }
    if !extras.is_empty() {
        println!("{} {}", "Extra modules (not declared):".bold(), extras.join(" "));
    }

    let mut mutated = false;

    if !missing.is_empty() && (yes || confirm("Install the missing modules?")?) {
        let mut a = vec!["install", "-y"];
        a.extend(missing.iter().map(|s| s.as_str()));
        run_apt(&a)?;
        mutated = true;
    }

    if !extras.is_empty() {
        if remove_extras {
            if yes || confirm("Remove the extra modules?")? {
                let mut a = vec!["remove", "-y"];
                a.extend(extras.iter().map(|s| s.as_str()));
                run_apt(&a)?;
                mutated = true;
            }
        } else {
            println!(
                "  {}",
                "(extras left in place — re-run with --remove-extras to remove them)".yellow()
            );
        }
    }

    if mutated {
        restart_server()?;
        println!("{}", "Waiting for the server to come back healthy...".bold());
        let (ok, reason) = wait_healthy(config, HEALTH_TIMEOUT).await;
        if !ok {
            return Err(anyhow!("After sync the server is {}.", reason));
        }
    }

    // Post-op reconcile check: confirm nothing declared is still missing.
    let now_installed = installed_module_packages()?;
    let still_missing: Vec<&String> = declared_pkgs
        .iter()
        .filter(|p| !now_installed.contains(p))
        .collect();
    if still_missing.is_empty() {
        println!("{}", "Reconcile OK — all declared modules present.".green());
        Ok(())
    } else {
        Err(anyhow!(
            "Reconcile FAILED — declared modules still missing after sync: {}",
            still_missing
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        ))
    }
}

async fn cmd_modules_install(config: &Config, name: &str, yes: bool) -> Result<()> {
    local_root_guard(config)?;
    let pkg = to_module_package(name);
    println!("{}", "Refreshing package lists...".bold());
    apt_update_tolerant()?;
    if !apt_cache_has(&pkg) {
        return Err(anyhow!(
            "No package '{}' available. Check the channel/credential are set up \
             (`acctl update --list`).",
            pkg
        ));
    }
    gate_not_mid_test(config).await?;
    if !yes && !confirm(&format!("Install {}?", pkg))? {
        println!("Aborted.");
        return Ok(());
    }
    run_apt(&["install", "-y", &pkg])?;
    restart_server()?;
    let (ok, reason) = wait_healthy(config, HEALTH_TIMEOUT).await;
    if ok {
        println!("{}", format!("Installed {} — server healthy.", pkg).green());
        Ok(())
    } else {
        Err(anyhow!("After installing {}, server is {}.", pkg, reason))
    }
}

async fn cmd_modules_remove(config: &Config, name: &str, force: bool, yes: bool) -> Result<()> {
    local_root_guard(config)?;
    let pkg = to_module_package(name);
    gate_not_mid_test(config).await?;

    // Removing a module the project still declares will make the server degraded.
    let declared = declared_modules(config).await.unwrap_or_default();
    if declared.iter().any(|m| m.package == pkg) && !force {
        return Err(anyhow!(
            "The active project DECLARES '{}' — removing it will degrade the server. \
             Re-run with --force if you really mean to.",
            pkg
        ));
    }
    if !yes && !confirm(&format!("Remove {}?", pkg))? {
        println!("Aborted.");
        return Ok(());
    }
    run_apt(&["remove", "-y", &pkg])?;
    restart_server()?;
    let (ok, reason) = wait_healthy(config, HEALTH_TIMEOUT).await;
    if ok {
        println!("{}", format!("Removed {} — server healthy.", pkg).green());
    } else {
        // Expected when force-removing a still-declared module.
        println!("{} {}", "Note: server reports:".yellow(), reason);
    }
    Ok(())
}

async fn cmd_migrate(config: &Config, yes: bool) -> Result<()> {
    local_root_guard(config)?;

    const EC_BIN: &str = "/opt/autocore/bin/modules/autocore-ethercat";
    let owner = dpkg_owner(EC_BIN);
    let exists = Path::new(EC_BIN).exists();

    if exists && owner.as_deref() == Some("autocore-ethercat") {
        println!(
            "{}",
            "Already migrated — ethercat files are owned by autocore-ethercat.".green()
        );
        return Ok(());
    }

    if !exists {
        println!(
            "{}",
            format!(
                "ethercat binary MISSING ({}) — possibly the ADC-SN-3833 deletion. \
                 Installing the standalone packages will restore it.",
                EC_BIN
            )
            .yellow()
        );
    } else {
        println!(
            "Pre-split layout: {} is owned by '{}'.",
            EC_BIN,
            owner.as_deref().unwrap_or("no package (orphaned)")
        );
    }

    gate_not_mid_test(config).await?;
    println!(
        "This will install {} + {} (apt's Replaces/Breaks re-own the files).",
        "autocore-ethercat".bold(),
        "autocore-ethercat-esi".bold()
    );
    if !yes && !confirm("Proceed with migration?")? {
        println!("Aborted.");
        return Ok(());
    }

    apt_update_tolerant()?;
    run_apt(&["install", "-y", "autocore-ethercat", "autocore-ethercat-esi"])?;

    // Verify the file ownership actually moved.
    if dpkg_owner(EC_BIN).as_deref() == Some("autocore-ethercat") {
        println!(
            "{}",
            "Verified — ethercat is now owned by autocore-ethercat.".green()
        );
    } else {
        println!(
            "{}",
            "Warning: ethercat ownership is not autocore-ethercat after install — check dpkg -S."
                .yellow()
        );
    }

    restart_server()?;
    let (ok, reason) = wait_healthy(config, HEALTH_TIMEOUT).await;
    if ok {
        println!("{}", "Migration complete — server healthy.".green());
        Ok(())
    } else {
        Err(anyhow!("After migration, server is {}.", reason))
    }
}

// ============================================================================
// Offline version snapshots (UPDATE_SYSTEM_PLAN.md §11) — the rustup-like
// experience for air-gapped machines. A snapshot bundle built by
// packaging/apt/export-snapshot.sh is imported into a LOCAL apt repo; imported
// snapshots coexist (shared content-addressed pool) and `use` switches between
// them like toolchains, all without a network.
// ============================================================================

const OFFLINE_REPO: &str = "/var/lib/autocore/offline-repo";
const OFFLINE_SOURCES: &str = "/etc/apt/sources.list.d/autocore-offline.sources";
const KEYRING_PATH: &str = "/usr/share/keyrings/autocore-archive-keyring.gpg";

#[derive(Debug, Deserialize)]
struct SnapshotManifest {
    snapshot: String,
    #[serde(default)]
    since: Option<String>,
    #[serde(default)]
    created: String,
    #[serde(default)]
    packages: Vec<SnapPkg>,
    #[serde(default)]
    required_files: Vec<String>,
    #[serde(default)]
    shipped_files: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SnapPkg {
    name: String,
    version: String,
    #[allow(dead_code)]
    filename: String,
}

fn offline_paths() -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let root = PathBuf::from(OFFLINE_REPO);
    (
        root.join("pool"),
        root.join("dists"),
        root.join("manifests"),
        root.join("active"),
    )
}

fn read_active_snapshot() -> Option<String> {
    let (_, _, _, active) = offline_paths();
    std::fs::read_to_string(active)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn write_active_snapshot(name: &str) -> Result<()> {
    let (_, _, _, active) = offline_paths();
    std::fs::write(active, format!("{}\n", name))?;
    Ok(())
}

fn load_manifest(name: &str) -> Result<SnapshotManifest> {
    let (_, _, manifests, _) = offline_paths();
    let path = manifests.join(format!("{}.json", name));
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("snapshot '{}' is not imported ({})", name, path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("malformed manifest {}", path.display()))
}

fn all_manifests() -> Vec<SnapshotManifest> {
    let (_, _, manifests, _) = offline_paths();
    std::fs::read_dir(&manifests)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "json"))
        .filter_map(|p| std::fs::read_to_string(&p).ok())
        .filter_map(|t| serde_json::from_str::<SnapshotManifest>(&t).ok())
        .collect()
}

/// Write the offline apt source pointing at a given suite (imported snapshot).
fn write_offline_source(suite: &str) -> Result<()> {
    let body = format!(
        "# AutoCore OFFLINE snapshot source — managed by `acctl snapshot`.\n\
         # Points at the local repo under {}; the active snapshot is the suite below.\n\
         Types: deb\n\
         URIs: file://{}\n\
         Suites: {}\n\
         Components: main\n\
         Architectures: amd64\n\
         Signed-By: {}\n",
        OFFLINE_REPO, OFFLINE_REPO, suite, KEYRING_PATH
    );
    std::fs::write(OFFLINE_SOURCES, body)
        .with_context(|| format!("failed to write {}", OFFLINE_SOURCES))?;
    Ok(())
}

/// apt-get update scoped to ONLY the given sources file — never touches the
/// network (base Ubuntu mirrors are unreachable on an air-gapped box). Verifies
/// the local repo's signature against the keyring as a side effect.
fn apt_update_local(sources_file: &str) -> Result<()> {
    let parent = Path::new(sources_file)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/etc/apt/sources.list.d".into());
    let status = std::process::Command::new("apt-get")
        .args([
            "-o",
            "Dir::Etc::SourceList=/dev/null",
            "-o",
            &format!("Dir::Etc::SourceParts={}", parent),
            "update",
        ])
        .env("DEBIAN_FRONTEND", "noninteractive")
        .status()
        .context("failed to spawn apt-get update")?;
    if !status.success() {
        return Err(anyhow!(
            "apt-get update (local) failed — the snapshot signature may not verify against {}",
            KEYRING_PATH
        ));
    }
    Ok(())
}

/// Restart the server and wait for health. Returns Err(reason) if it doesn't
/// come back healthy — the caller owns rollback. Shared by `update` and
/// `snapshot use` so both behave identically.
async fn restart_and_wait_healthy(config: &Config) -> std::result::Result<(), String> {
    if let Err(e) = restart_server() {
        return Err(format!("restart failed: {}", e));
    }
    let (ok, reason) = wait_healthy(config, HEALTH_TIMEOUT).await;
    if ok {
        Ok(())
    } else {
        Err(reason)
    }
}

async fn cmd_snapshot_import(config: &Config, bundle: &str) -> Result<()> {
    local_root_guard(config)?;
    let bundle_path = Path::new(bundle);
    if !bundle_path.exists() {
        return Err(anyhow!("bundle not found: {}", bundle));
    }

    let (pool, dists, manifests, _) = offline_paths();
    for d in [&pool, &dists, &manifests] {
        std::fs::create_dir_all(d).with_context(|| format!("failed to create {}", d.display()))?;
    }

    // Extract into a temp dir on the same filesystem for a clean, atomic-ish merge.
    let tmp = PathBuf::from(OFFLINE_REPO).join(".import-tmp");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp)?;
    let status = std::process::Command::new("tar")
        .args(["-xzf", bundle, "-C"])
        .arg(&tmp)
        .status()
        .context("failed to spawn tar")?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err(anyhow!("failed to extract bundle {}", bundle));
    }

    let manifest: SnapshotManifest = {
        let mpath = tmp.join("manifest.json");
        let text = std::fs::read_to_string(&mpath)
            .context("bundle is missing manifest.json (not an autocore snapshot bundle?)")?;
        serde_json::from_str(&text).context("malformed manifest.json in bundle")?
    };
    let snap = manifest.snapshot.clone();
    println!(
        "Importing snapshot '{}'{} — {} packages, {} files in bundle.",
        snap,
        manifest
            .since
            .as_ref()
            .map(|s| format!(" (incremental since {})", s))
            .unwrap_or_default(),
        manifest.packages.len(),
        manifest.shipped_files.len()
    );

    // Merge the shared pool (no-clobber — content-addressed, identical bytes).
    let tmp_pool = tmp.join("pool");
    if tmp_pool.exists() {
        let status = std::process::Command::new("cp")
            .args(["-rn"])
            .arg(format!("{}/.", tmp_pool.display()))
            .arg(&pool)
            .status()
            .context("failed to merge pool")?;
        if !status.success() {
            let _ = std::fs::remove_dir_all(&tmp);
            return Err(anyhow!("failed to merge pool files"));
        }
    }

    // Install this snapshot's signed metadata as its own suite dir.
    let dest_dist = dists.join(&snap);
    let _ = std::fs::remove_dir_all(&dest_dist);
    let status = std::process::Command::new("cp")
        .arg("-r")
        .arg(tmp.join("dists").join(&snap))
        .arg(&dest_dist)
        .status()
        .context("failed to copy snapshot metadata")?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&tmp);
        return Err(anyhow!("bundle missing dists/{}", snap));
    }

    // Completeness check: every required deb must now be present in the pool
    // (already had it, or this bundle shipped it). A wrong --since fails HERE,
    // loudly, rather than corrupting the box.
    let root = PathBuf::from(OFFLINE_REPO);
    let missing: Vec<&String> = manifest
        .required_files
        .iter()
        .filter(|f| !root.join(f).exists())
        .collect();
    if !missing.is_empty() {
        // Roll back this partial import (leave shared pool; it's harmless).
        let _ = std::fs::remove_dir_all(&dest_dist);
        let _ = std::fs::remove_dir_all(&tmp);
        return Err(anyhow!(
            "Incomplete import: {} required file(s) are neither on the box nor in this bundle \
             — e.g. {}. This looks like an incremental bundle whose base snapshot ('{}') was \
             never imported. Import the full/base bundle first.",
            missing.len(),
            missing[0],
            manifest.since.as_deref().unwrap_or("?")
        ));
    }

    // Persist the manifest, then verify the signature via a scoped apt update
    // against a temp source pointing at this snapshot.
    std::fs::copy(tmp.join("manifest.json"), manifests.join(format!("{}.json", snap)))?;
    let _ = std::fs::remove_dir_all(&tmp);

    let verify_dir = PathBuf::from(OFFLINE_REPO).join(".verify");
    let _ = std::fs::remove_dir_all(&verify_dir);
    std::fs::create_dir_all(&verify_dir)?;
    write_verify_source(&verify_dir, &snap)?;
    let verify_res = apt_update_local(&verify_dir.join("autocore-verify.sources").to_string_lossy());
    let _ = std::fs::remove_dir_all(&verify_dir);
    if let Err(e) = verify_res {
        // Signature/verification failed — reject the import.
        let _ = std::fs::remove_dir_all(&dest_dist);
        let _ = std::fs::remove_file(manifests.join(format!("{}.json", snap)));
        return Err(anyhow!("Import rejected: {}", e));
    }

    println!(
        "{}",
        format!("Imported '{}'. Activate it with:  sudo acctl snapshot use {}", snap, snap).green()
    );
    Ok(())
}

/// A temporary source file (in its own dir) used only to verify a snapshot's
/// signature at import time, without disturbing the active source.
fn write_verify_source(dir: &Path, suite: &str) -> Result<()> {
    let body = format!(
        "Types: deb\n\
         URIs: file://{}\n\
         Suites: {}\n\
         Components: main\n\
         Architectures: amd64\n\
         Signed-By: {}\n",
        OFFLINE_REPO, suite, KEYRING_PATH
    );
    std::fs::write(dir.join("autocore-verify.sources"), body)?;
    Ok(())
}

async fn cmd_snapshot_list(_config: &Config) -> Result<()> {
    let active = read_active_snapshot();
    let mut manifests = all_manifests();
    manifests.sort_by(|a, b| a.snapshot.cmp(&b.snapshot));
    if manifests.is_empty() {
        println!("No snapshots imported. Import one with `acctl snapshot import <bundle>`.");
        return Ok(());
    }
    println!("{}", "Imported snapshots (offline library):".bold());
    for m in &manifests {
        let marker = if active.as_deref() == Some(m.snapshot.as_str()) {
            "*".green().to_string()
        } else {
            " ".to_string()
        };
        println!(
            " {} {}  ({} packages, imported {})",
            marker,
            m.snapshot.bold(),
            m.packages.len(),
            if m.created.is_empty() { "?" } else { &m.created }
        );
    }
    println!("\n{} = active. Switch with `acctl snapshot use <name>`.", "*".green());
    Ok(())
}

async fn cmd_snapshot_use(config: &Config, name: &str, yes: bool) -> Result<()> {
    local_root_guard(config)?;
    let manifest = load_manifest(name)?;
    gate_not_mid_test(config).await?;

    let previous = read_active_snapshot();
    if previous.as_deref() == Some(name) {
        println!("Snapshot '{}' is already active. Re-applying its package set.", name);
    }

    println!("{}", format!("Switching to snapshot '{}':", name).bold());
    for p in &manifest.packages {
        println!("  {} = {}", p.name, p.version);
    }
    if !yes && !confirm("Apply this snapshot?")? {
        println!("Aborted.");
        return Ok(());
    }

    // Record the currently-installed autocore set so we can roll back to exactly
    // it (not just "the previous snapshot manifest") if this goes wrong.
    let before = installed_autocore_packages()?;

    write_offline_source(name)?;
    apt_update_local(OFFLINE_SOURCES)?;

    let pkgs: Vec<(String, String)> = manifest
        .packages
        .iter()
        .map(|p| (p.name.clone(), p.version.clone()))
        .collect();
    apply_version_set(&pkgs)?;

    match restart_and_wait_healthy(config).await {
        Ok(()) => {
            write_active_snapshot(name)?;
            println!("{}", format!("Snapshot '{}' active — server healthy.", name).green());
            Ok(())
        }
        Err(reason) => {
            eprintln!("{} {}", "Server did not come back healthy:".red(), reason);
            // Roll back: repoint the source at the previous snapshot (so apt can
            // find the old versions) and reinstall the exact prior set.
            match &previous {
                Some(prev) => {
                    eprintln!("{}", format!("Rolling back to snapshot '{}'...", prev).yellow());
                    write_offline_source(prev)?;
                    apt_update_local(OFFLINE_SOURCES)?;
                    apply_version_set(&before)?;
                    match restart_and_wait_healthy(config).await {
                        Ok(()) => Err(anyhow!(
                            "Snapshot '{}' left the server degraded ({}); rolled back to '{}' (now healthy).",
                            name, reason, prev
                        )),
                        Err(r2) => Err(anyhow!(
                            "Snapshot '{}' degraded the server ({}) AND rollback to '{}' failed ({}). Manual intervention needed.",
                            name, reason, prev, r2
                        )),
                    }
                }
                None => Err(anyhow!(
                    "Snapshot '{}' left the server degraded ({}) and there is no previous snapshot to roll back to.",
                    name, reason
                )),
            }
        }
    }
}

async fn cmd_snapshot_remove(config: &Config, name: &str, yes: bool) -> Result<()> {
    local_root_guard(config)?;
    if read_active_snapshot().as_deref() == Some(name) {
        return Err(anyhow!(
            "'{}' is the ACTIVE snapshot — switch to another with `acctl snapshot use` first.",
            name
        ));
    }
    let (_, dists, manifests, _) = offline_paths();
    let dist_dir = dists.join(name);
    let manifest_file = manifests.join(format!("{}.json", name));
    if !manifest_file.exists() && !dist_dir.exists() {
        return Err(anyhow!("snapshot '{}' is not imported", name));
    }
    if !yes && !confirm(&format!("Remove snapshot '{}' and GC unused pool files?", name))? {
        println!("Aborted.");
        return Ok(());
    }

    let _ = std::fs::remove_dir_all(&dist_dir);
    let _ = std::fs::remove_file(&manifest_file);

    // Pool GC: keep only files still referenced by a remaining snapshot.
    let keep: std::collections::HashSet<String> = all_manifests()
        .iter()
        .flat_map(|m| m.required_files.iter().cloned())
        .collect();
    let root = PathBuf::from(OFFLINE_REPO);
    let out = std::process::Command::new("find")
        .arg(root.join("pool"))
        .args(["-type", "f"])
        .output()
        .context("failed to run find for pool GC")?;
    let mut removed = 0usize;
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let rel = match Path::new(line).strip_prefix(&root) {
            Ok(r) => r.to_string_lossy().to_string(),
            Err(_) => continue,
        };
        if !keep.contains(&rel) {
            if std::fs::remove_file(line).is_ok() {
                removed += 1;
            }
        }
    }
    println!(
        "{}",
        format!("Removed snapshot '{}'. GC reclaimed {} pool file(s).", name, removed).green()
    );
    Ok(())
}

async fn cmd_snapshot_detach(config: &Config, purge: bool, yes: bool) -> Result<()> {
    local_root_guard(config)?;

    let source_present = Path::new(OFFLINE_SOURCES).exists();
    let repo_present = Path::new(OFFLINE_REPO).exists();
    if !source_present && !repo_present {
        println!("Nothing to detach — this machine has no offline snapshot source or repo.");
        return Ok(());
    }

    println!("{}", "Detaching from offline snapshot mode:".bold());
    println!("  - remove the local offline apt source ({})", OFFLINE_SOURCES);
    if purge {
        println!("  - PURGE the local offline repo ({})", OFFLINE_REPO);
    } else {
        println!(
            "  - keep imported snapshots at {} (re-enter offline mode later with \
             `acctl snapshot use`)",
            OFFLINE_REPO
        );
    }
    if !yes && !confirm("Proceed?")? {
        println!("Aborted.");
        return Ok(());
    }

    // Remove the source file outright — the surest way to prevent a dangling
    // file:// source from ever blocking an online `apt-get update`. A later
    // `acctl snapshot use` recreates it.
    if source_present {
        std::fs::remove_file(OFFLINE_SOURCES)
            .with_context(|| format!("failed to remove {}", OFFLINE_SOURCES))?;
    }
    // Clear the active marker (nothing offline is active anymore).
    let (_, _, _, active) = offline_paths();
    let _ = std::fs::remove_file(active);

    if purge {
        let _ = std::fs::remove_dir_all(OFFLINE_REPO);
        println!("{}", "Detached and purged the local offline repo.".green());
    } else {
        println!(
            "{}",
            "Detached. Imported snapshots kept for a future offline switch.".green()
        );
    }
    println!("This machine now updates only from its online source (`acctl update`).");
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

    let empty: Vec<serde_json::Value> = Vec::new();
    let findings = response.data.get("errors").and_then(|v| v.as_array()).unwrap_or(&empty);

    // Partition into warnings (informational) vs errors (block sync).
    // Findings without an explicit `severity` field default to error
    // for back-compat with older server builds.
    let (warnings, errors): (Vec<&serde_json::Value>, Vec<&serde_json::Value>) =
        findings.iter().partition(|e| {
            e.get("severity").and_then(|v| v.as_str()) == Some("warning")
        });

    // Group helper: bucket by category for readable output.
    fn group<'a>(
        items: &[&'a serde_json::Value],
    ) -> std::collections::BTreeMap<&'a str, Vec<&'a serde_json::Value>> {
        let mut by_category = std::collections::BTreeMap::new();
        for e in items {
            let cat = e.get("category").and_then(|v| v.as_str()).unwrap_or("(unknown)");
            by_category.entry(cat).or_insert_with(Vec::new).push(*e);
        }
        by_category
    }

    if !warnings.is_empty() {
        eprintln!("{}", "Project validation warnings (not blocking):".yellow().bold());
        for (cat, entries) in &group(&warnings) {
            eprintln!();
            eprintln!("  {} ({})", cat.yellow(), entries.len());
            for e in entries {
                let path = e.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let message = e.get("message").and_then(|v| v.as_str()).unwrap_or("");
                eprintln!("    {} {}", path.dimmed(), message);
            }
        }
        eprintln!();
    }

    if ok && errors.is_empty() {
        if warnings.is_empty() {
            println!("{}", "Project validation: OK".green());
        } else {
            println!(
                "{} {}",
                "Project validation: OK".green(),
                format!("({} warning(s); fix in the AIS UI when convenient)", warnings.len()).dimmed(),
            );
        }
        return Ok(());
    }

    let by_category = group(&errors);
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
        errors.len(),
        by_category.len(),
        if by_category.len() == 1 { "y" } else { "ies" },
    ))
}

/// Fetch the server's effective TIS methods (test_methods.json sidecar,
/// legacy embedded block as fallback) via `system.get_test_methods`.
/// Returns `Ok(None)` when the server predates the command; the inner
/// value is `Value::Null` when the server has no methods at all.
async fn fetch_server_test_methods(client: &mut WsClient) -> Result<Option<serde_json::Value>> {
    let resp = client
        .send_command("system.get_test_methods", serde_json::json!({}))
        .await?;
    if !resp.success {
        // Old server without the command — caller skips the sidecar leg.
        return Ok(None);
    }
    Ok(Some(
        resp.data
            .get("test_methods")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    ))
}

/// Comparison form for a test-methods map: `null` (server/local has none)
/// and `{}` are both "no methods".
fn normalize_methods_for_compare(v: &serde_json::Value) -> serde_json::Value {
    if v.is_null() {
        serde_json::json!({})
    } else {
        v.clone()
    }
}

/// Fetch the server's effective AMS config (asset_management.json sidecar,
/// or a legacy embedded block). `Ok(None)` when the server predates
/// `system.get_asset_management` — the caller then skips the sidecar leg.
async fn fetch_server_asset_management(
    client: &mut WsClient,
) -> Result<Option<serde_json::Value>> {
    let resp = client
        .send_command("system.get_asset_management", serde_json::json!({}))
        .await?;
    if !resp.success {
        // Old server without the command — caller skips the sidecar leg.
        return Ok(None);
    }
    Ok(Some(
        resp.data
            .get("ams")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    ))
}

/// Comparison form for an AMS config object. `null`, a missing/`null`
/// `asset_types`, an omitted allowlist, and an empty `asset_refs` all read
/// as "not set", so an embedded-assembled block (explicit nulls) and a
/// canonical sidecar don't spuriously read as drift.
fn normalize_ams_for_compare(v: &serde_json::Value) -> serde_json::Value {
    let Some(obj) = v.as_object() else {
        return serde_json::json!({});
    };
    let mut out = serde_json::Map::new();
    if let Some(t) = obj.get("asset_types") {
        if !t.is_null() {
            out.insert("asset_types".to_string(), t.clone());
        }
    }
    if let Some(e) = obj.get("enabled_builtin_asset_types") {
        if !e.is_null() {
            out.insert("enabled_builtin_asset_types".to_string(), e.clone());
        }
    }
    if let Some(r) = obj.get("asset_refs") {
        let empty = r.is_null() || r.as_array().map(|a| a.is_empty()).unwrap_or(false);
        if !empty {
            out.insert("asset_refs".to_string(), r.clone());
        }
    }
    serde_json::Value::Object(out)
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

    // Sidecar leg: the server's effective test methods (Ok(None) when it
    // predates system.get_test_methods — skip the comparison then).
    let server_methods = fetch_server_test_methods(&mut client).await.ok().flatten();

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

    if let Some(server_m) = server_methods {
        let local_m = match test_methods::effective_test_methods(&project_path, &local_json) {
            Ok(m) => m.unwrap_or(serde_json::Value::Null),
            Err(e) => {
                eprintln!(
                    "{}",
                    format!("Warning: {:#}, skipping test-methods sync check.", e).yellow()
                );
                return Ok(());
            }
        };
        if normalize_methods_for_compare(&local_m) != normalize_methods_for_compare(&server_m) {
            return Err(anyhow!(
                "Test methods differ ({}) — 'acctl sync' will pull the machine's copy down. \
                 Run it first, or use '--force' to skip.",
                test_methods::TEST_METHODS_FILE
            ));
        }
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

async fn cmd_sync(config: &Config, full: bool, dry_run: bool) -> Result<()> {
    sync_project_json(config, dry_run).await?;
    println!();
    sync_datastore(config, full, dry_run).await?;
    if !full {
        println!(
            "{}",
            "  (critical files only — `acctl sync all` syncs the full datastore)".dimmed()
        );
    }
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

    // Sidecar leg — one-way, down. Methods are authored on the machine
    // via the HMI, so the machine is the source of truth for
    // test_methods.json: sync always pulls it, never pushes it. The
    // deliberate overwrite path is `acctl push test-methods`. None =
    // server predates system.get_test_methods; sidecar sits this one out.
    let server_methods = fetch_server_test_methods(&mut client).await?;
    match &server_methods {
        None => {}
        Some(server_m) if !server_m.is_null() => {
            // Compare against the local effective methods (sidecar, or a
            // legacy embedded block). A corrupt local sidecar counts as
            // "differs" so the server's good copy replaces it.
            let local_m = match test_methods::effective_test_methods(&project_path, &local_json)
            {
                Ok(m) => m.unwrap_or(serde_json::Value::Null),
                Err(e) => {
                    eprintln!(
                        "{}",
                        format!("Warning: {:#} — pulling the server's copy.", e).yellow()
                    );
                    serde_json::Value::Null
                }
            };
            if normalize_methods_for_compare(&local_m)
                == normalize_methods_for_compare(server_m)
            {
                println!(
                    "{}",
                    format!("{} is in sync.", test_methods::TEST_METHODS_FILE).green()
                );
            } else if dry_run {
                println!(
                    "{}",
                    format!(
                        "(dry-run) Would update local {} from server.",
                        test_methods::TEST_METHODS_FILE
                    )
                    .dimmed()
                );
            } else {
                let sidecar = test_methods::sidecar_path(&project_path);
                let pretty = serde_json::to_string_pretty(&test_methods::wrapped(server_m))?;
                fs::write(&sidecar, pretty)?;
                println!(
                    "{}",
                    format!("Local {} updated from server.", test_methods::TEST_METHODS_FILE)
                        .green()
                );
            }
        }
        Some(_) => {
            // Server has no methods at all. Never push from sync — leave
            // the local file for a deliberate `acctl push test-methods`.
            if test_methods::sidecar_path(&project_path).is_file() {
                println!(
                    "{}",
                    format!(
                        "Note: server has no test methods; local {} left alone \
                         (push deliberately with `acctl push test-methods`).",
                        test_methods::TEST_METHODS_FILE
                    )
                    .yellow()
                );
            }
        }
    }

    // AMS config sidecar leg — one-way, down. Mirrors the test_methods leg
    // above: AMS configuration is seeded/authored deliberately (or via
    // `acctl push asset-config`), so sync always pulls it, never pushes it.
    // None = server predates system.get_asset_management; sidecar sits this
    // one out.
    let server_ams = fetch_server_asset_management(&mut client).await?;
    match &server_ams {
        None => {}
        Some(server_a) if !server_a.is_null() => {
            // Compare against the local effective AMS config (sidecar, or a
            // legacy embedded block). A corrupt local sidecar counts as
            // "differs" so the server's good copy replaces it.
            let local_a = match asset_management::effective_asset_management(
                &project_path,
                &local_json,
            ) {
                Ok(a) => a.unwrap_or(serde_json::Value::Null),
                Err(e) => {
                    eprintln!(
                        "{}",
                        format!("Warning: {:#} — pulling the server's copy.", e).yellow()
                    );
                    serde_json::Value::Null
                }
            };
            if normalize_ams_for_compare(&local_a) == normalize_ams_for_compare(server_a) {
                println!(
                    "{}",
                    format!("{} is in sync.", asset_management::ASSET_MANAGEMENT_FILE).green()
                );
            } else if dry_run {
                println!(
                    "{}",
                    format!(
                        "(dry-run) Would update local {} from server.",
                        asset_management::ASSET_MANAGEMENT_FILE
                    )
                    .dimmed()
                );
            } else {
                let sidecar = asset_management::sidecar_path(&project_path);
                let pretty = serde_json::to_string_pretty(&asset_management::wrapped(server_a))?;
                fs::write(&sidecar, pretty)?;
                println!(
                    "{}",
                    format!(
                        "Local {} updated from server.",
                        asset_management::ASSET_MANAGEMENT_FILE
                    )
                    .green()
                );
            }
        }
        Some(_) => {
            // Server has no AMS config at all. Never push from sync — leave
            // the local file for a deliberate `acctl push asset-config`.
            if asset_management::sidecar_path(&project_path).is_file() {
                println!(
                    "{}",
                    format!(
                        "Note: server has no AMS config; local {} left alone \
                         (push deliberately with `acctl push asset-config`).",
                        asset_management::ASSET_MANAGEMENT_FILE
                    )
                    .yellow()
                );
            }
        }
    }

    // Semantic comparison (project.json only — the sidecar was handled above)
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
            // The sidecar is never pushed from sync (machine-authored);
            // see `acctl push test-methods` for the deliberate path.
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

/// Datastore-relative paths the server owns as live, machine-local state.
/// `acctl sync` may pull a fresher server copy down, but never pushes these
/// up — even if the local mtime is newer it's almost always stale, or a
/// deliberate restore the operator should do via the matching
/// `acctl push <thing>`. A trailing `/` marks a directory prefix; any other
/// entry is matched as an exact relative path.
///
/// - `autocore_gnv.ini`: NV writes from the running control program land
///   here. Restore with `acctl push gnv`.
/// - `assets/`: AMS asset + calibration records — the transducer actually
///   installed in *this* machine, its cert history, usage counters. The
///   shared `project.json` is the same across machines; this data is not, so
///   an auto-push would let one machine clobber another's assets on the
///   shared server. Publish deliberately with `acctl push assets`.
const SYNC_PULL_ONLY: &[&str] = &["autocore_gnv.ini", "assets/"];

/// Datastore-relative paths plain `acctl sync` (no `all`) reconciles: the
/// critical machine state worth backing up on every sync. Everything else
/// (captures/, scripts/, ...) only moves on an explicit `acctl sync all` —
/// large capture sets make a full default sync slow and fragile over remote
/// links. `methods/` holds the committed test-method `.seq.json` files the
/// control program runs; they are small and safety-relevant, so they ride
/// every sync (mtime-wins both ways, unlike the pull-only entries below).
const SYNC_CRITICAL: &[&str] = &["autocore_gnv.ini", "assets/", "methods/"];

/// True if `path` (a datastore-relative path, forward-slashed) matches one
/// of `entries`. Directory entries (trailing `/`) match the dir itself and
/// anything under it; other entries match exactly.
fn path_in_list(path: &str, entries: &[&str]) -> bool {
    entries.iter().any(|entry| match entry.strip_suffix('/') {
        Some(dir) => path == dir || path.starts_with(&format!("{dir}/")),
        None => path == *entry,
    })
}

/// True if `path` is pull-only per `SYNC_PULL_ONLY` (see that const).
fn is_pull_only(path: &str) -> bool {
    path_in_list(path, SYNC_PULL_ONLY)
}

/// Group `paths` into batches whose summed size stays under `max_bytes`,
/// so each datastore transfer fits comfortably in one websocket message.
/// A single file larger than `max_bytes` gets its own batch.
fn batch_paths_by_size(
    paths: &[String],
    sizes: &std::collections::HashMap<&str, u64>,
    max_bytes: u64,
) -> Vec<Vec<String>> {
    let mut batches: Vec<Vec<String>> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    let mut current_bytes = 0u64;
    for p in paths {
        let size = sizes.get(p.as_str()).copied().unwrap_or(0);
        if !current.is_empty() && current_bytes + size > max_bytes {
            batches.push(std::mem::take(&mut current));
            current_bytes = 0;
        }
        current.push(p.clone());
        current_bytes += size;
    }
    if !current.is_empty() {
        batches.push(current);
    }
    batches
}

/// Per-request byte budget for datastore transfers (sum of raw file sizes).
/// Keeps each websocket message small enough for old servers' frame limits
/// and slow links, instead of one giant zip that trips the 16 MiB default.
const SYNC_CHUNK_BYTES: u64 = 8 * 1024 * 1024;

/// Response timeout for datastore transfer requests — a chunk over a slow
/// tailscale link can legitimately take longer than the 30s default.
const SYNC_TRANSFER_TIMEOUT: Duration = Duration::from_secs(300);

/// Mtime-wins bidirectional sync of <project>/datastore (excluding
/// results/). With `full == false`, restricted to the `SYNC_CRITICAL`
/// paths (which are all pull-only, so nothing is pushed).
async fn sync_datastore(config: &Config, full: bool, dry_run: bool) -> Result<()> {
    let project_root = find_project_root()?;
    let local_datastore = project_root.join("datastore");
    let exclude_results = ["results/"];

    if full {
        println!("Syncing datastore (excluding results/)...");
    } else {
        println!(
            "Syncing critical datastore files ({})...",
            SYNC_CRITICAL.join(", ")
        );
    }

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

    let mut server_files: Vec<(String, i64, u64)> = server_resp.data["files"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| {
            Some((
                v.get("path")?.as_str()?.to_string(),
                v.get("mtime_ms")?.as_i64()?,
                v.get("size").and_then(|s| s.as_u64()).unwrap_or(0),
            ))
        }).collect())
        .unwrap_or_default();

    let mut local_files = walk_local_files(&local_datastore, &exclude_results)?;

    if !full {
        server_files.retain(|(p, _, _)| path_in_list(p, SYNC_CRITICAL));
        local_files.retain(|(p, _, _)| path_in_list(p, SYNC_CRITICAL));
    }

    use std::collections::HashMap;
    let server_map: HashMap<&str, i64> = server_files.iter()
        .map(|(p, m, _)| (p.as_str(), *m)).collect();
    let local_map:  HashMap<&str, i64> = local_files.iter()
        .map(|(p, m, _)| (p.as_str(), *m)).collect();
    let server_sizes: HashMap<&str, u64> = server_files.iter()
        .map(|(p, _, s)| (p.as_str(), *s)).collect();
    let local_sizes: HashMap<&str, u64> = local_files.iter()
        .map(|(p, _, s)| (p.as_str(), *s)).collect();

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

    // Pull-only paths (see `is_pull_only` / `SYNC_PULL_ONLY`): files the
    // server owns as live, machine-local state. Sync may pull a fresher
    // server copy down, but never push the other way. Filtering happens
    // here (after the mtime comparison) rather than at list time so that
    // server→local pulls still work.
    let push_blocked: Vec<String> = to_push
        .iter()
        .filter(|p| is_pull_only(p))
        .cloned()
        .collect();
    to_push.retain(|p| !is_pull_only(p));

    if to_pull.is_empty() && to_push.is_empty() && push_blocked.is_empty() {
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
    if !push_blocked.is_empty() {
        println!(
            "  {} {} file(s) skipped (pull-only — use the matching `acctl push` to restore):",
            "⊘".yellow(),
            push_blocked.len(),
        );
        for p in &push_blocked { println!("    {}", p); }
    }

    if dry_run {
        println!("  {}", "(dry-run) no changes applied".dimmed());
        client.close().await?;
        return Ok(());
    }

    // Pull: ask the server for zips of these paths, batched so no single
    // websocket message blows past frame limits, and extract on top of local.
    if !to_pull.is_empty() {
        std::fs::create_dir_all(&local_datastore)?;
        let batches = batch_paths_by_size(&to_pull, &server_sizes, SYNC_CHUNK_BYTES);
        let mut pulled = 0usize;
        for batch in &batches {
            let resp = client.send_command_timeout(
                "system.download_datastore",
                serde_json::json!({ "paths": batch }),
                SYNC_TRANSFER_TIMEOUT,
            ).await?;
            if !resp.success {
                client.close().await?;
                return Err(anyhow!("download_datastore failed: {}", resp.error_message));
            }
            let b64 = resp.data["data"].as_str()
                .ok_or_else(|| anyhow!("download_datastore: missing 'data'"))?;
            let bytes = base64::engine::general_purpose::STANDARD.decode(b64)
                .context("base64 decode")?;
            pulled += extract_zip_preserving_mtime(&bytes, &local_datastore)?;
            if batches.len() > 1 {
                print!("\r  pulled {}/{} file(s)", pulled, to_pull.len());
                use std::io::Write;
                let _ = std::io::stdout().flush();
            }
        }
        if batches.len() > 1 { println!(); } else { println!("  pulled {} file(s)", pulled); }
    }

    // Push: zip local versions of `to_push` and send, batched like pulls.
    if !to_push.is_empty() {
        let batches = batch_paths_by_size(&to_push, &local_sizes, SYNC_CHUNK_BYTES);
        let mut pushed = 0u64;
        for batch in &batches {
            let zip_bytes = build_zip_from_paths(&local_datastore, batch)?;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&zip_bytes);
            let resp = client.send_command_timeout(
                "system.upload_datastore",
                serde_json::json!({ "data": b64, "preserve_mtime": true }),
                SYNC_TRANSFER_TIMEOUT,
            ).await?;
            if !resp.success {
                client.close().await?;
                return Err(anyhow!("upload_datastore failed: {}", resp.error_message));
            }
            pushed += resp.data["files_extracted"].as_u64().unwrap_or(0);
            if batches.len() > 1 {
                print!("\r  pushed {}/{} file(s)", pushed, to_push.len());
                use std::io::Write;
                let _ = std::io::stdout().flush();
            }
        }
        if batches.len() > 1 { println!(); } else { println!("  pushed {} file(s)", pushed); }
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
    let files: Vec<(String, u64)> = list.data["files"].as_array()
        .map(|arr| arr.iter().filter_map(|v| Some((
            v.get("path")?.as_str()?.to_string(),
            v.get("size").and_then(|s| s.as_u64()).unwrap_or(0),
        ))).collect())
        .unwrap_or_default();
    if files.is_empty() {
        println!("{}", "Server has no results to pull.".dimmed());
        client.close().await?;
        return Ok(());
    }
    let paths: Vec<String> = files.iter().map(|(p, _)| p.clone()).collect();
    let sizes: std::collections::HashMap<&str, u64> =
        files.iter().map(|(p, s)| (p.as_str(), *s)).collect();

    println!("Pulling {} results file(s)...", paths.len());
    fs::create_dir_all(&local_datastore)?;
    let mut n = 0usize;
    for batch in batch_paths_by_size(&paths, &sizes, SYNC_CHUNK_BYTES) {
        let resp = client.send_command_timeout(
            "system.download_datastore",
            serde_json::json!({ "paths": batch }),
            SYNC_TRANSFER_TIMEOUT,
        ).await?;
        if !resp.success {
            client.close().await?;
            return Err(anyhow!("download_datastore: {}", resp.error_message));
        }
        let bytes = base64::engine::general_purpose::STANDARD.decode(
            resp.data["data"].as_str().ok_or_else(|| anyhow!("missing 'data'"))?,
        )?;
        n += extract_zip_preserving_mtime(&bytes, &local_datastore)?;
    }
    println!("{} pulled {} file(s) into {:?}", "✓".green(), n, local_datastore.join("results"));
    client.close().await?;
    Ok(())
}

/// `acctl push scripts` — upload local datastore/scripts/ to the server.
/// Push the local `datastore/methods/` directory (committed test-method
/// `.seq.json` files) to the server. Called during `acctl deploy` so a freshly
/// provisioned target has the canonical method before its first run; ongoing
/// reconcile is handled by `acctl sync` (`methods/` is in `SYNC_CRITICAL`).
async fn cmd_push_methods(config: &Config, target: Option<&str>) -> Result<()> {
    let project_root = find_project_root()?;
    let local_methods = project_root.join("datastore").join("methods");
    if !local_methods.is_dir() {
        return Ok(()); // nothing to provision; deploy skips silently
    }

    let local_datastore = project_root.join("datastore");
    let entries = walk_local_files(&local_datastore, &[])?;
    let paths: Vec<String> = entries.into_iter()
        .map(|(p, _, _)| p)
        .filter(|p| p.starts_with("methods/"))
        .collect();
    if paths.is_empty() {
        return Ok(());
    }

    println!("Pushing {} method file(s)...", paths.len());
    let zip_bytes = build_zip_from_paths(&local_datastore, &paths)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&zip_bytes);

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let mut payload = serde_json::json!({ "data": b64, "preserve_mtime": true });
    if let Some(t) = target {
        payload["project_name"] = serde_json::json!(t);
    }
    let resp = client.send_command("system.upload_datastore", payload).await?;
    if !resp.success {
        client.close().await?;
        return Err(anyhow!("upload_datastore: {}", resp.error_message));
    }
    let n = resp.data["files_extracted"].as_u64().unwrap_or(0);
    println!("{} pushed {} method file(s)", "✓".green(), n);
    client.close().await?;
    Ok(())
}

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

/// Publish the local AMS data (`datastore/assets/`) to the server.
///
/// `acctl sync` treats `assets/` as pull-only (see `SYNC_PULL_ONLY`): the
/// records are machine-local and the shared project.json must not carry one
/// machine's hardware state onto another. This command is the deliberate
/// publish path. Additive (no server-side deletes); after upload it calls
/// `ams.reinitialize` so the running servelet reloads from disk instead of
/// overwriting the pushed files from its stale in-memory registry — the AMS
/// analogue of `acctl push gnv` → `gm.reinitialize`.
async fn cmd_push_assets(config: &Config, no_reinit: bool) -> Result<()> {
    let project_root = find_project_root()?;
    let local_datastore = project_root.join("datastore");

    // Relative paths under datastore/ so server-side extraction lands them
    // back at <datastore>/assets/... .
    let entries = walk_local_files(&local_datastore, &[])?;
    let paths: Vec<String> = entries
        .into_iter()
        .map(|(p, _, _)| p)
        .filter(|p| p.starts_with("assets/"))
        .collect();
    if paths.is_empty() {
        println!("{}", "datastore/assets/ is empty; nothing to push.".dimmed());
        return Ok(());
    }

    println!("Pushing {} AMS file(s) to server...", paths.len());
    let zip_bytes = build_zip_from_paths(&local_datastore, &paths)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&zip_bytes);

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let resp = client
        .send_command(
            "system.upload_datastore",
            serde_json::json!({ "data": b64, "preserve_mtime": true }),
        )
        .await?;
    if !resp.success {
        client.close().await?;
        return Err(anyhow!("upload_datastore: {}", resp.error_message));
    }
    let n = resp.data["files_extracted"].as_u64().unwrap_or(0);
    println!("{} pushed {} file(s)", "✓".green(), n);

    if no_reinit {
        println!(
            "{} skipped ams.reinitialize (--no-reinit). Restart the server before any AMS write occurs to avoid stale-cache clobber.",
            "⚠".yellow()
        );
    } else {
        println!("Reinitializing AMS to load pushed asset records...");
        let reinit = client
            .send_command("ams.reinitialize", serde_json::json!({}))
            .await?;
        if !reinit.success {
            client.close().await?;
            return Err(anyhow!(
                "ams.reinitialize failed after upload: {}. Files are on disk but the in-memory registry is stale — restart the server before the next AMS write.",
                reinit.error_message
            ));
        }
        let assets = reinit.data["assets"].as_u64().unwrap_or(0);
        println!("{} AMS reinitialized ({} assets)", "✓".green(), assets);
    }

    client.close().await?;
    Ok(())
}

/// Push the local `test_methods.json` sidecar to the server
/// (restore-from-backup). See `PushCommands::TestMethods` for the
/// direction rationale — sync only ever pulls methods down, this is the
/// one deliberate way up. The server writes only the sidecar (previous
/// copy backed up to test_methods.json.bak) via
/// `system.upload_test_methods`; project.json is untouched.
async fn cmd_push_test_methods(config: &Config, yes: bool) -> Result<()> {
    let project_root = find_project_root()?;
    let project_path = project_root.join("project.json");
    let sidecar = test_methods::sidecar_path(&project_path);
    let methods = test_methods::load_sidecar_methods(&project_path)?.ok_or_else(|| {
        anyhow!("No local {} at {}. Nothing to push.", test_methods::TEST_METHODS_FILE, sidecar.display())
    })?;
    let count = methods.as_object().map(|m| m.len()).unwrap_or(0);

    println!(
        "{}",
        format!(
            "This OVERWRITES the machine's test methods with {} ({} method(s)).",
            sidecar.display(),
            count
        )
        .yellow()
        .bold()
    );
    println!("Methods are normally authored on the machine via the HMI's method editor;");
    println!("the previous file is backed up to test_methods.json.bak on the server.");

    if !yes {
        print!("Proceed? [y/N]: ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let resp = client
        .send_command(
            "system.upload_test_methods",
            serde_json::json!({ "test_methods_json": test_methods::wrapped(&methods) }),
        )
        .await?;
    client.close().await?;
    if !resp.success {
        return Err(anyhow!("upload_test_methods: {}", resp.error_message));
    }
    println!(
        "{} pushed {} ({} method(s))",
        "✓".green(),
        test_methods::TEST_METHODS_FILE,
        count
    );
    Ok(())
}

/// Push the local `asset_management.json` sidecar to the server
/// (restore-from-backup). See `PushCommands::AssetConfig` for the direction
/// rationale — sync only ever pulls AMS config down, this is the one
/// deliberate way up. The server writes only the sidecar (previous copy
/// backed up to asset_management.json.bak) via
/// `system.upload_asset_management`; project.json is untouched.
///
/// Distinct from `cmd_push_assets` (`acctl push assets`), which publishes
/// the machine-local asset instances under datastore/assets/.
async fn cmd_push_asset_config(config: &Config, yes: bool) -> Result<()> {
    let project_root = find_project_root()?;
    let project_path = project_root.join("project.json");
    let sidecar = asset_management::sidecar_path(&project_path);
    let ams = asset_management::load_sidecar_ams(&project_path)?.ok_or_else(|| {
        anyhow!(
            "No local {} at {}. Nothing to push.",
            asset_management::ASSET_MANAGEMENT_FILE,
            sidecar.display()
        )
    })?;
    let type_count = ams
        .get("asset_types")
        .and_then(|t| t.as_object())
        .map(|m| m.len())
        .unwrap_or(0);

    println!(
        "{}",
        format!(
            "This OVERWRITES the machine's AMS configuration with {} ({} custom asset type(s)).",
            sidecar.display(),
            type_count
        )
        .yellow()
        .bold()
    );
    println!("AMS config is normally seeded deliberately, not authored on the machine;");
    println!("the previous file is backed up to asset_management.json.bak on the server.");

    if !yes {
        print!("Proceed? [y/N]: ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let resp = client
        .send_command(
            "system.upload_asset_management",
            serde_json::json!({ "asset_management_json": asset_management::wrapped(&ams) }),
        )
        .await?;
    client.close().await?;
    if !resp.success {
        return Err(anyhow!("upload_asset_management: {}", resp.error_message));
    }
    println!(
        "{} pushed {} ({} custom asset type(s))",
        "✓".green(),
        asset_management::ASSET_MANAGEMENT_FILE,
        type_count
    );
    Ok(())
}

/// Push `datastore/autocore_gnv.ini` to the server (restore-from-backup).
///
/// The server's GNV file is live state: NV writes from the running control
/// program land there. `acctl sync` deliberately pulls it but does not
/// push it, so an accidental sync can't roll the server back to a stale
/// local snapshot. This command exists for the explicit-restore case
/// (local known-good copy, server's copy lost or corrupted).
///
/// After uploading the file we call `gm.reinitialize` so the GM servelet
/// re-reads GNV from disk into its in-memory cache. Without that step
/// the next NV write would merge against the stale in-memory state and
/// clobber the values we just restored.
async fn cmd_push_gnv(config: &Config, no_reinit: bool, target: Option<&str>) -> Result<()> {
    let project_root = find_project_root()?;
    let local_datastore = project_root.join("datastore");
    let gnv_path = local_datastore.join("autocore_gnv.ini");
    if !gnv_path.is_file() {
        return Err(anyhow!(
            "No local GNV file at {:?}. Nothing to restore.",
            gnv_path
        ));
    }

    let size = fs::metadata(&gnv_path).map(|m| m.len()).unwrap_or(0);
    println!(
        "Pushing {} ({} bytes) to server (restore-from-backup)...",
        gnv_path.display(),
        size
    );

    // Single-file zip with entry "autocore_gnv.ini" so the server
    // extracts it to <datastore>/autocore_gnv.ini.
    let zip_bytes = build_zip_from_paths(&local_datastore, &["autocore_gnv.ini".to_string()])?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&zip_bytes);

    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let mut payload = serde_json::json!({ "data": b64, "preserve_mtime": true });
    if let Some(t) = target {
        payload["project_name"] = serde_json::json!(t);
    }
    let resp = client
        .send_command("system.upload_datastore", payload)
        .await?;
    if !resp.success {
        client.close().await?;
        return Err(anyhow!("upload_datastore: {}", resp.error_message));
    }
    let n = resp.data["files_extracted"].as_u64().unwrap_or(0);
    println!("{} pushed {} file(s)", "✓".green(), n);

    if no_reinit {
        println!(
            "{} skipped gm.reinitialize (--no-reinit). Restart the server before any NV writes occur to avoid stale-cache clobber.",
            "⚠".yellow()
        );
    } else {
        println!("Reinitializing GM to load restored NV values...");
        let reinit = client
            .send_command("gm.reinitialize", serde_json::json!({}))
            .await?;
        if !reinit.success {
            client.close().await?;
            return Err(anyhow!(
                "gm.reinitialize failed after upload: {}. File is on disk but in-memory NV cache is stale — restart the server before the next NV write.",
                reinit.error_message
            ));
        }
        println!("{} GM reinitialized", "✓".green());
    }

    client.close().await?;
    Ok(())
}

/// List GNV snapshots on the server, newest first.
async fn cmd_list_gnv_snapshots(config: &Config) -> Result<()> {
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let resp = client
        .send_command("system.list_gnv_snapshots", serde_json::json!({}))
        .await?;
    client.close().await?;
    if !resp.success {
        return Err(anyhow!("list_gnv_snapshots failed: {}", resp.error_message));
    }
    let arr = resp.data["snapshots"].as_array().cloned().unwrap_or_default();
    if arr.is_empty() {
        println!("{}", "No GNV snapshots on server.".dimmed());
        return Ok(());
    }
    println!("GNV snapshots (newest first):");
    for s in &arr {
        let name = s["name"].as_str().unwrap_or("?");
        let size = s["size"].as_u64().unwrap_or(0);
        let mtime_ms = s["mtime_ms"].as_i64().unwrap_or(0);
        let when = Local.timestamp_millis_opt(mtime_ms).single()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "?".to_string());
        println!("  {}  {:>8} bytes  {}", when.dimmed(), size, name);
    }
    Ok(())
}

/// Restore a GNV snapshot by name. Triggers a server-side
/// `gm.reinitialize` after the copy so values land in SHM.
async fn cmd_restore_gnv_snapshot(config: &Config, name: &str) -> Result<()> {
    println!(
        "Restoring GNV snapshot {} on server (server will snapshot current state first)...",
        name
    );
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let resp = client
        .send_command(
            "system.restore_gnv_snapshot",
            serde_json::json!({ "name": name }),
        )
        .await?;
    client.close().await?;
    if !resp.success {
        return Err(anyhow!("restore_gnv_snapshot failed: {}", resp.error_message));
    }
    let target = resp.data["target"].as_str().unwrap_or("unknown");
    println!("{} restored to {}", "✓".green(), target);
    if resp.data["gm_reinitialized"].as_bool().unwrap_or(false) {
        println!("{} GM reinitialize requested", "✓".green());
    }
    Ok(())
}

// ============================================================================
// System backup / restore
// ============================================================================

/// Human-readable byte size (e.g. "12.3 MiB").
fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", v, UNITS[i])
    }
}

/// Print a numbered table of backups (newest first) from a `list_backups`
/// response array. The 1-based index is what `remote-restore` prompts for.
fn print_backup_table(arr: &[serde_json::Value]) {
    println!("Backups on server (newest first):");
    for (i, b) in arr.iter().enumerate() {
        let name = b["name"].as_str().unwrap_or("?");
        let version = b["server_version"].as_str().unwrap_or("?");
        let size = b["size_bytes"].as_u64().unwrap_or(0);
        let mtime_ms = b["mtime_ms"].as_i64().unwrap_or(0);
        let when = Local
            .timestamp_millis_opt(mtime_ms)
            .single()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "?".to_string());
        let note = b["note"].as_str().unwrap_or("");
        let incl = if b["include_results"].as_bool().unwrap_or(false) { " +results" } else { "" };
        print!(
            "  {:>2}) {}  v{:<10} {:>10}{}  {}",
            i + 1,
            when.dimmed(),
            version,
            human_size(size),
            incl,
            name.dimmed(),
        );
        if note.is_empty() {
            println!();
        } else {
            println!("  — {}", note);
        }
    }
}

/// Create a whole-system backup on the server (or just list existing ones).
async fn cmd_remote_backup(
    config: &Config,
    include_results: bool,
    note: Option<String>,
    list: bool,
) -> Result<()> {
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    if list {
        let resp = client.send_command("system.list_backups", serde_json::json!({})).await?;
        client.close().await?;
        if !resp.success {
            return Err(anyhow!("list_backups failed: {}", resp.error_message));
        }
        let arr = resp.data["backups"].as_array().cloned().unwrap_or_default();
        if arr.is_empty() {
            println!("{}", "No backups on server.".dimmed());
        } else {
            print_backup_table(&arr);
        }
        return Ok(());
    }

    println!("Creating system backup on server (this can take a moment)...");
    let data = serde_json::json!({
        "include_results": include_results,
        "note": note.unwrap_or_default(),
    });
    // Tarring all projects can exceed the 30s default; give it room.
    let resp = client
        .send_command_timeout("system.create_backup", data, Duration::from_secs(600))
        .await?;
    client.close().await?;
    if !resp.success {
        return Err(anyhow!("create_backup failed: {}", resp.error_message));
    }

    let name = resp.data["name"].as_str().unwrap_or("?");
    let version = resp.data["server_version"].as_str().unwrap_or("?");
    let size = resp.data["size_bytes"].as_u64().unwrap_or(0);
    println!("{} backup created: {}", "✓".green(), name);
    println!("    version v{}   {}", version, human_size(size));
    if !include_results {
        println!(
            "{}",
            "    (test results/captures excluded — pass --include-results to keep them)".dimmed()
        );
    }
    println!("{}", "    pull it to this machine with `acctl sync backups`".dimmed());
    Ok(())
}

/// Restore a system backup on the server. Lists + prompts when `name` is None.
async fn cmd_remote_restore(config: &Config, name: Option<String>, yes: bool) -> Result<()> {
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;

    let resp = client.send_command("system.list_backups", serde_json::json!({})).await?;
    if !resp.success {
        client.close().await?;
        return Err(anyhow!("list_backups failed: {}", resp.error_message));
    }
    let arr = resp.data["backups"].as_array().cloned().unwrap_or_default();
    if arr.is_empty() {
        client.close().await?;
        println!("{}", "No backups on server.".dimmed());
        return Ok(());
    }

    // Resolve the chosen backup, interactively if no name was given.
    let selected = match name {
        Some(n) => arr
            .iter()
            .find(|b| b["name"].as_str() == Some(n.as_str()))
            .cloned()
            .ok_or_else(|| anyhow!("Backup not found on server: {}", n))?,
        None => {
            print_backup_table(&arr);
            print!("Select a backup to restore [1-{}, or q to cancel]: ", arr.len());
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let t = input.trim();
            if t.is_empty() || t.eq_ignore_ascii_case("q") {
                client.close().await?;
                println!("Aborted.");
                return Ok(());
            }
            let idx: usize = t.parse().map_err(|_| anyhow!("Invalid selection: {}", t))?;
            if idx < 1 || idx > arr.len() {
                client.close().await?;
                return Err(anyhow!("Selection out of range: {}", idx));
            }
            arr[idx - 1].clone()
        }
    };

    let sel_name = selected["name"].as_str().unwrap_or("?").to_string();
    let sel_version = selected["server_version"].as_str().unwrap_or("?");

    // Show the version transition so the operator knows what they're rolling to.
    let current = client
        .send_command("system.get_server_version", serde_json::json!({}))
        .await
        .ok()
        .and_then(|r| r.data["version"].as_str().map(String::from))
        .unwrap_or_else(|| "?".to_string());

    println!();
    println!(
        "{}",
        "Restore overwrites binaries, config and projects on the target, then restarts the server.".yellow()
    );
    println!("  current server: v{}", current);
    println!("  restore to:     v{}   ({})", sel_version, sel_name);

    if !yes {
        print!("Proceed? [y/N]: ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            client.close().await?;
            println!("Aborted.");
            return Ok(());
        }
    }

    println!("Restoring {} ...", sel_name);
    let resp = client
        .send_command_timeout(
            "system.restore_backup",
            serde_json::json!({ "name": sel_name }),
            Duration::from_secs(600),
        )
        .await;
    // The server restarts ~500ms after responding, so a dropped socket right
    // after the request is the expected, successful path.
    match resp {
        Ok(r) if r.success => {
            println!("{} restore applied; server is restarting. Reconnect in a few seconds.", "✓".green());
        }
        Ok(r) => {
            let _ = client.close().await;
            return Err(anyhow!("restore_backup failed: {}", r.error_message));
        }
        Err(e) => {
            println!(
                "{} restore requested; server is restarting (connection dropped, as expected: {}).",
                "✓".green(),
                e
            );
        }
    }
    let _ = client.close().await;
    Ok(())
}

/// Pull all system backups from the server into a local `backups/` directory,
/// skipping any already present at the same size. Pull-only — never uploads.
async fn cmd_sync_backups(config: &Config) -> Result<()> {
    let mut client = WsClient::connect(&config.get_host(), config.get_port()).await?;
    let resp = client.send_command("system.list_backups", serde_json::json!({})).await?;
    if !resp.success {
        client.close().await?;
        return Err(anyhow!("list_backups failed: {}", resp.error_message));
    }
    let arr = resp.data["backups"].as_array().cloned().unwrap_or_default();
    if arr.is_empty() {
        client.close().await?;
        println!("{}", "No backups on server.".dimmed());
        return Ok(());
    }

    let local_dir = PathBuf::from("backups");
    fs::create_dir_all(&local_dir)?;

    let mut pulled = 0usize;
    for b in &arr {
        let name = match b["name"].as_str() {
            Some(n) => n,
            None => continue,
        };
        let size = b["size_bytes"].as_u64().unwrap_or(0);
        let local_path = local_dir.join(name);
        if let Ok(meta) = fs::metadata(&local_path) {
            if meta.len() == size {
                println!("  {} {}", "= have".dimmed(), name);
                continue;
            }
        }
        download_backup_file(&mut client, name, &local_path).await?;
        // Best-effort sibling manifest so the local copy is self-describing.
        let manifest = format!("{}.manifest.json", name.trim_end_matches(".tar.gz"));
        let _ = download_backup_file(&mut client, &manifest, &local_dir.join(&manifest)).await;
        println!("  {} {} ({})", "↓ pulled".green(), name, human_size(size));
        pulled += 1;
    }
    client.close().await?;
    println!("{} {} backup(s) into {}/", "✓".green(), pulled, local_dir.display());
    Ok(())
}

/// Download a single backup file from the server in chunks, writing to `dest`
/// atomically (via a `.partial` temp). Used by `acctl sync backups`.
async fn download_backup_file(client: &mut WsClient, name: &str, dest: &Path) -> Result<()> {
    let tmp = PathBuf::from(format!("{}.partial", dest.to_string_lossy()));
    let mut file = fs::File::create(&tmp)?;
    let mut offset: u64 = 0;
    loop {
        let resp = client
            .send_command_timeout(
                "system.download_backup",
                serde_json::json!({ "name": name, "offset": offset, "length": SYNC_CHUNK_BYTES }),
                SYNC_TRANSFER_TIMEOUT,
            )
            .await?;
        if !resp.success {
            return Err(anyhow!("download_backup({}) failed: {}", name, resp.error_message));
        }
        let b64 = resp.data["data"]
            .as_str()
            .ok_or_else(|| anyhow!("download_backup: missing 'data'"))?;
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64)?;
        file.write_all(&bytes)?;
        let advanced = resp.data["length"].as_u64().unwrap_or(bytes.len() as u64);
        offset += advanced;
        if resp.data["eof"].as_bool().unwrap_or(true) {
            break;
        }
        if advanced == 0 {
            return Err(anyhow!("download_backup({}): server returned 0 bytes before EOF", name));
        }
    }
    file.flush()?;
    drop(file);
    fs::rename(&tmp, dest)?;
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

/// Project.json for `acctl new-tis-project`. Carries a couple of
/// trivial GM variables so the gm.rs codegen has something to chew on
/// the first time the user runs `acctl codegen`. The demo test method
/// lives in the `test_methods.json` sidecar ([`TIS_TEST_METHODS_JSON`]),
/// never in project.json. The TIS-specific readiness scalars
/// (`tis_staged*`, `tis_active*`) are auto-injected by
/// `Project::normalize()` on server load and don't need to live here.
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
  }
}
"#;

/// TIS test methods sidecar (`test_methods.json`, next to project.json)
/// with the demo `translational_traction` method. Methods never live in
/// project.json — the sidecar is the storage the server (and the HMI's
/// TIS method editor) reads and writes.
const TIS_TEST_METHODS_JSON: &str = r#"{
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

    // Root files — TIS-flavored project.json plus the test_methods.json
    // sidecar holding the demo method (methods never live in project.json).
    write_template(&project_dir, "project.json", &sub(TIS_PROJECT_JSON))?;
    write_template(&project_dir, "test_methods.json", TIS_TEST_METHODS_JSON)?;
    write_template(&project_dir, ".gitignore", GITIGNORE)?;
    write_template(&project_dir, "datastore/autocore_gnv.ini", &sub(GNV_INI))?;
    println!("  Created project.json");
    println!("  Created test_methods.json (with translational_traction test method)");

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
    println!("  acctl push test-methods --yes           # Seed the machine's test_methods.json");
    println!("  acctl codegen-tags                      # Regenerate gm.rs + tis.ts");
    println!("  acctl push control --start              # Build, deploy, start control program");
    println!("  cd www && npm install && npm run dev    # Start the HMI dev server");
    println!();
    println!("From here, author your real schema in the HMI's TIS method editor — the");
    println!("machine owns test_methods.json and `acctl sync` pulls it back down. (Local");
    println!("edits can be seeded deliberately with `acctl push test-methods`.) The");
    println!("Project, Test, and Data tabs pick changes up on the next page reload.");

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
        Commands::AddAxis { name, link, axis_type, backend } => {
            return cmd_add_axis(name, link, axis_type, backend).await;
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
        Commands::AddAxis { .. } => unreachable!(),
        Commands::Ams { cmd } => match cmd {
            AmsCommand::Export { output } => cmd_ams_export(&config, &output).await,
            AmsCommand::Import { input, dry_run } => cmd_ams_import(&config, &input, dry_run).await,
            AmsCommand::Backfill { dry_run } => cmd_ams_backfill(&config, dry_run).await,
        },
        Commands::Pull { extract } => cmd_pull(&config, extract).await,
        Commands::Push { what } => match what {
            PushCommands::Project { restart } => cmd_push_project(&config, restart, None).await,
            PushCommands::Www { source, no_build } => cmd_push_www(&config, source, no_build, None).await,
            PushCommands::Control {
                source,
                no_build,
                start,
                force,
            } => cmd_push_control(&config, source, no_build, start, force).await,
            PushCommands::Doc { no_build } => cmd_push_doc(&config, no_build).await,
            PushCommands::Scripts => cmd_push_scripts(&config).await,
            PushCommands::Assets { no_reinit } => cmd_push_assets(&config, no_reinit).await,
            PushCommands::TestMethods { yes } => cmd_push_test_methods(&config, yes).await,
            PushCommands::AssetConfig { yes } => cmd_push_asset_config(&config, yes).await,
            PushCommands::Gnv { no_reinit } => cmd_push_gnv(&config, no_reinit, None).await,
        },
        Commands::Codegen { force } => cmd_codegen(&config, force).await,
        Commands::Switch {
            project_name,
            restart,
        } => cmd_switch(&config, &project_name, restart).await,
        Commands::Status => cmd_status(&config).await,
        Commands::Logs { follow } => cmd_logs(&config, follow).await,
        Commands::Control { action } => cmd_control(&config, &action).await,
        Commands::Sync { scope, dry_run } => match scope.as_deref() {
            Some("backups") => cmd_sync_backups(&config).await,
            other => cmd_sync(&config, other.is_some(), dry_run).await,
        },
        Commands::Deploy { project_name, no_control, no_www, no_build, no_restart } =>
            cmd_deploy(&config, project_name, no_control, no_www, no_build, no_restart).await,
        Commands::PullResults => cmd_pull_results(&config).await,
        Commands::ListGnvSnapshots => cmd_list_gnv_snapshots(&config).await,
        Commands::RestoreGnvSnapshot { name } => cmd_restore_gnv_snapshot(&config, &name).await,
        Commands::RemoteBackup { include_results, note, list } =>
            cmd_remote_backup(&config, include_results, note, list).await,
        Commands::Update { list, check, rollback, version, channel, yes } =>
            cmd_update(&config, UpdateArgs { list, check, rollback, version, channel, yes }).await,
        Commands::Modules { cmd } => match cmd {
            ModulesCommand::List => cmd_modules_list(&config).await,
            ModulesCommand::Sync { remove_extras, yes } => cmd_modules_sync(&config, remove_extras, yes).await,
            ModulesCommand::Install { name, yes } => cmd_modules_install(&config, &name, yes).await,
            ModulesCommand::Remove { name, force, yes } => cmd_modules_remove(&config, &name, force, yes).await,
        },
        Commands::Migrate { yes } => cmd_migrate(&config, yes).await,
        Commands::Snapshot { cmd } => match cmd {
            SnapshotCommand::Import { bundle } => cmd_snapshot_import(&config, &bundle).await,
            SnapshotCommand::List => cmd_snapshot_list(&config).await,
            SnapshotCommand::Use { name, yes } => cmd_snapshot_use(&config, &name, yes).await,
            SnapshotCommand::Remove { name, yes } => cmd_snapshot_remove(&config, &name, yes).await,
            SnapshotCommand::Detach { purge, yes } => cmd_snapshot_detach(&config, purge, yes).await,
        },
        Commands::RemoteRestore { name, yes } => cmd_remote_restore(&config, name, yes).await,
        Commands::Tools { cmd } => match cmd {
            ToolsCommand::List => cmd_tools_list(&config).await,
            ToolsCommand::Rescan => cmd_tools_rescan(&config).await,
        },
        Commands::Config { cmd } => match cmd {
            ConfigCommand::List => cmd_config_show(&config, true).await,
            ConfigCommand::Show => cmd_config_show(&config, false).await,
            ConfigCommand::Set { name, restart } => cmd_config_set(&config, Some(&name), restart).await,
            ConfigCommand::Clear { restart } => cmd_config_set(&config, None, restart).await,
            ConfigCommand::Validate => cmd_config_validate(&config).await,
        },
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
    // Keep in sync with rust_type_for_test() in autocore-server/src/codegen/codegen_tis.rs
    // (the codegen path is what actually determines what's emit-able).
    let valid_types = [
        "bool",
        "u8", "u16", "u32", "u64",
        "i8", "i16", "i32", "i64",
        "f32", "f64",
        "string",
    ];
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

    // Print local-check results first.
    for e in &errors {
        println!("{} {}", colored::Colorize::red("ERROR:"), e);
    }
    for w in &warnings {
        println!("{}  {}", colored::Colorize::yellow("WARN:"), w);
    }

    println!("  {} modules, {} variables ({} linked)", module_domains.len(), var_count, link_count);

    // Now hit the server for the comprehensive validator. This is what
    // catches AMS placeholder errors (missing `default`, typos), AMS
    // registry/asset integrity, and the per-module schema checks. Fails
    // gracefully when no server is reachable — local checks still count.
    let server_failed = match Config::load() {
        Ok(cfg) => match WsClient::connect(&cfg.get_host(), cfg.get_port()).await {
            Ok(mut client) => {
                println!();
                let result = validate_project_remote(&mut client, Some(&project)).await;
                let _ = client.close().await;
                result.is_err()
            }
            Err(e) => {
                eprintln!(
                    "{} Server-side validation skipped (no connection to {}:{}): {}",
                    colored::Colorize::yellow("Note:"),
                    cfg.get_host(), cfg.get_port(), e,
                );
                false
            }
        },
        Err(_) => {
            eprintln!(
                "{} Server-side validation skipped (no acctl.toml — run `acctl set-target`).",
                colored::Colorize::yellow("Note:"),
            );
            false
        }
    };

    if errors.is_empty() && !server_failed {
        println!("{}", colored::Colorize::green("✓ project.json is valid"));
        return Ok(());
    }

    let total = errors.len() + if server_failed { 1 } else { 0 };
    Err(anyhow!("{} validation issue(s) found", total))
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

/// `acctl add-axis` — append a CiA-402 axis to project.json. EtherCAT axes go
/// in `modules.ethercat.config.axes` (untagged = ethercat, back-compat with
/// existing projects); virtual axes go in the backend-neutral
/// `modules.motion.config.axes` with an explicit `backend: {kind: "virtual"}`.
/// Idempotent on the axis name.
///
/// Drive-behavior defaults (e.g. `soft_home_method`) are NOT seeded
/// here: acctl/autocore-server must not read the EtherCAT device library. The
/// IDE seeds them at bind time from `ethercat.list_devices` (or set them by
/// hand under the axis `options`).
async fn cmd_add_axis(name: &str, link: &str, axis_type: &str, backend: &str) -> Result<()> {
    let module = match backend {
        "ethercat" => "ethercat",
        "virtual" => "motion",
        other => return Err(anyhow!(
            "unknown backend '{}' (expected 'ethercat' or 'virtual')", other
        )),
    };
    if backend == "ethercat" && link.is_empty() {
        return Err(anyhow!("--link <slave> is required for an ethercat axis"));
    }

    let (path, mut value) = load_project_json_relaxed()?;
    let obj = value
        .as_object_mut()
        .ok_or_else(|| anyhow!("project.json is not a JSON object"))?;

    let modules = obj
        .entry("modules")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or_else(|| anyhow!("project.json `modules` is not an object"))?;
    // `motion` is a config-only namespace (codegen reads its axes; there is no
    // motion module binary), so a newly-created one must be disabled or the
    // supervisor would try to spawn a nonexistent "motion" executable. The real
    // `ethercat` module is enabled. (In practice the target module already
    // exists, so this default only applies when creating the home for the first
    // virtual axis.)
    let default_enabled = module != "motion";
    let m = modules
        .entry(module)
        .or_insert_with(|| serde_json::json!({ "enabled": default_enabled, "config": {} }))
        .as_object_mut()
        .ok_or_else(|| anyhow!("module `{}` is not an object", module))?;
    let config = m
        .entry("config")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or_else(|| anyhow!("module `{}` config is not an object", module))?;
    let axes = config
        .entry("axes")
        .or_insert_with(|| serde_json::json!([]))
        .as_array_mut()
        .ok_or_else(|| anyhow!("`axes` is not an array"))?;

    if axes.iter().any(|a| a.get("name").and_then(|n| n.as_str()) == Some(name)) {
        println!("{}", format!("Axis '{}' already exists; nothing to do.", name).yellow());
        return Ok(());
    }

    // Build the axis entry. EtherCAT axes stay untagged (backend defaults to
    // ethercat) to match every existing project; virtual axes carry the tag.
    let mut axis = serde_json::Map::new();
    axis.insert("name".to_string(), serde_json::json!(name));
    if backend == "ethercat" {
        axis.insert("link".to_string(), serde_json::json!(link));
    } else {
        axis.insert("backend".to_string(), serde_json::json!({ "kind": "virtual" }));
    }
    axis.insert("type".to_string(), serde_json::json!(axis_type));
    axes.push(serde_json::Value::Object(axis));

    save_project_json_relaxed(&path, &value)?;
    println!(
        "{}",
        format!(
            "Added {} axis '{}' to modules.{}.config.axes in {}",
            backend, name, module, path.display()
        )
        .green()
    );
    println!();
    println!("Next steps:");
    if backend == "ethercat" {
        println!("  1. (Optional) seed drive defaults: in the IDE, bind the axis to its");
        println!("     slave — it reads the drive's motion_profile and sets axis options");
        println!("     like `soft_home_method`. Or set them by hand under `options`.");
        println!("  2. Run `acctl codegen` to generate the drive handle in control/src/gm.rs.");
    } else {
        println!("  1. Run `acctl codegen` to generate the SimDrive-backed handle in");
        println!("     control/src/gm.rs (no fieldbus needed).");
    }
    Ok(())
}

async fn cmd_add_tis() -> Result<()> {
    let (path, value) = load_project_json_relaxed()?;
    let sidecar = test_methods::sidecar_path(&path);
    if sidecar.is_file() {
        println!(
            "{}",
            format!("TIS already enabled ({} present).", sidecar.display()).yellow()
        );
        return Ok(());
    }
    if value.get("test_methods").is_some() {
        // Legacy layout: methods still embedded in project.json. Leave it
        // alone — the server migrates the block into the sidecar on the
        // next TIS save (tis.save_config), and writing an empty sidecar
        // here would shadow the embedded methods (sidecar wins on load).
        println!(
            "{}",
            "TIS already enabled (legacy embedded test_methods block in project.json).".yellow()
        );
        println!(
            "Note: the server migrates it into {} on the next save from the HMI's TIS editor.",
            test_methods::TEST_METHODS_FILE
        );
        return Ok(());
    }
    let empty = test_methods::wrapped(&serde_json::json!({}));
    fs::write(&sidecar, serde_json::to_vec_pretty(&empty)?)
        .with_context(|| format!("writing {}", sidecar.display()))?;
    println!("{}", format!("Wrote {} (project.json untouched)", sidecar.display()).green());
    println!();
    println!("Next steps:");
    println!("  1. Declare at least one method in {} (or use the HMI's", test_methods::TEST_METHODS_FILE);
    println!("     TIS method editor). See doc/ch15-test-information-system.md for the schema.");
    println!("  2. Run `acctl push test-methods` to seed the machine (sync only ever");
    println!("     pulls methods down), then `acctl push project --restart` and `acctl codegen`.");
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

    let effective = test_methods::effective_test_methods(&path, &value)?;
    if let Some(methods) = effective.as_ref().and_then(|v| v.as_object()) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_size_scales_units() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(1024), "1.0 KiB");
        assert_eq!(human_size(1024 * 1024), "1.0 MiB");
        assert_eq!(human_size(3 * 1024 * 1024 * 1024), "3.0 GiB");
    }

    #[test]
    fn parse_semver_handles_plain_and_prerelease() {
        assert_eq!(parse_semver("3.3.55"), Some((3, 3, 55)));
        assert_eq!(parse_semver("3.3.55-rc.1"), Some((3, 3, 55)));
        assert_eq!(parse_semver("10.0.2"), Some((10, 0, 2)));
        assert_eq!(parse_semver("not.a.version"), None);
        assert_eq!(parse_semver("3.3"), None);
    }

    #[test]
    fn required_min_std_scales_with_project_features() {
        // No axes → just the unconditional GmCompat floor.
        let bare = serde_json::json!({ "modules": {} });
        assert_eq!(required_min_autocore_std(&bare), STD_MIN_GMCOMPAT);

        // A plain ethercat axis → still just the GmCompat floor.
        let ec = serde_json::json!({ "modules": { "ethercat": { "config": { "axes": [
            { "name": "Press", "link": "AKD_3" }
        ] } } } });
        assert_eq!(required_min_autocore_std(&ec), STD_MIN_GMCOMPAT);

        // A virtual axis → 3.3.55 (SimDrive), the highest.
        let virt = serde_json::json!({ "modules": { "motion": { "config": { "axes": [
            { "name": "Sim", "backend": { "kind": "virtual" } }
        ] } } } });
        assert_eq!(required_min_autocore_std(&virt), STD_MIN_SIMDRIVE);
    }

    #[test]
    fn version_tuple_ordering_gates_correctly() {
        // The comparison the gate relies on.
        assert!((3, 3, 51) < STD_MIN_GMCOMPAT);
        assert!((3, 3, 54) < STD_MIN_SIMDRIVE);
        assert!(!((3, 3, 55) < STD_MIN_SIMDRIVE));
        assert!(!((3, 4, 0) < STD_MIN_SIMDRIVE));
    }

    #[test]
    fn is_pull_only_matches_gnv_and_assets_tree() {
        // Exact-match entry.
        assert!(is_pull_only("autocore_gnv.ini"));

        // Directory-prefix entry: the dir itself and everything under it.
        assert!(is_pull_only("assets"));
        assert!(is_pull_only("assets/registry.json"));
        assert!(is_pull_only("assets/load_cell/LC-0001/asset.json"));
        assert!(is_pull_only("assets/load_cell/LC-0001/calibrations/CAL-1.json"));
        assert!(is_pull_only("assets/load_cell/LC-0001/usage.json"));

        // Not pull-only: other datastore subtrees stay bidirectional.
        assert!(!is_pull_only("scripts/foo.lua"));
        assert!(!is_pull_only("results/run-1/test.json"));
        assert!(!is_pull_only("autocore_gnv.ini.bak"));
        // A path that merely starts with the dir name but isn't under it.
        assert!(!is_pull_only("assets_archive/old.json"));
    }

    #[test]
    fn batch_paths_by_size_respects_budget() {
        let paths: Vec<String> = ["a", "b", "c", "d", "e"].iter().map(|s| s.to_string()).collect();
        let sizes: std::collections::HashMap<&str, u64> =
            [("a", 4), ("b", 5), ("c", 1), ("d", 10), ("e", 2)].into_iter().collect();

        // Budget 10: a+b+c = 10 exactly fits; d (10) opens a new batch and
        // fills it; e goes in a third.
        let batches = batch_paths_by_size(&paths, &sizes, 10);
        assert_eq!(batches, vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string()],
            vec!["e".to_string()],
        ]);

        // A single file over budget still gets its own batch (never dropped).
        let batches = batch_paths_by_size(&paths, &sizes, 3);
        assert_eq!(batches.iter().map(|b| b.len()).sum::<usize>(), paths.len());
        assert!(batches.iter().all(|b| !b.is_empty()));

        // Unknown sizes default to 0 and everything lands in one batch.
        let batches = batch_paths_by_size(&paths, &Default::default(), 10);
        assert_eq!(batches.len(), 1);

        // Empty input → no batches.
        assert!(batch_paths_by_size(&[], &sizes, 10).is_empty());
    }

    #[test]
    fn critical_scope_matches_expected_paths() {
        assert!(path_in_list("autocore_gnv.ini", SYNC_CRITICAL));
        assert!(path_in_list("assets/registry.json", SYNC_CRITICAL));
        assert!(!path_in_list("captures/capture_20260610_154739.json", SYNC_CRITICAL));
        assert!(!path_in_list("scripts/foo.lua", SYNC_CRITICAL));
    }
}
