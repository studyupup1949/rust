use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

/// Derive macro for the Spanned trait.
///
/// This macro automatically implements the `Spanned` trait by:
/// - For structs: looking for a field of type `Option<SrcLoc>`
/// - For enums: delegating to each variant's `span()` method (assumes each variant implements Spanned)
#[proc_macro_derive(Spanned)]
pub fn derive_spanned(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = match &input.data {
        Data::Struct(data) => {
            let field_name = match &data.fields {
                Fields::Named(fields) => {
                    fields.named.iter()
                        .find(|f| is_option_srcloc(&f.ty))
                        .and_then(|f| f.ident.as_ref())
                        .expect("Spanned requires a field of type Option<SrcLoc>")
                    }
                _ => panic!("Spanned can only be derived for structs with named fields"),
            };

            quote! {
                impl Spanned for #name {
                    fn span(&self) -> Option<&SrcLoc> {
                        self.#field_name.as_ref()
                    }
                }
            }
        }
        Data::Enum(data) => {
            let match_arms = data.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                match &variant.fields {
                    Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                        // Single tuple field - delegate to it
                        quote! {
                            #name::#variant_name(inner) => inner.span()
                        }
                    }
                    Fields::Unnamed(fields) if fields.unnamed.len() > 1 => {
                        // Multiple tuple fields - delegate to the first one
                        quote! {
                            #name::#variant_name(inner, ..) => inner.span()
                        }
                    }
                    Fields::Named(fields) => {
                         let field_names: Vec<_> = fields.named.iter()
                            .filter_map(|f| f.ident.as_ref())
                            .collect();

                        if let Some(src_loc_field) = fields.named.iter()
                            .find(|f| is_option_srcloc(&f.ty))
                            .and_then(|f| f.ident.as_ref())
                            {
                            // Has a SrcLoc field
                            quote! {
                                #name::#variant_name { ref #src_loc_field, .. } => #src_loc_field.as_ref()
                            }
                        } else if let Some(first_field) = field_names.first() {
                            // Delegate to first field
                            quote! {
                                #name::#variant_name { #first_field, .. } => #first_field.span()
                            }
                        } else {
                            panic!("Enum variant {} has no fields", variant_name);
                        }
                    }
                    Fields::Unit => {
                        panic!("Unit enum variants cannot implement Spanned")
                    }
                    &Fields::Unnamed(_) => todo!()
                }
            });

            quote! {
                impl Spanned for #name {
                    fn span(&self) -> Option<&SrcLoc> {
                        match self {
                            #(#match_arms,)*
                        }
                    }
                }
            }
        }
        Data::Union(_) => panic!("Spanned cannot be derived for unions"),
    };

    TokenStream::from(expanded)
}

fn is_option_srcloc(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(syn::Type::Path(inner_path))) = args.args.first() {
                        return inner_path.path.segments.last()
                            .map(|seg| seg.ident == "SrcLoc")
                            .unwrap_or(false);
                    }
                }
            }
        }
    }
    false
}
