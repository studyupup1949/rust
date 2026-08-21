use crate::UnitRule;

mod cfg_order;
mod common;
mod module_item_order;
mod no_allow_unused_imports;

pub mod no_comment;

mod normalized_use_stmt;
mod optimal_control_flow;
mod path_depth;
mod scope_ident_length_correspondance;
mod theta_actor_fields_gated;
mod theta_actor_private_gate;
mod theta_actors_at_bottom;
mod theta_no_private_type_leak;
mod unnecessary_import_alias;
mod use_group_order;
mod use_order;
mod use_toplevel;

pub static BASE_RULES: &[&dyn UnitRule] = &[
    &cfg_order::UnitRule,
    &module_item_order::UnitRule,
    &no_allow_unused_imports::UnitRule,
    &path_depth::UnitRule,
    &normalized_use_stmt::UnitRule,
    &optimal_control_flow::UnitRule,
    &scope_ident_length_correspondance::UnitRule,
    &unnecessary_import_alias::UnitRule,
    &use_group_order::UnitRule,
    &use_order::UnitRule,
    &use_toplevel::UnitRule,
];
pub static THETA_RULES: &[&dyn UnitRule] = &[
    &theta_actor_fields_gated::UnitRule,
    &theta_actor_private_gate::UnitRule,
    &theta_actors_at_bottom::UnitRule,
    &theta_no_private_type_leak::UnitRule,
];

#[cfg(test)]
mod meta_tests {
    use std::fs;

    use super::{BASE_RULES, THETA_RULES};
    use crate::{Rule, project_rules::PROJECT_RULES};

    fn all_rules() -> Vec<&'static dyn Rule> {
        let mut rules: Vec<&'static dyn Rule> =
            BASE_RULES.iter().map(|r| *r as &dyn Rule).collect();

        rules.extend(THETA_RULES.iter().map(|r| *r as &dyn Rule));
        rules.extend(PROJECT_RULES.iter().map(|r| *r as &dyn Rule));

        rules
    }

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

    #[test]
    fn all_rules_fit_in_1024_lines() {
        let rules_dir = "src/rules";
        let entries = fs::read_dir(rules_dir).expect("Failed to read src/rules directory");

        for entry in entries {
            let path = entry.expect("Failed to read directory entry").path();

            if !path.is_file() {
                continue;
            }

            let file_name = path.file_name().unwrap().to_string_lossy().to_string();

            if file_name == "mod.rs" || file_name == "common.rs" {
                continue;
            }

            if path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }

            let source = fs::read_to_string(&path).unwrap_or_default();
            let lines = source.lines().count();

            assert!(
                lines <= 1024,
                "Rule file `{file_name}` has {lines} lines, exceeds the 1024-line limit"
            );
        }
    }

    #[test]
    fn all_rule_file_names_match_rule_names() {
        let unit_rules: Vec<&dyn Rule> = BASE_RULES
            .iter()
            .map(|r| *r as &dyn Rule)
            .chain(THETA_RULES.iter().map(|r| *r as &dyn Rule))
            .collect();

        for rule in &unit_rules {
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

        for rule in PROJECT_RULES.iter() {
            let expected_stem = rule.name().replace('-', "_");
            let expected_file = format!("{expected_stem}.rs");
            let path = std::path::Path::new("src/project_rules").join(&expected_file);

            assert!(
                path.exists(),
                "Rule `{}` expects file `src/project_rules/{}` but it does not exist",
                rule.name(),
                expected_file,
            );
        }
    }
}
