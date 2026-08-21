use syn::{
    Ident, Path, PathArguments, PathSegment, punctuated::Punctuated, spanned::Spanned,
    token::PathSep,
};

use crate::common::syn::ext::{PathConstructExt, PathSegmentConstructExt};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathSplitLastArgs {
    pub leading_colon: Option<PathSep>,
    pub segments: Punctuated<PathSegment, PathSep>,
    pub last_segment_ident: Ident,
}

#[allow(dead_code)]
impl PathSplitLastArgs {
    pub fn from_parts(
        leading_colon: Option<PathSep>,
        segments: Punctuated<PathSegment, PathSep>,
        last_segment_ident: Ident,
    ) -> Self {
        Self {
            leading_colon,
            segments,
            last_segment_ident,
        }
    }

    pub fn from_path(mut path: Path) -> syn::Result<(Self, PathArguments)> {
        if let Some(last_pair) = path.segments.pop() {
            let last = last_pair.into_value();
            Ok((
                Self::from_parts(path.leading_colon, path.segments, last.ident),
                last.arguments,
            ))
        } else {
            let msg = if path.leading_colon.is_some() {
                "expected a path segment after `::`"
            } else {
                "expected a non-empty path (at least one segment)"
            };
            Err(syn::Error::new(path.span(), msg))
        }
    }

    pub fn into_path(mut self, last_segment_arguments: PathArguments) -> Path {
        self.segments.push(PathSegment::from_parts(
            self.last_segment_ident,
            last_segment_arguments,
        ));
        Path::from_parts(self.leading_colon, self.segments)
    }
}
