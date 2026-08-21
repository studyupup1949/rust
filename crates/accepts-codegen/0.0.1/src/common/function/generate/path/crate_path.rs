use quote::format_ident;
use syn::{Path, PathSegment, punctuated::Punctuated, token::PathSep};

use crate::common::{
    context::CodegenContext,
    syn::ext::{PathConstructExt, PathSegmentConstructExt, PunctuatedConstructExt},
};

pub fn crate_path(ctx: &CodegenContext) -> Path {
    if !ctx.internal {
        Path::from_parts(
            Some(PathSep::default()),
            Punctuated::from_value(PathSegment::from_ident(format_ident!("accepts"))),
        )
    } else {
        Path::from_parts(
            None,
            Punctuated::from_value(PathSegment::from_ident(format_ident!("crate"))),
        )
    }
}
