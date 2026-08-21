use quote::format_ident;
use syn::{Ident, Path, Token, Type, TypePath, fold::Fold, parse::Parse};

use crate::common::syn::{ast::helpers::fold::TypeIdentReplacer, ext::TypePathConstructExt};

#[derive(Debug, Clone)]
pub struct ReferenceSourceType {
    pub wrapper: Option<Type>,
    pub state_type: Type,
}

impl ReferenceSourceType {
    pub fn from_parts(wrapper: Option<Type>, state_type: Type) -> Self {
        Self {
            wrapper,
            state_type,
        }
    }

    pub fn into_state_source(self, accepts_t_ident: Ident) -> (Type, Type) {
        let inner_ident = format_ident!("__Inner");
        let item_ident = format_ident!("__Item");

        let state = TypeIdentReplacer::from_needle_replacement(
            &item_ident,
            &Type::Path(TypePath::from_path(Path::from(accepts_t_ident))),
        )
        .fold_type(self.state_type);

        let source = match self.wrapper {
            None => state.clone(),
            Some(wrapper) => {
                TypeIdentReplacer::from_needle_replacement(&inner_ident, &state).fold_type(wrapper)
            }
        };

        (state, source)
    }
}

impl Parse for ReferenceSourceType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let first = input.parse::<Type>()?;

        Ok(match input.parse::<Token![=>]>() {
            Ok(_) => {
                let second = input.parse::<Type>()?;

                Self::from_parts(Some(first), second)
            }
            Err(_) => Self::from_parts(None, first),
        })
    }
}
