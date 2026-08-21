use quote::format_ident;
use syn::{Path, Token, parse::Parse, token::Brace};

use crate::common::syn::parsers::KeyValueMap;

use super::{ForwardSource, MutGuardErrorConfig};

#[derive(Debug, Clone)]
pub struct MutGuardConfig {
    pub guard_access_path: Path,
    pub guard_access_error: Option<MutGuardErrorConfig>,
}
impl MutGuardConfig {
    pub fn from_parts(
        guard_access_path: Path,
        guard_access_error: Option<MutGuardErrorConfig>,
    ) -> Self {
        Self {
            guard_access_path,
            guard_access_error,
        }
    }
}
impl Parse for MutGuardConfig {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let guard_access_path = input.call(Path::parse_mod_style)?;

        let guard_access_error = if input.peek(Token![<]) {
            input.parse::<Token![<]>()?;

            let guard_access_error = input.parse()?;

            input.parse::<Token![>]>()?;

            let mut option_config = if input.peek(Brace) {
                Some(input.parse::<KeyValueMap<Token![=]>>()?)
            } else {
                None
            };

            let accepts_generics_ident = if let Some(ident) =
                option_config.as_mut().map_or(Ok(None), |option_config| {
                    option_config.try_take_parse("error_accepts_generics_ident")
                })? {
                ident
            } else {
                format_ident!("AccessErrorAccepts")
            };

            let accepts_field_ident = if let Some(ident) =
                option_config.as_mut().map_or(Ok(None), |option_config| {
                    option_config.try_take_parse("error_accepts_field_ident")
                })? {
                ident
            } else {
                format_ident!("access_error_acceptor")
            };

            let forward_source = if let Some(ident) =
                option_config.as_mut().map_or(Ok(None), |option_config| {
                    option_config.try_take_parse("forward_source")
                })? {
                ident
            } else {
                ForwardSource::None
            };

            if let Some(option_config) = option_config.as_mut() {
                option_config.error_if_not_empty("unknown option")?;
            }

            let guard_access_error = MutGuardErrorConfig::from_parts(
                guard_access_error,
                accepts_generics_ident,
                accepts_field_ident,
                forward_source,
            );

            Some(guard_access_error)
        } else {
            None
        };

        Ok(Self::from_parts(guard_access_path, guard_access_error))
    }
}
