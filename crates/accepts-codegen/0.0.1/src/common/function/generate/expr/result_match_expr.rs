use syn::{Arm, Expr, ExprMatch, Ident, Pat, PatIdent, PatTupleStruct, punctuated::Punctuated};

use crate::common::syn::ext::{
    ArmConstructExt, ExprMatchConstructExt, PatIdentConstructExt, PatTupleStructConstructExt,
    PunctuatedConstructExt,
};

use super::{result_err_path, result_ok_path};

pub fn result_match_expr(
    scrutinee: Expr,
    ok_binding: Ident,
    ok_body: Expr,
    err_binding: Ident,
    err_body: Expr,
) -> ExprMatch {
    ExprMatch::from_expr_arms(Box::new(scrutinee), {
        let mut arms = Vec::with_capacity(2);

        arms.push(Arm::from_pat_guard_body(
            Pat::TupleStruct(PatTupleStruct::from_path_elems(
                result_ok_path(),
                Punctuated::from_value(Pat::Ident(PatIdent::from_ident(ok_binding))),
            )),
            None,
            Box::new(ok_body),
        ));

        arms.push(Arm::from_pat_guard_body(
            Pat::TupleStruct(PatTupleStruct::from_path_elems(
                result_err_path(),
                Punctuated::from_value(Pat::Ident(PatIdent::from_ident(err_binding))),
            )),
            None,
            Box::new(err_body),
        ));

        arms
    })
}
