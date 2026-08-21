use syn::{
    AngleBracketedGenericArguments, GenericArgument, Ident, Path, PathArguments, PathSegment, Type,
    punctuated::Punctuated,
};

use crate::common::{
    context::CodegenContext,
    syn::ext::{
        AngleBracketedGenericArgumentsConstructExt, IdentConstructExt, PathSegmentConstructExt,
        PunctuatedConstructExt,
    },
};

use super::crate_internal_path;

pub fn box_path(ctx: &CodegenContext, box_t_type: Type) -> Path {
    let mut path = crate_internal_path(ctx);
    let segments = &mut path.segments;

    segments.push(PathSegment::from_ident(Ident::from_str("alloc")));
    segments.push(PathSegment::from_ident(Ident::from_str("boxed")));
    segments.push(PathSegment::from_parts(
        Ident::from_str("Box"),
        PathArguments::AngleBracketed(AngleBracketedGenericArguments::from_args(
            Punctuated::from_value(GenericArgument::Type(box_t_type)),
        )),
    ));

    path
}
