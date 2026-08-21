use proc_macro2::Span;
use syn::{Attribute, Item, ItemUse};

use super::{
    RsRule, Rule,
    common::{Origin, origin_name, use_item_origin},
};

pub struct UseGroupOrder;

fn has_cfg(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("cfg"))
}

fn span_line(span: Span) -> usize {
    span.start().line
}

impl Rule for UseGroupOrder {
    fn name(&self) -> &'static str {
        "use-group-order"
    }

    fn description(&self) -> &'static str {
        "use groups must be ordered: std → external crates → crate/self"
    }
}

impl RsRule for UseGroupOrder {
    fn check(&self, file: &syn::File) -> Vec<(usize, String)> {
        check_items(&file.items)
    }
}

fn check_items(items: &[Item]) -> Vec<(usize, String)> {
    let mut violations = vec![];

    let unconditional: Vec<&ItemUse> = items
        .iter()
        .filter_map(|item| match item {
            Item::Use(u) if !has_cfg(&u.attrs) => Some(u),
            _ => None,
        })
        .collect();

    let mut prev: Option<(&ItemUse, Origin)> = None;

    for u in unconditional {
        let origin = use_item_origin(u);

        if let Some((prev_u, prev_origin)) = prev {
            if prev_origin > origin {
                violations.push((
                    span_line(u.use_token.span),
                    format!(
                        "{} import must not appear after {} import; order: std → external-crate → crate",
                        origin_name(origin),
                        origin_name(prev_origin),
                    ),
                ));
            }

            if prev_origin != origin {
                let prev_last_line = span_line(prev_u.semi_token.span);
                let curr_first_line = span_line(u.use_token.span);
                if curr_first_line <= prev_last_line + 1 {
                    violations.push((
                        span_line(u.use_token.span),
                        format!(
                            "missing blank line between {} and {} import groups",
                            origin_name(prev_origin),
                            origin_name(origin),
                        ),
                    ));
                }
            }
        }

        prev = Some((u, origin));
    }

    // ── Recurse into inline modules ──────────────────────────────────────
    for item in items {
        if let Item::Mod(m) = item
            && let Some((_, content)) = &m.content
        {
            violations.extend(check_items(content));
        }
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<(usize, String)> {
        let file = syn::parse_file(src).unwrap();
        UseGroupOrder.check(&file)
    }

    #[test]
    fn top_level_wrong_order_is_violation() {
        // Wrong order AND missing blank line → 2 violations.
        let src = "use crate::foo;\nuse std::io;";
        assert_eq!(check(src).len(), 2);
    }

    #[test]
    fn violation_inside_nested_mod_is_detected() {
        let src = "mod inner { use crate::foo;\nuse std::io; }";
        assert!(
            !check(src).is_empty(),
            "expected violation inside nested mod"
        );
    }
}
