//! # Auto Convert Enums
//! This crate provides a proc macro that generates [From] impls for unnamed enum fields that have a type.
//!
//! Ever get tired of writing the same From impls to enable using `?` on some [Result] types? This crate is for you!
//!
//! This is useful for enums that are used as a wrapper for a collecting multiple errors into one big enum.
//!
//! ## Example
//! ```
//! #[macro_use]
//! extern crate ace_it;
//!
//! #[derive(Debug)]
//! #[ace_it]
//! enum Error {
//!   Io(std::io::Error),
//!   ParseInt(std::num::ParseIntError),
//!   ParseFloat(std::num::ParseFloatError),
//! }
//! ```
//! After this, Error has three [From] impls:
//! * [From]<[std::io::Error]> for Error
//! * [From]<[std::num::ParseIntError]> for Error
//! * [From]<[std::num::ParseFloatError]> for Error
//!
//! Now you can use `?` on any of these types and get an Error back.
//! ```
//! use std::io::Read;
//!
//! fn read_int<R: Read>(reader: &mut R) -> Result<i32, Error> {
//!     let mut buf = String::new();
//!     reader.read_to_string(&mut buf)?;
//!     Ok(buf.parse()?)
//! }

use std::collections::HashSet;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Fields, Variant};

#[proc_macro_attribute]
pub fn ace_it(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let parsed = syn::parse(input);

    let parsed = match parsed {
        Ok(parsed) => parsed,
        Err(e) => return e.to_compile_error().into(),
    };

    ace_it_impl(parsed).into()
}

/// Generates From impls for the given enum.
fn process_variants<'a>(
    variants: impl Iterator<Item = &'a Variant>,
    enum_name: &Ident,
) -> Vec<TokenStream> {
    let mut from_impls = Vec::new();

    for variant in variants {
        let variant_name = &variant.ident;

        if let Fields::Unnamed(fields) = &variant.fields {
            let types = &fields.unnamed;
            let imp = quote! {
                impl From<#types> for #enum_name {
                    fn from(value: #types) -> Self {
                        Self::#variant_name(value)
                    }
                }
            };

            from_impls.push(imp);
        }
    }

    from_impls
}

fn find_duplicate_variant_type<'a>(variants: impl Iterator<Item = &'a Variant>) -> Option<Span> {
    let mut types_map = HashSet::new();
    for variant in variants {
        if let Fields::Unnamed(fields) = &variant.fields {
            let types = fields.unnamed.to_token_stream().to_string();

            if !types_map.insert(types) {
                return Some(variant.span());
            }
        }
    }
    None
}

fn ace_it_impl(parsed: syn::ItemEnum) -> TokenStream {
    let mut enum_def = parsed.to_token_stream();
    if let Some(var) = find_duplicate_variant_type(parsed.variants.iter()) {
        return syn::Error::new(
            var,
            "Duplicate variant type, can't auto-generate From impls",
        )
        .to_compile_error();
    }

    let for_impls = process_variants(parsed.variants.iter(), &parsed.ident);

    for impls in for_impls {
        impls.to_tokens(&mut enum_def);
    }

    enum_def
}

#[cfg(test)]
mod tests {
    use super::*;

    use quote::quote;
    use syn::parse2;

    #[test]
    fn ace_it() {
        let input = quote! {
            enum Test {
                A,
                B(u32),
                C { a: u32, b: u32 },
            }
        };
        let expected = quote! {
            enum Test {
                A,
                B(u32),
                C { a: u32, b: u32 },
            }

            impl From<u32> for Test {
                fn from(value: u32) -> Self {
                    Self::B(value)
                }
            }
        };
        let parsed: syn::ItemEnum = parse2(input).unwrap();
        let result = ace_it_impl(parsed);
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn repeating_types_error() {
        let input = quote! {
            enum Test {
                A,
                B(u32),
                C(u32),
            }
        };
        let parsed: syn::ItemEnum = parse2(input).unwrap();
        let result = ace_it_impl(parsed);
        assert!(result.to_string().contains("Duplicate variant type"));
    }
}
