use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(DefaultControllerInit)]
pub fn derive_auto_new(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as syn::DeriveInput);
  let name = input.ident;

  let expanded = quote! {
    impl adjust::controller::ControllerInitialization<AppState> for #name {
      async fn new() -> anyhow::Result<Box<Self>> {
        Ok(Box::new(Self))
      }
    }
  };

  TokenStream::from(expanded)
}