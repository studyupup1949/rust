use syn::{Ident, Path, PathSegment, Type};

use crate::common::syn::ext::{IdentConstructExt, PathSegmentConstructExt};

use super::option_path;

pub fn option_some_path(option_t: Option<Type>) -> Path {
    let mut path = option_path(option_t);

    path.segments
        .push(PathSegment::from_ident(Ident::from_str("Some")));

    path
}
