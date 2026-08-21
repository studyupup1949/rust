use crate::{
    attr::ResolveStatus,
    generics::InferredBounds,
    input::{Enum, Field, Input, Struct},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::collections::BTreeSet;
use syn::{DeriveInput, Member, Result};

pub trait BodyExpander {
    fn expand_struct(input: &Struct) -> TokenStream;
    fn expand_enum(input: &Enum) -> TokenStream;
}

pub fn expand<E: BodyExpander>(node: &DeriveInput) -> Result<TokenStream> {
    match Input::from_syn(node)? {
        Input::Struct(s) => Ok(impl_struct::<E>(&s)),
        Input::Enum(e) => Ok(impl_enum::<E>(&e)),
    }
}

fn impl_struct<E: BodyExpander>(input: &Struct) -> TokenStream {
    let ty = &input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    let mut implied_response_bounds = BTreeSet::new();
    let status_body = match &input.attrs.status {
        Some(ResolveStatus::Transparent(_)) => {
            let only_field = &input.fields[0].member;
            implied_response_bounds.insert(0);
            Some(quote! {
                fn status_code(&self) -> ::actix_web::http::StatusCode {
                    ::actix_web::ResponseError::status_code(&self.#only_field)
                }
            })
        }
        Some(ResolveStatus::Fixed(status)) => Some(quote! {
            fn status_code(&self) -> ::actix_web::http::StatusCode {
                #status
            }
        }),
        None => None,
    };
    let mut inferred_response_bounds = InferredBounds::new();
    for field in implied_response_bounds {
        let field = &input.fields[field];
        if field.contains_generic {
            inferred_response_bounds.insert(field.ty, quote! { ::actix_web::ResponseError });
        }
    }
    let response_where_clause = inferred_response_bounds.augment_where_clause(input.generics);
    let error_expansion = E::expand_struct(input);

    quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics ::actix_web::ResponseError for #ty #ty_generics #response_where_clause {
            #status_body

            #error_expansion
        }
    }
}

fn impl_enum<E: BodyExpander>(input: &Enum) -> TokenStream {
    let ty = &input.ident;
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    let mut inferred_bounds = InferredBounds::new();
    let arms = input.variants.iter().filter_map(|variant| {
        variant.attrs.status.as_ref().map(|s| {
            let status = match s {
                ResolveStatus::Transparent(_) => {
                    let field = &variant.fields[0];
                    if field.contains_generic {
                        inferred_bounds.insert(field.ty, quote! { ::actix_web::ResponseError });
                    }
                    let only_field = match &field.member {
                        Member::Named(ident) => ident.clone(),
                        Member::Unnamed(idx) => format_ident!("_{}", idx),
                    };
                    quote! { ::actix_web::ResponseError::status_code(#only_field) }
                }
                ResolveStatus::Fixed(status) => status.code.to_token_stream(),
            };
            let ident = &variant.ident;
            let pat = fields_pat(&variant.fields);
            quote! { #ty::#ident #pat => #status }
        })
    });
    let arms: Vec<_> = arms.collect();

    let status_body = if arms.is_empty() {
        None
    } else {
        Some(quote! {
            fn status_code(&self) -> ::actix_web::http::StatusCode {
                #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
                match &self {
                    #(#arms,)*
                }
            }
        })
    };

    let where_clause = inferred_bounds.augment_where_clause(input.generics);
    let error_expansion = E::expand_enum(input);

    quote! {
        #[allow(unused_qualifications)]
        impl #impl_generics ::actix_web::ResponseError for #ty #ty_generics #where_clause {
            #status_body

            #error_expansion
        }
    }
}

fn fields_pat(fields: &[Field]) -> TokenStream {
    let mut members = fields.iter().map(|field| &field.member).peekable();
    match members.peek() {
        Some(Member::Named(_)) => quote!({ #(#members),* }),
        Some(Member::Unnamed(_)) => {
            let vars = members.map(|member| match member {
                Member::Unnamed(member) => format_ident!("_{}", member),
                Member::Named(_) => unreachable!(),
            });
            quote!((#(#vars),*))
        }
        None => quote!({}),
    }
}
