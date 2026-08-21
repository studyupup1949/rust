use std::path::{Path, PathBuf};

use super::{ProjectRule, Rule};

pub struct PreCommitHook;

impl Rule for PreCommitHook {
    fn name(&self) -> &'static str {
        "pre-commit-hook"
    }

    fn description(&self) -> &'static str {
        "pre-commit hook must check fmt, clippy, test, and a9-lint"
    }
}

fn has_fmt_check(content: &str) -> bool {
    content.lines().any(|l| {
        l.contains("cargo fmt") && l.contains("--check") && !l.trim_start().starts_with('#')
    })
}

fn has_clippy_check(content: &str) -> bool {
    content.lines().any(|l| {
        l.contains("cargo clippy") && l.contains("-D warnings") && !l.trim_start().starts_with('#')
    })
}

fn has_test(content: &str) -> bool {
    content
        .lines()
        .any(|l| l.contains("cargo test") && !l.trim_start().starts_with('#'))
}

fn has_a9_lint(content: &str) -> bool {
    content.lines().any(|l| {
        !l.trim_start().starts_with('#') && (l.contains("cargo a9-lint") || l.contains("cargo run"))
    })
}

impl ProjectRule for PreCommitHook {
    fn check_project(&self, root: &Path) -> Vec<(PathBuf, usize, String)> {
        let hook = root.join(".githooks").join("pre-commit");

        if !hook.exists() {
            return vec![(
                root.join(".githooks"),
                1,
                "missing .githooks/pre-commit".to_string(),
            )];
        }

        let Ok(content) = std::fs::read_to_string(&hook) else {
            return vec![];
        };

        let mut violations = vec![];
        if !has_fmt_check(&content) {
            violations.push((
                hook.clone(),
                1,
                "hook must run `cargo fmt -- --check`".into(),
            ));
        }
        if !has_clippy_check(&content) {
            violations.push((
                hook.clone(),
                1,
                "hook must run `cargo clippy -- -D warnings`".into(),
            ));
        }
        if !has_test(&content) {
            violations.push((hook.clone(), 1, "hook must run `cargo test`".into()));
        }
        if !has_a9_lint(&content) {
            violations.push((
                hook.clone(),
                1,
                "hook must run a9-lint; run `cargo a9-lint fix` to auto-fix".into(),
            ));
        }
        violations
    }

    fn has_fixer_project(&self) -> bool {
        false
    }

    fn try_fix_project(&self, _root: &Path) -> Result<(), String> {
        Err("pre-commit hook must be configured manually per project".into())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use fs::File;

    use tempfile::TempDir;

    use super::*;

    fn check_project(hook_content: Option<&str>) -> Vec<(PathBuf, usize, String)> {
        let dir = TempDir::new().unwrap();
        let hooks_dir = dir.path().join(".githooks");
        fs::create_dir_all(&hooks_dir).unwrap();
        if let Some(content) = hook_content {
            let mut f = File::create(hooks_dir.join("pre-commit")).unwrap();
            write!(f, "{content}").unwrap();
        }
        PreCommitHook.check_project(dir.path())
    }

    #[test]
    fn missing_hook_is_violation() {
        let dir = TempDir::new().unwrap();
        let v = PreCommitHook.check_project(dir.path());
        assert_eq!(v.len(), 1);
        assert!(v[0].2.contains("missing"));
    }

    #[test]
    fn complete_hook_no_violations() {
        let content = "\
#!/usr/bin/env bash
set -euo pipefail
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test --quiet
cargo a9-lint check
";
        assert!(check_project(Some(content)).is_empty());
    }

    #[test]
    fn missing_fmt_check_is_violation() {
        let content = "\
cargo clippy -- -D warnings
cargo test
cargo a9-lint check
";
        let v = check_project(Some(content));
        assert!(v.iter().any(|(_, _, m)| m.contains("fmt -- --check")));
    }

    #[test]
    fn missing_clippy_check_is_violation() {
        let content = "\
cargo fmt -- --check
cargo test
cargo a9-lint check
";
        let v = check_project(Some(content));
        assert!(v.iter().any(|(_, _, m)| m.contains("-D warnings")));
    }

    #[test]
    fn missing_test_is_violation() {
        let content = "\
cargo fmt -- --check
cargo clippy -- -D warnings
cargo a9-lint check
";
        let v = check_project(Some(content));
        assert!(v.iter().any(|(_, _, m)| m.contains("cargo test")));
    }

    #[test]
    fn missing_a9_lint_is_violation() {
        let content = "\
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
";
        let v = check_project(Some(content));
        assert!(v.iter().any(|(_, _, m)| m.contains("a9-lint")));
    }
}
