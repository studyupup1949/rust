use quote::format_ident;
use syn::{
    AngleBracketedGenericArguments, AssocType, GenericArgument, Path, PathArguments, PathSegment,
    Type, punctuated::Punctuated,
};

use crate::common::syn::ext::{
    AngleBracketedGenericArgumentsConstructExt, AssocTypeConstructExt, PathSegmentConstructExt,
    PunctuatedConstructExt,
};

use super::core_path;

pub fn future_path(future_output_ty: Type) -> Path {
    let mut path = core_path();
    let segments = &mut path.segments;

    segments.push(PathSegment::from_ident(format_ident!("future")));
    segments.push(PathSegment::from_parts(
        format_ident!("Future"),
        PathArguments::AngleBracketed(AngleBracketedGenericArguments::from_args(
            Punctuated::from_value(GenericArgument::AssocType(AssocType::from_ident_ty(
                format_ident!("Output"),
                future_output_ty,
            ))),
        )),
    ));

    path
}
