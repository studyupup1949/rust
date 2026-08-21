use syn::{
    AngleBracketedGenericArguments, GenericArgument,
    punctuated::Punctuated,
    token::{Comma, Gt, Lt, PathSep},
};

pub trait AngleBracketedGenericArgumentsConstructExt {
    fn from_parts(
        colon2_token: Option<PathSep>,
        lt_token: Lt,
        args: Punctuated<GenericArgument, Comma>,
        gt_token: Gt,
    ) -> AngleBracketedGenericArguments;

    fn from_colon2_args(
        colon2_token: Option<PathSep>,
        args: Punctuated<GenericArgument, Comma>,
    ) -> AngleBracketedGenericArguments;

    fn from_args(args: Punctuated<GenericArgument, Comma>) -> AngleBracketedGenericArguments;
}

impl AngleBracketedGenericArgumentsConstructExt for AngleBracketedGenericArguments {
    fn from_parts(
        colon2_token: Option<PathSep>,
        lt_token: Lt,
        args: Punctuated<GenericArgument, Comma>,
        gt_token: Gt,
    ) -> AngleBracketedGenericArguments {
        Self {
            colon2_token,
            lt_token,
            args,
            gt_token,
        }
    }

    fn from_colon2_args(
        colon2_token: Option<PathSep>,
        args: Punctuated<GenericArgument, Comma>,
    ) -> AngleBracketedGenericArguments {
        <Self as AngleBracketedGenericArgumentsConstructExt>::from_parts(
            colon2_token,
            Lt::default(),
            args,
            Gt::default(),
        )
    }

    fn from_args(args: Punctuated<GenericArgument, Comma>) -> AngleBracketedGenericArguments {
        <Self as AngleBracketedGenericArgumentsConstructExt>::from_colon2_args(None, args)
    }
}
