use proc_macro2::Span;
use syn::{File, Item};

use crate::{Rule as RuleTrait, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl RuleTrait for UnitRule {
    fn name(&self) -> &'static str {
        "use-order"
    }

    fn description(&self) -> &'static str {
        "Top-level items must follow the order: extern crate → use → mod (declaration-only) → everything else"
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

/// Category rank for top-level item ordering per Rust Style Guide:
///   extern crate → use → mod (declaration-only) → everything else
///
/// Only declaration-only modules (`mod foo;`) are rank 2.
/// Inline modules (`mod foo { ... }`) are rank 3 — they are regular items
/// and can appear anywhere after declarations (e.g. `#[cfg(test)] mod tests { ... }`).
const fn item_rank(item: &Item) -> u8 {
    match item {
        Item::ExternCrate(_) => 0,
        Item::Use(_) => 1,
        Item::Mod(m) if m.content.is_none() => 2,
        _ => 3,
    }
}

const fn rank_label(rank: u8) -> &'static str {
    match rank {
        0 => "extern crate",
        1 => "use",
        2 => "mod",
        _ => "item",
    }
}

fn item_line(item: &Item) -> usize {
    let span = match item {
        Item::ExternCrate(i) => i.extern_token.span,
        Item::Use(i) => i.use_token.span,
        Item::Mod(i) => i.mod_token.span,
        Item::Fn(i) => i.sig.fn_token.span,
        Item::Struct(i) => i.struct_token.span,
        Item::Enum(i) => i.enum_token.span,
        Item::Trait(i) => i.trait_token.span,
        Item::Impl(i) => i.impl_token.span,
        Item::Const(i) => i.const_token.span,
        Item::Static(i) => i.static_token.span,
        Item::Type(i) => i.type_token.span,
        Item::Macro(i) => i
            .mac
            .path
            .segments
            .first()
            .map_or_else(Span::call_site, |s| s.ident.span()),
        _ => Span::call_site(),
    };

    span.start().line
}

fn check_items(items: &[Item]) -> Vec<(usize, String)> {
    let mut violations = vec![];
    let mut max_rank: u8 = 0;
    let mut max_rank_line: usize = 0;

    for item in items {
        let rank = item_rank(item);
        let line = item_line(item);

        if rank < max_rank {
            violations.push((
                line,
                format!(
                    "`{}` must appear before `{}` (line {})",
                    rank_label(rank),
                    rank_label(max_rank),
                    max_rank_line,
                ),
            ));
        }

        if rank > max_rank {
            max_rank = rank;
            max_rank_line = line;
        }

        if let Item::Mod(m) = item
            && let Some((_, content)) = &m.content
        {
            violations.extend(check_items(content));
        }
    }

    violations
}

/// Stable-sort items by rank, preserving relative order within the same rank.
/// Recurse into inline mods.
fn fix_items(items: &mut [Item]) {
    items.sort_by_key(item_rank);

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
    fn mod_before_use_violation() {
        let src = "mod foo;\nuse std::io;\n";
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
        assert!(vs[0].message.contains("`use` must appear before `mod`"));
    }

    #[test]
    fn use_after_struct_violation() {
        let src = "struct Foo;\nuse std::io;\n";
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
        assert!(vs[0].message.contains("`use` must appear before `item`"));
    }

    #[test]
    fn multiple_violations() {
        let src = "struct Foo;\nmod bar;\nuse std::io;\n";

        assert_eq!(detect(src).len(), 2);
    }

    #[test]
    fn use_before_extern_crate_violation() {
        let src = "use std::io;\nextern crate alloc;\n";
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
        assert!(
            vs[0]
                .message
                .contains("`extern crate` must appear before `use`")
        );
    }

    #[test]
    fn declaration_mod_after_struct_violation() {
        let src = "struct Foo;\nmod bar;\n";
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
        assert!(vs[0].message.contains("`mod` must appear before `item`"));
    }

    #[test]
    fn violation_inside_nested_mod() {
        let src = "mod inner { struct Foo; use std::io; }";

        assert_eq!(detect(src).len(), 1);
    }

    #[test]
    fn correct_order_no_violations() {
        let src = "use std::io;\nmod private;\nstruct Foo;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn pub_use_treated_as_use() {
        let src = "use std::io;\npub use crate::Foo;\nmod private;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn cfg_attrs_dont_change_rank() {
        let src = "use std::io;\n#[cfg(feature = \"private\")]\npub use private::Args;\n#[cfg(feature = \"private\")]\nmod private;\nstruct Foo;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn inline_mod_is_regular_item() {
        let src =
            "use std::io;\nmod foo;\nstruct Bar;\n#[cfg(test)]\nmod tests { use super::*; }\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn extern_crate_first() {
        let src = "extern crate alloc;\nuse std::io;\nmod foo;\nstruct Bar;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn all_fixable() {
        let src = "struct Foo;\nuse std::io;\n";

        for v in detect(src) {
            assert!(v.fixable);
        }
    }

    #[test]
    fn fix_moves_use_before_mod() {
        let src = "mod foo;\nuse std::io;\n";
        let fixed = fix(src);
        let use_pos = fixed.find("use std").unwrap();
        let mod_pos = fixed.find("mod foo").unwrap();

        assert!(
            use_pos < mod_pos,
            "use should come before mod\nfixed:\n{fixed}"
        );
    }

    #[test]
    fn fix_sorts_full_order() {
        let src = "struct Bar;\nuse std::io;\nextern crate alloc;\n";
        let fixed = fix(src);
        let ext_pos = fixed.find("extern crate").unwrap();
        let use_pos = fixed.find("use std").unwrap();
        let struct_pos = fixed.find("struct Bar").unwrap();

        assert!(
            ext_pos < use_pos && use_pos < struct_pos,
            "expected extern crate < use < struct\nfixed:\n{fixed}"
        );
    }

    #[test]
    fn fix_idempotent_on_correct_order() {
        let src = "use std::io;\nmod foo;\nstruct Bar;\n";
        let file = syn::parse_file(src).unwrap();

        assert!(UnitRule.detect(&file).is_empty());
    }

    #[test]
    fn fix_then_detect_is_clean() {
        let src = "struct Bar;\nmod foo;\nuse std::io;\n";
        let file = syn::parse_file(src).unwrap();
        let fixed = UnitRule.fix(file);

        assert!(UnitRule.detect(&fixed).is_empty(), "fixpoint not reached");
    }
}
