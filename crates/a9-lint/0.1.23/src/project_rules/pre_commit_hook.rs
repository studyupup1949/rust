use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

use crate::{ProjectRule, Rule as RuleTrait, Violation};

const HOOK_DIR: &str = ".githooks";
const HOOK_FILE: &str = "pre-commit";
const CANONICAL_HOOK: &str = r#"#!/usr/bin/env bash
# DO NOT EDIT — regenerate with: cargo a9-lint

if cargo fmt -- --check && cargo clippy -- -D warnings && cargo a9-lint --check; then
    exit 0
fi

git stash push --keep-index -m "pre-commit-auto-fix-backup"
STASH_CREATED=$?

cargo fmt
cargo clippy --fix --allow-staged --allow-dirty -- -D warnings || true
cargo a9-lint

git add -A

if cargo fmt -- --check && cargo clippy -- -D warnings && cargo a9-lint --check; then
    [ "$STASH_CREATED" -eq 0 ] && echo "WIP saved in stash — restore with: git stash pop"
    exit 0
fi

echo "Auto-fix incomplete. Fix remaining errors manually."
[ "$STASH_CREATED" -eq 0 ] && echo "To undo auto-fix changes: git stash pop"
exit 1
"#;

pub struct Rule;

impl RuleTrait for Rule {
    fn name(&self) -> &'static str {
        "pre-commit-hook"
    }

    fn description(&self) -> &'static str {
        ".githooks/pre-commit must match canonical smart hook (auto-fixable)"
    }
}

impl ProjectRule for Rule {
    fn detect(&self, project_root: &Path) -> Vec<Violation> {
        let mut violations = vec![];

        let git_root = find_git_root(project_root);
        let hooks_base = git_root.as_deref().unwrap_or(project_root);
        let hook_path = hooks_base.join(HOOK_DIR).join(HOOK_FILE);
        let content = fs::read_to_string(&hook_path).unwrap_or_default();

        if content != CANONICAL_HOOK {
            violations.push(Violation {
                line: 0,
                message: format!(
                    "{HOOK_DIR}/{HOOK_FILE} content mismatch — regenerate with `cargo a9-lint`"
                ),
                fixable: true,
            });
        }

        check_hooks_path(hooks_base, &mut violations);

        violations
    }

    fn fix(&self, project_root: &Path) {
        let git_root = find_git_root(project_root);
        let hooks_base = git_root.as_deref().unwrap_or(project_root);
        let hook_dir = hooks_base.join(HOOK_DIR);
        let hook_path = hook_dir.join(HOOK_FILE);
        let _ = fs::create_dir_all(&hook_dir);
        let _ = fs::write(&hook_path, CANONICAL_HOOK);
        let _ = fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755));
    }
}

fn find_git_root(start: &Path) -> Option<std::path::PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(start)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Some(std::path::PathBuf::from(path))
}

fn check_hooks_path(hooks_base: &Path, violations: &mut Vec<Violation>) {
    let output = Command::new("git")
        .args(["config", "--get", "core.hooksPath"])
        .current_dir(hooks_base)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let configured = String::from_utf8_lossy(&o.stdout).trim().to_string();

            if configured != HOOK_DIR {
                violations.push(Violation {
                    line: 0,
                    message: format!(
                        "git core.hooksPath is \"{configured}\", expected \"{HOOK_DIR}\""
                    ),
                    fixable: false,
                });
            }
        }
        _ => {
            violations.push(Violation {
                line: 0,
                message: format!(
                    "git core.hooksPath is not set; run `git config core.hooksPath {HOOK_DIR}`"
                ),
                fixable: false,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use tempfile::TempDir;

    use super::*;

    fn init_repo(dir: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "core.hooksPath", HOOK_DIR])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    fn write_hook(dir: &Path, content: &str) {
        let hook_dir = dir.join(HOOK_DIR);

        fs::create_dir_all(&hook_dir).unwrap();

        let hook_path = hook_dir.join(HOOK_FILE);

        fs::write(&hook_path, content).unwrap();
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn passes_with_canonical_hook() {
        let dir = TempDir::new().unwrap();

        init_repo(dir.path());
        write_hook(dir.path(), CANONICAL_HOOK);
        assert!(Rule.detect(dir.path()).is_empty());
    }

    #[test]
    fn flags_wrong_content() {
        let dir = TempDir::new().unwrap();

        init_repo(dir.path());
        write_hook(dir.path(), "#!/bin/bash\necho hello\n");

        let vio = Rule.detect(dir.path());

        assert_eq!(vio.len(), 1);
        assert!(vio[0].message.contains("content mismatch"));
        assert!(vio[0].fixable);
    }

    #[test]
    fn flags_missing_hook() {
        let dir = TempDir::new().unwrap();

        init_repo(dir.path());

        let vio = Rule.detect(dir.path());

        assert!(vio.iter().any(|v| v.message.contains("content mismatch")));
    }

    #[test]
    fn fix_writes_canonical_hook() {
        let dir = TempDir::new().unwrap();

        init_repo(dir.path());
        Rule.fix(dir.path());
        assert!(Rule.detect(dir.path()).is_empty());
    }

    #[test]
    fn flags_missing_hooks_path_config() {
        let dir = TempDir::new().unwrap();

        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        write_hook(dir.path(), CANONICAL_HOOK);

        let vio = Rule.detect(dir.path());

        assert!(vio.iter().any(|v| v.message.contains("core.hooksPath")));
    }

    #[test]
    fn passes_when_project_root_is_subdirectory_of_git_root() {
        let dir = TempDir::new().unwrap();

        init_repo(dir.path());
        write_hook(dir.path(), CANONICAL_HOOK);

        let subdir = dir.path().join("rust-workspace");

        fs::create_dir_all(&subdir).unwrap();
        assert!(Rule.detect(&subdir).is_empty());
    }
}
