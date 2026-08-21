use syn::{Expr, ExprAwait, Type};

use crate::{
    acceptor::common::ast::accepts_trait_ast::AcceptsInfo,
    common::{context::CodegenContext, syn::ext::ExprAwaitConstructExt},
};

use super::generate_accept_expr_call;

pub fn generate_accept_expr_maybe_await<A: AcceptsInfo>(
    ctx: &CodegenContext,
    accepts_info: A,
    accepts_t_type: Type,
    accepts_self_expr: Expr,
    accepts_value_expr: Expr,
) -> Expr {
    let mut accept_expr = Expr::Call(generate_accept_expr_call(
        ctx,
        accepts_info,
        accepts_t_type,
        accepts_self_expr,
        accepts_value_expr,
    ));

    if accepts_info.accept_is_async() {
        accept_expr = Expr::Await(ExprAwait::from_base(Box::new(accept_expr)))
    }

    accept_expr
}
