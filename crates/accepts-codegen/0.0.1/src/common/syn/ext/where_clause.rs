use syn::{
    WhereClause, WherePredicate,
    punctuated::Punctuated,
    token::{Comma, Where},
};

pub trait WhereClauseConstructExt {
    fn from_predicates(predicates: Punctuated<WherePredicate, Comma>) -> WhereClause;
}

impl WhereClauseConstructExt for WhereClause {
    fn from_predicates(predicates: Punctuated<WherePredicate, Comma>) -> WhereClause {
        WhereClause {
            where_token: Where::default(),
            predicates,
        }
    }
}
