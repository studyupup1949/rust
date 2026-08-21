use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(AddressEq)]
pub fn address_eq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let lifetime = &input.generics;
    (quote! {
        #[automatically_derived]
        impl #lifetime ::std::cmp::PartialEq for #name #lifetime {
            fn eq(&self, other: &Self) -> bool {
                ::std::ptr::eq(self as *const #name, other as *const #name)
            }
        }

        #[automatically_derived]
        impl #lifetime ::std::cmp::Eq for #name #lifetime {}
    })
    .into()
}

#[proc_macro_derive(AddressHash)]
pub fn address_hash(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let lifetime = &input.generics;
    (quote! {
        #[automatically_derived]
        impl #lifetime ::std::hash::Hash for #name #lifetime {
            fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                (self as *const #name).hash(state)
            }
        }
    })
    .into()
}

#[proc_macro_derive(AddressOrd)]
pub fn address_ord(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let lifetime = &input.generics;
    (quote! {
        #[automatically_derived]
        impl #lifetime ::std::cmp::Ord for #name #lifetime {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                (self as *const #name).cmp(&(other as *const #name))
            }
        }

        #[automatically_derived]
        impl #lifetime ::std::cmp::PartialOrd for #name #lifetime {
            fn partial_cmp(&self, other: &Self) -> ::std::option::Option<::std::cmp::Ordering> {
                ::std::option::Option::Some(self.cmp(other))
            }
        }
    })
    .into()
}
