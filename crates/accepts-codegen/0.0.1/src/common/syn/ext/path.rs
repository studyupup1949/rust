use syn::{Path, PathSegment, punctuated::Punctuated, token::PathSep};

pub trait PathConstructExt {
    fn from_parts(
        leading_colon: Option<PathSep>,
        segments: Punctuated<PathSegment, PathSep>,
    ) -> Path;

    fn from_segments(segments: Punctuated<PathSegment, PathSep>) -> Path;
}

impl PathConstructExt for Path {
    fn from_parts(
        leading_colon: Option<PathSep>,
        segments: Punctuated<PathSegment, PathSep>,
    ) -> Path {
        Path {
            leading_colon,
            segments,
        }
    }

    fn from_segments(segments: Punctuated<PathSegment, PathSep>) -> Path {
        Path {
            leading_colon: None,
            segments,
        }
    }
}
