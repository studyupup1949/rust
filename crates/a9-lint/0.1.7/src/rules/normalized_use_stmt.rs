use std::collections::HashMap;

use proc_macro2::Span;
use syn::{
    Attribute, Item, Meta, UseTree, Visibility,
    punctuated::Punctuated,
    token::{Brace, PathSep},
};

use crate::{Rule as RuleTrait, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl RuleTrait for UnitRule {
    fn name(&self) -> &'static str {
        "normalized-use-stmt"
    }

    fn description(&self) -> &'static str {
        "Use statements sharing the same root must be merged; duplicate root segments are flagged"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &syn::File) -> Vec<Violation> {
        check_items(&ast.items)
            .into_iter()
            .map(|(line, message, fixable)| Violation {
                line,
                message,
                fixable,
            })
            .collect()
    }

    fn fix(&self, mut ast: syn::File) -> syn::File {
        fix_items(&mut ast.items);

        ast
    }
}

fn span_line(span: Span) -> usize {
    span.start().line
}

fn first_segment(tree: &UseTree) -> Option<String> {
    match tree {
        UseTree::Path(p) => Some(p.ident.to_string()),
        UseTree::Name(n) => Some(n.ident.to_string()),
        UseTree::Rename(r) => Some(r.ident.to_string()),
        UseTree::Group(g) => g.items.first().and_then(first_segment),
        UseTree::Glob(_) => None,
    }
}

fn cfg_fingerprint(attrs: &[Attribute]) -> String {
    let mut parts: Vec<String> = attrs
        .iter()
        .filter(|a| a.path().is_ident("cfg"))
        .map(|a| {
            if let Meta::List(l) = &a.meta {
                l.tokens.to_string()
            } else {
                String::new()
            }
        })
        .collect();

    parts.sort();

    parts.join("|")
}

fn vis_key(vis: &Visibility) -> &'static str {
    match vis {
        Visibility::Public(_) => "pub",
        Visibility::Restricted(r) if r.path.is_ident("crate") => "pub(crate)",
        Visibility::Restricted(r) if r.path.is_ident("super") => "pub(super)",
        Visibility::Restricted(_) => "restricted",
        Visibility::Inherited => "",
    }
}

/// Check within a single use tree for siblings sharing a root segment.
fn check_within_group(
    tree: &UseTree,
    prefix: &str,
    violations: &mut Vec<(usize, String)>,
    line: usize,
) {
    match tree {
        UseTree::Group(group) => {
            let mut root_counts: HashMap<String, usize> = HashMap::new();

            for sub in &group.items {
                if let Some(root) = first_segment(sub) {
                    *root_counts.entry(root).or_insert(0) += 1;
                }
            }

            let mut dups: Vec<&String> = root_counts
                .iter()
                .filter(|&(_, &c)| c > 1)
                .map(|(k, _)| k)
                .collect();

            if !dups.is_empty() {
                dups.sort();

                let context = if prefix.is_empty() {
                    "use { }".to_string()
                } else {
                    format!("use {prefix}::{{ }}")
                };

                violations.push((
                    line,
                    format!(
                        "{context} has items with shared root that should be merged: {}",
                        dups.iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                ));
            }

            for sub in &group.items {
                check_within_group(sub, prefix, violations, line);
            }
        }
        UseTree::Path(p) => {
            let new_prefix = if prefix.is_empty() {
                p.ident.to_string()
            } else {
                format!("{prefix}::{}", p.ident)
            };

            check_within_group(&p.tree, &new_prefix, violations, line);
        }
        _ => {}
    }
}

/// Returns (line, message, fixable) triples.
fn check_items(items: &[Item]) -> Vec<(usize, String, bool)> {
    let mut violations = vec![];
    let mut root_first_line: HashMap<(String, String), usize> = HashMap::new();
    let mut root_count: HashMap<(String, String), usize> = HashMap::new();

    for item in items {
        let Item::Use(u) = item else { continue };
        let line = span_line(u.use_token.span);
        let cfg = cfg_fingerprint(&u.attrs);

        if let Some(root) = first_segment(&u.tree) {
            let key = (cfg, root);

            *root_count.entry(key.clone()).or_insert(0) += 1;
            root_first_line.entry(key).or_insert(line);
        }
    }

    let mut cross: Vec<(String, usize)> = root_count
        .into_iter()
        .filter(|(_, c)| *c > 1)
        .map(|(key, _)| {
            let line = root_first_line[&key];

            (key.1, line)
        })
        .collect();

    cross.sort_by_key(|&(_, line)| line);

    for (root, line) in cross {
        violations.push((
            line,
            format!(
                "multiple `use {root}::...` statements should be merged into `use {root}::{{...}}`"
            ),
            true,
        ));
    }

    for item in items {
        let Item::Use(u) = item else { continue };
        let line = span_line(u.use_token.span);

        let mut within = vec![];

        check_within_group(&u.tree, "", &mut within, line);

        for (l, msg) in within {
            violations.push((l, msg, false));
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

/// Flatten a `UseTree`'s top-level group into individual sub-trees.
fn flatten_tree(tree: &UseTree) -> Vec<UseTree> {
    match tree {
        UseTree::Group(g) => g.items.iter().flat_map(flatten_tree).collect(),
        other => vec![other.clone()],
    }
}

fn use_tree_sort_key(tree: &UseTree) -> String {
    match tree {
        UseTree::Path(p) => format!("{}::{}", p.ident, use_tree_sort_key(&p.tree)),
        UseTree::Name(n) => n.ident.to_string(),
        UseTree::Rename(r) => format!("{} as {}", r.ident, r.rename),
        UseTree::Group(g) => {
            let items: Vec<String> = g.items.iter().map(use_tree_sort_key).collect();

            format!("{{{}}}", items.join(", "))
        }
        UseTree::Glob(_) => "*".to_string(),
    }
}

fn merge_use_items(items: &mut Vec<Item>) {
    type Key = (String, String, String);

    let mut groups: Vec<(Key, Vec<usize>)> = vec![];
    let mut key_to_idx: HashMap<Key, usize> = HashMap::new();

    for (i, item) in items.iter().enumerate() {
        let Item::Use(u) = item else { continue };

        if !matches!(&u.tree, UseTree::Path(_)) {
            continue;
        }

        let cfg = cfg_fingerprint(&u.attrs);
        let vis = vis_key(&u.vis).to_string();

        if let Some(root) = first_segment(&u.tree) {
            let key = (cfg, root, vis);

            if let Some(&gi) = key_to_idx.get(&key) {
                groups[gi].1.push(i);
            } else {
                let gi = groups.len();

                key_to_idx.insert(key.clone(), gi);
                groups.push((key, vec![i]));
            }
        }
    }

    let mut to_remove: Vec<usize> = vec![];

    for (_, indices) in &groups {
        if indices.len() < 2 {
            continue;
        }

        let mut all_subs: Vec<UseTree> = vec![];

        for &i in indices {
            let Item::Use(u) = &items[i] else {
                unreachable!()
            };

            if let UseTree::Path(p) = &u.tree {
                all_subs.extend(flatten_tree(&p.tree));
            }
        }

        all_subs.sort_by_key(use_tree_sort_key);
        all_subs.dedup_by(|a, b| use_tree_sort_key(a) == use_tree_sort_key(b));

        let first_idx = indices[0];

        let root_ident = {
            let Item::Use(u) = &items[first_idx] else {
                unreachable!()
            };

            match &u.tree {
                UseTree::Path(p) => p.ident.clone(),
                _ => continue,
            }
        };

        let merged_sub = if all_subs.len() == 1 {
            all_subs.into_iter().next().unwrap()
        } else {
            let mut punct = Punctuated::new();

            for sub in all_subs {
                punct.push(sub);
            }

            UseTree::Group(syn::UseGroup {
                brace_token: Brace::default(),
                items: punct,
            })
        };

        let merged_tree = UseTree::Path(syn::UsePath {
            ident: root_ident,
            colon2_token: PathSep::default(),
            tree: Box::new(merged_sub),
        });

        if let Item::Use(u) = &mut items[first_idx] {
            u.tree = merged_tree;
        }

        for &i in &indices[1..] {
            to_remove.push(i);
        }
    }

    to_remove.sort_unstable();
    to_remove.dedup();

    for &i in to_remove.iter().rev() {
        items.remove(i);
    }
}

fn fix_items(items: &mut Vec<Item>) {
    merge_use_items(items);

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
    fn top_level_duplicate_root_is_violation() {
        let src = "use std::io;\nuse std::fmt;\n";
        let v = detect(src);

        assert_eq!(v.len(), 1);
        assert!(v[0].fixable);
        assert!(v[0].message.contains("std"));
    }

    #[test]
    fn no_violation_when_roots_differ() {
        let src = "use std::io;\nuse syn::File;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn violation_inside_nested_mod() {
        let src = "mod inner {\n    use std::io;\n    use std::fmt;\n}\n";
        let v = detect(src);

        assert_eq!(v.len(), 1, "expected violation in nested mod: {v:?}");
    }

    #[test]
    fn cfg_gated_groups_not_merged() {
        let src = r#"
use syn::ItemUse;

#[cfg(feature = "theta")]
use syn::{Attribute, Meta};
"#;

        assert!(
            detect(src).is_empty(),
            "different cfg groups should not merge"
        );
    }

    #[test]
    fn same_cfg_group_duplicate_root_is_violation() {
        let src = r#"
#[cfg(feature = "theta")]
use syn::Attribute;

#[cfg(feature = "theta")]
use syn::Meta;
"#;
        let v = detect(src);

        assert_eq!(v.len(), 1);
        assert!(v[0].fixable);
    }

    #[test]
    fn within_group_duplicate_is_unfixable() {
        let src = "use std::{io, io::Read};\n";
        let v = detect(src);

        assert_eq!(v.len(), 1);
        assert!(!v[0].fixable);
    }

    #[test]
    fn fix_merges_same_root() {
        let src = "use std::io;\nuse std::fmt;\n";
        let fixed = fix(src);

        assert!(fixed.contains("std::{"), "expected merged use: {fixed}");
        assert!(fixed.contains("fmt"));
        assert!(fixed.contains("io"));
    }

    #[test]
    fn fix_preserves_different_roots() {
        let src = "use std::io;\nuse syn::File;\n";
        let fixed = fix(src);

        assert!(fixed.contains("use std::io;"));
        assert!(fixed.contains("use syn::File;"));
    }

    #[test]
    fn fix_idempotent() {
        let src = "use std::io;\nuse std::fmt;\n";
        let once = fix(src);
        let twice = fix(&once);

        assert_eq!(once, twice);
    }

    #[test]
    fn fix_then_detect_clean_for_fixable() {
        let src = "use std::io;\nuse std::fmt;\n";
        let fixed = fix(src);
        let v: Vec<_> = detect(&fixed).into_iter().filter(|v| v.fixable).collect();

        assert!(v.is_empty(), "fixable violations remain after fix: {v:?}");
    }

    #[test]
    fn fix_merges_in_nested_mod() {
        let src = "mod inner {\n    use std::io;\n    use std::fmt;\n}\n";
        let fixed = fix(src);
        let v: Vec<_> = detect(&fixed).into_iter().filter(|v| v.fixable).collect();

        assert!(
            v.is_empty(),
            "fixable violations remain in nested mod: {v:?}"
        );
    }
}
