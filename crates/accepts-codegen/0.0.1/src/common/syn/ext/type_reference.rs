use syn::{
    Lifetime, Token, Type, TypeReference,
    token::{And, Mut},
};

pub trait TypeReferenceConstructExt {
    fn from_path(
        and_token: Token![&],
        lifetime: Option<Lifetime>,
        mutability: Option<Token![mut]>,
        elem: Box<Type>,
    ) -> TypeReference;

    fn from_lifetime_mutability_elem(
        lifetime: Option<Lifetime>,
        mutability: Option<Token![mut]>,
        elem: Box<Type>,
    ) -> TypeReference;

    fn from_mutability_elem(mutability: Option<Token![mut]>, elem: Box<Type>) -> TypeReference;

    fn from_elem(elem: Box<Type>) -> TypeReference;

    fn mut_from_elem(elem: Box<Type>) -> TypeReference;
}

impl TypeReferenceConstructExt for TypeReference {
    fn from_path(
        and_token: Token![&],
        lifetime: Option<Lifetime>,
        mutability: Option<Token![mut]>,
        elem: Box<Type>,
    ) -> TypeReference {
        TypeReference {
            and_token,
            lifetime,
            mutability,
            elem,
        }
    }

    fn from_lifetime_mutability_elem(
        lifetime: Option<Lifetime>,
        mutability: Option<Token![mut]>,
        elem: Box<Type>,
    ) -> TypeReference {
        Self::from_path(And::default(), lifetime, mutability, elem)
    }

    fn from_mutability_elem(mutability: Option<Token![mut]>, elem: Box<Type>) -> TypeReference {
        Self::from_lifetime_mutability_elem(None, mutability, elem)
    }

    fn from_elem(elem: Box<Type>) -> TypeReference {
        Self::from_mutability_elem(None, elem)
    }

    fn mut_from_elem(elem: Box<Type>) -> TypeReference {
        Self::from_mutability_elem(Some(Mut::default()), elem)
    }
}
