use quote::format_ident;
use syn::{Path, PathSegment};

use crate::common::{context::CodegenContext, syn::ext::PathSegmentConstructExt};

use super::crate_common_path;

pub fn crate_common_error_path(ctx: &CodegenContext) -> Path {
    let mut path = crate_common_path(ctx);
    let segments = &mut path.segments;

    segments.push(PathSegment::from_ident(format_ident!("error")));

    path
}
