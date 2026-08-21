use crate::ProjectRule;

mod clippy_lints;
mod pre_commit_hook;

pub static PROJECT_RULES: &[&dyn ProjectRule] = &[&clippy_lints::Rule, &pre_commit_hook::Rule];
