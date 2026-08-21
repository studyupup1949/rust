use proc_macro2::Span;
use syn::Item;

use super::Rule;

pub struct ItemOrder;

/// Category rank for top-level item ordering per Rust Style Guide:
///   extern crate → use → mod (declaration-only) → everything else
///
/// Only declaration-only modules (`mod foo;`) are rank 2.
/// Inline modules (`mod foo { ... }`) are rank 3 — they are regular items
/// and can appear anywhere after declarations (e.g. `#[cfg(test)] mod tests { ... }`).
fn item_rank(item: &Item) -> u8 {
    match item {
        Item::ExternCrate(_) => 0,
        Item::Use(_) => 1,
        Item::Mod(m) if m.content.is_none() => 2,
        _ => 3,
    }
}

fn rank_label(rank: u8) -> &'static str {
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

impl Rule for ItemOrder {
    fn name(&self) -> &'static str {
        "item-order"
    }

    fn check(&self, file: &syn::File) -> Vec<(usize, String)> {
        let mut violations = vec![];
        let mut max_rank: u8 = 0;
        let mut max_rank_line: usize = 0;

        for item in &file.items {
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
        }

        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<(usize, String)> {
        let file = syn::parse_file(src).unwrap();
        ItemOrder.check(&file)
    }

    #[test]
    fn correct_order_no_violations() {
        let src = r#"
use std::io;
use serde::Serialize;

mod private;

struct Foo;
"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn mod_before_use_violation() {
        let src = r#"
mod foo;

use std::io;
"#;
        let v = check(src);
        assert_eq!(v.len(), 1);
        assert!(v[0].1.contains("`use` must appear before `mod`"));
    }

    #[test]
    fn use_after_struct_violation() {
        let src = r#"
struct Foo;

use std::io;
"#;
        let v = check(src);
        assert_eq!(v.len(), 1);
        assert!(v[0].1.contains("`use` must appear before `item`"));
    }

    #[test]
    fn pub_use_treated_as_use() {
        let src = r#"
use std::io;
pub use crate::Foo;

mod private;
"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn cfg_attrs_dont_change_rank() {
        let src = r#"
use std::io;

#[cfg(feature = "private")]
pub use private::Args;

#[cfg(feature = "private")]
mod private;

struct Foo;
"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn inline_mod_is_regular_item() {
        let src = r#"
use std::io;

mod foo;

struct Bar;

impl Bar {}

#[cfg(test)]
mod tests {
    use super::*;
}
"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn inline_mod_after_struct_ok() {
        let src = r#"
struct Foo;

mod inline_mod {
    pub fn bar() {}
}
"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn declaration_mod_after_struct_violation() {
        let src = r#"
struct Foo;

mod bar;
"#;
        let v = check(src);
        assert_eq!(v.len(), 1);
        assert!(v[0].1.contains("`mod` must appear before `item`"));
    }

    #[test]
    fn multiple_violations() {
        let src = r#"
struct Foo;
mod bar;
use std::io;
"#;
        let v = check(src);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn extern_crate_first() {
        let src = r#"
extern crate alloc;
use std::io;
mod foo;
struct Bar;
"#;
        assert!(check(src).is_empty());
    }

    #[test]
    fn use_before_extern_crate_violation() {
        let src = r#"
use std::io;
extern crate alloc;
"#;
        let v = check(src);
        assert_eq!(v.len(), 1);
        assert!(v[0].1.contains("`extern crate` must appear before `use`"));
    }
}
