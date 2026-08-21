use syn::{Ident, Path, PathSegment};

use crate::common::{
    context::CodegenContext,
    syn::ext::{IdentConstructExt, PathSegmentConstructExt},
};

use super::crate_path;

pub fn crate_codegen_path(ctx: &CodegenContext) -> Path {
    let mut crate_path = crate_path(ctx);
    let segments = &mut crate_path.segments;

    segments.push(PathSegment::from_ident(Ident::from_str("codegen")));

    crate_path
}
