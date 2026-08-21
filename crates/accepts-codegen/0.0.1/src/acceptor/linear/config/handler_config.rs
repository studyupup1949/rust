use quote::format_ident;
use syn::{Ident, Path, Token, parenthesized, parse::Parse, token::Brace};

use crate::common::syn::parsers::KeyValueMap;

use super::{HandlerErrorConfig, MutGuardConfig, MutableSourceType, ReferenceSourceType};

#[derive(Debug, Clone)]
pub struct HandlerConfig {
    pub source_field_ident: Ident,
    pub handler: HandlerConfig2,
    pub handler_path: Path,
    pub handler_error: Option<HandlerErrorConfig>,
}

impl HandlerConfig {
    pub fn from_parts(
        source_ident: Ident,
        handler: HandlerConfig2,
        handler_path: Path,
        handler_error: Option<HandlerErrorConfig>,
    ) -> Self {
        Self {
            source_field_ident: source_ident,
            handler,
            handler_path,
            handler_error,
        }
    }

    fn parse_ref(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let paren_content;
        parenthesized!(paren_content in input);

        let handler_path = paren_content.call(Path::parse_mod_style)?;

        paren_content.parse::<Token![<]>()?;

        let source_type = paren_content.parse()?;

        let handler_error_ty = if paren_content.peek(Token![,]) {
            paren_content.parse::<Token![,]>()?;
            Some(paren_content.parse()?)
        } else {
            None
        };

        paren_content.parse::<Token![>]>()?;

        if !paren_content.is_empty() {
            return Err(syn::Error::new(
                paren_content.span(),
                "unexpected tokens in `Ref` handler",
            ));
        }

        let mut paren_option_config = if paren_content.peek(Brace) {
            Some(paren_content.parse::<KeyValueMap>()?)
        } else {
            None
        };

        let handler_error = if let Some(handler_error_ty) = handler_error_ty {
            let accepts_generics_ident = if let Some(ident) = paren_option_config
                .as_mut()
                .map_or(Ok(None), |option_config| {
                    option_config.try_take_parse("error_accepts_generics_ident")
                })? {
                ident
            } else {
                format_ident!("ErrorAccepts")
            };

            let accepts_field_ident = if let Some(ident) = paren_option_config
                .as_mut()
                .map_or(Ok(None), |option_config| {
                    option_config.try_take_parse("error_accepts_field_ident")
                })? {
                ident
            } else {
                format_ident!("error_acceptor")
            };

            Some(HandlerErrorConfig::from_parts(
                handler_error_ty,
                accepts_generics_ident,
                accepts_field_ident,
            ))
        } else {
            None
        };

        if let Some(paren_option_config) = &paren_option_config {
            paren_option_config.error_if_not_empty("unknown option")?;
        }

        let mut option_config = if input.peek(Brace) {
            Some(input.parse::<KeyValueMap>()?)
        } else {
            None
        };

        let source_ident = if let Some(ident) =
            option_config.as_mut().map_or(Ok(None), |option_config| {
                option_config.try_take_parse("source_ident")
            })? {
            ident
        } else {
            format_ident!("source")
        };

        Ok(Self::from_parts(
            source_ident,
            HandlerConfig2::ret_from_parts(source_type),
            handler_path,
            handler_error,
        ))
    }

    fn parse_mut(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let paren_content;
        parenthesized!(paren_content in input);

        let handler_path = paren_content.call(Path::parse_mod_style)?;

        paren_content.parse::<Token![<]>()?;

        let source_type = paren_content.parse()?;

        let handler_error_ty = if paren_content.peek(Token![,]) {
            paren_content.parse::<Token![,]>()?;
            Some(paren_content.parse()?)
        } else {
            None
        };

        paren_content.parse::<Token![>]>()?;
        paren_content.parse::<Token![,]>()?;

        let guard_access = paren_content.parse()?;

        if !paren_content.is_empty() {
            return Err(syn::Error::new(
                paren_content.span(),
                "unexpected tokens in `Mut` handler",
            ));
        }

        let mut paren_option_config = if paren_content.peek(Brace) {
            Some(paren_content.parse::<KeyValueMap>()?)
        } else {
            None
        };

        let handler_error = if let Some(handler_error_ty) = handler_error_ty {
            let accepts_generics_ident = if let Some(option_config) = &mut paren_option_config {
                option_config.take_parse("error_accepts_generics_ident")?
            } else {
                format_ident!("ErrorAccepts")
            };

            let accepts_field_ident = if let Some(ident) = paren_option_config
                .as_mut()
                .map_or(Ok(None), |option_config| {
                    option_config.try_take_parse("error_accepts_field_ident")
                })? {
                ident
            } else {
                format_ident!("error_acceptor")
            };
            Some(HandlerErrorConfig::from_parts(
                handler_error_ty,
                accepts_generics_ident,
                accepts_field_ident,
            ))
        } else {
            None
        };

        if let Some(paren_option_config) = &paren_option_config {
            paren_option_config.error_if_not_empty("unknown option")?;
        }

        let mut option_config = if input.peek(Brace) {
            Some(input.parse::<KeyValueMap>()?)
        } else {
            None
        };

        let source_ident = if let Some(ident) =
            option_config.as_mut().map_or(Ok(None), |option_config| {
                option_config.try_take_parse("source_ident")
            })? {
            ident
        } else {
            format_ident!("source")
        };

        Ok(Self::from_parts(
            source_ident,
            HandlerConfig2::mut_from_parts(source_type, guard_access),
            handler_path,
            handler_error,
        ))
    }
}

//Mut(CustomHandler<Arc<__Inner> => Mutex<__Inner> => Vec<__Item>, SendError<T>>, GuardAccess), //
impl Parse for HandlerConfig {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;

        match ident.to_string().as_str() {
            "Ref" => Self::parse_ref(input),
            "Mut" => Self::parse_mut(input),

            _ => return Err(syn::Error::new(ident.span(), "unknown option")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum HandlerConfig2 {
    Ref {
        source_type: ReferenceSourceType,
    },
    Mut {
        source_type: MutableSourceType,
        mut_guard: MutGuardConfig,
    },
}
#[allow(dead_code)]
impl HandlerConfig2 {
    pub fn ret_from_parts(source_type: ReferenceSourceType) -> Self {
        Self::Ref { source_type }
    }

    pub fn mut_from_parts(source_type: MutableSourceType, mut_guard: MutGuardConfig) -> Self {
        Self::Mut {
            source_type,
            mut_guard,
        }
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, Self::Ref { .. })
    }
    pub fn is_mut(&self) -> bool {
        matches!(self, Self::Mut { .. })
    }
}
