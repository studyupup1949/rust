use std::path::{Path, PathBuf};

use super::{ProjectRule, Rule};

pub struct PreCommitHook;

impl Rule for PreCommitHook {
    fn name(&self) -> &'static str {
        "pre-commit-hook"
    }

    fn description(&self) -> &'static str {
        "pre-commit hook must exist with fmt --check, clippy -D warnings, and a9-lint check"
    }
}

impl ProjectRule for PreCommitHook {
    fn check_project(&self, root: &Path) -> Vec<(PathBuf, usize, String)> {
        let hook = root.join(".githooks").join("pre-commit");

        if !hook.exists() {
            return vec![(
                root.join(".githooks"),
                1,
                "missing .githooks/pre-commit — create a pre-commit hook with fmt, clippy, and a9-lint checks".to_string(),
            )];
        }

        let Ok(content) = std::fs::read_to_string(&hook) else {
            return vec![];
        };

        let mut violations = vec![];

        if !content
            .lines()
            .any(|l| l.contains("fmt") && l.contains("--check"))
        {
            violations.push((
                hook.clone(),
                1,
                "pre-commit hook must run `cargo fmt -- --check`".to_string(),
            ));
        }

        if !content
            .lines()
            .any(|l| l.contains("cargo clippy") && l.contains("-D warnings"))
        {
            violations.push((
                hook.clone(),
                1,
                "pre-commit hook must run `cargo clippy -- -D warnings`".to_string(),
            ));
        }

        let has_a9_lint_check = content.lines().any(|l| {
            (l.contains("cargo a9-lint") && l.contains("check"))
                || (l.contains("cargo run") && l.contains("check"))
        });
        if !has_a9_lint_check {
            violations.push((
                hook.clone(),
                1,
                "pre-commit hook must run an a9-lint check (`cargo a9-lint check` or `cargo run -- check`)".to_string(),
            ));
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use tempfile::TempDir;

    use super::*;

    fn check_project(hook_content: Option<&str>) -> Vec<(PathBuf, usize, String)> {
        let dir = TempDir::new().unwrap();
        let hooks_dir = dir.path().join(".githooks");
        fs::create_dir_all(&hooks_dir).unwrap();
        if let Some(content) = hook_content {
            let mut f = fs::File::create(hooks_dir.join("pre-commit")).unwrap();
            write!(f, "{content}").unwrap();
        }
        PreCommitHook.check_project(dir.path())
    }

    #[test]
    fn missing_hook_is_violation() {
        // No hook created — just the .githooks dir
        let dir = tempfile::TempDir::new().unwrap();
        let v = PreCommitHook.check_project(dir.path());
        assert_eq!(v.len(), 1);
        assert!(v[0].2.contains("missing"));
    }

    #[test]
    fn complete_hook_no_violations() {
        let content = "#!/usr/bin/env bash\ncargo fmt -- --check\ncargo clippy -- -D warnings\ncargo a9-lint check";
        assert!(check_project(Some(content)).is_empty());
    }

    #[test]
    fn complete_hook_with_run_no_violations() {
        let content = "#!/usr/bin/env bash\ncargo fmt -- --check\ncargo clippy -- -D warnings\ncargo run --quiet -- check";
        assert!(check_project(Some(content)).is_empty());
    }

    #[test]
    fn missing_fmt_check_is_violation() {
        let content = "cargo clippy -- -D warnings\ncargo a9-lint check";
        let v = check_project(Some(content));
        assert!(v.iter().any(|(_, _, m)| m.contains("fmt")));
    }

    #[test]
    fn missing_clippy_d_warnings_is_violation() {
        let content = "cargo fmt -- --check\ncargo clippy\ncargo a9-lint check";
        let v = check_project(Some(content));
        assert!(v.iter().any(|(_, _, m)| m.contains("clippy")));
    }

    #[test]
    fn missing_a9_lint_check_is_violation() {
        let content = "cargo fmt -- --check\ncargo clippy -- -D warnings";
        let v = check_project(Some(content));
        assert!(v.iter().any(|(_, _, m)| m.contains("a9-lint")));
    }
}
