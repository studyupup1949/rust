use quote::format_ident;
use syn::{Ident, Path, Token, Type, TypePath, fold::Fold, parse::Parse};

use crate::common::syn::{ast::helpers::fold::TypeIdentReplacer, ext::TypePathConstructExt};

#[derive(Debug, Clone)]
pub struct MutableSourceType {
    pub wrapper: Option<Type>,
    pub resource_type: Type,
    pub state_type: Type,
}

impl MutableSourceType {
    pub fn from_parts(wrapper: Option<Type>, resource_type: Type, state_type: Type) -> Self {
        Self {
            wrapper,
            resource_type,
            state_type,
        }
    }

    pub fn into_state_guarded_source(self, accepts_t_ident: Ident) -> (Type, Type, Type) {
        let inner_ident = format_ident!("__Inner");
        let item_ident = format_ident!("__Item");
        let state = TypeIdentReplacer::from_needle_replacement(
            &item_ident,
            &Type::Path(TypePath::from_path(Path::from(accepts_t_ident))),
        )
        .fold_type(self.state_type);

        let guarded = TypeIdentReplacer::from_needle_replacement(&inner_ident, &state)
            .fold_type(self.resource_type);

        let source = match self.wrapper {
            None => guarded.clone(),
            Some(wrapper) => TypeIdentReplacer::from_needle_replacement(&inner_ident, &guarded)
                .fold_type(wrapper),
        };
        (state, guarded, source)
    }
}

impl Parse for MutableSourceType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let first = input.parse::<Type>()?;

        input.parse::<Token![=>]>()?;

        let second = input.parse::<Type>()?;

        if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;

            let third = input.parse::<Type>()?;

            Ok(Self::from_parts(Some(first), second, third))
        } else {
            Ok(Self::from_parts(None, first, second))
        }
    }
}
