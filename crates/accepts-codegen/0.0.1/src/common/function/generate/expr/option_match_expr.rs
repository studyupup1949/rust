use syn::{
    Arm, Expr, ExprMatch, ExprPath, Ident, Pat, PatIdent, PatTupleStruct, Type,
    punctuated::Punctuated,
};

use crate::common::syn::ext::{
    ArmConstructExt, ExprMatchConstructExt, ExprPathConstructExt, PatIdentConstructExt,
    PatTupleStructConstructExt, PunctuatedConstructExt,
};

use super::{option_none_path, option_some_path};

pub fn option_match_expr(
    option_t: Option<Type>,
    scrutinee: Expr,
    none_body: Expr,
    some_binding: Ident,
    some_body: Expr,
) -> ExprMatch {
    ExprMatch::from_expr_arms(Box::new(scrutinee), {
        let mut arms = Vec::with_capacity(2);

        arms.push(Arm::from_pat_guard_body(
            Pat::Path(ExprPath::from_path(option_none_path(option_t.clone()))),
            None,
            Box::new(none_body),
        ));

        arms.push(Arm::from_pat_guard_body(
            Pat::TupleStruct(PatTupleStruct::from_path_elems(
                option_some_path(option_t),
                Punctuated::from_value(Pat::Ident(PatIdent::from_ident(some_binding))),
            )),
            None,
            Box::new(some_body),
        ));

        arms
    })
}
