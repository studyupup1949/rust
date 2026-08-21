use std::path::Path;

mod cfg_order;
mod common;
mod item_order;
mod no_allow_unused_imports;
mod normalized_use_stmt;
mod path_depth;
mod use_group_order;
mod use_toplevel;

pub struct Violation {
    pub rule: &'static str,
    pub description: &'static str,
    pub file: std::path::PathBuf,
    pub line: usize,
    pub message: String,
}

pub trait Rule: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    /// Returns (line, message) pairs for each violation found.
    fn check(&self, file: &syn::File) -> Vec<(usize, String)>;
}

static RULES: &[&dyn Rule] = &[
    &cfg_order::CfgOrder,
    &item_order::ItemOrder,
    &no_allow_unused_imports::NoAllowUnusedImports,
    &normalized_use_stmt::NormalizedUseStmt,
    &path_depth::PathDepth,
    &use_group_order::UseGroupOrder,
    &use_toplevel::UseToplevel,
];

pub fn run_all(file_path: &Path, source: &str, disabled: &[String]) -> Vec<Violation> {
    let Ok(parsed) = syn::parse_file(source) else {
        return vec![];
    };

    let mut violations = vec![];
    for rule in RULES {
        if disabled.iter().any(|d| d == rule.name()) {
            continue;
        }
        for (line, message) in rule.check(&parsed) {
            violations.push(Violation {
                rule: rule.name(),
                description: rule.description(),
                file: file_path.to_path_buf(),
                line,
                message,
            });
        }
    }
    violations
}

#[cfg(test)]
mod meta_tests {
    use std::fs;

    use super::RULES;

    /// Every rule registered in RULES must have a corresponding entry in README.md,
    /// and the README description must exactly match the rule's description().
    /// This is a private meta check — it does not ship in the published binary.
    #[test]
    fn all_rules_documented_in_readme() {
        let readme = include_str!("../../README.md");
        for rule in RULES {
            assert!(
                readme.contains(rule.name()),
                "README.md is missing an entry for rule `{}`",
                rule.name()
            );
            assert!(
                readme.contains(rule.description()),
                "README.md description for rule `{}` does not match description()",
                rule.name()
            );
        }
    }

    /// Every rule description must be a single line and at most 128 characters.
    #[test]
    fn all_rule_descriptions_are_short_one_liners() {
        for rule in RULES {
            let desc = rule.description();
            assert!(
                !desc.contains('\n'),
                "Rule `{}` description must be a single line",
                rule.name()
            );
            assert!(
                desc.len() <= 128,
                "Rule `{}` description is {} chars, exceeds the 128-char limit",
                rule.name(),
                desc.len()
            );
        }
    }

    /// Every rule source file must fit within 256 lines.
    #[test]
    fn all_rules_fit_in_256_lines() {
        let rules_dir = "src/rules";
        let entries = fs::read_dir(rules_dir).expect("Failed to read src/rules directory");
        for entry in entries {
            let path = entry.expect("Failed to read directory entry").path();
            if !path.is_file() {
                continue;
            }
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            // Skip non-rule files
            if file_name == "mod.rs" || file_name == "common.rs" {
                continue;
            }
            if !file_name.ends_with(".rs") {
                continue;
            }
            let source = fs::read_to_string(&path).expect(&format!("Failed to read {}", file_name));
            let lines = source.lines().count();
            assert!(
                lines <= 256,
                "Rule file `{}` has {} lines, exceeds the 256-line limit",
                file_name,
                lines
            );
        }
    }
}
