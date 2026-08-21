use std::{collections::BTreeMap, io::Write, path::Path, process};

use a9_lint::{LintError, fix, project_rules::PROJECT_RULES, workspace};
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
    let scan_dirs = effective_scan_dirs(&config, &project_root);
    let file_paths = collect_rs_files(&project_root, &scan_dirs);

    if parsed.check {
        run_check(&file_paths, &config, &project_root);
    } else {
        run_fix(&file_paths, &config, &project_root);
    }
}

fn run_check(file_paths: &[std::path::PathBuf], config: &config::Config, project_root: &Path) {
    let mut has_errors = false;

    let edition = config::detect_edition(project_root);

    has_errors |= run_project_rules(project_root, &config.rules.allow);

    for path in file_paths {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        let result = fix(&source, &config.features, &config.rules.allow);
        let final_source = rustfmt_string(&result.source, &edition);

        if final_source != source {
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

fn run_fix(file_paths: &[std::path::PathBuf], config: &config::Config, project_root: &Path) {
    let edition = config::detect_edition(project_root);
    let disabled: Vec<&str> = config.rules.allow.iter().map(String::as_str).collect();

    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();

    for rule in PROJECT_RULES {
        if disabled.contains(&rule.name()) {
            continue;
        }

        rule.fix(project_root);

        for v in rule.detect(project_root) {
            *counts.entry(rule.name()).or_default() += 1;
            eprintln!("[{}] — {}", rule.name(), v.message);
        }
    }

    for path in file_paths {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        let result = fix(&source, &config.features, &config.rules.allow);
        let final_source = rustfmt_string(&result.source, &edition);

        if final_source != source {
            let _ = std::fs::write(path, &final_source);
        }

        for err in &result.errors {
            match err {
                LintError::ParseError(msg) => {
                    *counts.entry("parse-error").or_default() += 1;
                    eprintln!("{}:1 — parse error: {msg}", path.display());
                }
                LintError::RuleError {
                    rule,
                    line,
                    message,
                } => {
                    *counts.entry(rule).or_default() += 1;
                    eprintln!("[{rule}] {}:{line} — {message}", path.display());
                }
            }
        }
    }

    if counts.is_empty() {
        eprintln!("a9-lint: all files clean");

        return;
    }

    let total: usize = counts.values().sum();

    eprintln!("a9-lint: {total} violations remain after fix");

    for (rule, count) in &counts {
        eprintln!("  {rule}: {count}");
    }

    process::exit(1);
}

fn effective_scan_dirs(config: &config::Config, project_root: &std::path::Path) -> Vec<String> {
    workspace::effective_scan_dirs(&config.scan, project_root)
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

fn rustfmt_string(source: &str, edition: &str) -> String {
    let mut child = process::Command::new("rustfmt")
        .arg("--edition")
        .arg(edition)
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
        .expect("failed to run rustfmt — is it installed?");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .expect("failed to write to rustfmt stdin");

    let output = child.wait_with_output().expect("failed to wait on rustfmt");

    if !output.status.success() {
        return source.to_string();
    }

    String::from_utf8(output.stdout).expect("rustfmt produced non-UTF-8 output")
}

fn run_project_rules(project_root: &Path, disabled: &[String]) -> bool {
    let disabled: Vec<&str> = disabled.iter().map(String::as_str).collect();

    let mut has_errors = false;

    for rule in PROJECT_RULES {
        if disabled.contains(&rule.name()) {
            continue;
        }

        for v in rule.detect(project_root) {
            has_errors = true;
            eprintln!("[{}] — {}", rule.name(), v.message);
        }
    }

    has_errors
}
