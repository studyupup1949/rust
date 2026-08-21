use syn::{
    Type, TypeTuple,
    punctuated::Punctuated,
    token::{Comma, Paren},
};

pub trait TypeTupleConstructExt {
    fn from_parts(paren_token: Paren, elems: Punctuated<Type, Comma>) -> TypeTuple;

    fn from_elems(elems: Punctuated<Type, Comma>) -> TypeTuple;
}

impl TypeTupleConstructExt for TypeTuple {
    fn from_parts(paren_token: Paren, elems: Punctuated<Type, Comma>) -> TypeTuple {
        TypeTuple { paren_token, elems }
    }

    fn from_elems(elems: Punctuated<Type, Comma>) -> TypeTuple {
        Self::from_parts(Paren::default(), elems)
    }
}
