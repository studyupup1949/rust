#[allow(dead_code)]
pub fn parse2_or_compile_error<T: syn::parse::Parse>(
    ts2: proc_macro2::TokenStream,
) -> Result<T, proc_macro2::TokenStream> {
    syn::parse2::<T>(ts2).map_err(|e| e.to_compile_error())
}
