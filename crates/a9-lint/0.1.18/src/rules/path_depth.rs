use std::collections::HashSet;

use syn::{
    File, Ident, Item, ItemMod, ItemUse, Path, UseName, UsePath, UseTree,
    token::{self, PathSep, Semi},
    visit::{Visit, visit_file, visit_item_mod, visit_path},
    visit_mut::{VisitMut, visit_file_mut, visit_path_mut},
};

use crate::{Rule, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl Visit<'_> for PathVisitor<'_> {
    fn visit_item_use(&mut self, _: &ItemUse) {}

    fn visit_item_mod(&mut self, node: &ItemMod) {
        if is_cfg_test(node) {
            return;
        }

        visit_item_mod(self, node);
    }

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
                if entry.leaf != *leaf || !is_redundant_prefix(&entry.prefix, code_prefix) {
                    continue;
                }

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

            if has_sibling_import(self.bound, code_prefix)
                && !self
                    .glob_prefixes
                    .iter()
                    .any(|gp| gp.as_slice() == code_prefix)
                && !self.occupied.contains(leaf)
            {
                let line = path.segments.last().unwrap().ident.span().start().line;

                self.violations.push((
                    line,
                    format!(
                        "sibling of `{leaf}` is imported from `{}`; import `{leaf}` directly",
                        code_prefix.join("::"),
                    ),
                    true,
                ));
                visit_path(self, path);

                return;
            }

            if segs.len() > 3 {
                let line = path.segments.last().unwrap().ident.span().start().line;

                self.violations.push((
                    line,
                    format!(
                        "path `{}` has depth {}; import sub-path directly",
                        path_to_string(path),
                        segs.len(),
                    ),
                    false,
                ));
            }
        }

        visit_path(self, path);
    }
}

impl VisitMut for PathFixer<'_> {
    fn visit_item_use_mut(&mut self, _: &mut ItemUse) {}

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
                if entry.leaf != *leaf || !is_redundant_prefix(&entry.prefix, code_prefix) {
                    continue;
                }

                let last = path.segments.last().unwrap().clone();

                path.segments.clear();
                path.segments.push(last);
                path.leading_colon = None;
                visit_path_mut(self, path);

                return;
            }
        }

        visit_path_mut(self, path);
    }
}

impl Rule for UnitRule {
    fn name(&self) -> &'static str {
        "path-depth"
    }

    fn description(&self) -> &'static str {
        "Flags redundant path prefixes on already-imported names"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &File) -> Vec<Violation> {
        let bound = collect_bound(ast);
        let occupied = collect_occupied_names(ast, &bound);
        let glob_prefixes = collect_glob_prefixes(ast);

        let mut visitor = PathVisitor {
            bound: &bound,
            occupied: &occupied,
            glob_prefixes: &glob_prefixes,
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

    fn fix(&self, mut ast: File) -> File {
        let bound = collect_bound(&ast);

        let mut fixer = PathFixer { bound: &bound };

        visit_file_mut(&mut fixer, &mut ast);

        let bound = collect_bound(&ast);
        let occupied = collect_occupied_names(&ast, &bound);
        let glob_prefixes = collect_glob_prefixes(&ast);
        let new_imports = collect_sibling_imports(&ast, &bound, &occupied, &glob_prefixes);

        for (prefix, leaf) in &new_imports {
            ast.items.push(make_use_item(prefix, leaf));
        }

        let bound = collect_bound(&ast);

        let mut fixer = PathFixer { bound: &bound };

        visit_file_mut(&mut fixer, &mut ast);

        ast
    }
}

impl Visit<'_> for SiblingCollector<'_> {
    fn visit_item_use(&mut self, _: &ItemUse) {}

    fn visit_item_mod(&mut self, node: &ItemMod) {
        if is_cfg_test(node) {
            return;
        }

        visit_item_mod(self, node);
    }

    fn visit_path(&mut self, path: &Path) {
        let segs: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();

        if segs.len() >= 2 {
            if matches!(segs[0].as_str(), "std" | "core" | "alloc") {
                visit_path(self, path);

                return;
            }

            let leaf = &segs[segs.len() - 1];
            let code_prefix = &segs[..segs.len() - 1];
            let already_imported = self
                .bound
                .iter()
                .any(|e| e.leaf == *leaf && is_redundant_prefix(&e.prefix, code_prefix));

            if !already_imported
                && has_sibling_import(self.bound, code_prefix)
                && !self
                    .glob_prefixes
                    .iter()
                    .any(|gp| gp.as_slice() == code_prefix)
                && !self.occupied.contains(leaf)
            {
                self.to_import.insert((code_prefix.to_vec(), leaf.clone()));
            }
        }

        visit_path(self, path);
    }
}

struct BoundEntry {
    prefix: Vec<String>,
    leaf: String,
}

struct PathVisitor<'a> {
    bound: &'a [BoundEntry],
    occupied: &'a HashSet<String>,
    glob_prefixes: &'a [Vec<String>],
    violations: Vec<(usize, String, bool)>,
}

struct PathFixer<'a> {
    bound: &'a [BoundEntry],
}

struct SiblingCollector<'a> {
    bound: &'a [BoundEntry],
    occupied: &'a HashSet<String>,
    glob_prefixes: &'a [Vec<String>],
    to_import: HashSet<(Vec<String>, String)>,
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

fn is_cfg_test(m: &ItemMod) -> bool {
    m.attrs.iter().any(|a| {
        if !a.path().is_ident("cfg") {
            return false;
        }

        a.parse_args::<Ident>().is_ok_and(|id| id == "test")
    })
}

fn path_to_string(path: &Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn collect_bound(ast: &File) -> Vec<BoundEntry> {
    let mut bound = vec![];

    for item in &ast.items {
        let Item::Use(u) = item else { continue };

        collect_bound_entries(&u.tree, vec![], &mut bound);
    }

    bound
}

fn collect_occupied_names(ast: &File, bound: &[BoundEntry]) -> HashSet<String> {
    let mut names: HashSet<String> = bound.iter().map(|e| e.leaf.clone()).collect();

    for item in &ast.items {
        let Some(name) = item_defined_name(item) else {
            continue;
        };

        names.insert(name);
    }

    names
}

fn item_defined_name(item: &Item) -> Option<String> {
    match item {
        Item::Struct(s) => Some(s.ident.to_string()),
        Item::Enum(e) => Some(e.ident.to_string()),
        Item::Fn(f) => Some(f.sig.ident.to_string()),
        Item::Type(t) => Some(t.ident.to_string()),
        Item::Trait(t) => Some(t.ident.to_string()),
        Item::Const(c) => Some(c.ident.to_string()),
        Item::Static(s) => Some(s.ident.to_string()),
        Item::Mod(m) => Some(m.ident.to_string()),
        Item::Union(u) => Some(u.ident.to_string()),
        _ => None,
    }
}

fn has_sibling_import(bound: &[BoundEntry], code_prefix: &[String]) -> bool {
    bound.iter().any(|e| e.prefix == code_prefix)
}

fn collect_glob_prefixes(ast: &File) -> Vec<Vec<String>> {
    let mut result = vec![];

    for item in &ast.items {
        let Item::Use(u) = item else { continue };

        collect_globs(&u.tree, vec![], &mut result);
    }

    result
}

fn collect_globs(tree: &UseTree, prefix: Vec<String>, acc: &mut Vec<Vec<String>>) {
    match tree {
        UseTree::Glob(_) => acc.push(prefix),
        UseTree::Path(p) => {
            let mut next = prefix;

            next.push(p.ident.to_string());
            collect_globs(&p.tree, next, acc);
        }
        UseTree::Group(g) => {
            for item in &g.items {
                collect_globs(item, prefix.clone(), acc);
            }
        }
        _ => {}
    }
}

fn collect_sibling_imports(
    ast: &File,
    bound: &[BoundEntry],
    occupied: &HashSet<String>,
    glob_prefixes: &[Vec<String>],
) -> Vec<(Vec<String>, String)> {
    let mut collector = SiblingCollector {
        bound,
        occupied,
        glob_prefixes,
        to_import: HashSet::new(),
    };

    visit_file(&mut collector, ast);

    collector.to_import.into_iter().collect()
}

fn make_use_item(prefix: &[String], leaf: &str) -> Item {
    let leaf_tree = UseTree::Name(UseName {
        ident: syn::Ident::new(leaf, proc_macro2::Span::call_site()),
    });
    let tree = prefix.iter().rev().fold(leaf_tree, |inner, seg| {
        UseTree::Path(UsePath {
            ident: syn::Ident::new(seg, proc_macro2::Span::call_site()),
            colon2_token: PathSep::default(),
            tree: Box::new(inner),
        })
    });

    Item::Use(ItemUse {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        use_token: token::Use::default(),
        leading_colon: None,
        tree,
        semi_token: Semi::default(),
    })
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
    pub token: my_crate::auth::tokens::Token,
}
"#;
        let v = detect(src);

        assert!(!v.is_empty(), "expected violation for deep path");
        assert!(!v[0].fixable);
    }

    #[test]
    fn depth_three_path_is_ok() {
        let src = r#"
pub struct Foo {
    pub token: my_crate::auth::Token,
}
"#;

        assert!(detect(src).is_empty());
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

    #[test]
    fn sibling_inconsistency_flagged() {
        let src = r#"
use syn::{Item, Path};

fn foo(_: &syn::ItemUse) {}
"#;
        let v = detect(src);

        assert_eq!(v.len(), 1);
        assert!(v[0].fixable);
        assert!(
            v[0].message.contains("sibling"),
            "expected sibling message: {}",
            v[0].message
        );
    }

    #[test]
    fn sibling_no_import_at_all_no_violation() {
        let src = r#"
fn foo(_: syn::ItemUse) {}
"#;

        assert!(detect(src).is_empty());
    }

    #[test]
    fn sibling_collision_suppressed() {
        let src = r#"
use my_crate::auth::Token;

struct Session;

fn foo(_: my_crate::auth::Session) {}
"#;
        let v = detect(src);

        assert!(
            v.is_empty(),
            "collision with local name should suppress sibling violation: {v:?}"
        );
    }

    #[test]
    fn sibling_glob_no_violation() {
        let src = r#"
use syn::*;

fn foo(_: syn::ItemUse) {}
"#;

        assert!(detect(src).is_empty());
    }

    #[test]
    fn fix_adds_sibling_import() {
        let src = r#"
use syn::{Item, Path};

fn foo(_: &syn::ItemUse) {}
"#;
        let fixed = fix(src);

        assert!(
            fixed.contains("ItemUse"),
            "fixed source should contain ItemUse: {fixed}"
        );
        assert!(
            fixed.contains("_: &ItemUse"),
            "qualified path should be shortened in usage: {fixed}"
        );
    }

    #[test]
    fn fix_sibling_then_detect_clean() {
        let src = r#"
use syn::{Item, Path};

fn foo(_: &syn::ItemUse) {}
"#;
        let fixed = fix(src);
        let v: Vec<_> = detect(&fixed).into_iter().filter(|v| v.fixable).collect();

        assert!(v.is_empty(), "fixable violations remain after fix: {v:?}");
    }

    #[test]
    fn fix_sibling_collision_leaves_path() {
        let src = r#"
use my_crate::auth::Token;

struct Session;

fn foo(_: my_crate::auth::Session) {}
"#;
        let fixed = fix(src);

        assert!(
            fixed.contains("my_crate::auth::Session") || fixed.contains("auth::Session"),
            "collision should prevent shortening: {fixed}"
        );
    }
}
