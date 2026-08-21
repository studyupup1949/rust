use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

/// Derive macro for the Spanned trait.
///
/// This macro automatically implements the `Spanned` trait by looking
/// for a field of type `SrcLoc`.
#[proc_macro_derive(Spanned)]
pub fn derive_spanned(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Find the field that contains SrcLoc
    let field_name = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                // Find the first field of type SrcLoc
                fields.named.iter().find(|f| {
                    if let syn::Type::Path(type_path) = &f.ty {
                        type_path.path.segments.last()
                            .map(|seg| seg.ident == "SrcLoc")
                            .unwrap_or(false)
                    } else {
                        false
                    }
                })
		    .and_then(|f| f.ident.as_ref())
                    .expect("Spanned requires a field named 'src_loc', 'location', 'span', or a field of type SrcLoc")
            }
            _ => panic!("Spanned can only be derived for structs with named fields"),
        },
        _ => panic!("Spanned can only be derived for structs"),
    };

    let expanded = quote! {
        impl Spanned for #name {
            fn span(&self) -> &SrcLoc {
                &self.#field_name
            }
        }
    };

    TokenStream::from(expanded)
}
