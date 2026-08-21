use proc_macro2::TokenStream;
use quote::quote;

pub fn expand(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    quote! {
        impl ::actix::Supervised for #name {}
    }
}
