use quote::format_ident;
use syn::{Path, PathSegment};

use crate::common::{
    context::CodegenContext, function::generate::crate_codegen_path,
    syn::ext::PathSegmentConstructExt,
};

pub fn codegen_acceptor_path(ctx: &CodegenContext) -> Path {
    let mut path = crate_codegen_path(ctx);
    let segments = &mut path.segments;

    segments.push(PathSegment::from_ident(format_ident!("acceptor")));

    path
}
