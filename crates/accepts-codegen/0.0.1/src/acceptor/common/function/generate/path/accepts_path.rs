use syn::{
    AngleBracketedGenericArguments, GenericArgument, Ident, Path, PathArguments, PathSegment, Type,
    punctuated::Punctuated, token::PathSep,
};

use crate::{
    acceptor::common::ast::accepts_trait_ast::AcceptsInfo,
    common::{
        context::CodegenContext,
        function::generate::crate_core_traits_path,
        syn::ext::{
            AngleBracketedGenericArgumentsConstructExt, IdentConstructExt, PathSegmentConstructExt,
            PunctuatedConstructExt,
        },
    },
};

pub fn accepts_path<A: AcceptsInfo>(
    ctx: &CodegenContext,
    accepts_info: A,
    accepts_t_type: Type,
) -> Path {
    let mut path = crate_core_traits_path(ctx);
    let segments = &mut path.segments;

    segments.push(PathSegment::from_parts(
        Ident::from_str(accepts_info.accepts_name()),
        PathArguments::AngleBracketed(AngleBracketedGenericArguments::from_colon2_args(
            Some(PathSep::default()),
            Punctuated::from_value(GenericArgument::Type(accepts_t_type)),
        )),
    ));

    path
}
