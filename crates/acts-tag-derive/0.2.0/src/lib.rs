use proc_macro::TokenStream;
use quote::quote;
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
                let field_name = &f.ident;
                let tag_attrs: Vec<&syn::Attribute> = f
                    .attrs
                    .iter()
                    .filter(|at| at.path().is_ident(TAG_ATTR_NAME))
                    .collect();
                for attr in tag_attrs {
                    let mut name = format!("{}", field_name.clone().unwrap());
                    if let Some(tag_name) = parse_tag_name(attr).unwrap() {
                        name = format!("{}", tag_name)
                    }
                    tags.push(Tag {
                        name,
                        ident: field_name.clone(),
                    });
                }
            });
        }
    }

    let mut tag_names = Vec::new();
    let mut tag_value_matches = Vec::new();
    for tag in tags {
        let name = tag.name;
        let ident = tag.ident;
        tag_names.push(quote! { #name.to_string() });
        tag_value_matches.push(quote! { #name => Some( Value::from(&self.#ident).unwrap() ) });
    }

    let output = quote! {

        impl Tags for #ident {
            fn value(&self, name: &str) -> Option<Value> {
                match name {
                    #(#tag_value_matches),*,
                    _ => None
                }
            }

            fn keys() -> Vec<String> {
                vec![#(#tag_names),*]
            }
        }
    };
    output.into()
}

fn parse_tag_name(attr: &syn::Attribute) -> Result<Option<syn::Ident>, syn::Error> {
    let mut key = None;
    let _ = attr.parse_nested_meta(|meta| {
        if let Some(name) = meta.path.get_ident() {
            key = Some(name.clone());
        }
        Ok(())
    });

    Ok(key)
}
