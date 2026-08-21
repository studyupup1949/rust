use proc_macro2::Span;
use syn::{Attribute, File, Item, ItemUse, Meta};

use super::common::{Origin, use_item_origin};
use crate::{Rule as RuleTrait, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl RuleTrait for UnitRule {
    fn name(&self) -> &'static str {
        "cfg-order"
    }

    fn description(&self) -> &'static str {
        "cfg-gated use items must come after unconditional ones, ordered by complexity"
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

fn cfg_attr(attrs: &[Attribute]) -> Option<&Attribute> {
    attrs.iter().find(|a| a.path().is_ident("cfg"))
}

/// Rough complexity: number of commas in the cfg token stream + 1.
fn cfg_complexity(attr: &Attribute) -> usize {
    if let Meta::List(list) = &attr.meta {
        list.tokens
            .to_string()
            .chars()
            .filter(|&c| c == ',')
            .count()
            + 1
    } else {
        1
    }
}

fn cfg_string(attr: &Attribute) -> String {
    if let Meta::List(list) = &attr.meta {
        list.tokens.to_string()
    } else {
        String::new()
    }
}

fn span_line(span: Span) -> usize {
    span.start().line
}

/// Sort key for a use item: (`has_cfg`, complexity, `cfg_string`, origin).
/// Unconditional uses (`has_cfg`=false) sort before cfg-gated ones.
fn use_sort_key(u: &ItemUse) -> (bool, usize, String, Origin) {
    match cfg_attr(&u.attrs) {
        None => (false, 0, String::new(), use_item_origin(u)),
        Some(attr) => (
            true,
            cfg_complexity(attr),
            cfg_string(attr),
            use_item_origin(u),
        ),
    }
}

fn check_items(items: &[Item]) -> Vec<(usize, String)> {
    let mut violations = vec![];
    let mut last_cfg: Option<(usize, String, Origin)> = None;
    let mut seen_any_cfg = false;

    for item in items {
        match item {
            Item::Use(u) => match cfg_attr(&u.attrs) {
                None => {
                    if seen_any_cfg {
                        violations.push((
                            span_line(u.use_token.span),
                            "unconditional use item must come before all cfg-gated use items"
                                .into(),
                        ));
                    }
                }
                Some(attr) => {
                    seen_any_cfg = true;

                    let complexity = cfg_complexity(attr);
                    let s = cfg_string(attr);
                    let origin = use_item_origin(u);

                    if let Some((pc, ref ps, po)) = last_cfg
                        && (pc, ps.as_str(), po) > (complexity, s.as_str(), origin)
                    {
                        violations
                            .push((
                                span_line(u.use_token.span),
                                format!(
                                    "cfg-gated use out of order: current cfg({s}) should not precede cfg({ps})"
                                ),
                            ));
                    }

                    last_cfg = Some((complexity, s, origin));
                }
            },
            Item::Mod(m) => {
                if let Some((_, content)) = &m.content {
                    violations.extend(check_items(content));
                }
            }
            _ => {}
        }
    }

    violations
}

/// Sort use items within a list: unconditional first, then cfg-gated by
/// (complexity, `cfg_string`, origin). Non-use items stay in place.
fn sort_uses(items: &mut [Item]) {
    let use_indices: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| matches!(item, Item::Use(_)))
        .map(|(i, _)| i)
        .collect();

    if use_indices.len() < 2 {
        return;
    }

    let mut uses: Vec<Item> = use_indices.iter().map(|&i| items[i].clone()).collect();

    uses.sort_by(|a, b| {
        let Item::Use(ua) = a else { unreachable!() };

        let Item::Use(ub) = b else { unreachable!() };

        use_sort_key(ua).cmp(&use_sort_key(ub))
    });

    for (&idx, sorted) in use_indices.iter().zip(uses) {
        items[idx] = sorted;
    }
}

/// Recursively sort uses in a list of items (including inline mods).
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
    fn empty_file_no_violations() {
        assert!(detect("").is_empty());
    }

    #[test]
    fn no_use_items_no_violations() {
        assert!(detect("fn foo() {}").is_empty());
    }

    #[test]
    fn single_cfg_use_no_violations() {
        let src = "#[cfg(feature = \"a\")]\nuse std::io;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn all_unconditional_no_violations() {
        assert!(detect("use std::io;\nuse std::fmt;\n").is_empty());
    }

    #[test]
    fn cfg_before_unconditional_is_violation() {
        let src = "#[cfg(feature = \"a\")]\nuse std::io;\nuse std::fmt;\n";
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
        assert!(vs[0].fixable);
        assert!(vs[0].message.contains("unconditional"));
    }

    #[test]
    fn two_unconditional_after_cfg_are_two_violations() {
        let src = "#[cfg(feature = \"a\")]\nuse std::io;\nuse std::fmt;\nuse std::vec;\n";

        assert_eq!(detect(src).len(), 2);
    }

    #[test]
    fn cfg_order_by_complexity_violation() {
        let src = "#[cfg(all(feature = \"a\", feature = \"b\"))]\nuse std::io;\n#[cfg(feature = \"a\")]\nuse std::fmt;\n";
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
        assert!(vs[0].message.contains("out of order"));
    }

    #[test]
    fn violation_inside_nested_mod() {
        let src =
            "mod inner {\n    #[cfg(feature = \"a\")]\n    use std::io;\n    use std::fmt;\n}\n";

        assert_eq!(detect(src).len(), 1);
    }

    #[test]
    fn unconditional_before_cfg_is_ok() {
        let src = "use std::fmt;\n#[cfg(feature = \"a\")]\nuse std::io;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn simpler_cfg_before_complex_is_ok() {
        let src = "#[cfg(feature = \"a\")]\nuse std::fmt;\n#[cfg(all(feature = \"a\", feature = \"b\"))]\nuse std::io;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn fix_moves_unconditional_before_cfg() {
        let src = "#[cfg(feature = \"a\")]\nuse std::io;\nuse std::fmt;\n";
        let fixed = fix(src);
        let fmt_pos = fixed.find("std::fmt").unwrap();
        let io_pos = fixed.find("std::io").unwrap();

        assert!(
            fmt_pos < io_pos,
            "unconditional (fmt) should come before cfg-gated (io)\nfixed:\n{fixed}"
        );
    }

    #[test]
    fn fix_idempotent_on_correct_order() {
        let src = "use std::fmt;\n#[cfg(feature = \"a\")]\nuse std::io;\n";
        let fixed = fix(src);

        assert!(fixed.contains("std::fmt"));
        assert!(fixed.contains("std::io"));
    }

    #[test]
    fn fix_sorts_cfg_by_complexity() {
        let src = "use std::fmt;\n#[cfg(all(feature = \"a\", feature = \"b\"))]\nuse std::io;\n#[cfg(feature = \"a\")]\nuse std::vec;\n";
        let fixed = fix(src);
        let vec_pos = fixed.find("std::vec").unwrap();
        let io_pos = fixed.find("std::io").unwrap();

        assert!(
            vec_pos < io_pos,
            "simpler cfg should come before complex\nfixed:\n{fixed}"
        );
    }

    #[test]
    fn fix_preserves_non_use_items_in_place() {
        let src =
            "fn foo() {}\n#[cfg(feature = \"a\")]\nuse std::io;\nuse std::fmt;\nstruct Bar;\n";
        let fixed = fix(src);
        let fmt_pos = fixed.find("std::fmt").unwrap();
        let io_pos = fixed.find("std::io").unwrap();
        let foo_pos = fixed.find("fn foo").unwrap();
        let bar_pos = fixed.find("struct Bar").unwrap();

        assert!(fmt_pos < io_pos, "uses reordered");
        assert!(foo_pos < fmt_pos, "fn stays before uses");
        assert!(bar_pos > io_pos, "struct stays after uses");
    }

    #[test]
    fn fix_then_detect_is_clean() {
        let src = "#[cfg(feature = \"a\")]\nuse std::io;\nuse std::fmt;\n";
        let file = syn::parse_file(src).unwrap();
        let fixed = UnitRule.fix(file);

        assert!(UnitRule.detect(&fixed).is_empty(), "fixpoint not reached");
    }
}
