use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream};
use syn::{
    File, Item, Meta, TraitBound, Type, TypePath, Visibility,
    visit::{self, Visit},
};

use crate::{Rule, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl<'ast> Visit<'ast> for TypeRefCollector {
    fn visit_type_path(&mut self, tp: &'ast TypePath) {
        if let Some(seg) = tp.path.segments.first()
            && tp.path.segments.len() == 1
        {
            self.refs.insert(seg.ident.to_string());
        }

        visit::visit_type_path(self, tp);
    }

    fn visit_trait_bound(&mut self, tb: &'ast TraitBound) {
        if let Some(seg) = tb.path.segments.first()
            && tb.path.segments.len() == 1
        {
            self.refs.insert(seg.ident.to_string());
        }

        visit::visit_trait_bound(self, tb);
    }
}

impl Rule for UnitRule {
    fn name(&self) -> &'static str {
        "module-item-order"
    }

    fn description(&self) -> &'static str {
        "top-level items must follow group order with pub before private, topologically sorted within groups"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &File) -> Vec<Violation> {
        check_items(&ast.items)
    }

    fn fix(&self, mut ast: File) -> File {
        fix_items(&mut ast.items);

        ast
    }
}

/// Visibility tier used for secondary ordering within a kind-group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum VisLevel {
    Pub,
    Crate,
    Restricted,
    Private,
}

/// The 18 ordering groups from top to bottom.
/// Primary sort: kind (traits → types → fns).
/// Secondary sort within each kind: visibility (pub → pub(crate) → restricted → private).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
enum Group {
    ExternCrate = 0,
    Use = 1,
    Mod = 2,
    Macro = 3,
    ConstStatic = 4,
    PubTrait = 5,
    PubCrateTrait = 6,
    RestrictedTrait = 7,
    PrivTrait = 8,
    PubType = 9,
    PubCrateType = 10,
    RestrictedType = 11,
    PrivType = 12,
    PubFn = 13,
    PubCrateFn = 14,
    RestrictedFn = 15,
    PrivFn = 16,
    Test = 17,
}

/// Collect all type-name references from an item (struct fields, fn signatures, trait bounds, etc.)
struct TypeRefCollector {
    refs: HashSet<String>,
}

fn vis_level(vis: &Visibility) -> VisLevel {
    match vis {
        Visibility::Public(_) => VisLevel::Pub,
        Visibility::Restricted(r) => {
            if r.path.is_ident("crate") {
                VisLevel::Crate
            } else {
                VisLevel::Restricted
            }
        }
        Visibility::Inherited => VisLevel::Private,
    }
}

fn is_test_mod(item: &Item) -> bool {
    let Item::Mod(m) = item else { return false };

    m.content.is_some()
        && m.attrs.iter().any(|a| {
            if let Meta::List(list) = &a.meta {
                list.path.is_ident("cfg") && list.tokens.to_string().contains("test")
            } else {
                false
            }
        })
}

const fn item_vis(item: &Item) -> Option<&Visibility> {
    match item {
        Item::Struct(s) => Some(&s.vis),
        Item::Enum(e) => Some(&e.vis),
        Item::Trait(t) => Some(&t.vis),
        Item::TraitAlias(t) => Some(&t.vis),
        Item::Type(t) => Some(&t.vis),
        Item::Union(u) => Some(&u.vis),
        Item::Fn(f) => Some(&f.vis),
        Item::Mod(m) => Some(&m.vis),
        Item::Const(c) => Some(&c.vis),
        Item::Static(s) => Some(&s.vis),
        _ => None,
    }
}

fn classify(item: &Item) -> Group {
    if is_test_mod(item) {
        return Group::Test;
    }

    match item {
        Item::ExternCrate(_) => Group::ExternCrate,
        Item::Use(_) => Group::Use,
        Item::Mod(m) if m.content.is_none() => Group::Mod,
        Item::Macro(_) => Group::Macro,
        Item::Const(_) | Item::Static(_) => Group::ConstStatic,
        Item::Trait(_) | Item::TraitAlias(_) => {
            match item_vis(item).map(vis_level).unwrap_or(VisLevel::Private) {
                VisLevel::Pub => Group::PubTrait,
                VisLevel::Crate => Group::PubCrateTrait,
                VisLevel::Restricted => Group::RestrictedTrait,
                VisLevel::Private => Group::PrivTrait,
            }
        }
        Item::Struct(_) | Item::Enum(_) | Item::Type(_) | Item::Union(_) | Item::Mod(_) => {
            match item_vis(item).map(vis_level).unwrap_or(VisLevel::Private) {
                VisLevel::Pub => Group::PubType,
                VisLevel::Crate => Group::PubCrateType,
                VisLevel::Restricted => Group::RestrictedType,
                VisLevel::Private => Group::PrivType,
            }
        }
        Item::Impl(_) => Group::PubType,
        Item::Fn(_) => match item_vis(item).map(vis_level).unwrap_or(VisLevel::Private) {
            VisLevel::Pub => Group::PubFn,
            VisLevel::Crate => Group::PubCrateFn,
            VisLevel::Restricted => Group::RestrictedFn,
            VisLevel::Private => Group::PrivFn,
        },
        _ => Group::PrivType,
    }
}

const fn group_label(group: Group) -> &'static str {
    match group {
        Group::ExternCrate => "extern crate",
        Group::Use => "use",
        Group::Mod => "mod declaration",
        Group::Macro => "macro",
        Group::ConstStatic => "const/static",
        Group::PubTrait => "pub trait",
        Group::PubCrateTrait => "pub(crate) trait",
        Group::RestrictedTrait => "pub(super)/pub(in) trait",
        Group::PrivTrait => "private trait",
        Group::PubType => "pub type/struct/enum",
        Group::PubCrateType => "pub(crate) type/struct/enum",
        Group::RestrictedType => "pub(super)/pub(in) type/struct/enum",
        Group::PrivType => "private type/struct/enum",
        Group::PubFn => "pub fn",
        Group::PubCrateFn => "pub(crate) fn",
        Group::RestrictedFn => "pub(super)/pub(in) fn",
        Group::PrivFn => "private fn",
        Group::Test => "#[cfg(test)] mod tests",
    }
}

fn item_line(item: &Item) -> usize {
    let span = match item {
        Item::ExternCrate(i) => i.extern_token.span,
        Item::Use(i) => i.use_token.span,
        Item::Mod(i) => i.mod_token.span,
        Item::Fn(i) => i.sig.fn_token.span,
        Item::Struct(i) => i.struct_token.span,
        Item::Enum(i) => i.enum_token.span,
        Item::Trait(i) => i.trait_token.span,
        Item::Impl(i) => i.impl_token.span,
        Item::Const(i) => i.const_token.span,
        Item::Static(i) => i.static_token.span,
        Item::Type(i) => i.type_token.span,
        Item::Macro(i) => i
            .mac
            .path
            .segments
            .first()
            .map_or_else(Span::call_site, |s| s.ident.span()),
        _ => Span::call_site(),
    };

    span.start().line
}

/// Get the defining name of an item (struct/enum/trait/type/fn/const/static name).
fn item_name(item: &Item) -> Option<String> {
    match item {
        Item::Struct(s) => Some(s.ident.to_string()),
        Item::Enum(e) => Some(e.ident.to_string()),
        Item::Trait(t) => Some(t.ident.to_string()),
        Item::TraitAlias(t) => Some(t.ident.to_string()),
        Item::Type(t) => Some(t.ident.to_string()),
        Item::Union(u) => Some(u.ident.to_string()),
        Item::Fn(f) => Some(f.sig.ident.to_string()),
        Item::Const(c) => Some(c.ident.to_string()),
        Item::Static(s) => Some(s.ident.to_string()),
        Item::Mod(m) => Some(m.ident.to_string()),
        _ => None,
    }
}

fn collect_type_refs(item: &Item) -> HashSet<String> {
    let mut collector = TypeRefCollector {
        refs: HashSet::new(),
    };

    collector.visit_item(item);

    if let Item::Impl(i) = item
        && let Some((_, path, _)) = &i.trait_
        && path.segments.len() == 1
    {
        collector.refs.insert(path.segments[0].ident.to_string());
    }

    collector.refs
}

/// Get the "self type" name for an impl block (for grouping impl with its struct).
fn impl_self_type_name(item: &Item) -> Option<String> {
    let Item::Impl(i) = item else { return None };

    if let Type::Path(tp) = &*i.self_ty {
        tp.path.segments.last().map(|s| s.ident.to_string())
    } else {
        None
    }
}

fn topo_sort(items: &mut [Item], defined_names: &HashMap<String, usize>) {
    let count = items.len();

    if count <= 1 {
        return;
    }

    let mut in_edges: Vec<HashSet<usize>> = vec![HashSet::new(); count];

    for (i, item) in items.iter().enumerate() {
        let mut refs = collect_type_refs(item);

        if let Some(name) = impl_self_type_name(item) {
            refs.insert(name);
        }

        for r in &refs {
            if let Some(&j) = defined_names.get(r)
                && j != i
            {
                in_edges[i].insert(j);
            }
        }
    }

    let mut out_edges: Vec<HashSet<usize>> = vec![HashSet::new(); count];
    let mut in_degree: Vec<usize> = vec![0; count];

    for (i, deps) in in_edges.iter().enumerate() {
        in_degree[i] = deps.len();

        for &j in deps {
            out_edges[j].insert(i);
        }
    }

    let mut queue: Vec<usize> = (0..count).filter(|&i| in_degree[i] == 0).collect();
    let mut order: Vec<usize> = Vec::with_capacity(count);

    while let Some(idx) = queue.first().copied() {
        queue.remove(0);
        order.push(idx);

        let mut freed: Vec<usize> = out_edges[idx]
            .iter()
            .filter(|&&dep| {
                in_degree[dep] -= 1;

                in_degree[dep] == 0
            })
            .copied()
            .collect();

        freed.sort_unstable();

        for fi in freed {
            let pos = queue.partition_point(|&x| x < fi);

            queue.insert(pos, fi);
        }
    }

    if order.len() < count {
        for i in 0..count {
            if order.contains(&i) {
                continue;
            }

            order.push(i);
        }
    }

    let mut sorted: Vec<Option<Item>> = items
        .iter_mut()
        .map(|i| Some(std::mem::replace(i, Item::Verbatim(TokenStream::default()))))
        .collect();

    for (dest, &src) in order.iter().enumerate() {
        items[dest] = sorted[src].take().unwrap();
    }
}

fn check_items(items: &[Item]) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut max_group = Group::ExternCrate;
    let mut max_group_line: usize = 0;

    for item in items {
        let group = classify(item);
        let line = item_line(item);

        if group < max_group {
            violations.push(Violation {
                line,
                message: format!(
                    "`{}` must appear before `{}` (line {max_group_line})",
                    group_label(group),
                    group_label(max_group),
                ),
                fixable: true,
            });
        }

        if group > max_group {
            max_group = group;
            max_group_line = line;
        }

        if let Item::Mod(m) = item
            && !is_test_mod(item)
            && let Some((_, content)) = &m.content
        {
            violations.extend(check_items(content));
        }
    }

    let mut group_names: HashMap<Group, HashMap<String, usize>> = HashMap::new();
    let mut group_idx: HashMap<Group, usize> = HashMap::new();

    for item in items {
        let group = classify(item);
        let local_idx = *group_idx.entry(group).and_modify(|v| *v += 1).or_insert(0);

        let Some(name) = item_name(item) else {
            continue;
        };

        group_names
            .entry(group)
            .or_default()
            .insert(name, local_idx);
    }

    group_idx.clear();

    for item in items {
        let group = classify(item);
        let local_idx = *group_idx.entry(group).and_modify(|v| *v += 1).or_insert(0);
        let names = group_names.get(&group);

        let mut refs = collect_type_refs(item);

        if let Some(name) = impl_self_type_name(item) {
            refs.insert(name);
        }

        for r in &refs {
            if let Some(names) = names
                && let Some(&dep_idx) = names.get(r.as_str())
                && dep_idx > local_idx
            {
                violations.push(Violation {
                    line: item_line(item),
                    message: format!(
                        "`{}` depends on `{r}` which appears later",
                        item_name(item).unwrap_or_else(|| "item".into()),
                    ),
                    fixable: true,
                });

                break;
            }
        }
    }

    violations
}

fn fix_items(items: &mut [Item]) {
    items.sort_by_key(|item| classify(item) as u8);

    let mut start = 0;

    while start < items.len() {
        let group = classify(&items[start]);

        let mut end = start + 1;

        while end < items.len() && classify(&items[end]) == group {
            end += 1;
        }

        if end - start > 1 {
            {
                let mut names: HashMap<String, usize> = HashMap::new();

                for (i, item) in items[start..end].iter().enumerate() {
                    let Some(name) = item_name(item) else {
                        continue;
                    };

                    names.insert(name, i);
                }

                topo_sort(&mut items[start..end], &names);
            }
        }

        start = end;
    }

    for item in items.iter_mut() {
        if let Item::Mod(m) = item
            && !is_test_mod(&Item::Mod(m.clone()))
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
    fn empty_file_no_violations() {
        assert!(detect("").is_empty());
    }

    #[test]
    fn correct_full_order() {
        let src = "\
use std::io;\n\
pub mod api;\n\
mod foo;\n\
const X: i32 = 1;\n\
pub trait MyTrait {}\n\
trait PrivTrait {}\n\
pub struct Foo;\n\
struct PrivStruct;\n\
pub fn bar() {}\n\
fn priv_fn() {}\n\
#[cfg(test)]\nmod tests {}\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn pub_mod_and_priv_mod_order_not_enforced() {
        let src = "mod foo;\npub mod api;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn use_after_struct_violation() {
        let src = "struct Foo;\nuse std::io;\n";
        let vs = detect(src);

        assert!(!vs.is_empty());
    }

    #[test]
    fn mod_before_use_violation() {
        let src = "mod foo;\nuse std::io;\n";
        let vs = detect(src);

        assert_eq!(vs.len(), 1);
        assert!(
            vs[0]
                .message
                .contains("`use` must appear before `mod declaration`")
        );
    }

    #[test]
    fn pub_struct_after_private_fn_violation() {
        let src = "fn priv_fn() {}\npub struct Foo;\n";
        let vs = detect(src);

        assert!(!vs.is_empty());
    }

    #[test]
    fn pub_trait_before_pub_struct_ok() {
        let src = "pub trait MyTrait {}\npub struct Foo;\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn private_trait_after_pub_fn_violation() {
        let src = "pub fn bar() {}\ntrait PrivTrait {}\n";
        let vs = detect(src);

        assert!(
            !vs.is_empty(),
            "private trait after pub fn should be a violation"
        );
    }

    #[test]
    fn test_mod_always_last() {
        let src = "#[cfg(test)]\nmod tests {}\nfn foo() {}\n";
        let vs = detect(src);

        assert!(!vs.is_empty());
    }

    #[test]
    fn dependent_struct_after_dependency_ok() {
        let src = "pub struct A;\npub struct B { pub a: A }\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn dependent_struct_before_dependency_violation() {
        let src = "pub struct B { pub a: A }\npub struct A;\n";
        let vs = detect(src);

        assert!(!vs.is_empty(), "B depends on A but comes first");
    }

    #[test]
    fn supertrait_before_subtrait_ok() {
        let src = "pub trait Base {}\npub trait Derived: Base {}\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn impl_after_struct_ok() {
        let src = "pub struct Foo;\nimpl Foo {}\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn fix_moves_use_before_struct() {
        let src = "struct Foo;\nuse std::io;\n";
        let fixed = fix(src);

        assert!(
            fixed.find("use std").unwrap() < fixed.find("struct Foo").unwrap(),
            "use should come before struct\n{fixed}"
        );
    }

    #[test]
    fn fix_sorts_pub_before_private() {
        let src = "fn priv_fn() {}\npub fn pub_fn() {}\n";
        let fixed = fix(src);

        assert!(
            fixed.find("pub fn pub_fn").unwrap() < fixed.find("fn priv_fn").unwrap(),
            "pub fn should come before private fn\n{fixed}"
        );
    }

    #[test]
    fn fix_topo_sorts_structs() {
        let src = "pub struct B { pub a: A }\npub struct A;\n";
        let fixed = fix(src);

        assert!(
            fixed.find("struct A").unwrap() < fixed.find("struct B").unwrap(),
            "A should come before B\n{fixed}"
        );
    }

    #[test]
    fn fix_then_detect_clean() {
        let src = "struct Bar;\nmod foo;\nuse std::io;\nfn priv_fn() {}\npub fn pub_fn() {}\n";
        let fixed = fix(src);
        let v = detect(&fixed);

        assert!(v.is_empty(), "violations remain after fix: {v:?}");
    }

    #[test]
    fn fix_idempotent() {
        let src = "use std::io;\nmod foo;\npub struct Foo;\nfn bar() {}\n";
        let first = fix(src);
        let second = fix(&first);

        assert_eq!(first, second, "fix should be idempotent");
    }

    #[test]
    fn inline_mod_is_type_not_mod_decl() {
        let src = "use std::io;\nmod foo;\nmod inline { use std::io; }\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn pub_crate_trait_after_pub_trait_ok() {
        let src = "pub trait A {}\npub(crate) trait B {}\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn pub_crate_trait_before_pub_trait_violation() {
        let src = "pub(crate) trait B {}\npub trait A {}\n";
        let vs = detect(src);

        assert!(
            !vs.is_empty(),
            "pub(crate) trait before pub trait should be a violation"
        );
    }

    #[test]
    fn pub_super_trait_after_pub_crate_trait_ok() {
        let src = "pub(crate) trait A {}\npub(super) trait B {}\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn pub_crate_fn_after_pub_fn_ok() {
        let src = "pub fn a() {}\npub(crate) fn b() {}\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn pub_crate_fn_before_pub_fn_violation() {
        let src = "pub(crate) fn b() {}\npub fn a() {}\n";
        let vs = detect(src);

        assert!(
            !vs.is_empty(),
            "pub(crate) fn before pub fn should be a violation"
        );
    }

    #[test]
    fn kind_first_pub_crate_trait_before_pub_fn_ok() {
        let src = "pub(crate) trait A {}\npub fn b() {}\n";

        assert!(detect(src).is_empty());
    }

    #[test]
    fn fix_sorts_pub_crate_after_pub_fn() {
        let src = "pub(crate) fn b() {}\npub fn a() {}\n";
        let fixed = fix(src);

        assert!(
            fixed.find("pub fn a").unwrap() < fixed.find("pub(crate) fn b").unwrap(),
            "pub fn should come before pub(crate) fn\n{fixed}"
        );
    }

    #[test]
    fn fix_kind_first_trait_before_fn() {
        let src = "pub fn foo() {}\npub trait Bar {}\n";
        let fixed = fix(src);

        assert!(
            fixed.find("trait Bar").unwrap() < fixed.find("fn foo").unwrap(),
            "trait should come before fn (kind-first)\n{fixed}"
        );
    }
}
