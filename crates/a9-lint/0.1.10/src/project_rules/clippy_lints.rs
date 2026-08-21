use std::{fs, path::Path};

use crate::{ProjectRule as ProjectRuleTrait, Rule as RuleTrait, Violation};

pub struct Rule;

impl RuleTrait for Rule {
    fn name(&self) -> &'static str {
        "clippy-lints"
    }

    fn description(&self) -> &'static str {
        "Cargo.toml must enable clippy all, pedantic, and nursery lint groups as warnings"
    }
}

impl ProjectRuleTrait for Rule {
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

        let mut violations = vec![];

        let clippy = cargo_toml
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
            });

        let Some(clippy) = clippy else {
            violations.push(Violation {
                line: 0,
                message: "missing [lints.clippy] section".into(),
                fixable: false,
            });

            return violations;
        };

        for group in ["all", "pedantic", "nursery"] {
            match clippy.get(group) {
                Some(val) => {
                    let ok = match val {
                        toml::Value::Table(t) => {
                            t.get("level").and_then(toml::Value::as_str) == Some("warn")
                                && t.get("priority")
                                    .and_then(toml::Value::as_integer)
                                    .is_some_and(|p| p < 0)
                        }
                        toml::Value::String(s) => s == "warn",
                        _ => false,
                    };

                    if !ok {
                        violations
                            .push(Violation {
                                line: 0,
                                message: format!(
                                    "[lints.clippy] {group} must be {{ level = \"warn\", priority = -1 }}"
                                ),
                                fixable: false,
                            });
                    }
                }
                None => {
                    violations.push(Violation {
                        line: 0,
                        message: format!(
                            "[lints.clippy] missing {group} = {{ level = \"warn\", priority = -1 }}"
                        ),
                        fixable: false,
                    });
                }
            }
        }

        for (lint, expected) in [
            ("option_if_let_else", "allow"),
            ("must_use_candidate", "allow"),
        ] {
            match clippy.get(lint) {
                Some(val) => {
                    let actual = val.as_str().unwrap_or("");

                    if actual != expected {
                        violations.push(Violation {
                            line: 0,
                            message: format!(
                                "[lints.clippy] {lint} must be \"{expected}\", got \"{actual}\""
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

        violations
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
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
option_if_let_else = "allow"
must_use_candidate = "allow"
"#,
        );
        assert!(Rule.detect(dir.path()).is_empty());
    }

    #[test]
    fn flags_missing_section() {
        let dir = TempDir::new().unwrap();

        write_cargo(dir.path(), "[package]\nname = \"test\"\n");

        let vio = Rule.detect(dir.path());

        assert_eq!(vio.len(), 1);
        assert!(vio[0].message.contains("missing [lints.clippy] section"));
    }

    #[test]
    fn flags_missing_group() {
        let dir = TempDir::new().unwrap();

        write_cargo(
            dir.path(),
            r#"
[lints.clippy]
all = { level = "warn", priority = -1 }
option_if_let_else = "allow"
must_use_candidate = "allow"
"#,
        );

        let vio = Rule.detect(dir.path());

        assert_eq!(vio.len(), 2);
        assert!(vio[0].message.contains("pedantic"));
        assert!(vio[1].message.contains("nursery"));
    }

    #[test]
    fn flags_wrong_level() {
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

        let vio = Rule.detect(dir.path());

        assert_eq!(vio.len(), 1);
        assert!(vio[0].message.contains("all must be"));
    }

    #[test]
    fn flags_missing_allow_overrides() {
        let dir = TempDir::new().unwrap();

        write_cargo(
            dir.path(),
            r#"
[lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }
"#,
        );

        let vio = Rule.detect(dir.path());

        assert_eq!(vio.len(), 2);
        assert!(vio[0].message.contains("option_if_let_else"));
        assert!(vio[1].message.contains("must_use_candidate"));
    }
}
