use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DeriveInput};

#[proc_macro_attribute]
pub fn template(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as AttributeArgs);
    let input = parse_macro_input!(item as DeriveInput);

    let ident = input.ident.clone();
    let mut path_str = String::new();
    for att in attrs.into_iter() {
        if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
            path,
            lit: syn::Lit::Str(lit_str),
            ..
        })) = att
        {
            if path.is_ident("path") {
                path_str = lit_str.value();
                eprintln!("path_str: {}", path_str);
            }
        }
    }

    let output = quote! {
        #input

        impl super::GenerateADF for #ident {
            fn to_adf(&self) -> Result<String, Box<dyn std::error::Error>> {
                let template_str = include_str!(#path_str);
                let mut template = super::__private::tinytemplate::TinyTemplate::new();
                template.add_template(stringify!(#ident), &template_str)?;
                let rendered = template.render(stringify!(#ident), &self)?;
                let adf_str = super::__private::convert_html_str_to_adf_str(rendered);
                Ok(adf_str)
            }
        }
    };

    output.into()
}
