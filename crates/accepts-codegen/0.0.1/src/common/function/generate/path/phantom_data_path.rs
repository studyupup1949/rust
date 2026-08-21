use syn::{
    AngleBracketedGenericArguments, GenericArgument, Ident, Path, PathArguments, PathSegment,
    Token, Type, TypeTuple, punctuated::Punctuated, token::PathSep,
};

use crate::common::syn::ext::{
    AngleBracketedGenericArgumentsConstructExt, IdentConstructExt, PathSegmentConstructExt,
    PunctuatedConstructExt, TypeTupleConstructExt,
};

use super::core_path;

pub fn phantom_data_path(args: Punctuated<Type, Token![,]>) -> Path {
    let mut path = core_path();
    let segments = &mut path.segments;

    segments.push(PathSegment::from_ident(Ident::from_str("marker")));
    segments.push(PathSegment::from_parts(
        Ident::from_str("PhantomData"),
        if args.is_empty() {
            PathArguments::None
        } else {
            PathArguments::AngleBracketed(AngleBracketedGenericArguments::from_colon2_args(
                Some(PathSep::default()),
                Punctuated::from_value(GenericArgument::Type({
                    if args.len() == 1 {
                        args.into_iter().next().unwrap()
                    } else {
                        Type::Tuple(TypeTuple::from_elems(args))
                    }
                })),
            ))
        },
    ));

    path
}
