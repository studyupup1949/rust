use syn::{Attribute, Lifetime, Receiver, Token, Type, token::SelfValue};

pub trait ReceiverConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        reference: Option<(Token![&], Option<Lifetime>)>,
        mutability: Option<Token![mut]>,
        self_token: Token![self],
        colon_token: Option<Token![:]>,
        ty: Box<Type>,
    ) -> Receiver;

    fn from_ref_mut_colon_ty(
        reference: Option<(Token![&], Option<Lifetime>)>,
        mutability: Option<Token![mut]>,
        colon_token: Option<Token![:]>,
        ty: Box<Type>,
    ) -> Receiver;
}

impl ReceiverConstructExt for Receiver {
    fn from_parts(
        attrs: Vec<Attribute>,
        reference: Option<(Token![&], Option<Lifetime>)>,
        mutability: Option<Token![mut]>,
        self_token: Token![self],
        colon_token: Option<Token![:]>,
        ty: Box<Type>,
    ) -> Receiver {
        Receiver {
            attrs,
            reference,
            mutability,
            self_token,
            colon_token,
            ty,
        }
    }

    fn from_ref_mut_colon_ty(
        reference: Option<(Token![&], Option<Lifetime>)>,
        mutability: Option<Token![mut]>,
        colon_token: Option<Token![:]>,
        ty: Box<Type>,
    ) -> Receiver {
        Self::from_parts(
            Vec::new(),
            reference,
            mutability,
            SelfValue::default(),
            colon_token,
            ty,
        )
    }
}
