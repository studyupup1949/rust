use syn::{
    Attribute, Pat, PatTupleStruct, Path, QSelf,
    punctuated::Punctuated,
    token::{Comma, Paren},
};

pub trait PatTupleStructConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        qself: Option<QSelf>,
        path: Path,
        paren_token: Paren,
        elems: Punctuated<Pat, Comma>,
    ) -> PatTupleStruct;

    fn from_qself_path_elems(
        qself: Option<QSelf>,
        path: Path,
        elems: Punctuated<Pat, Comma>,
    ) -> PatTupleStruct;

    fn from_path_elems(path: Path, elems: Punctuated<Pat, Comma>) -> PatTupleStruct;
}

impl PatTupleStructConstructExt for PatTupleStruct {
    fn from_parts(
        attrs: Vec<Attribute>,
        qself: Option<QSelf>,
        path: Path,
        paren_token: Paren,
        elems: Punctuated<Pat, Comma>,
    ) -> PatTupleStruct {
        PatTupleStruct {
            attrs,
            qself,
            path,
            paren_token,
            elems,
        }
    }

    fn from_qself_path_elems(
        qself: Option<QSelf>,
        path: Path,
        elems: Punctuated<Pat, Comma>,
    ) -> PatTupleStruct {
        Self::from_parts(Vec::new(), qself, path, Paren::default(), elems)
    }

    fn from_path_elems(path: Path, elems: Punctuated<Pat, Comma>) -> PatTupleStruct {
        Self::from_qself_path_elems(None, path, elems)
    }
}
