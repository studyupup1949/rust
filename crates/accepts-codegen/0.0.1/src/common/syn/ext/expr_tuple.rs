use syn::{
    Attribute, Expr, ExprTuple,
    punctuated::Punctuated,
    token::{Comma, Paren},
};

pub trait ExprTupleConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        paren_token: Paren,
        elems: Punctuated<Expr, Comma>,
    ) -> ExprTuple;

    fn from_attrs_elems(attrs: Vec<Attribute>, elems: Punctuated<Expr, Comma>) -> ExprTuple;

    fn from_elems(elems: Punctuated<Expr, Comma>) -> ExprTuple;
}

impl ExprTupleConstructExt for ExprTuple {
    fn from_parts(
        attrs: Vec<Attribute>,
        paren_token: Paren,
        elems: Punctuated<Expr, Comma>,
    ) -> ExprTuple {
        ExprTuple {
            attrs,
            paren_token,
            elems,
        }
    }

    fn from_attrs_elems(attrs: Vec<Attribute>, elems: Punctuated<Expr, Comma>) -> ExprTuple {
        <Self as ExprTupleConstructExt>::from_parts(attrs, Paren::default(), elems)
    }

    fn from_elems(elems: Punctuated<Expr, Comma>) -> ExprTuple {
        <Self as ExprTupleConstructExt>::from_attrs_elems(Vec::new(), elems)
    }
}
