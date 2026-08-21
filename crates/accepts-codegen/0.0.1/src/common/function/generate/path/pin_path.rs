use syn::{
    AngleBracketedGenericArguments, GenericArgument, Ident, Path, PathArguments, PathSegment, Type,
    punctuated::Punctuated,
};

use crate::common::syn::ext::{
    AngleBracketedGenericArgumentsConstructExt, IdentConstructExt, PathSegmentConstructExt,
    PunctuatedConstructExt,
};

use super::core_path;

pub fn pin_path(pin_ptr_type: Type) -> Path {
    let mut path = core_path();
    let segments = &mut path.segments;

    segments.push(PathSegment::from_ident(Ident::from_str("pin")));
    segments.push(PathSegment::from_parts(
        Ident::from_str("Pin"),
        PathArguments::AngleBracketed(AngleBracketedGenericArguments::from_args(
            Punctuated::from_value(GenericArgument::Type(pin_ptr_type)),
        )),
    ));

    path
}
