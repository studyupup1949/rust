#![allow(clippy::pedantic, clippy::nursery)]
use syn::{
    BinOp, Block, Expr, ExprBlock, ExprContinue, ExprForLoop, ExprIf, ExprLoop, ExprReturn,
    ExprUnary, ExprWhile, File, ImplItem, ImplItemFn, Item, ItemFn, ItemImpl, ItemTrait, Local,
    LocalInit, ReturnType, Stmt, UnOp, token,
    visit::Visit,
    visit_mut::{self, VisitMut},
};

use crate::{Rule as RuleTrait, UnitRule as UnitRuleTrait, Violation};

pub struct UnitRule;

impl RuleTrait for UnitRule {
    fn name(&self) -> &'static str {
        "optimal-control-flow"
    }

    fn description(&self) -> &'static str {
        "the lighter branch should guard; prefer early exit when the then-body dominates"
    }
}

impl UnitRuleTrait for UnitRule {
    fn detect(&self, ast: &File) -> Vec<Violation> {
        let mut visitor = Visitor { violations: vec![] };

        visitor.visit_file(ast);

        visitor.violations
    }

    fn fix(&self, mut ast: File) -> File {
        Fixer.visit_file_mut(&mut ast);

        ast
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let fn_body = matches!(node.sig.output, ReturnType::Default);

        check_block(&node.block, false, fn_body, &mut self.violations);
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        for item in &node.items {
            let ImplItem::Fn(f) = item else { continue };

            let fn_body = matches!(f.sig.output, ReturnType::Default);

            check_block(&f.block, false, fn_body, &mut self.violations);
        }
    }

    fn visit_item_trait(&mut self, _: &'ast ItemTrait) {}
}

impl VisitMut for Fixer {
    fn visit_item_fn_mut(&mut self, node: &mut ItemFn) {
        visit_mut::visit_item_fn_mut(self, node);

        if matches!(node.sig.output, ReturnType::Default) {
            fix_block_last(&mut node.block, &ExitKind::Return);
        }
    }

    fn visit_impl_item_fn_mut(&mut self, node: &mut ImplItemFn) {
        visit_mut::visit_impl_item_fn_mut(self, node);

        if matches!(node.sig.output, ReturnType::Default) {
            fix_block_last(&mut node.block, &ExitKind::Return);
        }
    }

    fn visit_expr_for_loop_mut(&mut self, node: &mut ExprForLoop) {
        visit_mut::visit_expr_for_loop_mut(self, node);
        fix_block_last(&mut node.body, &ExitKind::Continue);
    }

    fn visit_expr_while_mut(&mut self, node: &mut ExprWhile) {
        visit_mut::visit_expr_while_mut(self, node);
        fix_block_last(&mut node.body, &ExitKind::Continue);
    }

    fn visit_expr_loop_mut(&mut self, node: &mut ExprLoop) {
        visit_mut::visit_expr_loop_mut(self, node);
        fix_block_last(&mut node.body, &ExitKind::Continue);
    }

    fn visit_item_trait_mut(&mut self, _: &mut ItemTrait) {}
}

struct Visitor {
    violations: Vec<Violation>,
}

struct Fixer;

enum ExitKind {
    Continue,
    Return,
}

fn check_block(
    block: &Block,
    is_loop_body: bool,
    is_fn_body: bool,
    violations: &mut Vec<Violation>,
) {
    let n = block.stmts.len();

    for (i, stmt) in block.stmts.iter().enumerate() {
        let is_last = i + 1 == n;

        check_stmt(
            stmt,
            is_loop_body && is_last,
            is_fn_body && is_last,
            violations,
        );
    }
}

fn check_stmt(
    stmt: &Stmt,
    is_last_in_loop: bool,
    is_last_in_fn: bool,
    violations: &mut Vec<Violation>,
) {
    match stmt {
        Stmt::Expr(expr, _) => {
            if let Expr::If(if_expr) = expr {
                check_if(if_expr, is_last_in_loop, is_last_in_fn, violations);
            }

            recurse_expr(expr, violations);
        }
        Stmt::Item(Item::Fn(f)) => {
            let fn_body = matches!(f.sig.output, ReturnType::Default);

            check_block(&f.block, false, fn_body, violations);
        }
        _ => {}
    }
}

fn check_if(
    if_expr: &ExprIf,
    is_last_in_loop: bool,
    is_last_in_fn: bool,
    violations: &mut Vec<Violation>,
) {
    match &if_expr.else_branch {
        Some((_, else_expr)) => {
            if cond_has_let(&if_expr.cond) {
                return;
            }

            let then_w = block_weight(&if_expr.then_branch);
            let else_w = else_weight(else_expr);

            if then_w > else_w {
                violations
                    .push(Violation {
                        line: if_expr.if_token.span.start().line,
                        message: format!(
                            "then-branch (weight {then_w}) is heavier than else-branch (weight {else_w}); prefer `if !cond {{ … }}` guard"
                        ),
                        fixable: is_last_in_loop || is_last_in_fn,
                    });
            }
        }
        None => {
            if !is_last_in_loop {
                return;
            }

            if cond_is_let_chain(&if_expr.cond) {
                return;
            }

            let then_w = block_weight(&if_expr.then_branch);

            if then_w > 1 {
                let is_if_let = cond_has_let(&if_expr.cond);

                violations
                    .push(Violation {
                        line: if_expr.if_token.span.start().line,
                        message: if is_if_let {
                            format!(
                                "if-let body (weight {then_w}) dominates loop iteration; prefer `let … = … else {{ continue }}`"
                            )
                        } else {
                            format!(
                                "if body (weight {then_w}) dominates loop iteration; prefer `if !cond {{ continue }}`"
                            )
                        },
                        fixable: true,
                    });
            }
        }
    }
}

fn recurse_expr(expr: &Expr, violations: &mut Vec<Violation>) {
    match expr {
        Expr::If(i) => {
            check_block(&i.then_branch, false, false, violations);

            if let Some((_, e)) = &i.else_branch {
                match e.as_ref() {
                    Expr::Block(b) => check_block(&b.block, false, false, violations),
                    Expr::If(_) => recurse_expr(e, violations),
                    _ => {}
                }
            }
        }
        Expr::ForLoop(f) => check_block(&f.body, true, false, violations),
        Expr::While(w) => check_block(&w.body, true, false, violations),
        Expr::Loop(l) => check_block(&l.body, true, false, violations),
        Expr::Block(b) => check_block(&b.block, false, false, violations),
        Expr::Closure(c) => {
            if let Expr::Block(b) = c.body.as_ref() {
                check_block(&b.block, false, false, violations);
            }
        }
        Expr::Match(m) => {
            for arm in &m.arms {
                match arm.body.as_ref() {
                    Expr::Block(b) => check_block(&b.block, false, false, violations),
                    _ => recurse_expr(&arm.body, violations),
                }
            }
        }
        _ => {}
    }
}

/// Returns true if the condition contains any `let` binding (if-let or let-chain).
fn cond_has_let(cond: &Expr) -> bool {
    match cond {
        Expr::Let(_) => true,
        Expr::Binary(b) => cond_has_let(&b.left) || cond_has_let(&b.right),
        Expr::Paren(p) => cond_has_let(&p.expr),
        _ => false,
    }
}

/// Returns true when the condition is a multi-let `&&` chain (e.g. `let a = x && let b = y`).
/// A single `if let` is NOT a let-chain.
fn cond_is_let_chain(cond: &Expr) -> bool {
    match cond {
        Expr::Binary(b) if matches!(b.op, BinOp::And(_)) => {
            cond_has_let(&b.left) || cond_has_let(&b.right)
        }
        Expr::Paren(p) => cond_is_let_chain(&p.expr),
        _ => false,
    }
}

fn fix_block_last(block: &mut Block, exit_kind: &ExitKind) {
    while try_fix_last(block, exit_kind) {}
}

fn try_fix_last(block: &mut Block, exit_kind: &ExitKind) -> bool {
    let Some(Stmt::Expr(Expr::If(if_expr), _)) = block.stmts.last() else {
        return false;
    };

    let has_else = if_expr.else_branch.is_some();
    let then_w = block_weight(&if_expr.then_branch);

    if !has_else {
        if !matches!(exit_kind, ExitKind::Continue) {
            return false;
        }

        if cond_is_let_chain(&if_expr.cond) {
            return false;
        }

        if then_w <= 1 {
            return false;
        }
    } else {
        if cond_has_let(&if_expr.cond) {
            return false;
        }

        let else_w = else_weight(if_expr.else_branch.as_ref().unwrap().1.as_ref());

        if then_w <= else_w {
            return false;
        }
    }

    let is_if_let = !has_else && cond_has_let(&if_expr.cond);

    let Stmt::Expr(Expr::If(if_expr), _) = block.stmts.pop().unwrap() else {
        unreachable!()
    };

    if is_if_let {
        apply_let_else(block, if_expr, exit_kind);
    } else if !has_else {
        apply_bool_guard(block, if_expr, exit_kind);
    } else {
        apply_case1_guard(block, if_expr, exit_kind);
    }

    true
}

/// Case 2, if-let → let PAT = EXPR else { continue/return };
fn apply_let_else(block: &mut Block, if_expr: ExprIf, exit_kind: &ExitKind) {
    let Expr::Let(let_expr) = *if_expr.cond else {
        unreachable!()
    };

    let local = Local {
        attrs: vec![],
        let_token: let_expr.let_token,
        pat: *let_expr.pat,
        init: Some(LocalInit {
            eq_token: let_expr.eq_token,
            expr: let_expr.expr,
            diverge: Some((
                token::Else::default(),
                Box::new(Expr::Block(ExprBlock {
                    attrs: vec![],
                    label: None,
                    block: Block {
                        brace_token: token::Brace::default(),
                        stmts: vec![make_exit_stmt(exit_kind)],
                    },
                })),
            )),
        }),
        semi_token: token::Semi::default(),
    };

    block.stmts.push(Stmt::Local(local));
    block.stmts.extend(if_expr.then_branch.stmts);
}

/// Case 2, boolean → if !cond { continue; }
fn apply_bool_guard(block: &mut Block, if_expr: ExprIf, exit_kind: &ExitKind) {
    let guard = Expr::If(ExprIf {
        attrs: vec![],
        if_token: if_expr.if_token,
        cond: Box::new(negate_expr(*if_expr.cond)),
        then_branch: Block {
            brace_token: token::Brace::default(),
            stmts: vec![make_exit_stmt(exit_kind)],
        },
        else_branch: None,
    });

    block.stmts.push(Stmt::Expr(guard, None));
    block.stmts.extend(if_expr.then_branch.stmts);
}

/// Case 1 → if !cond { LIGHT; continue/return; } HEAVY
fn apply_case1_guard(block: &mut Block, if_expr: ExprIf, exit_kind: &ExitKind) {
    let (_, else_expr) = if_expr.else_branch.unwrap();

    let mut guard_stmts = match *else_expr {
        Expr::Block(b) => b.block.stmts,
        other => vec![Stmt::Expr(other, Some(token::Semi::default()))],
    };

    if !last_stmt_diverges(&guard_stmts) {
        guard_stmts.push(make_exit_stmt(exit_kind));
    }

    let guard = Expr::If(ExprIf {
        attrs: vec![],
        if_token: if_expr.if_token,
        cond: Box::new(negate_expr(*if_expr.cond)),
        then_branch: Block {
            brace_token: token::Brace::default(),
            stmts: guard_stmts,
        },
        else_branch: None,
    });

    block.stmts.push(Stmt::Expr(guard, None));
    block.stmts.extend(if_expr.then_branch.stmts);
}

fn make_exit_stmt(exit_kind: &ExitKind) -> Stmt {
    match exit_kind {
        ExitKind::Continue => make_continue_stmt(),
        ExitKind::Return => Stmt::Expr(
            Expr::Return(ExprReturn {
                attrs: vec![],
                return_token: token::Return::default(),
                expr: None,
            }),
            Some(token::Semi::default()),
        ),
    }
}

fn make_continue_stmt() -> Stmt {
    Stmt::Expr(
        Expr::Continue(ExprContinue {
            attrs: vec![],
            continue_token: token::Continue::default(),
            label: None,
        }),
        Some(token::Semi::default()),
    )
}

fn last_stmt_diverges(stmts: &[Stmt]) -> bool {
    let Some(Stmt::Expr(expr, _)) = stmts.last() else {
        return false;
    };

    matches!(expr, Expr::Return(_) | Expr::Continue(_) | Expr::Break(_))
}

/// Negate an expression using the cleanest transformation available.
fn negate_expr(expr: Expr) -> Expr {
    match expr {
        Expr::Unary(u) if matches!(u.op, UnOp::Not(_)) => *u.expr,
        Expr::Binary(mut b) => match b.op {
            BinOp::Eq(_) => {
                b.op = BinOp::Ne(token::Ne::default());

                Expr::Binary(b)
            }
            BinOp::Ne(_) => {
                b.op = BinOp::Eq(token::EqEq::default());

                Expr::Binary(b)
            }
            BinOp::Lt(_) => {
                b.op = BinOp::Ge(token::Ge::default());

                Expr::Binary(b)
            }
            BinOp::Le(_) => {
                b.op = BinOp::Gt(token::Gt::default());

                Expr::Binary(b)
            }
            BinOp::Gt(_) => {
                b.op = BinOp::Le(token::Le::default());

                Expr::Binary(b)
            }
            BinOp::Ge(_) => {
                b.op = BinOp::Lt(token::Lt::default());

                Expr::Binary(b)
            }
            BinOp::And(_) => {
                b.left = Box::new(negate_expr(*b.left));
                b.right = Box::new(negate_expr(*b.right));
                b.op = BinOp::Or(token::OrOr::default());

                Expr::Binary(b)
            }
            BinOp::Or(_) => {
                b.left = Box::new(negate_expr(*b.left));
                b.right = Box::new(negate_expr(*b.right));
                b.op = BinOp::And(token::AndAnd::default());

                Expr::Binary(b)
            }
            _ => wrap_not(Expr::Binary(b)),
        },
        Expr::Paren(mut p) => {
            p.expr = Box::new(negate_expr(*p.expr));

            Expr::Paren(p)
        }
        other => wrap_not(other),
    }
}

fn wrap_not(expr: Expr) -> Expr {
    Expr::Unary(ExprUnary {
        attrs: vec![],
        op: UnOp::Not(token::Not::default()),
        expr: Box::new(expr),
    })
}

fn block_weight(block: &Block) -> usize {
    block.stmts.iter().map(stmt_weight).sum()
}

fn else_weight(expr: &Expr) -> usize {
    match expr {
        Expr::Block(b) => block_weight(&b.block),
        _ => 1 + call_count(expr),
    }
}

fn stmt_weight(stmt: &Stmt) -> usize {
    1 + match stmt {
        Stmt::Local(l) => l.init.as_ref().map_or(0, |i| call_count(&i.expr)),
        Stmt::Expr(e, _) => call_count(e),
        _ => 0,
    }
}

fn call_count(expr: &Expr) -> usize {
    match expr {
        Expr::MethodCall(mc) => {
            1 + call_count(&mc.receiver) + mc.args.iter().map(call_count).sum::<usize>()
        }
        Expr::Call(c) => 1 + c.args.iter().map(call_count).sum::<usize>(),
        Expr::Macro(_) => 1,
        Expr::Struct(s) => {
            s.fields.len() + s.fields.iter().map(|f| call_count(&f.expr)).sum::<usize>()
        }
        Expr::If(i) => {
            call_count(&i.cond)
                + block_weight(&i.then_branch)
                + i.else_branch.as_ref().map_or(0, |(_, e)| call_count(e))
        }
        Expr::Block(b) => block_weight(&b.block),
        Expr::Match(m) => {
            call_count(&m.expr) + m.arms.iter().map(|a| call_count(&a.body)).sum::<usize>()
        }
        Expr::ForLoop(f) => call_count(&f.expr) + block_weight(&f.body),
        Expr::While(w) => call_count(&w.cond) + block_weight(&w.body),
        Expr::Loop(l) => block_weight(&l.body),
        Expr::Paren(p) => call_count(&p.expr),
        Expr::Reference(r) => call_count(&r.expr),
        Expr::Unary(u) => call_count(&u.expr),
        Expr::Binary(b) => call_count(&b.left) + call_count(&b.right),
        Expr::Return(r) => r.expr.as_ref().map_or(0, |e| call_count(e)),
        Expr::Assign(a) => call_count(&a.left) + call_count(&a.right),
        Expr::Index(i) => call_count(&i.expr) + call_count(&i.index),
        Expr::Field(f) => call_count(&f.base),
        Expr::Closure(c) => call_count(&c.body),
        Expr::Await(a) => call_count(&a.base),
        Expr::Try(t) => call_count(&t.expr),
        Expr::Cast(c) => call_count(&c.expr),
        Expr::Range(r) => {
            r.start.as_ref().map_or(0, |e| call_count(e))
                + r.end.as_ref().map_or(0, |e| call_count(e))
        }
        Expr::Tuple(t) => t.elems.iter().map(call_count).sum(),
        Expr::Array(a) => a.elems.iter().map(call_count).sum(),
        Expr::Repeat(r) => call_count(&r.expr),
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
    fn case1_then_heavier_than_else_violation() {
        let v = detect(
            r#"
fn foo() {
    if some_condition() {
        do_a();
        do_b();
        do_c();
    } else {
        return;
    }
}
"#,
        );

        assert_eq!(v.len(), 1);
        assert_eq!(v[0].line, 3);
    }

    #[test]
    fn case1_else_heavier_no_violation() {
        let v = detect(
            r#"
fn foo() {
    if cond() {
        do_a();
    } else {
        do_b();
        do_c();
        do_d();
    }
}
"#,
        );

        assert!(v.is_empty());
    }

    #[test]
    fn case1_equal_weight_no_violation() {
        let v = detect(
            r#"
fn foo() {
    if cond() {
        do_a();
        do_b();
    } else {
        do_c();
        do_d();
    }
}
"#,
        );

        assert!(v.is_empty());
    }

    #[test]
    fn case1_negated_cond_still_flagged() {
        let v = detect(
            r#"
fn foo() {
    if !some_condition() {
        do_a();
        do_b();
        do_c();
    } else {
        return;
    }
}
"#,
        );

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn case1_if_let_no_violation() {
        let v = detect(
            r#"
fn foo(val: Option<i32>) {
    if let Some(x) = val {
        do_a(x);
        do_b(x);
        do_c(x);
    } else {
        return;
    }
}
"#,
        );

        assert!(v.is_empty());
    }

    #[test]
    fn case1_let_chain_no_violation() {
        let v = detect(
            r#"
fn foo(a: Option<i32>, b: Option<i32>) {
    if let Some(x) = a && let Some(y) = b {
        do_a(x);
        do_b(y);
        do_c();
    } else {
        return;
    }
}
"#,
        );

        assert!(v.is_empty());
    }

    #[test]
    fn case1_method_chain_weight() {
        let v = detect(
            r#"
fn foo() {
    if some_condition() {
        group_names.entry(group).or_default().insert(name, local_idx);
    } else {
        return;
    }
}
"#,
        );

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn case1_impl_fn() {
        let v = detect(
            r#"
struct Foo;
impl Foo {
    fn bar(&self) {
        if some_condition() {
            do_a();
            do_b();
            do_c();
        } else {
            return;
        }
    }
}
"#,
        );

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn case2_if_let_last_in_for_loop_violation() {
        let v = detect(
            r#"
fn foo(items: &[Item]) {
    for item in items {
        let group = classify(item);
        if let Some(name) = item_name(item) {
            group_names.entry(group).or_default().insert(name, local_idx);
        }
    }
}
"#,
        );

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn case2_plain_if_last_in_while_loop_violation() {
        let v = detect(
            r#"
fn foo() {
    while running() {
        if some_condition() {
            do_heavy_work();
            process_result();
            finalize();
        }
    }
}
"#,
        );

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn case2_if_not_last_in_loop_no_violation() {
        let v = detect(
            r#"
fn foo(items: &[Item]) {
    for item in items {
        if let Some(name) = item_name(item) {
            group_names.entry(group).or_default().insert(name, local_idx);
        }
        cleanup(item);
    }
}
"#,
        );

        assert!(v.is_empty());
    }

    #[test]
    fn case2_let_chain_no_violation() {
        let v = detect(
            r#"
fn foo(refs: &[Ref]) {
    for r in refs {
        if let Some(names) = names
            && let Some(&dep_idx) = names.get(r.as_str())
        {
            violations.push(Violation { line: 1, message: String::new(), fixable: true });
            break;
        }
    }
}
"#,
        );

        assert!(v.is_empty());
    }

    #[test]
    fn case2_single_stmt_weight_2_violation() {
        let v = detect(
            r#"
fn foo(items: &[i32]) {
    for item in items {
        if valid(item) {
            process(item);
        }
    }
}
"#,
        );

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn case2_single_trivial_stmt_no_violation() {
        let v = detect(
            r#"
fn foo(items: &[i32]) {
    for item in items {
        if valid(*item) {
            x;
        }
    }
}
"#,
        );

        assert!(v.is_empty());
    }

    #[test]
    fn case2_loop_body_violation() {
        let v = detect(
            r#"
fn foo() {
    loop {
        let val = next();
        if let Some(x) = val {
            work(x);
            store(x);
        }
    }
}
"#,
        );

        assert_eq!(v.len(), 1);
    }

    #[test]
    fn case2_not_in_loop_no_violation() {
        let v = detect(
            r#"
fn foo(val: Option<i32>) {
    if let Some(x) = val {
        work(x);
        store(x);
    }
}
"#,
        );

        assert!(v.is_empty());
    }

    #[test]
    fn weight_method_chain_three_calls() {
        let block: Block =
            syn::parse_str("{ group_names.entry(group).or_default().insert(name, local_idx); }")
                .unwrap();

        assert_eq!(block_weight(&block), 4);
    }

    #[test]
    fn weight_single_variable() {
        let block: Block = syn::parse_str("{ x; }").unwrap();

        assert_eq!(block_weight(&block), 1);
    }

    #[test]
    fn weight_single_method_call() {
        let block: Block = syn::parse_str("{ x.m(); }").unwrap();

        assert_eq!(block_weight(&block), 2);
    }
}
