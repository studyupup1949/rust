use std::{fs, path::Path, process::Command};

use crate::{ProjectRule as ProjectRuleTrait, Rule as RuleTrait, Violation};

const HOOK_DIR: &str = ".githooks";
const HOOK_FILE: &str = "pre-commit";

pub struct Rule;

impl RuleTrait for Rule {
    fn name(&self) -> &'static str {
        "pre-commit-hook"
    }

    fn description(&self) -> &'static str {
        ".githooks/pre-commit must exist with fmt, clippy, and a9-lint checks, and core.hooksPath must be set"
    }
}

impl ProjectRuleTrait for Rule {
    fn detect(&self, project_root: &Path) -> Vec<Violation> {
        let mut violations = vec![];

        let hook_path = project_root.join(HOOK_DIR).join(HOOK_FILE);

        let Ok(content) = fs::read_to_string(&hook_path) else {
            violations.push(Violation {
                line: 0,
                message: format!("{HOOK_DIR}/{HOOK_FILE} not found"),
                fixable: false,
            });

            return violations;
        };

        if !content.contains("cargo fmt") {
            violations.push(Violation {
                line: 0,
                message: format!("{HOOK_DIR}/{HOOK_FILE} must run `cargo fmt`"),
                fixable: false,
            });
        }

        if !content.contains("cargo clippy") {
            violations.push(Violation {
                line: 0,
                message: format!("{HOOK_DIR}/{HOOK_FILE} must run `cargo clippy`"),
                fixable: false,
            });
        }

        if !content.contains("a9-lint") {
            violations.push(Violation {
                line: 0,
                message: format!("{HOOK_DIR}/{HOOK_FILE} must run a9-lint check"),
                fixable: false,
            });
        }

        check_hooks_path(project_root, &mut violations);

        violations
    }
}

fn check_hooks_path(project_root: &Path, violations: &mut Vec<Violation>) {
    let output = Command::new("git")
        .args(["config", "--get", "core.hooksPath"])
        .current_dir(project_root)
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
    fn passes_with_correct_hook() {
        let dir = TempDir::new().unwrap();

        init_repo(dir.path());
        write_hook(
            dir.path(),
            "#!/bin/bash\ncargo fmt -- --check\ncargo clippy -- -D warnings\ncargo a9-lint --check\n",
        );
        assert!(Rule.detect(dir.path()).is_empty());
    }

    #[test]
    fn flags_missing_hook() {
        let dir = TempDir::new().unwrap();

        init_repo(dir.path());

        let vio = Rule.detect(dir.path());

        assert!(vio.iter().any(|v| v.message.contains("not found")));
    }

    #[test]
    fn flags_missing_fmt() {
        let dir = TempDir::new().unwrap();

        init_repo(dir.path());
        write_hook(
            dir.path(),
            "#!/bin/bash\ncargo clippy\ncargo a9-lint --check\n",
        );

        let vio = Rule.detect(dir.path());

        assert!(vio.iter().any(|v| v.message.contains("cargo fmt")));
    }

    #[test]
    fn flags_missing_clippy() {
        let dir = TempDir::new().unwrap();

        init_repo(dir.path());
        write_hook(
            dir.path(),
            "#!/bin/bash\ncargo fmt -- --check\ncargo a9-lint --check\n",
        );

        let vio = Rule.detect(dir.path());

        assert!(vio.iter().any(|v| v.message.contains("cargo clippy")));
    }

    #[test]
    fn flags_missing_a9_lint() {
        let dir = TempDir::new().unwrap();

        init_repo(dir.path());
        write_hook(
            dir.path(),
            "#!/bin/bash\ncargo fmt -- --check\ncargo clippy\n",
        );

        let vio = Rule.detect(dir.path());

        assert!(vio.iter().any(|v| v.message.contains("a9-lint")));
    }

    #[test]
    fn flags_missing_hooks_path_config() {
        let dir = TempDir::new().unwrap();

        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        write_hook(
            dir.path(),
            "#!/bin/bash\ncargo fmt -- --check\ncargo clippy\ncargo a9-lint --check\n",
        );

        let vio = Rule.detect(dir.path());

        assert!(vio.iter().any(|v| v.message.contains("core.hooksPath")));
    }
}
