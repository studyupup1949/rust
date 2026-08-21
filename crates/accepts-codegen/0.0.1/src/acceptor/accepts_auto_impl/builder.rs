use syn::{
    Attribute, Block, Expr, ExprCall, ExprPath, Ident, Meta, Path, PathSegment, Stmt, Type,
    punctuated::Punctuated, token::Comma,
};

use crate::{
    acceptor::common::{
        ast::accepts_trait_ast::{
            Accepts, AcceptsBuilder, AcceptsGenerics, AcceptsInfo, AcceptsT, AsyncAccepts,
            DynAsyncAccepts, PartialAcceptsTraitImpl,
        },
        function::generate::{generate_accept_expr_call, generate_pinboxed_accept_async_expr_call},
    },
    common::{
        context::CodegenContext,
        syn::ext::{
            AttributeConstructExt, BlockConstructExt, ExprCallConstructExt, ExprPathConstructExt,
            IdentConstructExt, PathConstructExt, PathSegmentConstructExt, PunctuatedConstructExt,
        },
    },
};

pub fn build_async_auto_impl(
    ctx: &CodegenContext,
    base_accepts_t_type: Type,
    accepts_outer_attrs: Punctuated<Meta, Comma>,
) -> PartialAcceptsTraitImpl<AsyncAccepts> {
    let expr = {
        Expr::Call(ExprCall::from_func_args(
            Box::new(Expr::Path(ExprPath::from_path(Path::from_segments({
                let mut segments = Punctuated::new();

                segments.push(PathSegment::from_ident(Ident::from_str("core")));
                segments.push(PathSegment::from_ident(Ident::from_str("future")));
                segments.push(PathSegment::from_ident(Ident::from_str("ready")));

                segments
            })))),
            Punctuated::from_value(Expr::Call(generate_accept_expr_call(
                ctx,
                Accepts,
                base_accepts_t_type.clone(),
                Expr::Path(ExprPath::from_path(Path::from(Ident::from_str("self")))),
                Expr::Path(ExprPath::from_path(Path::from(Ident::from_str("value")))),
            ))),
        ))
    };
    build_auto_impl_inner(
        base_accepts_t_type,
        accepts_outer_attrs,
        AsyncAccepts,
        Block::from_stmts(vec![Stmt::Expr(expr, None)]),
    )
}

pub fn build_dyn_auto_impl(
    ctx: &CodegenContext,
    base_accepts_t_type: Type,
    accepts_outer_attrs: Punctuated<Meta, Comma>,
) -> PartialAcceptsTraitImpl<DynAsyncAccepts> {
    build_auto_impl_inner(
        base_accepts_t_type.clone(),
        accepts_outer_attrs,
        DynAsyncAccepts,
        Block::from_stmts(vec![Stmt::Expr(
            Expr::Call(generate_pinboxed_accept_async_expr_call(
                ctx,
                base_accepts_t_type,
                Expr::Path(ExprPath::from_path(Path::from(Ident::from_str("self")))),
                Expr::Path(ExprPath::from_path(Path::from(Ident::from_str("value")))),
            )),
            None,
        )]),
    )
}

fn build_auto_impl_inner<A: AcceptsInfo + AcceptsBuilder>(
    base_accepts_t_type: Type,
    accepts_outer_attrs: Punctuated<Meta, Comma>,
    accepts: A,
    accepts_block: Block,
) -> PartialAcceptsTraitImpl<A> {
    let mut accepts = PartialAcceptsTraitImpl::from_accepts(
        AcceptsGenerics::default(),
        accepts,
        AcceptsT::Type(base_accepts_t_type),
        accepts_block,
    );

    accepts.attrs.extend(
        accepts_outer_attrs
            .into_iter()
            .map(|meta| Attribute::from_style_meta(syn::AttrStyle::Outer, meta)),
    );

    accepts
}
