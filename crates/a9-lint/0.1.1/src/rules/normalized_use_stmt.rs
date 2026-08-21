use std::collections::HashMap;

use proc_macro2::Span;
use syn::{Item, UseTree};

use super::Rule;

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

    fn check(&self, file: &syn::File) -> Vec<(usize, String)> {
        let mut violations = vec![];

        // ── Cross-statement check ────────────────────────────────────────────
        // Multiple separate `use root::...` statements with the same root
        // should be merged into one `use root::{...}`.
        let mut root_first_line: HashMap<String, usize> = HashMap::new();
        let mut root_count: HashMap<String, usize> = HashMap::new();

        for item in &file.items {
            let Item::Use(u) = item else { continue };
            let line = span_line(u.use_token.span);

            if let Some(root) = first_segment(&u.tree) {
                *root_count.entry(root.clone()).or_insert(0) += 1;
                root_first_line.entry(root).or_insert(line);
            }
        }

        let mut cross: Vec<(String, usize)> = root_count
            .into_iter()
            .filter(|(_, c)| *c > 1)
            .map(|(root, _)| (root_first_line[&root], root))
            .map(|(line, root)| (root, line))
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
        // Inside a single `use root::{...}`, sibling entries that share a
        // first segment should be merged one level deeper.
        for item in &file.items {
            let Item::Use(u) = item else { continue };
            let line = span_line(u.use_token.span);
            check_within_group(&u.tree, "", &mut violations, line);
        }

        violations
    }
}
