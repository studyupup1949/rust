use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Data, DeriveInput, Variant};

use crate::args::{RestErrorMacroArguments, RestErrorVariantArguments};

pub fn rest_error_macro(
    args: RestErrorMacroArguments,
    mut input: DeriveInput,
) -> Result<TokenStream, syn::Error> {
    let enum_name = input.ident.clone();
    let enum_data = match input.data {
        Data::Enum(ref mut enum_data) => enum_data,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "rest_error can only be applied on enum",
            ))
        }
    };

    // Add InternalError variant.
    if let Some(RestErrorVariantArguments { status_code }) = args.internal_error {
        let variant_meta = quote! {
            #[rest(status_code = #status_code)]
            #[error(transparent)]
            InternalError(
                #[from]
                anyhow::Error)
        };
        let variant: Variant = parse_quote!(#variant_meta);
        enum_data.variants.push(variant);
    }

    // Generate the implementation code for the desired traits.
    let mut expanded = quote! {
        #[derive(
            ::actix_web_rest::macros::RestError,
            ::strum::AsRefStr,
            ::thiserror::Error,
        )]
    };
    expanded.extend(quote!(#input));
    expanded.extend(quote! {
        impl ::std::fmt::Debug for #enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                writeln!(f, "{}\n", self)?;
                let mut current = std::error::Error::source(self);
                while let Some(cause) = current {
                    writeln!(f, "Caused by:\n\t{}", cause)?;
                    current = std::error::Error::source(cause);
                }
                Ok(())
            }
        }
    });

    // Return the generated implementation as a token stream.
    Ok(expanded)
}
