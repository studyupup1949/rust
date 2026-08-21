use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream, Parser};

use crate::{
    acceptor::{
        accepts_auto_impl::args::{AcceptsImplData, AsyncAutoImplArgs, AutoPinDynAcceptsArgs},
        common::ast::accepts_trait_ast::{Accepts, AsyncAccepts},
    },
    common::{context::CodegenContext, function::map_err_to_compile_error, macros::ok_or_return},
};

use super::builder::{build_async_auto_impl, build_dyn_auto_impl};

pub fn expand_async_impl(
    ctx: &CodegenContext,
    attr: TokenStream,
    mut item: TokenStream,
) -> TokenStream {
    let args: AsyncAutoImplArgs = ok_or_return!(map_err_to_compile_error(
        (|i: ParseStream| Parse::parse(i)).parse2(attr)
    ));

    let base: AcceptsImplData = ok_or_return!(map_err_to_compile_error(
        (|i: ParseStream| AcceptsImplData::parse(Accepts, i)).parse2(item.clone())
    ));

    build_async_auto_impl(ctx, base.accepts_t_type, args.outer_attrs)
        .into_item_impl_from_type(ctx, base.self_ty, base.generics)
        .to_tokens(&mut item);

    item
}

pub fn expand_dyn_impl(
    ctx: &CodegenContext,
    attr: TokenStream,
    mut item: TokenStream,
) -> TokenStream {
    let args: AutoPinDynAcceptsArgs = ok_or_return!(map_err_to_compile_error(
        (|i: ParseStream| Parse::parse(i)).parse2(attr)
    ));
    let base: AcceptsImplData = ok_or_return!(map_err_to_compile_error(
        (|i: ParseStream| AcceptsImplData::parse(AsyncAccepts, i)).parse2(item.clone())
    ));

    build_dyn_auto_impl(ctx, base.accepts_t_type, args.outer_attrs)
        .into_item_impl_from_type(ctx, base.self_ty, base.generics)
        .to_tokens(&mut item);

    item
}
