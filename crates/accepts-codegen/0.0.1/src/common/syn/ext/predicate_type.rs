use syn::{
    BoundLifetimes, PredicateType, Type, TypeParamBound,
    punctuated::Punctuated,
    token::{Colon, Plus},
};

pub trait PredicateTypeConstructExt {
    fn from_parts(
        lifetimes: Option<BoundLifetimes>,
        bounded_ty: Type,
        colon_token: Colon,
        bounds: Punctuated<TypeParamBound, Plus>,
    ) -> PredicateType;

    fn from_bounds(bounded_ty: Type, bounds: Punctuated<TypeParamBound, Plus>) -> PredicateType;
}

impl PredicateTypeConstructExt for PredicateType {
    fn from_parts(
        lifetimes: Option<BoundLifetimes>,
        bounded_ty: Type,
        colon_token: Colon,
        bounds: Punctuated<TypeParamBound, Plus>,
    ) -> PredicateType {
        PredicateType {
            lifetimes,
            bounded_ty,
            colon_token,
            bounds,
        }
    }

    fn from_bounds(bounded_ty: Type, bounds: Punctuated<TypeParamBound, Plus>) -> PredicateType {
        Self::from_parts(None, bounded_ty, Colon::default(), bounds)
    }
}
