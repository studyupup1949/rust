use proc_macro2::Span;
use syn::{Attribute, Expr, Item, ItemUse, Lit, Meta, Type, UseTree, Visibility};

/// Import origin classification used across multiple rules.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Origin {
    Std = 0,
    External = 1,
    Crate = 2,
}

pub fn classify_root(name: &str) -> Origin {
    match name {
        "std" | "core" | "alloc" => Origin::Std,
        "crate" | "self" | "super" => Origin::Crate,
        _ => Origin::External,
    }
}

pub fn use_tree_origin(tree: &UseTree) -> Origin {
    match tree {
        UseTree::Path(p) => classify_root(&p.ident.to_string()),
        UseTree::Name(n) => classify_root(&n.ident.to_string()),
        UseTree::Rename(r) => classify_root(&r.ident.to_string()),
        UseTree::Group(g) => g
            .items
            .iter()
            .map(use_tree_origin)
            .min()
            .unwrap_or(Origin::External),
        UseTree::Glob(_) => Origin::External,
    }
}

pub fn use_item_origin(u: &ItemUse) -> Origin {
    use_tree_origin(&u.tree)
}

pub fn origin_name(o: Origin) -> &'static str {
    match o {
        Origin::Std => "std",
        Origin::External => "external-crate",
        Origin::Crate => "crate/self",
    }
}

// ── actor crate helpers (theta feature) ─────────────────────────────────────
/// Returns true if `attr` is `#[cfg(feature = "<feature_name>")]`.
pub fn is_cfg_feature(attr: &Attribute, feature_name: &str) -> bool {
    if !attr.path().is_ident("cfg") {
        return false;
    }
    let Meta::List(list) = &attr.meta else {
        return false;
    };
    let Ok(nv) = list.parse_args::<syn::MetaNameValue>() else {
        return false;
    };
    if !nv.path.is_ident("feature") {
        return false;
    }
    let Expr::Lit(expr_lit) = &nv.value else {
        return false;
    };
    let Lit::Str(s) = &expr_lit.lit else {
        return false;
    };
    s.value() == feature_name
}

/// Returns true if any attribute in `attrs` has path `actor`.
pub fn has_actor_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("actor"))
}

/// Extracts the simple struct name from `impl Actor for <Name>`.
pub fn actor_self_type_name(impl_item: &syn::ItemImpl) -> Option<String> {
    let Type::Path(tp) = impl_item.self_ty.as_ref() else {
        return None;
    };
    tp.path.segments.last().map(|s| s.ident.to_string())
}

// ── fixer helpers ────────────────────────────────────────────────────────────
/// Returns the 1-based start line of an item, using its first attribute line
/// when present (so leading doc-comments are included in the item's range).
pub fn item_start_line(item: &Item) -> usize {
    fn attr_first_line(attrs: &[Attribute]) -> Option<usize> {
        attrs.first().map(|a| a.pound_token.spans[0].start().line)
    }
    let from_attrs = match item {
        Item::Fn(i) => attr_first_line(&i.attrs),
        Item::Struct(i) => attr_first_line(&i.attrs),
        Item::Enum(i) => attr_first_line(&i.attrs),
        Item::Impl(i) => attr_first_line(&i.attrs),
        Item::Mod(i) => attr_first_line(&i.attrs),
        Item::Use(i) => attr_first_line(&i.attrs),
        Item::Trait(i) => attr_first_line(&i.attrs),
        Item::Const(i) => attr_first_line(&i.attrs),
        Item::Type(i) => attr_first_line(&i.attrs),
        Item::Static(i) => attr_first_line(&i.attrs),
        Item::ExternCrate(i) => attr_first_line(&i.attrs),
        _ => None,
    };
    from_attrs.unwrap_or_else(|| item_keyword_line(item))
}

/// Returns the 1-based line of an item's primary keyword token.
pub fn item_keyword_line(item: &Item) -> usize {
    let span: Span = match item {
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
        _ => Span::call_site(),
    };
    span.start().line
}

/// Splits source text into (preamble, item_texts).
/// `preamble` is everything before the first item; `item_texts[i]` is the
/// source for `items[i]` including any leading attributes.
pub fn split_item_sources(items: &[Item], source: &str) -> (String, Vec<String>) {
    if items.is_empty() {
        return (source.to_string(), vec![]);
    }
    let lines: Vec<&str> = source.lines().collect();
    let total = lines.len();
    let starts: Vec<usize> = items.iter().map(item_start_line).collect();

    let preamble = if starts[0] > 1 {
        lines[..starts[0] - 1].join("\n")
    } else {
        String::new()
    };

    let mut texts = Vec::with_capacity(items.len());
    for i in 0..items.len() {
        let s0 = starts[i].saturating_sub(1);
        let e0 = if i + 1 < starts.len() {
            starts[i + 1].saturating_sub(2).max(s0)
        } else {
            total.saturating_sub(1)
        }
        .min(total.saturating_sub(1));
        texts.push(lines[s0..=e0].join("\n"));
    }
    (preamble, texts)
}

/// Reconstructs source from preamble + ordered item texts, preserving the
/// trailing newline of the original source.
pub fn rejoin_sources(preamble: &str, texts: &[String], trailing_newline: bool) -> String {
    let mut out = String::new();
    if !preamble.is_empty() {
        out.push_str(preamble);
        out.push('\n');
    }
    for (i, t) in texts.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(t);
    }
    if trailing_newline && !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Serialize a `UseTree` back to source text.
pub fn use_tree_to_str(tree: &UseTree) -> String {
    match tree {
        UseTree::Path(p) => format!("{}::{}", p.ident, use_tree_to_str(&p.tree)),
        UseTree::Name(n) => n.ident.to_string(),
        UseTree::Rename(r) => format!("{} as {}", r.ident, r.rename),
        UseTree::Group(g) => {
            let mut items: Vec<String> = g.items.iter().map(use_tree_to_str).collect();
            items.sort();
            format!("{{{}}}", items.join(", "))
        }
        UseTree::Glob(_) => "*".to_string(),
    }
}

/// Serialize a `Visibility` to a source prefix string (e.g. `"pub "` or `""`).
pub fn vis_to_str(vis: &Visibility) -> &'static str {
    match vis {
        Visibility::Public(_) => "pub ",
        Visibility::Restricted(r) if r.path.is_ident("crate") => "pub(crate) ",
        Visibility::Restricted(r) if r.path.is_ident("super") => "pub(super) ",
        Visibility::Restricted(r) if r.path.is_ident("self") => "pub(self) ",
        Visibility::Restricted(_) => "pub(?) ",
        Visibility::Inherited => "",
    }
}

/// Merge multiple use sub-trees (after the shared root is removed) into a
/// single normalized `{item1, item2, ...}` or bare item string.
/// Each entry in `sub_trees` is the string after the shared root segment.
pub fn merge_sub_trees(sub_trees: Vec<String>) -> String {
    if sub_trees.len() == 1 {
        return sub_trees.into_iter().next().unwrap();
    }
    // Group sub_trees that start with the same leading path segment and merge
    // one level deeper (handles cases like ["a::x", "a::y"] → "a::{x, y}").
    let mut order: Vec<String> = Vec::new();
    let mut groups: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for s in sub_trees {
        if let Some(colon_pos) = s.find("::") {
            let root = s[..colon_pos].to_string();
            let rest = s[colon_pos + 2..].to_string();
            groups
                .entry(root.clone())
                .or_insert_with(|| {
                    order.push(root);
                    Vec::new()
                })
                .push(rest);
        } else {
            // Leaf item or glob — treat as atomic
            order.push(s.clone());
            groups.entry(s).or_default();
        }
    }

    let mut parts: Vec<String> = Vec::with_capacity(order.len());
    for key in &order {
        let rests = &groups[key];
        if rests.is_empty() {
            parts.push(key.clone());
        } else if rests.len() == 1 {
            parts.push(format!("{}::{}", key, rests[0]));
        } else {
            let merged = merge_sub_trees(rests.clone());
            parts.push(format!("{}::{}", key, merged));
        }
    }

    parts.sort();
    format!("{{{}}}", parts.join(", "))
}
