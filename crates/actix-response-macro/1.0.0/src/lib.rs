use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemEnum};

#[proc_macro_derive(ResponseError, attributes(response_code))]
pub fn derive_response_error(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item: TokenStream = item.into();
    let item: ItemEnum = syn::parse2(item).unwrap();
    let name = item.ident;
    let mut variants: Vec<(Ident, Ident)> = Vec::new();
    for variant in item.variants {
        let Some(attr) = variant
            .attrs
            .iter()
            .find(|a| a.path().is_ident("response_code"))
        else {
            panic!("missing response_code for variant {}", variant.ident);
        };

        let code: Ident = attr.parse_args().unwrap();
        variants.push((variant.ident, code));
    }

    let matches = variants
        .iter()
        .map(|(k, v)| {
            quote! {
                Self::#k => ::actix_web::http::StatusCode::#v
            }
        })
        .collect::<Vec<_>>();

    let res = quote! {
        impl ::actix_web::ResponseError for #name {
            fn error_response(&self) -> ::actix_web::HttpResponse {
                let status = match self {
                    #(#matches),*
                };

                ::actix_web::HttpResponse::build(status).json(::actix_error_helper::ApiResponse::<()>::Error(self.to_string()))
            }
        }
    };

    eprintln!("{}", res.to_string());

    res.into()
}
