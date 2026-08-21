use syn::{
    GenericParam, Generics, WhereClause,
    punctuated::Punctuated,
    token::{Comma, Gt, Lt},
};

pub trait GenericsConstructExt {
    fn from_parts(
        lt_token: Option<Lt>,
        params: Punctuated<GenericParam, Comma>,
        gt_token: Option<Gt>,
        where_clause: Option<WhereClause>,
    ) -> Generics;
}

impl GenericsConstructExt for Generics {
    fn from_parts(
        lt_token: Option<Lt>,
        params: Punctuated<GenericParam, Comma>,
        gt_token: Option<Gt>,
        where_clause: Option<WhereClause>,
    ) -> Generics {
        Generics {
            lt_token,
            params,
            gt_token,
            where_clause,
        }
    }
}
