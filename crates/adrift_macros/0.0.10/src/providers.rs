extern crate glob;
extern crate proc_macro;
use std::env;

use convert_case::{Case, Casing};
use glob::glob;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn setup_providers_impl(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let glob_path = format!(
        "{}/{}/*.rs",
        env::var("CARGO_MANIFEST_DIR").unwrap(),
        "providers"
    );

    let mut files = vec![];

    for entry in glob(&glob_path).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let file_name = path
                    .clone()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                files.push((file_name.clone(), file_name.to_case(Case::Pascal)));
            }
            _ => (),
        }
    }

    let mut items = TokenStream::new();
    let mut mods = TokenStream::new();

    files.iter().for_each(|(file, struc)| {
        let a = format_ident!("{}", file);
        let b = format_ident!("{}", struc);

        let token: TokenStream = quote!(Box::new(#a::#b),);

        items.extend(token);

        let token: TokenStream = quote!(pub mod #a;);

        mods.extend(token);
    });

    quote! {
        use adrift::service_providers::{ServiceProvider, ServiceProviders};
        #mods

        #[allow(clippy::all)] pub const PROVIDERS: adrift::once_cell::sync::Lazy<ServiceProviders> = adrift::once_cell::sync::Lazy::new(|| ServiceProviders {
            items: vec![
                #items
            ]
        });
    }
    .into()
}
