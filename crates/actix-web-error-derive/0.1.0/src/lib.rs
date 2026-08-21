extern crate proc_macro;

mod attr;
mod expand;
mod expander;
mod generics;
mod input;

use expand::expand;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Json, attributes(status))]
pub fn derive_json(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::<expander::Json>(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Text, attributes(status))]
pub fn derive_text(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::<expander::Text>(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
