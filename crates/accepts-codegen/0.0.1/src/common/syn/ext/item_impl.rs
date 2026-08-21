use syn::{Attribute, Generics, ImplItem, ItemImpl, Path, Token, Type, token};

pub trait ItemImplConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        defaultness: Option<Token![default]>,
        unsafety: Option<Token![unsafe]>,
        impl_token: Token![impl],
        generics: Generics,
        trait_: Option<(Option<Token![!]>, Path, Token![for])>,
        self_ty: Box<Type>,
        brace_token: token::Brace,
        items: Vec<ImplItem>,
    ) -> ItemImpl;
}

impl ItemImplConstructExt for ItemImpl {
    fn from_parts(
        attrs: Vec<Attribute>,
        defaultness: Option<Token![default]>,
        unsafety: Option<Token![unsafe]>,
        impl_token: Token![impl],
        generics: Generics,
        trait_: Option<(Option<Token![!]>, Path, Token![for])>,
        self_ty: Box<Type>,
        brace_token: token::Brace,
        items: Vec<ImplItem>,
    ) -> ItemImpl {
        ItemImpl {
            attrs,
            defaultness,
            unsafety,
            impl_token,
            generics,
            trait_,
            self_ty,
            brace_token,
            items,
        }
    }
}
