use syn::{Ident, Path, PathSegment, punctuated::Punctuated, token::PathSep};

use crate::common::syn::ext::{
    IdentConstructExt, PathConstructExt, PathSegmentConstructExt, PunctuatedConstructExt,
};

pub fn core_path() -> Path {
    Path::from_parts(
        Some(PathSep::default()),
        Punctuated::from_value(PathSegment::from_ident(Ident::from_str("core"))),
    )
}
