use syn::{
    Attribute, Ident, Pat, PatIdent,
    token::{At, Mut, Ref},
};

pub trait PatIdentConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        by_ref: Option<Ref>,
        mutability: Option<Mut>,
        ident: Ident,
        subpat: Option<(At, Box<Pat>)>,
    ) -> PatIdent;

    fn from_ref_mut_ident(by_ref: Option<Ref>, mutability: Option<Mut>, ident: Ident) -> PatIdent;

    fn from_ident(ident: Ident) -> PatIdent;

    fn mut_from_ident(ident: Ident) -> PatIdent;

    fn ref_from_ident(ident: Ident) -> PatIdent;

    fn mut_ref_from_ident(ident: Ident) -> PatIdent;
}

impl PatIdentConstructExt for PatIdent {
    fn from_parts(
        attrs: Vec<Attribute>,
        by_ref: Option<Ref>,
        mutability: Option<Mut>,
        ident: Ident,
        subpat: Option<(At, Box<Pat>)>,
    ) -> PatIdent {
        PatIdent {
            attrs,
            by_ref,
            mutability,
            ident,
            subpat,
        }
    }

    fn from_ident(ident: Ident) -> PatIdent {
        Self::from_parts(Vec::new(), None, None, ident, None)
    }

    fn from_ref_mut_ident(by_ref: Option<Ref>, mutability: Option<Mut>, ident: Ident) -> PatIdent {
        Self::from_parts(Vec::new(), by_ref, mutability, ident, None)
    }

    fn mut_from_ident(ident: Ident) -> PatIdent {
        Self::from_ref_mut_ident(None, Some(Mut::default()), ident)
    }

    fn ref_from_ident(ident: Ident) -> PatIdent {
        Self::from_ref_mut_ident(Some(Ref::default()), None, ident)
    }

    fn mut_ref_from_ident(ident: Ident) -> PatIdent {
        Self::from_ref_mut_ident(Some(Ref::default()), Some(Mut::default()), ident)
    }
}
