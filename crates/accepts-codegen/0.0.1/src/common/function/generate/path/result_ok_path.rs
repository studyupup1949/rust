use syn::{Ident, Path, PathSegment};

use crate::common::syn::ext::{IdentConstructExt, PathSegmentConstructExt};

use super::result_path;

pub fn result_ok_path() -> Path {
    let mut path = result_path();
    let segments = &mut path.segments;

    segments.push(PathSegment::from_ident(Ident::from_str("Ok")));

    path
}
