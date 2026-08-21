use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{discouraged::AnyDelimiter, Parse},
    token, Ident,
};

#[derive(Debug)]
pub struct RestErrorMacroArguments {
    pub internal_error: Option<RestErrorVariantArguments>,
}

impl Parse for RestErrorMacroArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut internal_error = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "internal_error" => {
                    if input.peek(token::Paren) {
                        let (_, _, inner_input) = input.parse_any_delimiter()?;
                        internal_error = Some(inner_input.parse::<RestErrorVariantArguments>()?)
                    } else {
                        internal_error = Some(RestErrorVariantArguments {
                            status_code: quote!(
                                ::actix_web_rest::http::StatusCode::INTERNAL_SERVER_ERROR
                            ),
                        })
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        "unsupported rest_error attribute",
                    ))
                }
            }
        }

        Ok(Self { internal_error })
    }
}

#[derive(Debug)]
pub struct RestErrorVariantArguments {
    pub status_code: TokenStream,
}

impl Parse for RestErrorVariantArguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut status_code: Option<TokenStream> = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "status_code" => {
                    let _: token::Eq = input.parse()?;
                    status_code = Some(input.parse::<TokenStream>()?)
                }
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        "unsupported rest_error variant attribute",
                    ))
                }
            }
        }

        Ok(Self {
            status_code: status_code.ok_or_else(|| {
                syn::Error::new(
                    input.span(),
                    "Missing status_code attribute on rest_error variant",
                )
            })?,
        })
    }
}
