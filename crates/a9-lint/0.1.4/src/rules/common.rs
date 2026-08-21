use syn::{Attribute, ItemUse, Meta, Type, UseTree};

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
    let syn::Expr::Lit(expr_lit) = &nv.value else {
        return false;
    };
    let syn::Lit::Str(s) = &expr_lit.lit else {
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
