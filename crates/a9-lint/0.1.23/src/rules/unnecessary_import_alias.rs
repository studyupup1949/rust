use std::collections::{HashMap, HashSet};

use syn::{
    File, Ident, Item, ItemUse, Path, UseName, UseTree,
    visit_mut::{VisitMut, visit_file_mut, visit_path_mut},
};

use crate::{Rule, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl Rule for UnitRule {
    fn name(&self) -> &'static str {
        "unnecessary-import-alias"
    }

    fn description(&self) -> &'static str {
        "Flags `use A::B as C` when a direct import or 2-segment qualified path would suffice"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &File) -> Vec<Violation> {
        let entries = collect_rename_entries(ast);
        let all_bound = collect_all_bound_leaves(ast);
        let local_defs = collect_local_defs(ast);

        detect_violations(&entries, &all_bound, &local_defs)
    }

    fn fix(&self, mut ast: File) -> File {
        let entries = collect_rename_entries(&ast);
        let all_bound = collect_all_bound_leaves(&ast);
        let local_defs = collect_local_defs(&ast);
        let fixes = build_fixes(&entries, &all_bound, &local_defs);

        if fixes.is_empty() {
            return ast;
        }

        let mut alias_fixer = AliasFixer { fixes: &fixes };

        visit_file_mut(&mut alias_fixer, &mut ast);

        ast
    }
}

impl VisitMut for AliasFixer<'_> {
    fn visit_item_use_mut(&mut self, node: &mut ItemUse) {
        fix_use_tree(&mut node.tree, self.fixes);
    }

    fn visit_path_mut(&mut self, path: &mut Path) {
        if path.leading_colon.is_none() && path.segments.len() == 1 {
            let ident_str = path.segments[0].ident.to_string();

            if let Some(FixKind::UseOriginal(original)) = self.fixes.get(&ident_str) {
                if ident_str != *original {
                    let span = path.segments[0].ident.span();

                    path.segments[0].ident = Ident::new(original, span);
                }

                return;
            }
        }

        visit_path_mut(self, path);
    }
}

#[derive(Clone)]
enum FixKind {
    UseOriginal(String),
}

struct RenameEntry {
    prefix: Vec<String>,
    original: String,
    alias: String,
    line: usize,
}

struct AliasFixer<'a> {
    fixes: &'a HashMap<String, FixKind>,
}

fn fix_use_tree(tree: &mut UseTree, fixes: &HashMap<String, FixKind>) {
    match tree {
        UseTree::Rename(r) if fixes.contains_key(&r.rename.to_string()) => {
            let ident = r.ident.clone();

            *tree = UseTree::Name(UseName { ident });
        }
        UseTree::Path(p) => fix_use_tree(&mut p.tree, fixes),
        UseTree::Group(g) => {
            for item in &mut g.items {
                fix_use_tree(item, fixes);
            }
        }
        _ => {}
    }
}

fn detect_violations(
    entries: &[RenameEntry],
    all_bound: &HashSet<String>,
    local_defs: &HashSet<String>,
) -> Vec<Violation> {
    let mut violations = vec![];

    for entry in entries {
        let occupied_without_self: HashSet<&str> = all_bound
            .iter()
            .filter(|n| n.as_str() != entry.alias.as_str())
            .map(String::as_str)
            .chain(local_defs.iter().map(String::as_str))
            .collect();
        let other_rename_originals: HashSet<&str> = entries
            .iter()
            .filter(|e| e.alias != entry.alias)
            .map(|e| e.original.as_str())
            .collect();
        let original_conflicts = occupied_without_self.contains(entry.original.as_str())
            || other_rename_originals.contains(entry.original.as_str());

        if !original_conflicts {
            let full_path = format_full_path(&entry.prefix, &entry.original);

            violations.push(Violation {
                line: entry.line,
                message: format!(
                    "unnecessary alias `{}`; use `{}` directly",
                    entry.alias, full_path,
                ),
                fixable: true,
            });

            continue;
        }

        let same_orig: Vec<&RenameEntry> = entries
            .iter()
            .filter(|e| e.original == entry.original)
            .collect();

        if same_orig.len() <= 1 {
            continue;
        }

        let all_distinct = {
            let mut seen = HashSet::new();

            same_orig
                .iter()
                .filter_map(|e| e.prefix.last())
                .all(|s| seen.insert(s.as_str()))
        };

        if all_distinct && let Some(last_seg) = entry.prefix.last() {
            violations.push(Violation {
                line: entry.line,
                message: format!(
                    "unnecessary alias `{}`; use `{}::{}` directly",
                    entry.alias, last_seg, entry.original,
                ),
                fixable: false,
            });
        }
    }

    violations
}

fn build_fixes(
    entries: &[RenameEntry],
    all_bound: &HashSet<String>,
    local_defs: &HashSet<String>,
) -> HashMap<String, FixKind> {
    let mut fixes = HashMap::new();

    for entry in entries {
        let occupied_without_self: HashSet<&str> = all_bound
            .iter()
            .filter(|n| n.as_str() != entry.alias.as_str())
            .map(String::as_str)
            .chain(local_defs.iter().map(String::as_str))
            .collect();
        let other_rename_originals: HashSet<&str> = entries
            .iter()
            .filter(|e| e.alias != entry.alias)
            .map(|e| e.original.as_str())
            .collect();
        let original_conflicts = occupied_without_self.contains(entry.original.as_str())
            || other_rename_originals.contains(entry.original.as_str());

        if original_conflicts {
            continue;
        }

        fixes.insert(
            entry.alias.clone(),
            FixKind::UseOriginal(entry.original.clone()),
        );
    }

    fixes
}

fn format_full_path(prefix: &[String], original: &str) -> String {
    if prefix.is_empty() {
        original.to_string()
    } else {
        format!("{}::{}", prefix.join("::"), original)
    }
}

fn collect_rename_entries(ast: &File) -> Vec<RenameEntry> {
    let mut entries = vec![];

    for item in &ast.items {
        let Item::Use(u) = item else { continue };

        collect_renames_from_tree(&u.tree, vec![], &mut entries);
    }

    entries
}

fn collect_renames_from_tree(tree: &UseTree, prefix: Vec<String>, entries: &mut Vec<RenameEntry>) {
    match tree {
        UseTree::Path(p) => {
            let mut next = prefix;

            next.push(p.ident.to_string());
            collect_renames_from_tree(&p.tree, next, entries);
        }
        UseTree::Group(g) => {
            for item in &g.items {
                collect_renames_from_tree(item, prefix.clone(), entries);
            }
        }
        UseTree::Rename(r) => {
            entries.push(RenameEntry {
                prefix,
                original: r.ident.to_string(),
                alias: r.rename.to_string(),
                line: r.ident.span().start().line,
            });
        }
        _ => {}
    }
}

fn collect_all_bound_leaves(ast: &File) -> HashSet<String> {
    let mut leaves = HashSet::new();

    for item in &ast.items {
        let Item::Use(u) = item else { continue };

        collect_bound_leaves_from_tree(&u.tree, &mut leaves);
    }

    leaves
}

fn collect_bound_leaves_from_tree(tree: &UseTree, leaves: &mut HashSet<String>) {
    match tree {
        UseTree::Path(p) => collect_bound_leaves_from_tree(&p.tree, leaves),
        UseTree::Group(g) => {
            for item in &g.items {
                collect_bound_leaves_from_tree(item, leaves);
            }
        }
        UseTree::Name(n) if n.ident != "self" => {
            leaves.insert(n.ident.to_string());
        }
        UseTree::Rename(r) => {
            leaves.insert(r.rename.to_string());
        }
        _ => {}
    }
}

fn collect_local_defs(ast: &File) -> HashSet<String> {
    let mut defs = HashSet::new();

    for item in &ast.items {
        let Some(name) = item_defined_name(item) else {
            continue;
        };

        defs.insert(name);
    }

    defs
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

#[cfg(test)]
mod tests {
    use a9_prettyplease::unparse;

    use super::*;
    use crate::UnitRule as UnitRuleTrait;

    fn detect(src: &str) -> Vec<Violation> {
        let ast = syn::parse_file(src).unwrap();

        UnitRule.detect(&ast)
    }

    fn fix(src: &str) -> String {
        let ast = syn::parse_file(src).unwrap();

        unparse(&UnitRule.fix(ast))
    }

    #[test]
    fn trivial_self_alias_flagged() {
        let v = detect(
            r#"
use foo::Bar as Bar;
fn f() -> Bar { todo!() }
"#,
        );

        assert!(!v.is_empty(), "self-alias should be flagged: {v:?}");
        assert!(v[0].message.contains("`Bar`"));
        assert!(v[0].fixable);
    }

    #[test]
    fn unique_alias_flagged() {
        let v = detect(
            r#"
use std::io::Error as IoError;
fn f() -> IoError { todo!() }
"#,
        );

        assert!(!v.is_empty(), "unique alias should be flagged: {v:?}");
        assert!(v[0].message.contains("`IoError`"));
        assert!(v[0].fixable);
    }

    #[test]
    fn alias_allowed_when_original_conflicts_with_local() {
        let v = detect(
            r#"
use std::io::Error as IoError;
struct Error;
fn f() -> IoError { todo!() }
"#,
        );

        assert!(v.is_empty(), "alias needed due to local def: {v:?}");
    }

    #[test]
    fn alias_allowed_when_original_conflicts_with_import() {
        let v = detect(
            r#"
use std::io::Error as IoError;
use other::Error;
fn f() -> IoError { todo!() }
"#,
        );

        assert!(v.is_empty(), "alias needed due to import conflict: {v:?}");
    }

    #[test]
    fn two_seg_disambiguation_flagged() {
        let v = detect(
            r#"
use std::io::Error as IoError;
use std::fmt::Error as FmtError;
"#,
        );

        assert_eq!(v.len(), 2, "both aliases should be flagged: {v:?}");
        assert!(v.iter().any(|e| e.message.contains("`IoError`")));
        assert!(v.iter().any(|e| e.message.contains("`FmtError`")));
        assert!(v.iter().all(|e| !e.fixable));
    }

    #[test]
    fn two_seg_clash_allowed() {
        let v = detect(
            r#"
use crate::http::error::Error as HttpError;
use crate::db::error::Error as DbError;
"#,
        );

        assert!(v.is_empty(), "last segs clash, alias needed: {v:?}");
    }

    #[test]
    fn fix_trivial_self_alias() {
        let result = fix(r#"
use foo::Bar as Bar;
fn f() -> Bar { todo!() }
"#);

        assert!(
            !result.contains("as Bar"),
            "as Bar should be removed: {result}"
        );
        assert!(result.contains("use foo::Bar"), "import kept: {result}");
    }

    #[test]
    fn fix_unique_alias_renames_usages() {
        let result = fix(r#"
use std::io::Error as IoError;
fn f() -> IoError { todo!() }
"#);

        assert!(
            !result.contains("IoError"),
            "alias removed from code: {result}"
        );
        assert!(
            result.contains("use std::io::Error"),
            "import kept without alias: {result}"
        );
        assert!(result.contains("-> Error"), "usage renamed: {result}");
    }

    #[test]
    fn fix_leaves_conflicting_alias_intact() {
        let src = r#"
use std::io::Error as IoError;
struct Error;
fn f() -> IoError { todo!() }
"#;
        let result = fix(src);

        assert!(
            result.contains("as IoError"),
            "conflicting alias untouched: {result}"
        );
    }
}
