use proc_macro2::Span;
use syn::{Attribute, Item, ItemUse};

use super::{
    RsRule, Rule,
    common::{Origin, origin_name, rejoin_sources, split_item_sources, use_item_origin},
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
    fn check(&self, file: &syn::File, _source: &str) -> Vec<(usize, String)> {
        check_items(&file.items)
    }

    fn has_fixer(&self) -> bool {
        true
    }

    fn try_fix(&self, source: &str, file: syn::File) -> Result<String, String> {
        if check_items(&file.items).is_empty() {
            return Err("no violations to fix".into());
        }
        let (preamble, texts) = split_item_sources(&file.items, source);

        // Collect unconditional use indices sorted by origin; keep cfg-gated and
        // non-use items at their original positions.
        let mut uncond_uses: Vec<(usize, Origin, String)> = Vec::new();
        let mut other: Vec<(usize, String)> = Vec::new();

        for (i, (item, text)) in file.items.iter().zip(texts).enumerate() {
            match item {
                Item::Use(u) if !has_cfg(&u.attrs) => {
                    uncond_uses.push((i, use_item_origin(u), text));
                }
                _ => other.push((i, text)),
            }
        }

        // Stable-sort unconditional uses by origin.
        uncond_uses.sort_by_key(|(_, o, _)| *o);

        // Find where the first unconditional use appeared; insert all sorted
        // uses there (with blank lines between groups), then continue with
        // the remaining items.
        let first_uncond_pos = file
            .items
            .iter()
            .position(|i| matches!(i, Item::Use(u) if !has_cfg(&u.attrs)));
        let Some(first_uncond_pos) = first_uncond_pos else {
            return Err("no unconditional use items found".into());
        };

        // Build sorted use block with blank-line separators between groups.
        let mut sorted_use_texts: Vec<String> = Vec::new();
        let mut prev_origin: Option<Origin> = None;
        for (_, origin, text) in &uncond_uses {
            if prev_origin.is_some_and(|po| po != *origin) {
                sorted_use_texts.push(String::new()); // blank line
            }
            sorted_use_texts.push(text.clone());
            prev_origin = Some(*origin);
        }

        let mut result_texts: Vec<String> = Vec::new();
        let mut uses_inserted = false;

        for (i, item) in file.items.iter().enumerate() {
            if matches!(item, Item::Use(u) if !has_cfg(&u.attrs)) {
                if !uses_inserted && i == first_uncond_pos {
                    result_texts.extend(sorted_use_texts.clone());
                    uses_inserted = true;
                }
                // Skip original unconditional-use positions.
            } else if let Some(pos) = other.iter().position(|(idx, _)| *idx == i) {
                result_texts.push(other[pos].1.clone());
            }
        }

        Ok(rejoin_sources(
            &preamble,
            &result_texts,
            source.ends_with('\n'),
        ))
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
        UseGroupOrder.check(&file, src)
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
