use syn::{Attribute, Fields, Generics, Ident, ItemStruct, Token, Visibility};

pub trait ItemStructConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        vis: Visibility,
        struct_token: Token![struct],
        ident: Ident,
        generics: Generics,
        fields: Fields,
        semi_token: Option<Token![;]>,
    ) -> ItemStruct;
}

impl ItemStructConstructExt for ItemStruct {
    fn from_parts(
        attrs: Vec<Attribute>,
        vis: Visibility,
        struct_token: Token![struct],
        ident: Ident,
        generics: Generics,
        fields: Fields,
        semi_token: Option<Token![;]>,
    ) -> ItemStruct {
        ItemStruct {
            attrs,
            vis,
            struct_token,
            ident,
            generics,
            fields,
            semi_token,
        }
    }
}
