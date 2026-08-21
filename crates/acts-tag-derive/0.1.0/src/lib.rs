use proc_macro::TokenStream;
use quote::quote;
use std::stringify;
use syn::{self, parse_macro_input, DeriveInput, Fields};

const TAG_ATTR_NAME: &str = "tag";

struct Tag {
    name: String,
    ident: Option<syn::Ident>,
}

#[proc_macro_derive(Tags, attributes(tag))]
pub fn derive_tag(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    let mut tags = Vec::new();
    if let syn::Data::Struct(data_struct) = data {
        if let Fields::Named(fields) = data_struct.fields {
            fields.named.iter().for_each(|f| {
                let tag = &f.ident;
                for attr in &f.attrs {
                    if let Some(ident) = attr.path().get_ident() {
                        if ident == TAG_ATTR_NAME {
                            let name = format!("{}", tag.clone().unwrap());
                            tags.push(Tag {
                                name,
                                ident: tag.clone(),
                            });
                        }
                    }
                }
            });
        }
    }

    let mut tag_names = Vec::new();
    let mut tag_value_matches = Vec::new();
    for tag in tags {
        let name = tag.name;
        let ident = tag.ident;
        tag_names.push(quote! { #name });
        tag_value_matches.push(quote! { #name => Some( Value::from(&self.#ident)) });
    }

    let output = quote! {
        impl #ident {

            pub fn is_tag(&self, name: &str) -> bool {
                self.tag_value(name).is_some()
            }

            pub fn tag_value(&self, name: &str) -> Option<Value> {
                match name {
                    #(#tag_value_matches),*,
                    _ => None
                }
            }

            pub fn tag_keys(&self) -> Vec<&str> {
                vec![#(#tag_names),*]
            }
        }
    };
    output.into()
}
