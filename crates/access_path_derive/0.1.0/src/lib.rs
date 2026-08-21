use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

/// Convert snake_case / mixedCase to PascalCase.
/// `first_name` → `FirstName`, `HTTPError` → `HttpError`
fn to_pascal_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;
    let mut prev_upper = false;
    let mut after_lower = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
            prev_upper = false;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
            after_lower = c.is_lowercase();
            prev_upper = c.is_uppercase();
        } else if c.is_uppercase() {
            // HTTPError → HttpError (not H T T P E rror)
            if !after_lower && prev_upper {
                result.extend(c.to_lowercase());
            } else {
                result.push(c);
                after_lower = false;
            }
            prev_upper = true;
        } else {
            result.push(c);
            after_lower = true;
            prev_upper = false;
        }
    }
    result
}

/// Generate the path type ident and method name for a field.
fn gen_field_idents(struct_ident: &syn::Ident, field: &syn::Field) -> (syn::Ident, syn::Ident) {
    let field_ident = field.ident.as_ref().expect("named field");
    let field_name = field_ident.to_string();
    let pascal = to_pascal_case(&field_name);
    let path_type_name = format!("{}{}Path", struct_ident, pascal);
    let path_ident = syn::Ident::new(&path_type_name, field_ident.span());
    // Method: {field_name}_path — syn/quote handles `r#` prefix automatically
    let method_name = format!("{field_name}_path");
    let method_ident = syn::Ident::new(&method_name, field_ident.span());
    (path_ident, method_ident)
}

#[proc_macro_derive(KeyPath)]
pub fn derive_keypath(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_ident = &input.ident;

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named),
            ..
        }) => &named.named,
        _ => {
            return syn::Error::new(
                input.ident.span(),
                "KeyPath can only be derived on structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut path_types = Vec::new();
    let mut path_impls = Vec::new();
    let mut path_methods = Vec::new();

    for field in fields.iter() {
        let field_ty = &field.ty;
        let field_ident = field.ident.as_ref().expect("named field");
        let (path_ident, method_ident) = gen_field_idents(struct_ident, field);

        path_types.push(quote! {
            #[derive(Clone, Copy, Debug)]
            pub struct #path_ident;
        });

        let struct_path = quote! { #struct_ident #ty_generics };
        path_impls.push(quote! {
            impl #impl_generics access_path::KeyPath<#struct_path> for #path_ident
            #where_clause
            {
                type Value = #field_ty;

                fn get<'k>(&self, root: &'k #struct_path) -> &'k #field_ty
                where
                    #field_ty: 'k,
                {
                    &root.#field_ident
                }

                fn get_mut<'k>(&self, root: &'k mut #struct_path) -> &'k mut #field_ty
                where
                    #field_ty: 'k,
                {
                    &mut root.#field_ident
                }
            }
        });

        path_methods.push(quote! {
            pub fn #method_ident() -> #path_ident {
                #path_ident
            }
        });
    }

    let expanded = quote! {
        impl #impl_generics #struct_ident #ty_generics #where_clause {
            #(#path_methods)*
        }

        #(#path_types)*

        #(#path_impls)*
    };

    expanded.into()
}
