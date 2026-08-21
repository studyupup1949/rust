use syn::{AngleBracketedGenericArguments, AssocType, Ident, Type, token::Eq};

pub trait AssocTypeConstructExt {
    fn from_parts(
        ident: Ident,
        generics: Option<AngleBracketedGenericArguments>,
        eq_token: Eq,
        ty: Type,
    ) -> AssocType;

    fn from_ident_generics_ty(
        ident: Ident,
        generics: Option<AngleBracketedGenericArguments>,
        ty: Type,
    ) -> AssocType;

    fn from_ident_ty(ident: Ident, ty: Type) -> AssocType;
}

impl AssocTypeConstructExt for AssocType {
    fn from_parts(
        ident: Ident,
        generics: Option<AngleBracketedGenericArguments>,
        eq_token: Eq,
        ty: Type,
    ) -> AssocType {
        AssocType {
            ident,
            generics,
            eq_token,
            ty,
        }
    }

    fn from_ident_generics_ty(
        ident: Ident,
        generics: Option<AngleBracketedGenericArguments>,
        ty: Type,
    ) -> AssocType {
        Self::from_parts(ident, generics, Eq::default(), ty)
    }

    fn from_ident_ty(ident: Ident, ty: Type) -> AssocType {
        Self::from_ident_generics_ty(ident, None, ty)
    }
}
