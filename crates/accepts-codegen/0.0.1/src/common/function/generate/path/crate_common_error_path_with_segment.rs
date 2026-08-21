use syn::{Path, PathSegment};

use crate::common::context::CodegenContext;

use super::crate_common_error_path;

pub fn crate_common_error_path_with_segment(ctx: &CodegenContext, segment: PathSegment) -> Path {
    let mut path = crate_common_error_path(ctx);
    let segments = &mut path.segments;

    segments.push(segment);

    path
}
