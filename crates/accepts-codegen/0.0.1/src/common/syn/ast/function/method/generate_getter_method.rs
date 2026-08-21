use syn::{
    Attribute, Block, Expr, ExprField, ExprPath, ExprReference, FnArg, Ident, ImplItemFn, Member,
    Path, Receiver, ReturnType, Signature, Type, TypePath, TypeReference, Visibility,
    punctuated::Punctuated,
    token::{And, Mut, RArrow},
};

use crate::common::syn::ext::{
    BlockConstructExt, ExprFieldConstructExt, ExprPathConstructExt, ExprReferenceConstructExt,
    IdentConstructExt, PunctuatedConstructExt, ReceiverConstructExt, SignatureConstructExt,
    TypePathConstructExt, TypeReferenceConstructExt,
};

pub fn generate_getter_method(
    attrs: Vec<Attribute>,
    vis: Visibility,
    gen_fn_ident: Ident,
    field_ident: Ident,
    field_ty: Type,
    ref_: bool,
    mut_: bool,
) -> ImplItemFn {
    let mut_token = match mut_ {
        true => Some(Mut::default()),
        false => None,
    };

    let sig = {
        let inputs = Punctuated::from_value(FnArg::Receiver(Receiver::from_ref_mut_colon_ty(
            match ref_ {
                true => Some((And::default(), None)),
                false => None,
            },
            mut_token.clone(),
            None,
            Box::new({
                let mut ty = Type::Path(TypePath::from_path(Path::from(Ident::from_str("Self"))));

                if ref_ {
                    ty = Type::Reference(TypeReference::from_mutability_elem(
                        mut_token.clone(),
                        Box::new(ty),
                    ))
                }

                ty
            }),
        )));

        let output = {
            let mut ty = field_ty.clone();

            if ref_ {
                ty = Type::Reference(TypeReference::from_mutability_elem(
                    mut_token.clone(),
                    Box::new(ty),
                ))
            }

            ReturnType::Type(RArrow::default(), Box::new(ty))
        };

        Signature::from_ident_inputs_output(gen_fn_ident, inputs, output)
    };

    let block = {
        let stmt = {
            let mut expr = Expr::Field(ExprField::from_base_member(
                Box::new(Expr::Path(ExprPath::from_path(Path::from(
                    Ident::from_str("self"),
                )))),
                Member::Named(field_ident.clone()),
            ));

            if ref_ {
                expr = Expr::Reference(ExprReference::from_mutability_expr(
                    mut_token.clone(),
                    Box::new(expr),
                ))
            }

            syn::Stmt::Expr(expr, None)
        };

        Block::from_stmts(vec![stmt])
    };

    ImplItemFn {
        attrs,
        vis,
        defaultness: None,
        sig,
        block,
    }
}
