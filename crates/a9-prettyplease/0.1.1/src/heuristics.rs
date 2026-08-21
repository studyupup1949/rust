use syn::{Expr, Item, Local, Pat, Stmt, UseTree};

// ---------------------------------------------------------------------------
// Use-group classification (unchanged)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseGroup {
    Std,
    External,
    CrateLocal,
}

pub fn classify_use(item: &syn::ItemUse) -> UseGroup {
    let root_ident = use_tree_root_ident(&item.tree);
    match root_ident.as_deref() {
        Some("std" | "alloc" | "core") => UseGroup::Std,
        Some("crate" | "super" | "self") => UseGroup::CrateLocal,
        _ => UseGroup::External,
    }
}

fn use_tree_root_ident(tree: &UseTree) -> Option<String> {
    match tree {
        UseTree::Path(path) => Some(path.ident.to_string()),
        UseTree::Name(name) => Some(name.ident.to_string()),
        UseTree::Rename(rename) => Some(rename.ident.to_string()),
        UseTree::Glob(_) => None,
        UseTree::Group(group) => group.items.first().and_then(use_tree_root_ident),
    }
}

// ---------------------------------------------------------------------------
// Item-kind classification
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemKind {
    Use,
    ExternCrate,
    Mod,
    Const,
    Static,
    TypeAlias,
    Definition,
    Other,
}

fn classify_item_kind(item: &Item) -> ItemKind {
    match item {
        Item::Use(_) => ItemKind::Use,
        Item::ExternCrate(_) => ItemKind::ExternCrate,
        Item::Mod(_) => ItemKind::Mod,
        Item::Const(_) => ItemKind::Const,
        Item::Static(_) => ItemKind::Static,
        Item::Type(_) => ItemKind::TypeAlias,
        Item::Fn(_)
        | Item::Struct(_)
        | Item::Enum(_)
        | Item::Union(_)
        | Item::Trait(_)
        | Item::TraitAlias(_)
        | Item::Impl(_)
        | Item::Macro(_) => ItemKind::Definition,
        _ => ItemKind::Other,
    }
}

pub fn should_blank_between_items(prev: &Item, next: &Item) -> bool {
    let pk = classify_item_kind(prev);
    let nk = classify_item_kind(next);

    // Same lightweight kind clusters together
    match (pk, nk) {
        (ItemKind::Use, ItemKind::Use) => {
            let prev_group = classify_use(match prev {
                Item::Use(u) => u,
                _ => unreachable!(),
            });
            let next_group = classify_use(match next {
                Item::Use(u) => u,
                _ => unreachable!(),
            });
            prev_group != next_group
        }
        (ItemKind::ExternCrate, ItemKind::ExternCrate) => false,
        (ItemKind::Mod, ItemKind::Mod) => false,
        (ItemKind::Const, ItemKind::Const) => false,
        (ItemKind::Static, ItemKind::Static) => false,
        (ItemKind::TypeAlias, ItemKind::TypeAlias) => false,
        _ => true,
    }
}

// ---------------------------------------------------------------------------
// Statement weight classification
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StmtWeight {
    Light,
    Binding,
    Medium,
    Heavy,
    Item,
}

fn expr_is_heavy(expr: &Expr) -> bool {
    match expr {
        Expr::If(_)
        | Expr::Match(_)
        | Expr::ForLoop(_)
        | Expr::While(_)
        | Expr::Loop(_)
        | Expr::Block(_)
        | Expr::Unsafe(_)
        | Expr::TryBlock(_) => true,
        Expr::Closure(c) => matches!(*c.body, Expr::Block(_)),
        Expr::Assign(a) => expr_is_heavy(&a.right),
        _ => false,
    }
}

fn pat_contains_mut(pat: &Pat) -> bool {
    match pat {
        Pat::Ident(p) => p.mutability.is_some(),
        Pat::Reference(p) => p.mutability.is_some() || pat_contains_mut(&p.pat),
        Pat::Struct(p) => p.fields.iter().any(|f| pat_contains_mut(&f.pat)),
        Pat::Tuple(p) => p.elems.iter().any(pat_contains_mut),
        Pat::TupleStruct(p) => p.elems.iter().any(pat_contains_mut),
        Pat::Slice(p) => p.elems.iter().any(pat_contains_mut),
        Pat::Or(p) => p.cases.iter().any(pat_contains_mut),
        Pat::Type(p) => pat_contains_mut(&p.pat),
        _ => false,
    }
}

fn classify_local(local: &Local) -> StmtWeight {
    if let Some(init) = &local.init {
        if expr_is_heavy(&init.expr) {
            return StmtWeight::Heavy;
        }
    }
    if pat_contains_mut(&local.pat) {
        StmtWeight::Binding
    } else {
        StmtWeight::Light
    }
}

// ---------------------------------------------------------------------------
// Tracing / logging macro detection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TracingLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

fn is_tracing_macro(expr: &Expr) -> Option<TracingLevel> {
    if let Expr::Macro(m) = expr {
        let name = m.mac.path.segments.last()?.ident.to_string();
        match name.as_str() {
            "trace" => Some(TracingLevel::Trace),
            "debug" => Some(TracingLevel::Debug),
            "info" => Some(TracingLevel::Info),
            "warn" => Some(TracingLevel::Warn),
            "error" => Some(TracingLevel::Error),
            _ => None,
        }
    } else {
        None
    }
}

fn stmt_expr(stmt: &Stmt) -> Option<&Expr> {
    match stmt {
        Stmt::Expr(expr, _) => Some(expr),
        _ => None,
    }
}

/// Returns Some(should_blank) if tracing attachment rules apply.
fn tracing_blank_line(prev: &Stmt, next: &Stmt) -> Option<bool> {
    // Check if prev is a tracing macro
    if let Some(prev_expr) = stmt_expr(prev) {
        if let Some(level) = is_tracing_macro(prev_expr) {
            return Some(match level {
                TracingLevel::Trace => false,
                TracingLevel::Debug => true,
                TracingLevel::Info | TracingLevel::Warn | TracingLevel::Error => true,
            });
        }
    }

    // Check if next is a tracing macro
    if let Some(next_expr) = stmt_expr(next) {
        if let Some(level) = is_tracing_macro(next_expr) {
            return Some(match level {
                TracingLevel::Trace => true,
                TracingLevel::Debug => false,
                TracingLevel::Info | TracingLevel::Warn | TracingLevel::Error => true,
            });
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Statement blank-line decisions
// ---------------------------------------------------------------------------

fn classify_weight(stmt: &Stmt) -> StmtWeight {
    match stmt {
        Stmt::Local(local) => classify_local(local),
        Stmt::Expr(expr, _) => {
            if expr_is_heavy(expr) {
                StmtWeight::Heavy
            } else {
                StmtWeight::Medium
            }
        }
        Stmt::Item(_) => StmtWeight::Item,
        Stmt::Macro(_) => StmtWeight::Medium,
    }
}

fn is_jump_stmt(stmt: &Stmt) -> bool {
    matches!(
        stmt,
        Stmt::Expr(
            Expr::Return(_) | Expr::Continue(_) | Expr::Break(_),
            _
        )
    )
}

fn is_let_else(stmt: &Stmt) -> bool {
    if let Stmt::Local(local) = stmt {
        if let Some(init) = &local.init {
            return init.diverge.is_some();
        }
    }
    false
}

fn should_blank_between_stmts(prev: &Stmt, next: &Stmt) -> bool {
    // Tracing macro attachment takes priority
    if let Some(decision) = tracing_blank_line(prev, next) {
        return decision;
    }

    // return / continue / break always get breathing room before them
    if is_jump_stmt(next) {
        return true;
    }

    // let...else always gets a blank line before it
    if is_let_else(next) {
        return true;
    }

    let pw = classify_weight(prev);
    let nw = classify_weight(next);

    // Item stmts get separation
    if nw == StmtWeight::Item || pw == StmtWeight::Item {
        return true;
    }

    // Heavy constructs get breathing room
    if pw == StmtWeight::Heavy || nw == StmtWeight::Heavy {
        return true;
    }

    // Same weight clusters together
    if pw == nw {
        return false;
    }

    // Any weight transition
    true
}

/// Returns a Vec of length `stmts.len()` where `result[i]` is true
/// if a blank line should be inserted BEFORE `stmts[i]`.
/// `result[0]` is always false (no blank line before the first statement).
pub fn stmt_blank_lines(stmts: &[Stmt]) -> Vec<bool> {
    let len = stmts.len();
    let mut blanks = vec![false; len];
    if len <= 1 {
        return blanks;
    }
    for i in 1..len {
        blanks[i] = should_blank_between_stmts(&stmts[i - 1], &stmts[i]);
    }
    blanks
}
