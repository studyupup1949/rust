use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::__private::Span;
use syn::ext::IdentExt;
use syn::parse::discouraged::Speculative;
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, LitInt, LitStr, Token, Type};

/// Endpoint response documentation
///
/// # Examples
///
/// ```no_compile
/// (status = 200, description = "Request successful")
/// ```
///
pub(crate) struct Response {
    pub(crate) status_code: Option<u16>,
    pub(crate) responses_type: Option<Type>,
    pub(crate) description: Option<LitStr>,
    pub(crate) response: Option<Ident>,
    pub(crate) content_type: Option<LitStr>,
    pub(crate) response_body: Option<ResponseBody>,
}

pub(crate) enum ResponseBody {
    Type(Type),
    Inline {
        inline_token: Ident,
        body_type: Type,
    },
    Ref {
        ref_token: Token![ref],
        reference: LitStr,
    },
}

impl Parse for ResponseBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![ref]) {
            let ref_token = input.parse::<Token![ref]>()?;

            let content;
            parenthesized!(content in input);

            let reference = content.parse()?;

            Ok(Self::Ref {
                ref_token,
                reference,
            })
        } else if lookahead.peek(Ident::peek_any) {
            let fork = input.fork();

            let ident = fork.parse::<Ident>()?;

            if ident == Ident::new("inline", Span::call_site()) {
                input.advance_to(&fork);

                let body_type = input.parse::<Type>()?;

                Ok(Self::Inline {
                    inline_token: ident,
                    body_type,
                })
            } else {
                let body_type = input.parse::<Type>()?;

                Ok(Self::Type(body_type))
            }
        } else {
            eprintln!("Failed to parse response body");
            Err(lookahead.error())
        }
    }
}

impl ToTokens for ResponseBody {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expanded = match self {
            Self::Type(ty) => quote! { body = #ty },
            Self::Inline {
                body_type,
                inline_token,
            } => quote! { body = #inline_token(#body_type) },
            Self::Ref {
                reference,
                ref_token,
            } => quote! { body = #ref_token(#reference) },
        };

        tokens.append_all(expanded);
    }
}

impl Parse for Response {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);

        let mut status_code = None;
        let mut responses_type = None;
        let mut description = None;
        let mut response = None;
        let mut content_type = None;
        let mut response_body = None;

        let content_fork = content.fork();

        // Parse the status field
        let status_ident: Ident = content_fork.parse()?;
        if status_ident != "status" {
            responses_type = Some(content.parse::<Type>()?);

            return Ok(Self {
                status_code,
                responses_type,
                description,
                response,
                content_type,
                response_body,
            });
        } else {
            content.advance_to(&content_fork);
        }

        content.parse::<Token![=]>()?;
        let status_code_lit: LitInt = content.parse()?;
        status_code = Some(status_code_lit.base10_parse()?);

        content.parse::<Token![,]>()?;

        // Parse either description or response, ensuring mutual exclusion
        while !content.is_empty() {
            let ident: Ident = content.parse()?;
            content.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "description" => {
                    if description.is_some() || response.is_some() {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "Cannot have both 'description' and 'response'",
                        ));
                    }
                    description = Some(content.parse()?);
                }
                "response" => {
                    if description.is_some() || response.is_some() {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "Cannot have both 'description' and 'response'",
                        ));
                    }
                    response = Some(content.parse()?);
                }
                "content_type" => {
                    content_type = Some(content.parse()?);
                }
                "body" => {
                    if response_body.is_some() {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "Response body is already specified",
                        ));
                    }

                    response_body = Some(content.parse()?)
                }
                _ => return Err(syn::Error::new_spanned(ident, "Unexpected field")),
            }

            // Optionally consume a comma if present
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        // Ensure at least one of `description` or `response` is present
        if description.is_none() && response.is_none() {
            return Err(syn::Error::new_spanned(
                status_ident,
                "Either 'description' or 'response' must be present",
            ));
        }

        Ok(Self {
            status_code,
            responses_type,
            description,
            response,
            content_type,
            response_body,
        })
    }
}
