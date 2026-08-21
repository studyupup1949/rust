use syn::{Ident, Path, PathSegment};

use crate::common::syn::ext::{IdentConstructExt, PathSegmentConstructExt};

use super::core_path;

pub fn result_path() -> Path {
    let mut path = core_path();
    let segments = &mut path.segments;

    segments.push(PathSegment::from_ident(Ident::from_str("result")));
    segments.push(PathSegment::from_ident(Ident::from_str("Result")));

    path
}
