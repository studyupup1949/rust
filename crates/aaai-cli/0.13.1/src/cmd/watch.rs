//! `aaai watch` — re-run the audit when source files change.

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use clap::Args;
use colored::Colorize;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use aaai_core::{AuditEngine, AuditStatus, DiffEngine, DiffType,
                config::io as config_io};

#[derive(Args)]
pub struct WatchArgs {
    #[arg(short = 'l', long, value_name = "PATH")]
    pub left: PathBuf,
    #[arg(short = 'r', long, value_name = "PATH")]
    pub right: PathBuf,
    #[arg(short = 'c', long, value_name = "FILE")]
    pub config: PathBuf,
    /// Debounce interval in milliseconds before re-running after a change.
    #[arg(long, default_value = "500")]
    pub debounce_ms: u64,
}

pub fn run(args: WatchArgs) -> anyhow::Result<()> {
    println!("{}", "aaai watch".bold());
    println!("Before : {}", args.left.display());
    println!("After  : {}", args.right.display());
    println!("Config : {}", args.config.display());
    println!("{}", "Watching for changes… (Ctrl-C to stop)".dimmed());
    println!();

    // Run once immediately.
    run_audit(&args);

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

    // Watch before dir, after dir, and config file.
    watcher.watch(&args.left,   RecursiveMode::Recursive)?;
    watcher.watch(&args.right,  RecursiveMode::Recursive)?;
    watcher.watch(&args.config, RecursiveMode::NonRecursive)?;

    let debounce = Duration::from_millis(args.debounce_ms);
    let mut last_run = std::time::Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) => {
                match event.kind {
                    EventKind::Create(_)
                    | EventKind::Modify(_)
                    | EventKind::Remove(_) => {
                        // Debounce: only re-run if enough time passed since last run.
                        if last_run.elapsed() >= debounce {
                            println!();
                            println!("{} Change detected — re-running audit…", "↻".cyan().bold());
                            println!();
                            run_audit(&args);
                            last_run = std::time::Instant::now();
                        }
                    }
                    _ => {}
                }
            }
            Ok(Err(e)) => log::warn!("Watch error: {e}"),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

fn run_audit(args: &WatchArgs) {
    let definition = match config_io::load(&args.config) {
        Ok(d) => d,
        Err(e) => {
            println!("{}", format!("Config error: {e}").red());
            return;
        }
    };
    let diffs = match DiffEngine::compare(&args.left, &args.right) {
        Ok(d) => d,
        Err(e) => {
            println!("{}", format!("Diff error: {e}").red());
            return;
        }
    };
    let result = AuditEngine::evaluate(&diffs, &definition);
    let s = &result.summary;

    // Compact one-line output for watch mode.
    let ts = chrono::Local::now().format("%H:%M:%S").to_string();
    let verdict = if s.is_passing() {
        format!("[{ts}] {} — OK:{} Pend:{} Fail:{}", "PASSED".green().bold(), s.ok, s.pending, s.failed)
    } else {
        format!("[{ts}] {} — OK:{} Pend:{} Fail:{} Err:{}", "FAILED".red().bold(),
            s.ok, s.pending, s.failed, s.error)
    };
    println!("{verdict}");

    // Show failing entries.
    for r in &result.results {
        if matches!(r.status, AuditStatus::Failed | AuditStatus::Error | AuditStatus::Pending)
            && r.diff.diff_type != DiffType::Unchanged
        {
            let icon = match r.status {
                AuditStatus::Failed  => "✗".red().to_string(),
                AuditStatus::Error   => "!".magenta().to_string(),
                AuditStatus::Pending => "?".yellow().to_string(),
                _                    => " ".to_string(),
            };
            println!("  {icon} {}", r.diff.path);
        }
    }
}
