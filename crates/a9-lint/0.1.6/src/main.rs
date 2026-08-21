use std::{collections::BTreeMap, path::Path, process};

use walkdir::WalkDir;

mod config;
mod rules;

fn main() {
    // Support both direct invocation (`cargo-a9-lint check`) and
    // cargo subcommand invocation (`cargo a9-lint check`), where
    // cargo injects the subcommand name as the first argument.
    let mut args = std::env::args().skip(1);
    let first = args.next();
    let cmd = if first.as_deref() == Some("a9-lint") {
        args.next()
    } else {
        first
    };

    match cmd.as_deref() {
        Some("check") | Some("lint") => {
            let cwd = std::env::current_dir().expect("failed to get cwd");
            let (config, root) = config::find_config(&cwd);

            let scan_dirs = effective_scan_dirs(&config);

            let violations = {
                let mut v = lint_dirs(&root, &scan_dirs, &config.rules.disable, &config.features);
                v.extend(rules::run_project(
                    &root,
                    &config.rules.disable,
                    &config.features,
                ));
                v
            };

            // Group violations by rule; each rule's description prints once as a header.
            let mut by_rule: BTreeMap<&str, (&str, Vec<&rules::Violation>)> = BTreeMap::new();
            for v in &violations {
                by_rule
                    .entry(v.rule)
                    .or_insert_with(|| (v.description, vec![]))
                    .1
                    .push(v);
            }
            let mut first = true;
            for (rule, (description, group)) in &by_rule {
                if !first {
                    eprintln!();
                }
                first = false;
                eprintln!("[{rule}] {description}");
                for v in group {
                    eprintln!("  {}:{} — {}", v.file.display(), v.line, v.message);
                }
            }

            if !violations.is_empty() {
                process::exit(1);
            }
        }
        None | Some("fix") => {
            let cwd = std::env::current_dir().expect("failed to get cwd");
            let (config, root) = config::find_config(&cwd);

            let scan_dirs = effective_scan_dirs(&config);
            let file_paths = collect_rs_files(&root, &scan_dirs);

            let report =
                rules::run_fix(&root, &file_paths, &config.rules.disable, &config.features);

            if report.succeeded {
                eprintln!("a9-lint fix: all violations fixed");
            } else {
                eprintln!("a9-lint fix: some violations remain");
                let json =
                    serde_json::to_string_pretty(&report).expect("failed to serialize report");
                let report_path = root.join("a9-lint-fix-report.json");
                std::fs::write(&report_path, json).expect("failed to write report");
                eprintln!("  report written to {}", report_path.display());
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Usage: cargo a9-lint [check|fix]");
            process::exit(1);
        }
    }
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
            .filter_map(|e| e.ok())
            .filter(|e| is_rust_source(e.path()))
        {
            paths.push(entry.path().to_path_buf());
        }
    }
    paths
}

fn lint_dirs(
    root: &Path,
    scan_dirs: &[String],
    disabled: &[String],
    features: &[String],
) -> Vec<rules::Violation> {
    let mut violations = vec![];

    for path in collect_rs_files(root, scan_dirs) {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        violations.extend(rules::run_all(&path, &source, disabled, features));
    }

    violations.sort_by(|a, b| {
        a.file
            .cmp(&b.file)
            .then(a.line.cmp(&b.line))
            .then(a.rule.cmp(b.rule))
    });

    violations
}

fn is_rust_source(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("rs")
}
