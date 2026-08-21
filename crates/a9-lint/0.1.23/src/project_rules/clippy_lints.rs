use std::{fs, path::Path};

use crate::{ProjectRule, Rule as RuleTrait, Violation};

pub struct Rule;

impl RuleTrait for Rule {
    fn name(&self) -> &'static str {
        "clippy-lints"
    }

    fn description(&self) -> &'static str {
        "[lints.clippy] exact: all=deny, pedantic/nursery=warn + 2 allow exceptions"
    }
}

impl ProjectRule for Rule {
    fn detect(&self, project_root: &Path) -> Vec<Violation> {
        let cargo_path = project_root.join("Cargo.toml");

        let Ok(content) = fs::read_to_string(&cargo_path) else {
            return vec![Violation {
                line: 0,
                message: "Cargo.toml not found".into(),
                fixable: false,
            }];
        };

        let Ok(cargo_toml) = content.parse::<toml::Table>() else {
            return vec![Violation {
                line: 0,
                message: "Cargo.toml is not valid TOML".into(),
                fixable: false,
            }];
        };

        let clippy = find_clippy_table(&cargo_toml);

        let Some(clippy) = clippy else {
            return vec![Violation {
                line: 0,
                message: "missing [lints.clippy] or [workspace.lints.clippy] section".into(),
                fixable: false,
            }];
        };

        check_clippy_table(clippy)
    }
}

fn find_clippy_table(cargo_toml: &toml::Table) -> Option<&toml::Table> {
    cargo_toml
        .get("lints")
        .and_then(toml::Value::as_table)
        .and_then(|t| t.get("clippy"))
        .and_then(toml::Value::as_table)
        .or_else(|| {
            cargo_toml
                .get("workspace")
                .and_then(toml::Value::as_table)
                .and_then(|t| t.get("lints"))
                .and_then(toml::Value::as_table)
                .and_then(|t| t.get("clippy"))
                .and_then(toml::Value::as_table)
        })
}

fn check_clippy_table(clippy: &toml::Table) -> Vec<Violation> {
    let expected_groups = [("all", "deny"), ("pedantic", "warn"), ("nursery", "warn")];
    let expected_individual = [
        ("option_if_let_else", "allow"),
        ("must_use_candidate", "allow"),
    ];

    let mut violations = vec![];

    for (key, expected_level) in &expected_groups {
        check_group(clippy, key, expected_level, &mut violations);
    }

    for (lint, expected) in expected_individual {
        check_individual(clippy, lint, expected, &mut violations);
    }

    for (key, _val) in clippy {
        let is_group = expected_groups.iter().any(|(k, _)| k == &key.as_str());
        let is_individual = expected_individual.iter().any(|(k, _)| k == &key.as_str());

        if is_group || is_individual {
            continue;
        }

        violations.push(Violation {
            line: 0,
            message: format!("[lints.clippy] unexpected key \"{key}\""),
            fixable: false,
        });
    }

    violations
}

fn check_group(
    clippy: &toml::Table,
    key: &str,
    expected_level: &str,
    violations: &mut Vec<Violation>,
) {
    match clippy.get(key) {
        Some(toml::Value::Table(t)) => {
            let level = t.get("level").and_then(toml::Value::as_str);
            let priority = t.get("priority").and_then(toml::Value::as_integer);

            if level != Some(expected_level) {
                violations
                    .push(Violation {
                        line: 0,
                        message: format!(
                            "[lints.clippy] {key} level = \"{level:?}\" but must be \"{expected_level}\""
                        ),
                        fixable: false,
                    });
            }

            if priority != Some(-1) {
                violations.push(Violation {
                    line: 0,
                    message: format!("[lints.clippy] {key} priority = {priority:?} but must be -1"),
                    fixable: false,
                });
            }
        }
        Some(toml::Value::String(s)) => {
            violations
                .push(Violation {
                    line: 0,
                    message: format!(
                        "[lints.clippy] {key} must be {{ level = \"{expected_level}\", priority = -1 }}, not \"{s}\""
                    ),
                    fixable: false,
                });
        }
        Some(_) => {
            violations.push(Violation {
                line: 0,
                message: format!("[lints.clippy] {key} has invalid type"),
                fixable: false,
            });
        }
        None => {
            violations
                .push(Violation {
                    line: 0,
                    message: format!(
                        "[lints.clippy] missing {key} = {{ level = \"{expected_level}\", priority = -1 }}"
                    ),
                    fixable: false,
                });
        }
    }
}

fn check_individual(
    clippy: &toml::Table,
    lint: &str,
    expected: &str,
    violations: &mut Vec<Violation>,
) {
    match clippy.get(lint) {
        Some(val) => {
            let actual = val.as_str().unwrap_or("");

            if actual != expected {
                violations.push(Violation {
                    line: 0,
                    message: format!(
                        "[lints.clippy] {lint} = \"{actual}\" but must be \"{expected}\""
                    ),
                    fixable: false,
                });
            }
        }
        None => {
            violations.push(Violation {
                line: 0,
                message: format!("[lints.clippy] missing {lint} = \"{expected}\""),
                fixable: false,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn write_cargo(dir: &Path, content: &str) {
        fs::write(dir.join("Cargo.toml"), content).unwrap();
    }

    #[test]
    fn passes_with_correct_config() {
        let dir = TempDir::new().unwrap();

        write_cargo(
            dir.path(),
            r#"
[lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
option_if_let_else = "allow"
must_use_candidate = "allow"
"#,
        );
        assert!(Rule.detect(dir.path()).is_empty());
    }

    #[test]
    fn passes_with_own_cargo_toml() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let violations = Rule.detect(&manifest_dir);

        assert!(
            violations.is_empty(),
            "a9-lint's own Cargo.toml should pass: {violations:#?}"
        );
    }

    #[test]
    fn passes_with_workspace_lints_section() {
        let dir = TempDir::new().unwrap();

        write_cargo(
            dir.path(),
            r#"
[workspace]
members = ["crates/*"]

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
option_if_let_else = "allow"
must_use_candidate = "allow"
"#,
        );
        assert!(Rule.detect(dir.path()).is_empty());
    }

    #[test]
    fn flags_missing_section_simple_crate() {
        let dir = TempDir::new().unwrap();

        write_cargo(dir.path(), "[package]\nname = \"test\"\n");

        let vio = Rule.detect(dir.path());

        assert_eq!(vio.len(), 1);
        assert!(vio[0].message.contains("missing [lints.clippy]"));
    }

    #[test]
    fn flags_missing_section_workspace() {
        let dir = TempDir::new().unwrap();

        write_cargo(
            dir.path(),
            r#"
[workspace]
members = ["crates/*"]
"#,
        );

        let vio = Rule.detect(dir.path());

        assert_eq!(vio.len(), 1);
        assert!(vio[0].message.contains("missing [lints.clippy]"));
    }

    #[test]
    fn rejects_additional_key_in_workspace_lints() {
        let dir = TempDir::new().unwrap();

        write_cargo(
            dir.path(),
            r#"
[workspace]
members = ["crates/*"]

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
option_if_let_else = "allow"
must_use_candidate = "allow"
extra_lint = "allow"
"#,
        );

        let vio = Rule.detect(dir.path());

        assert!(
            vio.iter()
                .any(|v| v.message.contains("unexpected key") && v.message.contains("extra_lint"))
        );
    }

    #[test]
    fn flags_missing_priority() {
        let dir = TempDir::new().unwrap();

        write_cargo(
            dir.path(),
            r#"
[lints.clippy]
all = "deny"
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
option_if_let_else = "allow"
must_use_candidate = "allow"
"#,
        );

        let vio = Rule.detect(dir.path());

        assert!(
            vio.iter()
                .any(|v| v.message.contains("all") && v.message.contains("priority"))
        );
    }

    #[test]
    fn flags_wrong_level() {
        let dir = TempDir::new().unwrap();

        write_cargo(
            dir.path(),
            r#"
[lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
option_if_let_else = "allow"
must_use_candidate = "allow"
"#,
        );

        let vio = Rule.detect(dir.path());

        assert!(
            vio.iter()
                .any(|v| v.message.contains("all") && v.message.contains("level"))
        );
    }

    #[test]
    fn flags_extra_unexpected_keys() {
        let dir = TempDir::new().unwrap();

        write_cargo(
            dir.path(),
            r#"
[lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
option_if_let_else = "allow"
must_use_candidate = "allow"
some_random_lint = "warn"
"#,
        );

        let vio = Rule.detect(dir.path());

        assert!(vio.iter().any(
            |v| v.message.contains("unexpected key") && v.message.contains("some_random_lint")
        ));
    }

    #[test]
    fn flags_missing_allow_exceptions() {
        let dir = TempDir::new().unwrap();

        write_cargo(
            dir.path(),
            r#"
[lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
"#,
        );

        let vio = Rule.detect(dir.path());

        assert!(vio.iter().any(|v| v.message.contains("option_if_let_else")));
        assert!(vio.iter().any(|v| v.message.contains("must_use_candidate")));
    }
}
