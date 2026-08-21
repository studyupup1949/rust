use proc_macro2::Span;
use syn::{Field, Ident, Type, Visibility, token::Colon};

pub trait FieldConstructExt {
    fn from_ident(ident: Ident, ty: Type) -> Field;
}

impl FieldConstructExt for Field {
    fn from_ident(ident: Ident, ty: Type) -> Field {
        Field {
            attrs: Vec::new(),
            vis: Visibility::Inherited,
            mutability: syn::FieldMutability::None,
            ident: Some(ident),
            colon_token: Some(Colon {
                spans: [Span::call_site()],
            }),
            ty,
        }
    }
}
