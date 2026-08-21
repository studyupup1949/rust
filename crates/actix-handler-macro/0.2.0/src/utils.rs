use std::fmt::Display;

use proc_macro::TokenStream;
use proc_macro2::Span;

pub fn compilation_error<T: Display>(msg: T) -> TokenStream {
    TokenStream::from(syn::Error::new(Span::call_site(), msg).to_compile_error())
}
