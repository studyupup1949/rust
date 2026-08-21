use crate::generate_endpoint::documentation::response::Response;
use crate::generate_endpoint::documentation::Documentation;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

impl ToTokens for Documentation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.method.is_none() && self.path.is_none() {
            panic!("Method & Path is needed in order to construct documentation")
        }

        let method = &self.method;
        let path = &self.path;
        let context_path = self
            .context_path
            .as_ref()
            .map(|cp| quote! { context_path = #cp, });
        let tag = self.tag.as_ref().map(|t| quote! { tag = #t, });
        let responses = if let Some(responses) = &self.responses {
            expand_responses(responses)
        } else {
            TokenStream::new()
        };
        let params = self.params.as_ref().map(|params| {
            quote! { params( #( #params ),* ), }
        });
        let security = self.security.as_ref().map(|security| {
            let security_iter = security.iter().map(|security| {
                if let Some(name) = &security.name {
                    let scopes = &security.scopes;
                    quote! {
                        (#name = [#( #scopes ),*])
                    }
                } else {
                    quote! { () }
                }
            });
            quote! { security( #( #security_iter ),* ), }
        });
        let request_body = self.request_body.as_ref().map(|rb| {
            quote! { #rb, }
        });

        let expanded = quote! {
            #[::actix_helper_utils::utoipa::path(
                #method,
                path = #path,
                #context_path
                #tag
                #responses
                #params
                #security
                #request_body
            )]
        };

        tokens.extend(expanded);
    }
}

impl ToTokens for Response {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(responses_type) = &self.responses_type {
            tokens.extend(quote! {#responses_type});
        } else if let Some(status_code) = &self.status_code {
            let description = self
                .description
                .as_ref()
                .map_or(quote! {}, |desc| quote! {, description = #desc });
            let response_ty = self
                .response
                .as_ref()
                .map_or(quote! {}, |res| quote! {, response = #res });
            let content_type = if let Some(content_type) = &self.content_type {
                quote! {, content_type = #content_type}
            } else {
                TokenStream::new()
            };
            let body = self
                .response_body
                .as_ref()
                .map_or(TokenStream::new(), |body| quote! { #body });

            let expanded =
                quote! {(status = #status_code #description #response_ty #content_type, #body)};

            tokens.extend(expanded);
        }
    }
}

fn expand_responses(responses: &[Response]) -> TokenStream {
    let response_iter = responses.iter().map(|response| {
        quote! { #response }
    });

    quote! { responses(#( #response_iter ),*), }
}
