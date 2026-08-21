use quote::format_ident;
use syn::{Path, PathSegment, Type};

use crate::common::syn::ext::PathSegmentConstructExt;

use super::option_path;

pub fn option_none_path(option_t: Option<Type>) -> Path {
    let mut path = option_path(option_t);
    path.segments
        .push(PathSegment::from_ident(format_ident!("None")));
    path
}
