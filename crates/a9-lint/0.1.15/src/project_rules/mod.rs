use crate::ProjectRule;

mod clippy_lints;
mod pre_commit_hook;
mod scan_coverage;

pub static PROJECT_RULES: &[&dyn ProjectRule] = &[
    &clippy_lints::Rule,
    &pre_commit_hook::Rule,
    &scan_coverage::Rule,
];
