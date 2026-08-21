use std::path::Path;

use a9_prettyplease::unparse;
use rules::no_comment::check_source;
use syn::{File, parse_file};

pub mod project_rules;
pub mod rules;
pub mod workspace;

const MAX_ENFORCE_ITERATIONS: usize = 10;

/// Base trait for all rules.
pub trait Rule: Send + Sync {
    fn name(&self) -> &'static str;

    fn description(&self) -> &'static str;
}

/// A lint rule operating on a single file's AST.
///
/// Rules provide two primitives:
/// - `detect`: identify ALL violations (both fixable and unfixable).
///   This is the single source of truth for what constitutes a violation.
/// - `fix`: transform the AST to resolve fixable violations.
///
/// The pipeline composes these into an enforcement loop:
///   `detect → fix → detect → …` until no fixable violations remain.
/// After all rules reach fixpoint, `detect` runs one final time:
///   - remaining fixable violations → BUG (inter-rule conflict or fix/detect divergence)
///   - remaining unfixable violations → reported as `LintError`
pub trait UnitRule: Rule {
    /// Detect all violations in the AST. Required.
    fn detect(&self, ast: &File) -> Vec<Violation>;

    /// Fix fixable violations. Default: no-op (for check-only rules).
    fn fix(&self, ast: File) -> File {
        ast
    }
}

/// A lint rule operating on the project root (Cargo.toml, hooks, etc.).
pub trait ProjectRule: Rule {
    fn detect(&self, project_root: &Path) -> Vec<Violation>;

    fn fix(&self, project_root: &Path) {
        let _ = project_root;
    }
}

/// A single lint error produced by a rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LintError {
    ParseError(String),
    RuleError {
        rule: &'static str,
        line: usize,
        message: String,
    },
}

/// A violation detected by a rule.
#[derive(Debug)]
pub struct Violation {
    pub line: usize,
    pub message: String,
    /// If true, `fix` is expected to resolve this violation.
    pub fixable: bool,
}

pub struct FixResult {
    pub source: String,
    pub errors: Vec<LintError>,
    pub changed: bool,
}

#[must_use]
pub fn fix(src: &str, features: &[String], disabled: &[String]) -> FixResult {
    let mut errs = vec![];

    let ast = match parse_file(src) {
        Err(e) => {
            errs.push(LintError::ParseError(format!(
                "failed to parse source: {e}"
            )));

            return FixResult {
                source: src.into(),
                errors: errs,
                changed: false,
            };
        }
        Ok(ast) => ast,
    };

    let canonical_before = unparse(&ast);
    let features: Vec<&str> = features.iter().map(String::as_str).collect();
    let disabled: Vec<&str> = disabled.iter().map(String::as_str).collect();
    let rules = active_unit_rules(&features, &disabled);

    if !disabled.contains(&"no-comment") {
        errs.extend(check_source(src));
    }

    let ast = rules.iter().fold(ast, |ast, r| enforce_rule(*r, ast));

    for rule in &rules {
        for v in rule.detect(&ast) {
            if !v.fixable {
                errs.push(LintError::RuleError {
                    rule: rule.name(),
                    line: v.line,
                    message: v.message,
                });

                continue;
            }

            errs.push(LintError::RuleError {
                rule: rule.name(),
                line: v.line,
                message: format!(
                    "[BUG] fixable violation remains after enforce: {}",
                    v.message
                ),
            });
        }
    }

    let fixed = unparse(&ast);
    let changed = fixed != canonical_before;

    FixResult {
        source: fixed,
        errors: errs,
        changed,
    }
}

fn active_unit_rules(features: &[&str], disabled: &[&str]) -> Vec<&'static dyn UnitRule> {
    let mut rules: Vec<&'static dyn UnitRule> = rules::BASE_RULES.to_vec();

    if features.contains(&"theta") {
        rules.extend_from_slice(rules::THETA_RULES);
    }

    rules.retain(|r| !disabled.contains(&r.name()));

    rules
}

/// Enforce a single rule: detect → fix loop until fixpoint.
/// Not a trait method — rules cannot bypass detect.
fn enforce_rule(rule: &dyn UnitRule, mut ast: File) -> File {
    for _ in 0..MAX_ENFORCE_ITERATIONS {
        if !rule.detect(&ast).iter().any(|v| v.fixable) {
            break;
        }

        ast = rule.fix(ast);
    }

    ast
}
