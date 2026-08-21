use syn::{
    Block, Expr, ExprClosure, File, FnArg, Ident, ImplItem, Item, ItemFn, ItemImpl, ItemTrait, Pat,
    Stmt, TraitItem, visit::Visit,
};

use crate::{Rule as RuleTrait, UnitRule as UnitRuleTrait, Violation};

const IDIOMATIC_NAMES: &[&str] = &[
    "i", "j", "k", "n", "x", "y", "z", "e", "f", "s", "r", "w", "c", "b", "v", "m", "p", "t", "a",
    "tx", "rx", "ch", "db", "id", "fd", "fs", "io", "re", "wg", "cx", "cq", "op", "lk", "ok", "it",
    "idx", "buf", "ctx", "cfg", "src", "dst", "msg", "res", "req", "key", "val", "len", "cap",
    "pos", "ptr", "err", "opt", "tmp", "ret", "acc", "cmd", "env", "arg", "args", "out", "dir",
    "col", "row", "addr", "conn", "sock", "iter", "elem", "prev", "next", "min", "max", "sum",
    "cnt", "tok", "eof", "sep", "pat", "rng", "vec", "map", "set", "ast", "errs", "tree", "line",
    "expr", "node", "root", "path", "name", "span", "kind", "data", "body", "item", "file", "func",
    "spec", "flag", "mode", "type", "cwd", "cli", "segs",
];

pub struct UnitRule;

impl RuleTrait for UnitRule {
    fn name(&self) -> &'static str {
        "scope-ident-length-correspondance"
    }

    fn description(&self) -> &'static str {
        "variable name length should correspond to its scope size"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &File) -> Vec<Violation> {
        let mut visitor = ScopeVisitor {
            violations: vec![],
            trait_method_params: vec![],
        };

        for item in &ast.items {
            let Item::Trait(t) = item else {
                continue;
            };

            for ti in &t.items {
                let TraitItem::Fn(f) = ti else {
                    continue;
                };

                for arg in &f.sig.inputs {
                    if let FnArg::Typed(pt) = arg
                        && let Pat::Ident(pi) = &*pt.pat
                    {
                        visitor.trait_method_params.push(pi.ident.to_string());
                    }
                }
            }
        }

        visitor.visit_file(ast);

        visitor.violations
    }
}

impl ScopeVisitor {
    fn check_fn_bindings(&mut self, block: &Block, params: &[(&Ident, bool)]) {
        let scope_size = count_stmts_block(block);
        let scope = classify_scope(scope_size);

        for &(ident, is_trait_param) in params {
            if is_trait_param || ident.to_string().starts_with('_') {
                continue;
            }

            self.check_binding(ident, scope, scope_size);
        }

        self.check_bindings_in_block(block, scope, scope_size);
    }

    fn check_bindings_in_block(&mut self, block: &Block, scope: ScopeClass, scope_size: usize) {
        for stmt in &block.stmts {
            self.check_bindings_in_stmt(stmt, scope, scope_size);
        }
    }

    fn check_bindings_in_stmt(&mut self, stmt: &Stmt, scope: ScopeClass, scope_size: usize) {
        match stmt {
            Stmt::Local(local) => {
                self.check_pat_bindings(&local.pat, scope, scope_size);

                if let Some(init) = &local.init {
                    self.check_expr_for_closures(&init.expr, scope_size);
                }
            }
            Stmt::Expr(expr, _) => {
                self.check_expr_for_closures(expr, scope_size);
            }
            _ => {}
        }
    }

    fn check_pat_bindings(&mut self, pat: &Pat, scope: ScopeClass, scope_size: usize) {
        match pat {
            Pat::Ident(pi) if !pi.ident.to_string().starts_with('_') => {
                self.check_binding(&pi.ident, scope, scope_size);
            }
            Pat::Tuple(pt) => {
                for p in &pt.elems {
                    self.check_pat_bindings(p, scope, scope_size);
                }
            }
            Pat::TupleStruct(pts) => {
                for p in &pts.elems {
                    self.check_pat_bindings(p, scope, scope_size);
                }
            }
            Pat::Struct(ps) => {
                for fp in &ps.fields {
                    self.check_pat_bindings(&fp.pat, scope, scope_size);
                }
            }
            Pat::Slice(ps) => {
                for p in &ps.elems {
                    self.check_pat_bindings(p, scope, scope_size);
                }
            }
            Pat::Or(po) => {
                for p in &po.cases {
                    self.check_pat_bindings(p, scope, scope_size);
                }
            }
            Pat::Reference(pr) => {
                self.check_pat_bindings(&pr.pat, scope, scope_size);
            }
            Pat::Type(pt) => {
                self.check_pat_bindings(&pt.pat, scope, scope_size);
            }
            _ => {}
        }
    }

    fn check_binding(&mut self, ident: &Ident, scope: ScopeClass, scope_size: usize) {
        let name = ident.to_string();
        let name_class = classify_name(name.len());
        let required = min_name_class(scope);

        if name_class >= required {
            return;
        }

        let max_tolerated_scope = effective_min_scope(name_class, is_idiomatic(&name));

        if scope <= max_tolerated_scope {
            return;
        }

        let line = ident.span().start().line;

        let mut message = format!(
            "`{}` is too short for scope {} ({} stmts); minimum: {} name",
            name,
            scope_class_label(scope),
            scope_size,
            min_name_label(required),
        );

        if scope == ScopeClass::Xl {
            message.push_str("; consider splitting this function");
        }

        self.violations.push(Violation {
            line,
            message,
            fixable: false,
        });
    }

    fn check_expr_for_closures(&mut self, expr: &Expr, parent_scope_size: usize) {
        match expr {
            Expr::Closure(closure) => {
                self.check_closure(closure);
            }
            Expr::Block(b) => {
                for stmt in &b.block.stmts {
                    let Stmt::Expr(e, _) = stmt else {
                        continue;
                    };

                    self.check_expr_for_closures(e, parent_scope_size);
                }
            }
            Expr::If(i) => {
                self.check_expr_for_closures_in_block(&i.then_branch, parent_scope_size);

                if let Some((_, e)) = &i.else_branch {
                    self.check_expr_for_closures(e, parent_scope_size);
                }
            }
            Expr::Match(m) => {
                for arm in &m.arms {
                    self.check_expr_for_closures(&arm.body, parent_scope_size);
                }
            }
            Expr::MethodCall(mc) => {
                for arg in &mc.args {
                    self.check_expr_for_closures(arg, parent_scope_size);
                }

                self.check_expr_for_closures(&mc.receiver, parent_scope_size);
            }
            Expr::Call(c) => {
                for arg in &c.args {
                    self.check_expr_for_closures(arg, parent_scope_size);
                }

                self.check_expr_for_closures(&c.func, parent_scope_size);
            }
            _ => {}
        }
    }

    fn check_expr_for_closures_in_block(&mut self, block: &Block, parent_scope_size: usize) {
        for stmt in &block.stmts {
            let Stmt::Expr(e, _) = stmt else {
                continue;
            };

            self.check_expr_for_closures(e, parent_scope_size);
        }
    }

    fn check_closure(&mut self, closure: &ExprClosure) {
        let closure_stmts = count_stmts_expr(&closure.body);

        if closure_stmts <= 3 {
            return;
        }

        let scope = classify_scope(closure_stmts);

        for input in &closure.inputs {
            self.check_pat_bindings(input, scope, closure_stmts);
        }

        if let Expr::Block(b) = &*closure.body {
            self.check_bindings_in_block(&b.block, scope, closure_stmts);
        }
    }
}

impl<'ast> Visit<'ast> for ScopeVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let params: Vec<(&Ident, bool)> = node
            .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                FnArg::Typed(pt) => {
                    if let Pat::Ident(pi) = &*pt.pat {
                        let is_trait = self.trait_method_params.contains(&pi.ident.to_string());

                        Some((&pi.ident, is_trait))
                    } else {
                        None
                    }
                }
                FnArg::Receiver(_) => None,
            })
            .collect();

        self.check_fn_bindings(&node.block, &params);
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        for item in &node.items {
            let ImplItem::Fn(f) = item else {
                continue;
            };

            let is_trait_impl = node.trait_.is_some();
            let params: Vec<(&Ident, bool)> = f
                .sig
                .inputs
                .iter()
                .filter_map(|arg| match arg {
                    FnArg::Typed(pt) => {
                        if let Pat::Ident(pi) = &*pt.pat {
                            Some((&pi.ident, is_trait_impl))
                        } else {
                            None
                        }
                    }
                    FnArg::Receiver(_) => None,
                })
                .collect();

            self.check_fn_bindings(&f.block, &params);
        }
    }

    fn visit_item_trait(&mut self, _node: &'ast ItemTrait) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ScopeClass {
    Xs,
    S,
    M,
    L,
    Xl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum NameClass {
    OneChar,
    TwoChar,
    Short,
    Descriptive,
}

struct ScopeVisitor {
    violations: Vec<Violation>,
    trait_method_params: Vec<String>,
}

const fn classify_scope(stmt_count: usize) -> ScopeClass {
    match stmt_count {
        0..=1 => ScopeClass::Xs,
        2..=5 => ScopeClass::S,
        6..=15 => ScopeClass::M,
        16..=30 => ScopeClass::L,
        _ => ScopeClass::Xl,
    }
}

const fn classify_name(len: usize) -> NameClass {
    match len {
        0..=1 => NameClass::OneChar,
        2 => NameClass::TwoChar,
        3..=4 => NameClass::Short,
        _ => NameClass::Descriptive,
    }
}

/// Minimum name class required for a given scope class.
const fn min_name_class(scope: ScopeClass) -> NameClass {
    match scope {
        ScopeClass::Xs => NameClass::OneChar,
        ScopeClass::S => NameClass::TwoChar,
        ScopeClass::M => NameClass::Short,
        ScopeClass::L | ScopeClass::Xl => NameClass::Descriptive,
    }
}

/// Scope class name for diagnostic messages.
const fn scope_class_label(sc: ScopeClass) -> &'static str {
    match sc {
        ScopeClass::Xs => "XS (1 stmt)",
        ScopeClass::S => "S (2-5 stmts)",
        ScopeClass::M => "M (6-15 stmts)",
        ScopeClass::L => "L (16-30 stmts)",
        ScopeClass::Xl => "XL (31+ stmts)",
    }
}

const fn min_name_label(nc: NameClass) -> &'static str {
    match nc {
        NameClass::OneChar => "1-char",
        NameClass::TwoChar => "2-char",
        NameClass::Short => "short (3-4 chars)",
        NameClass::Descriptive => "descriptive (5+ chars)",
    }
}

fn is_idiomatic(name: &str) -> bool {
    IDIOMATIC_NAMES.contains(&name)
}

/// With idiomatic bonus, the effective minimum scope class is raised by one
/// (i.e., the name is tolerated in one larger scope class).
const fn effective_min_scope(name_class: NameClass, idiomatic: bool) -> ScopeClass {
    let base = match name_class {
        NameClass::OneChar => ScopeClass::Xs,
        NameClass::TwoChar => ScopeClass::S,
        NameClass::Short => ScopeClass::M,
        NameClass::Descriptive => ScopeClass::L,
    };

    if idiomatic {
        match base {
            ScopeClass::Xs => ScopeClass::S,
            ScopeClass::S => ScopeClass::M,
            ScopeClass::M => ScopeClass::L,
            ScopeClass::L | ScopeClass::Xl => ScopeClass::Xl,
        }
    } else {
        base
    }
}

fn count_stmts_block(block: &Block) -> usize {
    block.stmts.iter().map(count_stmts_recursive).sum()
}

fn count_stmts_recursive(stmt: &Stmt) -> usize {
    1 + match stmt {
        Stmt::Local(local) => local
            .init
            .as_ref()
            .map_or(0, |init| count_stmts_expr(&init.expr)),
        Stmt::Expr(expr, _) => count_stmts_expr(expr),
        Stmt::Item(_) | Stmt::Macro(_) => 0,
    }
}

fn count_stmts_expr(expr: &Expr) -> usize {
    match expr {
        Expr::Block(b) => count_stmts_block(&b.block),
        Expr::If(i) => {
            let then_count = count_stmts_block(&i.then_branch);
            let else_count = i
                .else_branch
                .as_ref()
                .map_or(0, |(_, e)| count_stmts_expr(e));

            then_count + else_count
        }
        Expr::While(w) => count_stmts_block(&w.body),
        Expr::ForLoop(f) => count_stmts_block(&f.body),
        Expr::Loop(l) => count_stmts_block(&l.body),
        Expr::Match(m) => m.arms.iter().map(|arm| count_stmts_expr(&arm.body)).sum(),
        Expr::Unsafe(u) => count_stmts_block(&u.block),
        Expr::Closure(c) => count_stmts_expr(&c.body),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UnitRule as UnitRuleTrait;

    fn detect(src: &str) -> Vec<Violation> {
        let ast = syn::parse_file(src).unwrap();

        UnitRule.detect(&ast)
    }

    #[test]
    fn xs_scope_allows_single_char() {
        let v = detect(
            r#"
fn f() {
    let x = 1;
}
"#,
        );

        assert!(v.is_empty(), "XS scope should allow 1-char: {v:?}");
    }

    #[test]
    fn s_scope_allows_two_char() {
        let v = detect(
            r#"
fn f() {
    let id = 1;
    let x = id + 1;
    let y = x + 1;
}
"#,
        );

        assert!(v.is_empty(), "S scope should allow 2-char: {v:?}");
    }

    #[test]
    fn s_scope_rejects_one_char_non_idiomatic() {
        let v = detect(
            r#"
fn f() {
    let q = 1;
    let w = q + 1;
    let u = w + 1;
}
"#,
        );

        assert!(!v.is_empty(), "S scope should reject non-idiomatic 1-char");
        assert!(v[0].message.contains("`q`"));
    }

    #[test]
    fn s_scope_allows_idiomatic_one_char() {
        let v = detect(
            r#"
fn f() {
    let i = 0;
    let x = i + 1;
    let y = x + 1;
}
"#,
        );

        assert!(
            v.is_empty(),
            "S scope should allow idiomatic 1-char `i`: {v:?}"
        );
    }

    #[test]
    fn m_scope_requires_short_name() {
        let v = detect(
            r#"
fn f() {
    let q = 1;
    let a1 = q;
    let a2 = a1;
    let a3 = a2;
    let a4 = a3;
    let a5 = a4;
    let a6 = a5;
}
"#,
        );

        assert!(!v.is_empty(), "M scope should reject 1-char `q`");
    }

    #[test]
    fn m_scope_allows_short_name() {
        let v = detect(
            r#"
fn f() {
    let buf = vec![];
    let idx = 0;
    let len = buf.len();
    let cap = buf.capacity();
    let pos = 0;
    let end = len;
    let res = pos + end;
}
"#,
        );

        assert!(v.is_empty(), "M scope should allow short names: {v:?}");
    }

    #[test]
    fn underscore_prefix_is_exempt() {
        let v = detect(
            r#"
fn f() {
    let _x = 1;
    let a1 = _x;
    let a2 = a1;
    let a3 = a2;
    let a4 = a3;
    let a5 = a4;
    let a6 = a5;
}
"#,
        );
        let has_x = v.iter().any(|v| v.message.contains("`_x`"));

        assert!(!has_x, "_x should be exempt: {v:?}");
    }

    #[test]
    fn closure_le_3_stmts_allows_short_names() {
        let v = detect(
            r#"
fn f() {
    let items = vec![1, 2, 3];
    items.iter().map(|x| x + 1);
}
"#,
        );

        assert!(
            v.is_empty(),
            "Small closure should allow short names: {v:?}"
        );
    }

    #[test]
    fn xl_scope_suggests_split() {
        let mut body = String::new();

        for i in 0..32 {
            body.push_str(&format!("    let a{i} = {i};\n"));
        }

        let src = format!("fn f() {{\n    let q = 0;\n{body}}}\n");
        let violations = detect(&src);
        let xl_violation = violations.iter().find(|viol| viol.message.contains("`q`"));

        assert!(
            xl_violation.is_some(),
            "XL scope should flag short name: {violations:?}"
        );
        assert!(
            xl_violation.unwrap().message.contains("splitting"),
            "XL should suggest splitting"
        );
    }

    #[test]
    fn trait_impl_params_are_exempt() {
        let v = detect(
            r#"
trait Foo {
    fn bar(&self, x: i32);
}
struct S;
impl Foo for S {
    fn bar(&self, x: i32) {
        let a1 = x;
        let a2 = a1;
        let a3 = a2;
        let a4 = a3;
        let a5 = a4;
        let a6 = a5;
        let a7 = a6;
    }
}
"#,
        );
        let has_x = v.iter().any(|v| v.message.contains("`x`"));

        assert!(!has_x, "Trait impl param `x` should be exempt: {v:?}");
    }

    #[test]
    fn idiomatic_two_char_in_m_scope() {
        let v = detect(
            r#"
fn f() {
    let tx = 1;
    let a1 = tx;
    let a2 = a1;
    let a3 = a2;
    let a4 = a3;
    let a5 = a4;
    let a6 = a5;
}
"#,
        );
        let has_tx = v.iter().any(|v| v.message.contains("`tx`"));

        assert!(
            !has_tx,
            "Idiomatic `tx` should be allowed in M scope: {v:?}"
        );
    }
}
