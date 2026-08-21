use syn::{Ident, PathArguments, PathSegment};

pub trait PathSegmentConstructExt {
    fn from_parts(ident: Ident, arguments: PathArguments) -> PathSegment;

    fn from_ident(ident: Ident) -> PathSegment;
}

impl PathSegmentConstructExt for PathSegment {
    fn from_parts(ident: Ident, arguments: PathArguments) -> PathSegment {
        PathSegment { ident, arguments }
    }

    fn from_ident(ident: Ident) -> PathSegment {
        PathSegment {
            ident,
            arguments: PathArguments::None,
        }
    }
}
