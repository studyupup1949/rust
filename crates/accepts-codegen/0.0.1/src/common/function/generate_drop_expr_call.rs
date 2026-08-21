use syn::{Expr, ExprCall, ExprPath, Ident, PathSegment, punctuated::Punctuated};

use crate::common::syn::ext::{
    ExprCallConstructExt, ExprPathConstructExt, IdentConstructExt, PathSegmentConstructExt,
    PunctuatedConstructExt,
};

use super::generate::core_path;

pub fn generate_drop_expr_call(drop_target_expr: Expr) -> ExprCall {
    let func = Expr::Path(ExprPath::from_path({
        let mut path = core_path();
        let segments = &mut path.segments;

        segments.push(PathSegment::from_ident(Ident::from_str("mem")));

        segments.push(PathSegment::from_ident(Ident::from_str("drop")));

        path
    }));

    let args = Punctuated::from_value(drop_target_expr);

    ExprCall::from_func_args(Box::new(func), args)
}
