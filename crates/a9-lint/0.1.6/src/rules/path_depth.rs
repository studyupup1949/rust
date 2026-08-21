use syn::{
    Item, Path, UseTree,
    visit::{Visit, visit_file, visit_path},
};

use super::{RsRule, Rule};

pub struct PathDepth;

/// A bound import entry: the prefix path segments and the imported leaf name.
/// E.g. `use crate::actor::Actor` -> (["crate","actor"], "Actor")
struct BoundEntry {
    prefix: Vec<String>,
    leaf: String,
}

fn collect_bound_entries(tree: &UseTree, prefix: Vec<String>, acc: &mut Vec<BoundEntry>) {
    match tree {
        UseTree::Path(p) => {
            let mut next = prefix;
            next.push(p.ident.to_string());
            collect_bound_entries(&p.tree, next, acc);
        }
        UseTree::Group(g) => {
            for item in &g.items {
                collect_bound_entries(item, prefix.clone(), acc);
            }
        }
        UseTree::Name(n) => {
            let leaf = n.ident.to_string();
            if leaf != "self" {
                acc.push(BoundEntry { prefix, leaf });
            }
        }
        UseTree::Rename(r) => {
            acc.push(BoundEntry {
                prefix,
                leaf: r.rename.to_string(),
            });
        }
        UseTree::Glob(_) => {}
    }
}

/// Returns true if `code_prefix` is a non-empty suffix of `bound_prefix`.
/// This identifies when a caller qualifies an imported name with part of its import path.
fn is_redundant_prefix(bound_prefix: &[String], code_prefix: &[String]) -> bool {
    if code_prefix.is_empty() || code_prefix.len() > bound_prefix.len() {
        return false;
    }
    let skip = bound_prefix.len() - code_prefix.len();
    bound_prefix[skip..] == *code_prefix
}

fn path_to_string(path: &Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

struct PathVisitor<'a> {
    bound: &'a [BoundEntry],
    violations: Vec<(usize, String)>,
}

impl Visit<'_> for PathVisitor<'_> {
    // Skip use items — their paths define the bound set, not violations.
    fn visit_item_use(&mut self, _: &syn::ItemUse) {}

    fn visit_path(&mut self, path: &Path) {
        let segs: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();

        if segs.len() >= 2 {
            // Exception: std/core/alloc qualified paths are always permitted at call sites.
            if matches!(segs[0].as_str(), "std" | "core" | "alloc") {
                visit_path(self, path);
                return;
            }

            let leaf = &segs[segs.len() - 1];
            let code_prefix = &segs[..segs.len() - 1];

            // Check 1: redundant re-qualification of an already-imported name.
            for entry in self.bound {
                if entry.leaf == *leaf && is_redundant_prefix(&entry.prefix, code_prefix) {
                    let line = path.segments.last().unwrap().ident.span().start().line;
                    self.violations.push((
                        line,
                        format!(
                            "`{}` is already imported; write `{}` instead of `{}`",
                            leaf,
                            leaf,
                            path_to_string(path),
                        ),
                    ));
                    visit_path(self, path);
                    return;
                }
            }

            // Check 2: deep unimported path (depth > 2 segments).
            if segs.len() > 2 {
                let line = path.segments.last().unwrap().ident.span().start().line;
                self.violations.push((
                    line,
                    format!(
                        "path `{}` has depth {}; import `{}` and use it directly",
                        path_to_string(path),
                        segs.len(),
                        leaf,
                    ),
                ));
            }
        }

        visit_path(self, path);
    }
}

impl Rule for PathDepth {
    fn name(&self) -> &'static str {
        "path-depth"
    }

    fn description(&self) -> &'static str {
        "Flags redundant path prefixes on already-imported names"
    }
}

impl RsRule for PathDepth {
    fn check(&self, file: &syn::File, _source: &str) -> Vec<(usize, String)> {
        let mut bound = vec![];
        for item in &file.items {
            let Item::Use(u) = item else { continue };
            collect_bound_entries(&u.tree, vec![], &mut bound);
        }

        let mut visitor = PathVisitor {
            bound: &bound,
            violations: vec![],
        };
        visit_file(&mut visitor, file);
        visitor.violations
    }

    fn has_fixer(&self) -> bool {
        true
    }

    // Fix redundant re-qualifications only (e.g. `crate::foo::Bar` → `Bar`).
    // Deep unimported paths are left for the developer to add an import.
    fn try_fix(&self, source: &str, file: syn::File) -> Result<String, String> {
        let violations = self.check(&file, source);
        let redundant: Vec<(usize, String)> = violations
            .into_iter()
            .filter(|(_, msg)| msg.contains("is already imported"))
            .collect();
        if redundant.is_empty() {
            return Err("no redundant-qualification violations to fix".into());
        }
        // For each redundant violation the message is:
        // "`Leaf` is already imported; write `Leaf` instead of `full::path::Leaf`"
        // We extract the full path and replace it with just the leaf on that line.
        let lines: Vec<&str> = source.lines().collect();
        let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        for (line_num, msg) in redundant {
            // Parse "`Leaf` is already imported; write `Leaf` instead of `full::path`"
            let instead_of = msg
                .split("instead of `")
                .nth(1)
                .and_then(|s| s.strip_suffix('`'));
            let write_just = msg
                .split("write `")
                .nth(1)
                .and_then(|s| s.split('`').next());
            if let (Some(full_path), Some(leaf)) = (instead_of, write_just)
                && line_num > 0
                && line_num <= out.len()
            {
                out[line_num - 1] = out[line_num - 1].replace(full_path, leaf);
            }
        }
        let mut new_src = out.join("\n");
        if source.ends_with('\n') {
            new_src.push('\n');
        }
        Ok(new_src)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Vec<(usize, String)> {
        let file = syn::parse_file(src).unwrap();
        PathDepth.check(&file, src)
    }

    #[test]
    fn deep_unimported_path_should_be_flagged() {
        // my_crate::auth::Token used inline — depth 3, no import.
        let src = r#"
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Foo {
    pub token: my_crate::auth::Token,
}
"#;
        let v = check(src);
        assert!(
            !v.is_empty(),
            "Expected violation for deep unimported path `my_crate::auth::Token`"
        );
    }

    #[test]
    fn depth_two_path_is_ok() {
        // my_crate::Token — depth 2, should be fine.
        let src = r#"
pub struct Foo {
    pub token: my_crate::Token,
}
"#;
        let v = check(src);
        assert!(v.is_empty(), "Depth-2 path should not be flagged: {v:?}");
    }

    #[test]
    fn std_deep_path_is_ok() {
        let src = r#"
fn foo() {
    let _ = std::collections::HashMap::<String, String>::new();
}
"#;
        let v = check(src);
        assert!(v.is_empty(), "std paths should be exempt: {v:?}");
    }

    #[test]
    fn redundant_requalification_flagged() {
        // Token is imported, then used fully qualified — should be flagged.
        let src = r#"
use my_crate::auth::Token;

pub struct Foo {
    pub token: my_crate::auth::Token,
}
"#;
        let v = check(src);
        assert!(
            !v.is_empty(),
            "Expected violation for re-qualified already-imported `Token`"
        );
    }
}
