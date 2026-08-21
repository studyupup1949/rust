mod config;
mod rules;

use std::{path::Path, process};

use walkdir::WalkDir;

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

            let scan_dirs: Vec<_> = if config.scan.is_empty() {
                vec!["src".to_string()]
            } else {
                config.scan.clone()
            };

            let violations = lint_dirs(&root, &scan_dirs, &config.rules.disable);

            for v in &violations {
                eprintln!(
                    "[{}] {}:{} — {}",
                    v.rule,
                    v.file.display(),
                    v.line,
                    v.message
                );
            }

            if !violations.is_empty() {
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Usage: cargo a9-lint check");
            process::exit(1);
        }
    }
}

fn lint_dirs(root: &Path, scan_dirs: &[String], disabled: &[String]) -> Vec<rules::Violation> {
    let mut violations = vec![];

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
            let path = entry.path();
            let Ok(source) = std::fs::read_to_string(path) else {
                continue;
            };
            violations.extend(rules::run_all(path, &source, disabled));
        }
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
        && !path
            .components()
            .any(|c| c.as_os_str() == "target" || c.as_os_str() == "build")
}
