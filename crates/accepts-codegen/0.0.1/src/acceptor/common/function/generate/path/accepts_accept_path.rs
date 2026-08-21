use syn::{Ident, Path, PathSegment, Type};

use crate::{
    acceptor::common::ast::accepts_trait_ast::AcceptsInfo,
    common::{
        context::CodegenContext,
        syn::ext::{IdentConstructExt, PathSegmentConstructExt},
    },
};

use super::accepts_path;

pub fn accepts_accept_path<A: AcceptsInfo>(
    ctx: &CodegenContext,
    accepts_info: A,
    accepts_t_type: Type,
) -> Path {
    let mut path = accepts_path(ctx, accepts_info, accepts_t_type);
    let segments = &mut path.segments;

    segments.push(PathSegment::from_ident(Ident::from_str(
        accepts_info.accept_fn_name(),
    )));

    path
}
