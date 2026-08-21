use crate::generate_endpoint::documentation::request_body::RequestBody;
use proc_macro2::Ident;
use response::Response;
use security::SecurityRequirement;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, bracketed, parenthesized, LitStr, Token};

pub(crate) mod expand;
mod request_body;
pub(crate) mod response;
pub(crate) mod security;

/// Endpoint documentation
///
/// # Examples
///
/// ```no_compile
/// context_path = "/api"
/// tag = "tag"
/// responses: {
///     (status = 200, description = "Request successful"),
///     (status = 404, description = "Not found")
/// }
/// ```
///
#[non_exhaustive]
pub(crate) struct Documentation {
    pub(crate) method: Option<Ident>,
    pub(crate) path: Option<LitStr>,
    pub(crate) context_path: Option<LitStr>,
    pub(crate) tag: Option<LitStr>,
    pub(crate) responses: Option<Vec<Response>>,
    pub(crate) security: Option<Vec<SecurityRequirement>>,
    pub(crate) params: Option<Vec<Ident>>,
    pub(crate) request_body: Option<RequestBody>,
}

impl Parse for Documentation {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut context_path: Option<LitStr> = None;
        let mut tag: Option<LitStr> = None;
        let mut responses: Option<Vec<Response>> = None;
        let mut params: Option<Vec<Ident>> = None;
        let mut security: Option<Vec<SecurityRequirement>> = None;
        let mut request_body = None;

        // Parse in a loop, allowing fields in any order
        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![:]>()?; // Expect a colon after each identifier

            match ident.to_string().as_str() {
                "context_path" => {
                    if context_path.is_some() {
                        return Err(input.error("Duplicate context_path"));
                    }
                    context_path = Some(input.parse()?);
                }
                "tag" => {
                    if tag.is_some() {
                        return Err(input.error("Duplicate tag"));
                    }
                    tag = Some(input.parse()?);
                }
                "responses" => {
                    if responses.is_some() {
                        return Err(input.error("Duplicate responses"));
                    }
                    let content;
                    braced!(content in input); // Expect a block around the responses
                    let parsed_responses =
                        Punctuated::<Response, Token![,]>::parse_terminated(&content)?;
                    responses = Some(parsed_responses.into_iter().collect());
                }
                "params" => {
                    if params.is_some() {
                        return Err(input.error("Duplicate params"));
                    }

                    let content;
                    parenthesized!(content in input);

                    let parsed_params = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?;
                    params = Some(parsed_params.into_iter().collect());
                }
                "security" => {
                    if security.is_some() {
                        return Err(input.error("Duplicate security"));
                    }

                    let content;
                    bracketed!(content in input);
                    let parsed_security =
                        Punctuated::<SecurityRequirement, Token![,]>::parse_terminated(&content)?;

                    security = Some(parsed_security.into_iter().collect());
                }
                "request_body" => {
                    if request_body.is_some() {
                        return Err(input.error("Duplicate 'request_body'"));
                    }

                    let content;
                    braced!(content in input);

                    request_body = Some(content.parse()?)
                }
                unknown => return Err(input.error(format!("Unknown field: {}, expected one of 'context_path', 'tag', 'responses', 'params', 'security' or 'request_body'", unknown))),
            }

            // Optionally consume a comma if present
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Documentation {
            method: None,
            path: None,
            context_path,
            tag,
            responses,
            params,
            security,
            request_body,
        })
    }
}
