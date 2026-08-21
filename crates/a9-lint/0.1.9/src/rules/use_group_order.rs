use proc_macro2::Span;
use syn::{Attribute, File, Item};

use super::common::{Origin, origin_name, use_item_origin};
use crate::{Rule as RuleTrait, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl RuleTrait for UnitRule {
    fn name(&self) -> &'static str {
        "use-group-order"
    }

    fn description(&self) -> &'static str {
        "use groups must be ordered: std → external crates → crate/self"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &File) -> Vec<Violation> {
        check_items(&ast.items)
            .into_iter()
            .map(|(line, message)| Violation {
                line,
                message,
                fixable: true,
            })
            .collect()
    }

    fn fix(&self, mut ast: File) -> File {
        fix_items(&mut ast.items);

        ast
    }
}

fn has_cfg(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("cfg"))
}

fn span_line(span: Span) -> usize {
    span.start().line
}

fn check_items(items: &[Item]) -> Vec<(usize, String)> {
    let mut violations = vec![];
    let mut prev_origin: Option<Origin> = None;

    for item in items {
        if let Item::Use(u) = item
            && !has_cfg(&u.attrs)
        {
            let origin = use_item_origin(u);

            if let Some(prev) = prev_origin
                && prev > origin
            {
                violations
                    .push((
                        span_line(u.use_token.span),
                        format!(
                            "{} import must not appear after {} import; order: std → external-crate → crate",
                            origin_name(origin), origin_name(prev),
                        ),
                    ));
            }

            prev_origin = Some(origin);
        }
    }

    for item in items {
        if let Item::Mod(m) = item
            && let Some((_, content)) = &m.content
        {
            violations.extend(check_items(content));
        }
    }

    violations
}

/// Sort unconditional use items by `Origin` (stable sort).
/// Non-use items and cfg-gated uses stay at their original indices.
fn sort_uses(items: &mut [Item]) {
    let use_indices: Vec<usize> = items
        .iter()
        .enumerate()
        .filter_map(|(i, item)| match item {
            Item::Use(u) if !has_cfg(&u.attrs) => Some(i),
            _ => None,
        })
        .collect();

    let mut uses: Vec<Item> = use_indices.iter().map(|&i| items[i].clone()).collect();

    uses.sort_by_key(|item| {
        let Item::Use(u) = item else { unreachable!() };

        use_item_origin(u)
    });

    for (&idx, sorted) in use_indices.iter().zip(uses) {
        items[idx] = sorted;
    }
}

fn fix_items(items: &mut [Item]) {
    sort_uses(items);

    for item in items.iter_mut() {
        if let Item::Mod(m) = item
            && let Some((_, content)) = &mut m.content
        {
            fix_items(content);
        }
    }
}

#[cfg(test)]
mod tests {
    use a9_prettyplease::unparse;

    use super::*;
    use crate::{UnitRule as UnitRuleTrait, Violation};

    fn detect(src: &str) -> Vec<Violation> {
        let file = syn::parse_file(src).unwrap();

        UnitRule.detect(&file)
    }

    fn fix(src: &str) -> String {
        let file = syn::parse_file(src).unwrap();
        let fixed = UnitRule.fix(file);

        unparse(&fixed)
    }

    #[test]
    fn correct_order_no_violations() {
        let src = "use std::io;\nuse clap::Arg;\nuse crate::config;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn crate_before_std_is_violation() {
        let src = "use crate::foo;\nuse std::io;\n";
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
        assert!(
            vs[0]
                .message
                .contains("std import must not appear after crate/self")
        );
    }

    #[test]
    fn external_before_std_is_violation() {
        let src = "use clap::Arg;\nuse std::io;\n";
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
        assert!(
            vs[0]
                .message
                .contains("std import must not appear after external-crate")
        );
    }

    #[test]
    fn crate_before_external_is_violation() {
        let src = "use crate::foo;\nuse clap::Arg;\n";
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
        assert!(
            vs[0]
                .message
                .contains("external-crate import must not appear after crate/self")
        );
    }

    #[test]
    fn cfg_gated_uses_are_ignored() {
        let src = "#[cfg(test)]\nuse crate::test_utils;\nuse std::io;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn std_only_no_violations() {
        let src = "use std::io;\nuse std::fmt;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn violation_inside_nested_mod() {
        let src = "mod inner { use crate::foo;\nuse std::io; }";

        assert!(!detect(src).is_empty());
    }

    #[test]
    fn fix_reorders_crate_after_std() {
        let src = "use crate::foo;\nuse std::io;\n";
        let fixed = fix(src);
        let std_pos = fixed.find("std::io").unwrap();
        let crate_pos = fixed.find("crate::foo").unwrap();

        assert!(
            std_pos < crate_pos,
            "std should come before crate\nfixed:\n{fixed}"
        );
    }

    #[test]
    fn fix_reorders_all_three_groups() {
        let src = "use crate::config;\nuse clap::Arg;\nuse std::io;\n";
        let fixed = fix(src);
        let std_pos = fixed.find("std::io").unwrap();
        let ext_pos = fixed.find("clap::Arg").unwrap();
        let crt_pos = fixed.find("crate::config").unwrap();

        assert!(
            std_pos < ext_pos && ext_pos < crt_pos,
            "expected std < external < crate\nfixed:\n{fixed}"
        );
    }

    #[test]
    fn fix_preserves_non_use_items() {
        let src = "fn foo() {}\nuse crate::bar;\nuse std::io;\nstruct Baz;\n";
        let fixed = fix(src);
        let std_pos = fixed.find("std::io").unwrap();
        let crate_pos = fixed.find("crate::bar").unwrap();
        let foo_pos = fixed.find("fn foo").unwrap();
        let baz_pos = fixed.find("struct Baz").unwrap();

        assert!(std_pos < crate_pos, "uses reordered");
        assert!(foo_pos < std_pos, "fn stays before uses");
        assert!(baz_pos > crate_pos, "struct stays after uses");
    }

    #[test]
    fn fix_idempotent_on_correct_order() {
        let src = "use std::io;\nuse clap::Arg;\nuse crate::config;\n";
        let file = syn::parse_file(src).unwrap();

        assert!(UnitRule.detect(&file).is_empty());
    }

    #[test]
    fn fix_then_detect_is_clean() {
        let src = "use crate::config;\nuse std::io;\nuse clap::Arg;\n";
        let file = syn::parse_file(src).unwrap();
        let fixed = UnitRule.fix(file);

        assert!(UnitRule.detect(&fixed).is_empty(), "fixpoint not reached");
    }

    #[test]
    fn super_treated_as_crate() {
        let src = "use super::bar;\nuse std::io;\n";
        let vs = detect(src);

        assert!(!vs.is_empty());
    }

    #[test]
    fn self_treated_as_crate() {
        let src = "use self::bar;\nuse std::io;\n";
        let vs = detect(src);

        assert!(!vs.is_empty());
    }
}
