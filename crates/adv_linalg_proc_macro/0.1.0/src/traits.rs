#![allow(dead_code)]

use quote::{quote, ToTokens};
use syn::{Ident, parse::Parse, Error};

#[derive(Clone)]
pub struct Add {
    ident: Ident
}
impl Parse for Add {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expected = "Add";
        match input.parse::<Ident>() {
            Ok(ident) if ident.to_string().as_str() == expected => Ok(Self { ident }),
            Err(error) => Err(error),
            Ok(ident) => Err(Error::new(
                ident.span(), 
                format!("expected trait identifier `{}`, instead found {}", expected, ident.to_string())
            ))
        }
    }
}
impl ToTokens for Add {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!{Add})
    }
}

#[derive(Clone)]
pub struct Mul {
    ident: Ident
}
impl Parse for Mul {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expected = "Mul";
        match input.parse::<Ident>() {
            Ok(ident) if ident.to_string().as_str() == expected => Ok(Self { ident }),
            Err(error) => Err(error),
            Ok(ident) => Err(Error::new(
                ident.span(), 
                format!("expected trait identifier `{}`, instead found {}", expected, ident.to_string())
            ))
        }
    }
}
impl ToTokens for Mul {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!{Mul})
    }
}

#[derive(Clone)]
pub struct Sub {
    ident: Ident
}
impl Parse for Sub {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expected = "Sub";
        match input.parse::<Ident>() {
            Ok(ident) if ident.to_string().as_str() == expected => Ok(Self { ident }),
            Err(error) => Err(error),
            Ok(ident) => Err(Error::new(
                ident.span(), 
                format!("expected trait identifier `{}`, instead found {}", expected, ident.to_string())
            ))
        }
    }
}
impl ToTokens for Sub {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!{Sub})
    }
}