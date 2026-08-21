use syn::{
    TypeImplTrait, TypeParamBound,
    punctuated::Punctuated,
    token::{Impl, Plus},
};

pub trait TypeImplTraitConstructExt {
    fn from_parts(impl_token: Impl, bounds: Punctuated<TypeParamBound, Plus>) -> TypeImplTrait;

    fn from_bounds(bounds: Punctuated<TypeParamBound, Plus>) -> TypeImplTrait;
}

impl TypeImplTraitConstructExt for TypeImplTrait {
    fn from_parts(impl_token: Impl, bounds: Punctuated<TypeParamBound, Plus>) -> TypeImplTrait {
        TypeImplTrait { impl_token, bounds }
    }

    fn from_bounds(bounds: Punctuated<TypeParamBound, Plus>) -> TypeImplTrait {
        Self::from_parts(Impl::default(), bounds)
    }
}
