use quote::format_ident;
use syn::{Path, PathSegment};

use crate::common::{context::CodegenContext, syn::ext::PathSegmentConstructExt};

use super::crate_path;

pub fn crate_internal_path(ctx: &CodegenContext) -> Path {
    let mut path = crate_path(ctx);
    let segments = &mut path.segments;

    segments.push(PathSegment::from_ident(format_ident!("__internal")));

    path
}
