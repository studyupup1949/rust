use syn::{BoundLifetimes, PredicateType, Token, Type, TypeParamBound, punctuated::Punctuated};

use crate::common::syn::ext::PredicateTypeConstructExt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptsTPredicateType {
    pub lifetimes: Option<BoundLifetimes>,
    pub colon_token: Token![:],
    pub bounds: Punctuated<TypeParamBound, Token![+]>,
}

impl AcceptsTPredicateType {
    pub fn into_predicate_type(self, accepts_t_type: Type) -> PredicateType {
        PredicateType::from_parts(
            self.lifetimes,
            accepts_t_type,
            self.colon_token,
            self.bounds,
        )
    }
}
