use syn::{
    Attribute, Pat, PatTuple,
    punctuated::Punctuated,
    token::{Comma, Paren},
};

pub trait PatTupleConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        paren_token: Paren,
        elems: Punctuated<Pat, Comma>,
    ) -> PatTuple;

    fn from_attrs_elems(attrs: Vec<Attribute>, elems: Punctuated<Pat, Comma>) -> PatTuple;

    fn from_elems(elems: Punctuated<Pat, Comma>) -> PatTuple;
}

impl PatTupleConstructExt for PatTuple {
    fn from_parts(
        attrs: Vec<Attribute>,
        paren_token: Paren,
        elems: Punctuated<Pat, Comma>,
    ) -> PatTuple {
        PatTuple {
            attrs,
            paren_token,
            elems,
        }
    }

    fn from_attrs_elems(attrs: Vec<Attribute>, elems: Punctuated<Pat, Comma>) -> PatTuple {
        <Self as PatTupleConstructExt>::from_parts(attrs, Paren::default(), elems)
    }

    fn from_elems(elems: Punctuated<Pat, Comma>) -> PatTuple {
        <Self as PatTupleConstructExt>::from_attrs_elems(Vec::new(), elems)
    }
}
