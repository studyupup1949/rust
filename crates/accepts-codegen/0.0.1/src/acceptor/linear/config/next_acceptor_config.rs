use quote::format_ident;
use syn::{
    Ident, Token,
    parse::{Parse, ParseStream},
    token::Brace,
};

use crate::common::syn::parsers::KeyValueMap;

#[derive(Debug, Clone)]
pub struct NextAcceptorConfig {
    pub next_accepts_generics_ident: Ident,
    pub next_acceptor_field_ident: Ident,
}
impl NextAcceptorConfig {
    pub fn from_parts(
        next_accepts_generics_ident: Ident,
        next_acceptor_field_ident: Ident,
    ) -> Self {
        Self {
            next_accepts_generics_ident,
            next_acceptor_field_ident,
        }
    }
}

impl Parse for NextAcceptorConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut option_config = if input.peek(Brace) {
            Some(input.parse::<KeyValueMap<Token![=]>>()?)
        } else {
            None
        };

        let next_accepts_generics_ident = if let Some(ident) =
            option_config.as_mut().map_or(Ok(None), |option_config| {
                option_config.try_take_parse("next_accepts_generics_ident")
            })? {
            ident
        } else {
            format_ident!("NextAccepts")
        };

        let next_accepts_field_ident = if let Some(ident) =
            option_config.as_mut().map_or(Ok(None), |option_config| {
                option_config.try_take_parse("next_acceptor_field_ident")
            })? {
            ident
        } else {
            format_ident!("next_acceptor")
        };

        if let Some(option_config) = option_config.as_mut() {
            option_config.error_if_not_empty("unknown option")?;
        }

        Ok(Self::from_parts(
            next_accepts_generics_ident,
            next_accepts_field_ident,
        ))
    }
}
impl Default for NextAcceptorConfig {
    fn default() -> Self {
        Self::from_parts(format_ident!("NextAccepts"), format_ident!("next_acceptor"))
    }
}
