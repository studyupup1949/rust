use syn::{Attribute, Pat, PatType, Token, Type, token::Colon};

pub trait PatTypeConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        pat: Box<Pat>,
        colon_token: Token![:],
        ty: Box<Type>,
    ) -> PatType;

    fn from_pat_ty(pat: Box<Pat>, ty: Box<Type>) -> PatType;
}

impl PatTypeConstructExt for PatType {
    fn from_parts(
        attrs: Vec<Attribute>,
        pat: Box<Pat>,
        colon_token: Token![:],
        ty: Box<Type>,
    ) -> PatType {
        PatType {
            attrs,
            pat,
            colon_token,
            ty,
        }
    }

    fn from_pat_ty(pat: Box<Pat>, ty: Box<Type>) -> PatType {
        Self::from_parts(Vec::new(), pat, Colon::default(), ty)
    }
}
