use quote::format_ident;
use syn::{
    AngleBracketedGenericArguments, AssocType, GenericArgument, Lifetime, Path, PathArguments,
    PathSegment, Type, punctuated::Punctuated, token::PathSep,
};

use crate::common::{
    context::CodegenContext,
    function::generate::crate_core_traits_path,
    syn::ext::{
        AngleBracketedGenericArgumentsConstructExt, AssocTypeConstructExt, PathSegmentConstructExt,
        PunctuatedConstructExt,
    },
};

fn next_acceptors_path_internal(
    ctx: &CodegenContext,
    mut_: bool,
    acceptor_type: Option<Type>,
    iter: Option<(Lifetime, Type)>,
) -> Path {
    let mut path = crate_core_traits_path(ctx);
    let segments = &mut path.segments;

    segments.push(PathSegment::from_parts(
        if !mut_ {
            format_ident!("NextAcceptors")
        } else {
            format_ident!("NextAcceptorsMut")
        },
        PathArguments::AngleBracketed(AngleBracketedGenericArguments::from_colon2_args(
            Some(PathSep::default()),
            {
                let mut args = Punctuated::new();

                //Acceptor = *
                if let Some(acceptor_type) = acceptor_type {
                    args.push(GenericArgument::AssocType(AssocType::from_ident_ty(
                        format_ident!("Acceptor"),
                        acceptor_type,
                    )));
                }

                //Iter<'*> = *
                if let Some((iter_lifetime, iter_type)) = iter {
                    args.push(GenericArgument::AssocType(
                        AssocType::from_ident_generics_ty(
                            format_ident!("Iter"),
                            Some(AngleBracketedGenericArguments::from_args(
                                Punctuated::from_value(GenericArgument::Lifetime(iter_lifetime)),
                            )),
                            iter_type,
                        ),
                    ));
                }

                args
            },
        )),
    ));

    path
}

pub fn next_acceptors_path(
    ctx: &CodegenContext,
    acceptor_type: Option<Type>,
    iter: Option<(Lifetime, Type)>,
) -> Path {
    next_acceptors_path_internal(ctx, false, acceptor_type, iter)
}

pub fn next_acceptors_path_mut(
    ctx: &CodegenContext,
    acceptor_type: Option<Type>,
    iter: Option<(Lifetime, Type)>,
) -> Path {
    next_acceptors_path_internal(ctx, true, acceptor_type, iter)
}
