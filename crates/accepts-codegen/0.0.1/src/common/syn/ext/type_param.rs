use syn::{Attribute, Ident, Token, Type, TypeParam, TypeParamBound, punctuated::Punctuated};

pub trait TypeParamConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        ident: Ident,
        colon_token: Option<Token![:]>,
        bounds: Punctuated<TypeParamBound, Token![+]>,
        eq_token: Option<Token![=]>,
        default: Option<Type>,
    ) -> TypeParam;

    fn from_ident(ident: Ident) -> TypeParam;
}

impl TypeParamConstructExt for TypeParam {
    fn from_parts(
        attrs: Vec<Attribute>,
        ident: Ident,
        colon_token: Option<Token![:]>,
        bounds: Punctuated<TypeParamBound, Token![+]>,
        eq_token: Option<Token![=]>,
        default: Option<Type>,
    ) -> TypeParam {
        TypeParam {
            attrs,
            ident,
            colon_token,
            bounds,
            eq_token,
            default,
        }
    }

    fn from_ident(ident: Ident) -> TypeParam {
        Self::from_parts(
            Vec::new(),
            ident,
            None,
            syn::punctuated::Punctuated::new(),
            None,
            None,
        )
    }
}
