use syn::{Expr, ExprCall, ExprPath, Ident, PathSegment, Type, TypeInfer, punctuated::Punctuated};

use crate::{
    acceptor::common::ast::accepts_trait_ast::AsyncAccepts,
    common::{
        context::CodegenContext,
        function::generate::box_path,
        syn::ext::{
            ExprCallConstructExt, ExprPathConstructExt, IdentConstructExt, PathSegmentConstructExt,
            PunctuatedConstructExt, TypeInferConstructExt,
        },
    },
};

use super::generate_accept_expr_call;

//Box::pin(
//    <Self as ::accepts_core::AsyncAccepts<_>>::accept_async(self, value),
//)
pub fn generate_pinboxed_accept_async_expr_call(
    ctx: &CodegenContext,
    accepts_t_type: Type,
    accepts_self_expr: Expr,
    accepts_value_expr: Expr,
) -> ExprCall {
    let func = Expr::Path(ExprPath::from_path({
        let mut boxpath = box_path(ctx, Type::Infer(TypeInfer::new_default()));
        let segments = &mut boxpath.segments;

        segments.push(PathSegment::from_ident(Ident::from_str("pin")));

        boxpath
    }));

    let args = Punctuated::from_value(Expr::Call(generate_accept_expr_call(
        ctx,
        AsyncAccepts,
        accepts_t_type,
        accepts_self_expr,
        accepts_value_expr,
    )));

    ExprCall::from_func_args(Box::new(func), args)
}
