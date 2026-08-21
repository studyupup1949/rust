use proc_macro2::Span;
use syn::{Attribute, Item, Meta};

use super::{
    Rule,
    common::{Origin, use_item_origin},
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

    fn check(&self, file: &syn::File) -> Vec<(usize, String)> {
        check_items(&file.items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<(usize, String)> {
        let file = syn::parse_file(src).unwrap();
        CfgOrder.check(&file)
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
