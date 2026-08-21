use quote::format_ident;
use syn::{Expr, ExprField, ExprPath, ExprReference, Member, Path, token::Mut};

use crate::common::syn::ext::{
    ExprFieldConstructExt, ExprPathConstructExt, ExprReferenceConstructExt,
};

#[allow(dead_code)]
pub enum AccessMode {
    Owned,
    Ref,
    RefMut,
}

#[inline]
pub fn self_field_access_expr(member: impl Into<Member>, mode: AccessMode) -> Expr {
    let member = member.into();

    let base = Expr::Path(ExprPath::from_path(Path::from(format_ident!("self"))));
    let field = Expr::Field(ExprField::from_base_member(Box::new(base), member));

    match mode {
        AccessMode::Owned => field,
        AccessMode::Ref => {
            Expr::Reference(ExprReference::from_mutability_expr(None, Box::new(field)))
        }
        AccessMode::RefMut => Expr::Reference(ExprReference::from_mutability_expr(
            Some(Mut::default()),
            Box::new(field),
        )),
    }
}
