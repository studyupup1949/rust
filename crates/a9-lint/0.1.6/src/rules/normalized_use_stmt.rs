use std::collections::HashMap;

use proc_macro2::Span;
use syn::{Attribute, Item, Meta, UseTree};

use super::{
    RsRule, Rule,
    common::{merge_sub_trees, use_tree_to_str, vis_to_str},
};

pub struct NormalizedUseStmt;

fn span_line(span: Span) -> usize {
    span.start().line
}

/// Return the first path segment of a UseTree (the crate / root name).
fn first_segment(tree: &UseTree) -> Option<String> {
    match tree {
        UseTree::Path(p) => Some(p.ident.to_string()),
        UseTree::Name(n) => Some(n.ident.to_string()),
        UseTree::Rename(r) => Some(r.ident.to_string()),
        UseTree::Group(g) => g.items.first().and_then(first_segment),
        UseTree::Glob(_) => None,
    }
}

/// Check that no sibling items inside a single `use { }` group share a first
/// segment — those should be merged one level deeper.
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

impl Rule for NormalizedUseStmt {
    fn name(&self) -> &'static str {
        "normalized-use-stmt"
    }

    fn description(&self) -> &'static str {
        "Use statements sharing the same root must be merged; duplicate root segments are flagged"
    }
}

impl RsRule for NormalizedUseStmt {
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
        let lines: Vec<&str> = source.lines().collect();
        let total = lines.len();
        let fixed = fix_use_items(&file.items, &lines, total);
        let mut out = fixed.join("\n");
        if source.ends_with('\n') {
            out.push('\n');
        }
        Ok(out)
    }
}

/// A canonical string representation of `#[cfg(...)]` attributes on an item.
/// Items with different fingerprints belong to different cfg groups and should
/// not be compared against each other for merge violations.
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

fn check_items(items: &[Item]) -> Vec<(usize, String)> {
    let mut violations = vec![];

    // ── Cross-statement check ────────────────────────────────────────────
    // Key: (cfg_fingerprint, root). Use items in different cfg groups are
    // independent and must not be flagged as needing to be merged.
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
        ));
    }

    // ── Within-group check ───────────────────────────────────────────────
    for item in items {
        let Item::Use(u) = item else { continue };
        let line = span_line(u.use_token.span);
        check_within_group(&u.tree, "", &mut violations, line);
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

/// Merge cross-statement use items with the same (cfg, root) key into one
/// statement; also normalize within-group duplicates.  Returns the source
/// lines of the file with the merged statements substituted in.
fn fix_use_items(items: &[Item], lines: &[&str], total: usize) -> Vec<String> {
    // Build mapping: (cfg_fp, root, vis) → sub-tree strings for each use item.
    // We process items in order; for each group the first item's line range is
    // replaced with the merged statement, subsequent items are dropped.
    let mut group_sub: HashMap<(String, String, String), Vec<String>> = HashMap::new();
    let mut group_first_line: HashMap<(String, String, String), usize> = HashMap::new();
    let mut group_last_line: HashMap<(String, String, String), usize> = HashMap::new();

    // Helper: compute the last line of a use item (semicolon line).
    let use_last_line = |u: &syn::ItemUse| u.semi_token.span.start().line;

    for item in items {
        let Item::Use(u) = item else { continue };
        let cfg = cfg_fingerprint(&u.attrs);
        let vis = vis_to_str(&u.vis).to_string();
        if let Some(root) = first_segment(&u.tree) {
            let sub = use_tree_to_str(sub_tree_after_root(&u.tree));
            let key = (cfg, root, vis);
            group_sub.entry(key.clone()).or_default().push(sub);
            let first_line = span_line(u.use_token.span);
            group_first_line.entry(key.clone()).or_insert(first_line);
            group_last_line.insert(key, use_last_line(u));
        }
    }

    // Build set of lines to drop and a replacement map line → merged statement.
    let mut drop_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut replace: HashMap<usize, String> = HashMap::new();

    for (key, subs) in &group_sub {
        if subs.len() < 2 {
            continue;
        }
        let (_, root, vis) = key;
        let merged = merge_sub_trees(subs.clone());
        let first = group_first_line[key];
        let last = group_last_line[key];
        // Mark all lines of the first use item for replacement with merged stmt.
        replace.insert(first, format!("{vis}use {root}::{merged};"));
        // Mark all lines of the first use item (first..=last) except first as drop.
        for l in (first + 1)..=last {
            drop_lines.insert(l);
        }
        // Mark all subsequent use items with the same key for dropping.
        let mut seen_first = false;
        for item in items {
            let Item::Use(u) = item else { continue };
            let cfg2 = cfg_fingerprint(&u.attrs);
            let vis2 = vis_to_str(&u.vis).to_string();
            let root2 = first_segment(&u.tree);
            if (cfg2, root2.unwrap_or_default(), vis2) == *key {
                if !seen_first {
                    seen_first = true;
                } else {
                    let start = span_line(u.use_token.span);
                    let end = use_last_line(u);
                    for l in start..=end {
                        drop_lines.insert(l);
                    }
                }
            }
        }
    }

    let mut result: Vec<String> = Vec::with_capacity(total);
    for (i, &line) in lines.iter().enumerate() {
        let ln = i + 1;
        if let Some(merged) = replace.get(&ln) {
            result.push(merged.clone());
        } else if !drop_lines.contains(&ln) {
            result.push(line.to_string());
        }
    }
    result
}

fn sub_tree_after_root(tree: &UseTree) -> &UseTree {
    if let UseTree::Path(p) = tree {
        &p.tree
    } else {
        tree
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<(usize, String)> {
        let file = syn::parse_file(src).unwrap();
        NormalizedUseStmt.check(&file, src)
    }

    #[test]
    fn top_level_duplicate_root_is_violation() {
        let src = r#"
use std::io;
use std::fmt;
"#;
        assert_eq!(check(src).len(), 1);
    }

    #[test]
    fn violation_inside_nested_mod_is_detected() {
        // Two `use std::` statements inside an inline mod — should be flagged.
        let src = r#"
mod inner {
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

    #[test]
    fn cfg_gated_groups_not_merged_no_violation() {
        // Two `use syn::...` in different cfg groups — must NOT be flagged.
        let src = r#"
use syn::ItemUse;

#[cfg(feature = "theta")]
use syn::{Attribute, Meta};
"#;
        assert!(
            check(src).is_empty(),
            "cfg-gated groups should not require merging"
        );
    }

    #[test]
    fn same_cfg_group_duplicate_root_is_violation() {
        // Two uses with the same cfg fingerprint must still be flagged.
        let src = r#"
#[cfg(feature = "theta")]
use syn::Attribute;

#[cfg(feature = "theta")]
use syn::Meta;
"#;
        assert_eq!(check(src).len(), 1);
    }
}
