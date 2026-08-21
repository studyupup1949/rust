use syn::{
    TypeParamBound, TypeTraitObject,
    punctuated::Punctuated,
    token::{self, Dyn, Plus},
};

pub trait TypeTraitObjectConstructExt {
    fn from_parts(
        dyn_token: Option<Dyn>,
        bounds: Punctuated<TypeParamBound, Plus>,
    ) -> TypeTraitObject;

    fn from_bounds(bounds: Punctuated<TypeParamBound, Plus>) -> TypeTraitObject;
}

impl TypeTraitObjectConstructExt for TypeTraitObject {
    fn from_parts(
        dyn_token: Option<Dyn>,
        bounds: Punctuated<TypeParamBound, Plus>,
    ) -> TypeTraitObject {
        Self { dyn_token, bounds }
    }

    fn from_bounds(bounds: Punctuated<TypeParamBound, Plus>) -> TypeTraitObject {
        Self::from_parts(Some(token::Dyn::default()), bounds)
    }
}
