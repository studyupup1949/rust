use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use venial::{parse_declaration, Declaration, Error};

/// Derives the empty `Sealed` trait.
/// 
/// Since this is just a marker trait, this derive is equivalent to:
/// ```
/// struct MyType {}
/// 
/// impl Sealed for MyType {}
/// ```
#[proc_macro_derive(Sealed)]
pub fn derive_sealed(input: TokenStream1) -> TokenStream1 {
    let struct_type = parse_declaration(TokenStream2::from(input));

    match struct_type {
        Err(error) => {
            TokenStream1::from(error.to_compile_error())
        }
        Ok(Declaration::Struct(struct_)) => {
            let ty_name = struct_.name;

            let generics = struct_.generic_params.unwrap_or_default().to_token_stream();

            let where_clause = struct_.where_clause.unwrap_or_default().to_token_stream();

            TokenStream1::from(quote! {
                impl #generics Sealed for #ty_name #generics #where_clause {}
            })
        }
        _ => {
            TokenStream1::from(
                Error::new("Must be a `struct` to derive `Sealed`").to_compile_error(),
            )
        }
    }
}
