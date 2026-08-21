use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
  let input = parse_macro_input!(item as ItemFn);
  let name = &input.sig.ident;
  let block = &input.block;

  let ret_type = match &input.sig.output {
    ReturnType::Type(_, ty) => ty,
    _ => panic!("main function should return Service<AppState>"),
  };

  let expanded = quote! {
    #[tokio::main]
    async fn #name() -> anyhow::Result<()> {
      adjust::server::WebServer::enviroment();

      let service: #ret_type = #block;
      service.run().await
    }
  };

  TokenStream::from(expanded)
}

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