use proc_macro2::Span;
use syn::{Attribute, Item, Meta};

use super::{
    RsRule, Rule,
    common::{Origin, rejoin_sources, split_item_sources, use_item_origin},
};

pub struct CfgOrder;

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
                        violations.push((
                            span_line(u.use_token.span),
                            format!("cfg-gated use out of order: current cfg({s}) should not precede cfg({ps})"),
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

impl Rule for CfgOrder {
    fn name(&self) -> &'static str {
        "cfg-order"
    }

    fn description(&self) -> &'static str {
        "cfg-gated use items must come after unconditional ones, ordered by complexity"
    }
}

impl RsRule for CfgOrder {
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

        // Separate non-use items, unconditional uses, and cfg-gated uses.
        // Sort: unconditional uses first, then cfg-gated sorted by (complexity, cfg_str, origin).
        let mut non_use_indexed: Vec<(usize, String)> = Vec::new();
        let mut uncond: Vec<(usize, String)> = Vec::new();
        let mut cfged: Vec<(usize, String, usize, String, Origin)> = Vec::new();

        for (i, (item, text)) in file.items.iter().zip(texts).enumerate() {
            let Item::Use(u) = item else {
                non_use_indexed.push((i, text));
                continue;
            };
            match cfg_attr(&u.attrs) {
                None => uncond.push((i, text)),
                Some(attr) => {
                    let complexity = cfg_complexity(attr);
                    let s = cfg_string(attr);
                    let origin = use_item_origin(u);
                    cfged.push((i, text, complexity, s, origin));
                }
            }
        }

        // Sort cfg-gated uses by (complexity, cfg_string, origin).
        cfged.sort_by(|a, b| a.2.cmp(&b.2).then(a.3.cmp(&b.3)).then(a.4.cmp(&b.4)));

        // Reconstruct: place non-use items by their original positions, weaving in
        // sorted uses at the position of the first use item.
        let first_use_pos = file.items.iter().position(|i| matches!(i, Item::Use(_)));
        let Some(first_use_pos) = first_use_pos else {
            return Err("no use items found".into());
        };

        let mut result_texts: Vec<String> = Vec::with_capacity(file.items.len());
        let mut uses_inserted = false;

        for (i, item) in file.items.iter().enumerate() {
            if matches!(item, Item::Use(_)) {
                if !uses_inserted && i == first_use_pos {
                    for (_, t) in &uncond {
                        result_texts.push(t.clone());
                    }
                    for (_, t, _, _, _) in &cfged {
                        result_texts.push(t.clone());
                    }
                    uses_inserted = true;
                }
                // Skip original use positions (they're inserted above).
            } else if let Some(pos) = non_use_indexed.iter().position(|(idx, _)| *idx == i) {
                result_texts.push(non_use_indexed[pos].1.clone());
            }
        }

        Ok(rejoin_sources(
            &preamble,
            &result_texts,
            source.ends_with('\n'),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<(usize, String)> {
        let file = syn::parse_file(src).unwrap();
        CfgOrder.check(&file, src)
    }

    #[test]
    fn top_level_out_of_order_cfg_is_violation() {
        let src = r#"
#[cfg(feature = "a")]
use std::io;
use std::fmt;
"#;
        assert_eq!(check(src).len(), 1);
    }

    #[test]
    fn violation_inside_nested_mod_is_detected() {
        // cfg-gated use before unconditional use inside an inline mod — should be flagged.
        let src = r#"
mod inner {
    #[cfg(feature = "a")]
    use std::io;
    use std::fmt;
}
"#;
        let v = check(src);
        assert_eq!(
            v.len(),
            1,
            "expected violation inside nested mod, got {:?}",
            v
        );
    }
}
