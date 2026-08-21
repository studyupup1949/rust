use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::Serialize;

mod cfg_order;
mod common;
mod item_order;
mod no_allow_unused_imports;
mod no_vague_comment;
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

#[derive(Serialize)]
pub struct FixReport {
    pub succeeded: bool,
    pub files: HashMap<String, FileFixReport>,
    pub project: ProjectFixReport,
}

#[derive(Serialize)]
pub struct FileFixReport {
    pub initial_violations: usize,
    pub final_violations: usize,
    pub remaining_violations: Vec<RemainingViolation>,
}

#[derive(Serialize)]
pub struct RemainingViolation {
    pub rule: String,
    pub line: usize,
    pub message: String,
    pub has_fixer: bool,
}

#[derive(Serialize)]
pub struct ProjectFixReport {
    pub initial_violations: usize,
    pub final_violations: usize,
    pub remaining_violations: Vec<ProjectRemainingViolation>,
}

#[derive(Serialize)]
pub struct ProjectRemainingViolation {
    pub rule: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub has_fixer: bool,
}

pub trait Rule: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

pub trait RsRule: Rule {
    /// Returns (line, message) pairs for each violation found in a parsed file.
    fn check(&self, file: &syn::File, source: &str) -> Vec<(usize, String)>;

    /// Returns true if this rule provides a meaningful `try_fix` implementation.
    fn has_fixer(&self) -> bool {
        false
    }

    /// Attempt to fix violations in `source`. Receives both the parsed AST
    /// and the original source string; returns the corrected source or an Err.
    fn try_fix(&self, source: &str, file: syn::File) -> Result<String, String>;
}

pub trait ProjectRule: Rule {
    /// Returns (file, line, message) triples for each project-level violation.
    fn check_project(&self, root: &Path) -> Vec<(PathBuf, usize, String)>;

    /// Returns true if this rule provides a meaningful `try_fix_project` implementation.
    fn has_fixer_project(&self) -> bool {
        false
    }

    /// Attempt to fix project-level violations under `root`.
    fn try_fix_project(&self, root: &Path) -> Result<(), String>;
}

static BASE_RS_RULES: &[&dyn RsRule] = &[
    &cfg_order::CfgOrder,
    &item_order::ItemOrder,
    &no_allow_unused_imports::NoAllowUnusedImports,
    &no_vague_comment::NoVagueComment,
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
        for (line, message) in rule.check(&parsed, source) {
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

/// Single-pass fix: check violations, apply fixers once, re-check, report remaining.
pub fn run_fix(
    root: &Path,
    file_paths: &[PathBuf],
    disabled: &[String],
    features: &[String],
) -> FixReport {
    let rs_rules = active_rs_rules(features);
    let proj_rules = active_project_rules(features);

    let mut files: HashMap<String, FileFixReport> = HashMap::new();

    for path in file_paths {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        let Ok(_parsed) = syn::parse_file(&source) else {
            continue;
        };
        let initial = build_rs_remaining(&source, &rs_rules, disabled);
        let initial_count = initial.len();

        if initial_count == 0 {
            files.insert(
                path.display().to_string(),
                FileFixReport {
                    initial_violations: 0,
                    final_violations: 0,
                    remaining_violations: vec![],
                },
            );
            continue;
        }

        // Apply each fixer once.
        let mut current = source;
        for rule in &rs_rules {
            if disabled.iter().any(|d| d == rule.name()) || !rule.has_fixer() {
                continue;
            }
            let Ok(parsed) = syn::parse_file(&current) else {
                break;
            };
            if let Ok(fixed) = rule.try_fix(&current, parsed) {
                current = fixed;
            }
        }

        // Write fixed source back.
        let _ = std::fs::write(path, &current);

        // Re-check.
        let remaining = build_rs_remaining(&current, &rs_rules, disabled);
        let final_count = remaining.len();
        files.insert(
            path.display().to_string(),
            FileFixReport {
                initial_violations: initial_count,
                final_violations: final_count,
                remaining_violations: remaining,
            },
        );
    }

    // Project rules.
    let proj_initial = build_proj_remaining(root, &proj_rules, disabled);
    let proj_initial_count = proj_initial.len();

    if proj_initial_count > 0 {
        for rule in &proj_rules {
            if disabled.iter().any(|d| d == rule.name()) || !rule.has_fixer_project() {
                continue;
            }
            let _ = rule.try_fix_project(root);
        }
    }

    let proj_remaining = build_proj_remaining(root, &proj_rules, disabled);
    let proj_final_count = proj_remaining.len();

    let project = ProjectFixReport {
        initial_violations: proj_initial_count,
        final_violations: proj_final_count,
        remaining_violations: proj_remaining,
    };

    let succeeded =
        files.values().all(|f| f.final_violations == 0) && project.final_violations == 0;

    FixReport {
        succeeded,
        files,
        project,
    }
}

fn build_rs_remaining(
    source: &str,
    rules: &[&'static dyn RsRule],
    disabled: &[String],
) -> Vec<RemainingViolation> {
    let Ok(parsed) = syn::parse_file(source) else {
        return vec![];
    };
    rules
        .iter()
        .filter(|r| !disabled.iter().any(|d| d == r.name()))
        .flat_map(|r| {
            r.check(&parsed, source)
                .into_iter()
                .map(|(line, message)| RemainingViolation {
                    rule: r.name().to_string(),
                    line,
                    message,
                    has_fixer: r.has_fixer(),
                })
        })
        .collect()
}

fn build_proj_remaining(
    root: &Path,
    rules: &[&'static dyn ProjectRule],
    disabled: &[String],
) -> Vec<ProjectRemainingViolation> {
    rules
        .iter()
        .filter(|r| !disabled.iter().any(|d| d == r.name()))
        .flat_map(|r| {
            r.check_project(root)
                .into_iter()
                .map(|(file, line, message)| ProjectRemainingViolation {
                    rule: r.name().to_string(),
                    file: file.display().to_string(),
                    line,
                    message,
                    has_fixer: r.has_fixer_project(),
                })
        })
        .collect()
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

    /// Every rule source file must fit within 512 lines.
    #[test]
    fn all_rules_fit_in_512_lines() {
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
                lines <= 512,
                "Rule file `{}` has {} lines, exceeds the 512-line limit",
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
