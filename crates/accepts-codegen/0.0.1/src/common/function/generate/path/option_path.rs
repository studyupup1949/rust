use syn::{
    AngleBracketedGenericArguments, GenericArgument, Ident, Path, PathArguments, PathSegment, Type,
    punctuated::Punctuated, token::PathSep,
};

use crate::common::syn::ext::{
    AngleBracketedGenericArgumentsConstructExt, IdentConstructExt, PathSegmentConstructExt,
    PunctuatedConstructExt,
};

use super::core_path;

pub fn option_path(option_t: Option<Type>) -> Path {
    let mut path = core_path();
    let segments = &mut path.segments;

    segments.push(PathSegment::from_ident(Ident::from_str("option")));
    segments.push(PathSegment::from_parts(
        Ident::from_str("Option"),
        if let Some(option_t) = option_t {
            PathArguments::AngleBracketed(AngleBracketedGenericArguments::from_colon2_args(
                Some(PathSep::default()),
                Punctuated::from_value(GenericArgument::Type(option_t)),
            ))
        } else {
            PathArguments::None
        },
    ));

    path
}
