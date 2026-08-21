use syn::{
    Item, Path, UseTree,
    visit::{Visit, visit_file, visit_path},
    visit_mut::{VisitMut, visit_file_mut, visit_path_mut},
};

use crate::{Rule as RuleTrait, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl Visit<'_> for PathVisitor<'_> {
    fn visit_item_use(&mut self, _: &syn::ItemUse) {}

    fn visit_path(&mut self, path: &Path) {
        let segs: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();

        if segs.len() >= 2 {
            if matches!(segs[0].as_str(), "std" | "core" | "alloc") {
                visit_path(self, path);

                return;
            }

            let leaf = &segs[segs.len() - 1];
            let code_prefix = &segs[..segs.len() - 1];

            for entry in self.bound {
                if entry.leaf == *leaf && is_redundant_prefix(&entry.prefix, code_prefix) {
                    let line = path.segments.last().unwrap().ident.span().start().line;

                    self.violations.push((
                        line,
                        format!(
                            "`{leaf}` is already imported; write `{leaf}` instead of `{}`",
                            path_to_string(path),
                        ),
                        true,
                    ));
                    visit_path(self, path);

                    return;
                }
            }

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
                    false,
                ));
            }
        }

        visit_path(self, path);
    }
}

impl VisitMut for PathFixer<'_> {
    fn visit_item_use_mut(&mut self, _: &mut syn::ItemUse) {}

    fn visit_path_mut(&mut self, path: &mut Path) {
        let segs: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();

        if segs.len() >= 2 {
            if matches!(segs[0].as_str(), "std" | "core" | "alloc") {
                visit_path_mut(self, path);

                return;
            }

            let leaf = &segs[segs.len() - 1];
            let code_prefix = &segs[..segs.len() - 1];

            for entry in self.bound {
                if entry.leaf == *leaf && is_redundant_prefix(&entry.prefix, code_prefix) {
                    let last = path.segments.last().unwrap().clone();

                    path.segments.clear();
                    path.segments.push(last);
                    path.leading_colon = None;
                    visit_path_mut(self, path);

                    return;
                }
            }
        }

        visit_path_mut(self, path);
    }
}

impl RuleTrait for UnitRule {
    fn name(&self) -> &'static str {
        "path-depth"
    }

    fn description(&self) -> &'static str {
        "Flags redundant path prefixes on already-imported names"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &syn::File) -> Vec<Violation> {
        let bound = collect_bound(ast);

        let mut visitor = PathVisitor {
            bound: &bound,
            violations: vec![],
        };

        visit_file(&mut visitor, ast);

        visitor
            .violations
            .into_iter()
            .map(|(line, message, fixable)| Violation {
                line,
                message,
                fixable,
            })
            .collect()
    }

    fn fix(&self, mut ast: syn::File) -> syn::File {
        let bound = collect_bound(&ast);

        let mut fixer = PathFixer { bound: &bound };

        visit_file_mut(&mut fixer, &mut ast);

        ast
    }
}

struct BoundEntry {
    prefix: Vec<String>,
    leaf: String,
}

struct PathVisitor<'a> {
    bound: &'a [BoundEntry],
    violations: Vec<(usize, String, bool)>,
}

struct PathFixer<'a> {
    bound: &'a [BoundEntry],
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

fn collect_bound(ast: &syn::File) -> Vec<BoundEntry> {
    let mut bound = vec![];

    for item in &ast.items {
        let Item::Use(u) = item else { continue };

        collect_bound_entries(&u.tree, vec![], &mut bound);
    }

    bound
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
    fn deep_unimported_path_flagged() {
        let src = r#"
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Foo {
    pub token: my_crate::auth::Token,
}
"#;
        let v = detect(src);

        assert!(!v.is_empty(), "expected violation for deep path");
        assert!(!v[0].fixable);
    }

    #[test]
    fn depth_two_path_is_ok() {
        let src = r#"
pub struct Foo {
    pub token: my_crate::Token,
}
"#;

        assert!(detect(src).is_empty());
    }

    #[test]
    fn std_deep_path_is_ok() {
        let src = r#"
fn foo() {
    let _ = std::collections::HashMap::<String, String>::new();
}
"#;

        assert!(detect(src).is_empty());
    }

    #[test]
    fn redundant_requalification_flagged() {
        let src = r#"
use my_crate::auth::Token;

pub struct Foo {
    pub token: my_crate::auth::Token,
}
"#;
        let v = detect(src);

        assert!(!v.is_empty(), "expected redundant qualification violation");
        assert!(v[0].fixable);
    }

    #[test]
    fn fix_removes_redundant_qualification() {
        let src = r#"
use my_crate::auth::Token;

pub struct Foo {
    pub token: my_crate::auth::Token,
}
"#;
        let fixed = fix(src);

        assert!(
            fixed.contains("pub token: Token"),
            "redundant path in field should be simplified: {fixed}"
        );
    }

    #[test]
    fn fix_idempotent() {
        let src = r#"
use my_crate::auth::Token;

pub struct Foo {
    pub token: my_crate::auth::Token,
}
"#;
        let once = fix(src);
        let twice = fix(&once);

        assert_eq!(once, twice);
    }

    #[test]
    fn fix_then_detect_clean_for_fixable() {
        let src = r#"
use my_crate::auth::Token;

pub struct Foo {
    pub token: my_crate::auth::Token,
}
"#;
        let fixed = fix(src);
        let v: Vec<_> = detect(&fixed).into_iter().filter(|v| v.fixable).collect();

        assert!(v.is_empty(), "fixable violations remain: {v:?}");
    }
}
