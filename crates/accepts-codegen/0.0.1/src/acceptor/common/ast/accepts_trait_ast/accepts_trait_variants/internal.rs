use syn::{
    FnArg, Ident, Pat, PatIdent, PatType, Path, TraitBound, Type, TypeParamBound, TypePath,
    TypeReference, TypeTuple, punctuated::Punctuated,
};

use crate::common::{
    function::generate::future_path,
    syn::ext::{
        IdentConstructExt, PatIdentConstructExt, PatTypeConstructExt, TraitBoundConstructExt,
        TypePathConstructExt, TypeReferenceConstructExt, TypeTupleConstructExt,
    },
};

//builder
pub fn accept_receiver_ty() -> Type {
    Type::Reference(TypeReference::from_elem(Box::new(Type::Path(
        TypePath::from_path(Path::from(Ident::from_str("Self"))),
    ))))
}

pub fn accept_value_fn_arg(accepts_t_type: Type) -> FnArg {
    FnArg::Typed(PatType::from_pat_ty(
        Box::new(Pat::Ident(PatIdent::from_ident(Ident::from_str("value")))),
        Box::new(accepts_t_type),
    ))
}

pub fn future_type_param_bound() -> TypeParamBound {
    TypeParamBound::Trait(TraitBound::from_path(future_path(Type::Tuple(
        TypeTuple::from_elems(Punctuated::new()),
    ))))
}
