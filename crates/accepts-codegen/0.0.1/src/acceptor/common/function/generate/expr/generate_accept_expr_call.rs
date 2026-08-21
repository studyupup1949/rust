use syn::{Expr, ExprCall, ExprPath, Type, punctuated::Punctuated};

use crate::{
    acceptor::common::ast::accepts_trait_ast::AcceptsInfo,
    common::{
        context::CodegenContext,
        syn::ext::{ExprCallConstructExt, ExprPathConstructExt},
    },
};

use super::accepts_accept_path;

pub fn generate_accept_expr_call<A: AcceptsInfo>(
    ctx: &CodegenContext,
    accepts_info: A,
    accepts_t_type: Type,
    accepts_self_expr: Expr,
    accepts_value_expr: Expr,
) -> ExprCall {
    let args = {
        let mut args = Punctuated::new();

        args.push(accepts_self_expr);

        args.push(accepts_value_expr);

        args
    };

    ExprCall::from_func_args(
        Box::new(Expr::Path(ExprPath::from_path(accepts_accept_path(
            ctx,
            accepts_info,
            accepts_t_type,
        )))),
        args,
    )
}

/*//
//<Self as Accepts<_>>::accept(self, value)
pub fn generate_accept_qself_expr_call<A: AcceptsInfo>(
    accepts_info: A,
    accepts_mod_path: AcceptsModPath,
    accepts_self_type: Type,
    accepts_value_type: Expr,
    accepts_t_type: Type,
) -> ExprCall {
    let accepts_path = {
        let mut accepts_path = accepts_mod_path.into_path(
            Ident::from_str(accepts_info.accepts_name()),
            Type::Infer(TypeInfer::new_default()),
        );

        accepts_path
            .segments
            .push(PathSegment::from_ident(Ident::from_str(
                accepts_info.accept_fn_name(),
            )));

        accepts_path
    };

    let func = Expr::Path(ExprPath::from_qself_path(
        Some({
            QSelf::from_ty_position_as(
                Box::new(accepts_self_type),
                accepts_path.segments.len() - 1,
                Some(As::default()),
            )
        }),
        accepts_path,
    ));

    let args = {
        let mut args = Punctuated::new();

        args.push(Expr::Path(ExprPath::from_path(Path::from(
            Ident::from_str("self"),
        ))));

        args.push(accepts_value_type);

        args
    };

    ExprCall::from_func_args(Box::new(func), args)
}
 */
