use args::RestErrorMacroArguments;
use syn::{parse_macro_input, DeriveInput};

mod args;
mod attr_rest_error;
mod derive_rest_error;

use attr_rest_error::*;
use derive_rest_error::*;

#[proc_macro_attribute]
pub fn rest_error(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse args and input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    let args = parse_macro_input!(args as RestErrorMacroArguments);

    match rest_error_macro(args, input) {
        Ok(ts) => ts.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

#[proc_macro_derive(RestError, attributes(rest))]
pub fn derive_rest_error(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    match derive_rest_error_macro(input) {
        Ok(ts) => ts.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
