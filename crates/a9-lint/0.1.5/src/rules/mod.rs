use std::path::{Path, PathBuf};

mod cfg_order;
mod common;
mod item_order;
mod no_allow_unused_imports;
mod normalized_use_stmt;
mod path_depth;
mod pre_commit_hook;
mod theta_actor_fields_gated;
mod theta_actor_private_gate;
mod theta_actors_at_bottom;
mod theta_no_private_type_leak;
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
}

pub trait RsRule: Rule {
    /// Returns (line, message) pairs for each violation found in a parsed file.
    fn check(&self, file: &syn::File) -> Vec<(usize, String)>;
}

pub trait ProjectRule: Rule {
    /// Returns (file, line, message) triples for each project-level violation.
    fn check_project(&self, root: &Path) -> Vec<(PathBuf, usize, String)>;
}

static BASE_RS_RULES: &[&dyn RsRule] = &[
    &cfg_order::CfgOrder,
    &item_order::ItemOrder,
    &no_allow_unused_imports::NoAllowUnusedImports,
    &normalized_use_stmt::NormalizedUseStmt,
    &path_depth::PathDepth,
    &use_group_order::UseGroupOrder,
    &use_toplevel::UseToplevel,
];

static BASE_PROJECT_RULES: &[&dyn ProjectRule] = &[&pre_commit_hook::PreCommitHook];

static THETA_RS_RULES: &[&dyn RsRule] = &[
    &theta_actors_at_bottom::ActorsAtBottom,
    &theta_actor_fields_gated::ActorFieldsGated,
    &theta_actor_private_gate::ActorPrivateGate,
    &theta_no_private_type_leak::NoPrivateTypeLeak,
];

static THETA_PROJECT_RULES: &[&dyn ProjectRule] = &[];

fn active_rs_rules(features: &[String]) -> Vec<&'static dyn RsRule> {
    let mut rules: Vec<&'static dyn RsRule> = BASE_RS_RULES.to_vec();
    if features.iter().any(|f| f == "theta") {
        rules.extend_from_slice(THETA_RS_RULES);
    }
    rules
}

fn active_project_rules(features: &[String]) -> Vec<&'static dyn ProjectRule> {
    let mut rules: Vec<&'static dyn ProjectRule> = BASE_PROJECT_RULES.to_vec();
    if features.iter().any(|f| f == "theta") {
        rules.extend_from_slice(THETA_PROJECT_RULES);
    }
    rules
}

pub fn run_all(
    file_path: &Path,
    source: &str,
    disabled: &[String],
    features: &[String],
) -> Vec<Violation> {
    let Ok(parsed) = syn::parse_file(source) else {
        return vec![];
    };

    let mut violations = vec![];
    for rule in active_rs_rules(features) {
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

pub fn run_project(root: &Path, disabled: &[String], features: &[String]) -> Vec<Violation> {
    let mut violations = vec![];
    for rule in active_project_rules(features) {
        if disabled.iter().any(|d| d == rule.name()) {
            continue;
        }
        for (file, line, message) in rule.check_project(root) {
            violations.push(Violation {
                rule: rule.name(),
                description: rule.description(),
                file,
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

    use super::{active_project_rules, active_rs_rules};

    fn all_rules() -> Vec<&'static dyn super::Rule> {
        let features = vec!["theta".to_string()];
        let mut rules: Vec<&'static dyn super::Rule> = active_rs_rules(&features)
            .into_iter()
            .map(|r| r as &dyn super::Rule)
            .collect();
        rules.extend(
            active_project_rules(&features)
                .into_iter()
                .map(|r| r as &dyn super::Rule),
        );
        rules
    }

    /// Every rule registered must have a corresponding entry in README.md,
    /// and the README description must exactly match the rule's description().
    /// This is a private meta check — it does not ship in the published binary.
    #[test]
    fn all_rules_documented_in_readme() {
        let readme = include_str!("../../README.md");
        for rule in all_rules() {
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
        for rule in all_rules() {
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

    /// Every rule file name must match the rule's name() with hyphens replaced by underscores.
    #[test]
    fn all_rule_file_names_match_rule_names() {
        for rule in all_rules() {
            let expected_stem = rule.name().replace('-', "_");
            let expected_file = format!("{expected_stem}.rs");
            let path = std::path::Path::new("src/rules").join(&expected_file);
            assert!(
                path.exists(),
                "Rule `{}` expects file `src/rules/{}` but it does not exist",
                rule.name(),
                expected_file,
            );
        }
    }
}
