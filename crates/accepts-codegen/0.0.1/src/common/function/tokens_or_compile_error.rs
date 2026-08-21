use proc_macro2::TokenStream;

use super::map_err_to_compile_error;

#[allow(dead_code)]
pub fn tokens_or_compile_error(res: syn::Result<TokenStream>) -> TokenStream {
    match map_err_to_compile_error(res) {
        Ok(tokens) => tokens,
        Err(err_tokens) => err_tokens,
    }
}
