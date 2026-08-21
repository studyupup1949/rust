use proc_macro2::TokenStream;

pub fn map_err_to_compile_error<T>(res: syn::Result<T>) -> Result<T, TokenStream> {
    match res {
        Ok(value) => Ok(value),
        Err(err) => Err(err.to_compile_error()),
    }
}
