use std::{path::Path, process};

use a9_lint::{LintError, fix};
use clap::Parser;
use walkdir::WalkDir;

mod config;

#[derive(Parser)]
#[command(name = "cargo-a9-lint", about = "An opinionated Rust style linter")]
struct Cli {
    /// When invoked as `cargo a9-lint`, cargo passes the subcommand name.
    #[arg(hide = true, default_value = "")]
    _cargo_subcommand: String,
    /// Check only — report violations without fixing.
    #[arg(long)]
    check: bool,
}

fn main() {
    let mut args: Vec<String> = std::env::args().collect();

    if args.get(1).map(String::as_str) == Some("a9-lint") {
        args.remove(1);
    }

    let parsed = Cli::parse_from(args);
    let working_dir = std::env::current_dir().expect("failed to get cwd");
    let (config, project_root) = config::find_config(&working_dir);
    let scan_dirs = effective_scan_dirs(&config);
    let file_paths = collect_rs_files(&project_root, &scan_dirs);

    if parsed.check {
        run_check(&file_paths, &config);
    } else {
        run_fix(&file_paths, &config);
    }
}

fn run_check(file_paths: &[std::path::PathBuf], config: &config::Config) {
    let mut has_errors = false;

    for path in file_paths {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        let result = fix(&source, &config.features, &config.rules.allow);

        if result.changed {
            has_errors = true;
            eprintln!(
                "[a9-lint] {} — source is not in canonical form (run `cargo a9-lint` to fix)",
                path.display()
            );
        }

        for err in &result.errors {
            has_errors = true;

            match err {
                LintError::ParseError(msg) => {
                    eprintln!("{}:1 — parse error: {msg}", path.display());
                }
                LintError::RuleError {
                    rule,
                    line,
                    message,
                } => {
                    eprintln!("[{rule}] {}:{line} — {message}", path.display());
                }
            }
        }
    }

    if has_errors {
        process::exit(1);
    }
}

fn run_fix(file_paths: &[std::path::PathBuf], config: &config::Config) {
    let mut has_errors = false;

    for path in file_paths {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        let result = fix(&source, &config.features, &config.rules.allow);

        if result.source != source {
            let _ = std::fs::write(path, &result.source);
        }

        for err in &result.errors {
            has_errors = true;

            match err {
                LintError::ParseError(msg) => {
                    eprintln!("{}:1 — parse error: {msg}", path.display());
                }
                LintError::RuleError {
                    rule,
                    line,
                    message,
                } => {
                    eprintln!("[{rule}] {}:{line} — {message}", path.display());
                }
            }
        }
    }

    if !has_errors {
        eprintln!("a9-lint: all files clean");

        return;
    }

    eprintln!("a9-lint: some violations remain after fix");
    process::exit(1);
}

fn effective_scan_dirs(config: &config::Config) -> Vec<String> {
    if config.scan.is_empty() {
        vec!["src".to_string()]
    } else {
        config.scan.clone()
    }
}

fn collect_rs_files(root: &Path, scan_dirs: &[String]) -> Vec<std::path::PathBuf> {
    let mut paths = vec![];

    for dir in scan_dirs {
        let base = root.join(dir);

        if !base.exists() {
            continue;
        }

        for entry in WalkDir::new(&base)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| is_rust_source(e.path()))
        {
            paths.push(entry.path().to_path_buf());
        }
    }

    paths
}

fn is_rust_source(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("rs")
}
