use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, token, Data, DeriveInput, Fields, Ident, Path, Variant,
};

#[proc_macro_attribute]
pub fn rest_error(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Generate the implementation code for the desired traits.
    let expanded = quote! {
        #[derive(
            Debug,
            ::actix_web_rest::actix_web_rest_macros::RestError,
            ::actix_web_rest::strum::AsRefStr,
            ::actix_web_rest::thiserror::Error,
        )]
        #input
    };

    // Return the generated implementation as a token stream.
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(RestError, attributes(rest, serde))]
pub fn derive_rest_error(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct or enum.
    let name = &input.ident;

    let rest_error_impl = match input.data {
        Data::Enum(ref enum_data) => match generate_rest_error_impl(&input, enum_data) {
            Ok(v) => v,
            Err(err) => err.to_compile_error(),
        },
        _ => panic!("RestError can only be derived for enums"),
    };

    let response_error_impl = generate_response_error_impl(&input, name);

    // Generate the implementation code for the desired traits.
    let mut expanded = TokenStream::new();
    expanded.extend(rest_error_impl);
    expanded.extend(response_error_impl);
    expanded.extend(quote! {
       impl ::actix_web_rest::serde::ser::Serialize for #name {
           fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
           where
               S: ::actix_web_rest::serde::ser::Serializer
           {
                use ::actix_web_rest::serde::ser::SerializeStruct;
                let mut struct_serializer = serializer.serialize_struct(stringify!(#name), 2)?;
                struct_serializer.serialize_field("error_code", self.as_ref())?;
                struct_serializer.serialize_field(
                   "error_message",
                   &self.to_string()
                )?;
                struct_serializer.end()
           }
        }
    });

    // Return the generated implementation as a token stream.
    proc_macro::TokenStream::from(expanded)
}

fn generate_rest_error_impl(
    input: &DeriveInput,
    enum_data: &syn::DataEnum,
) -> Result<TokenStream, syn::Error> {
    let enum_name = &input.ident;

    let match_arms = enum_data
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let status_code = extract_status_code(variant)?;

            let params = match variant.fields {
                Fields::Unit => quote! {},
                Fields::Unnamed(..) => quote! { (..) },
                Fields::Named(..) => quote! { {..} },
            };
            Ok(quote_spanned! {variant.span()=>
                Self::#variant_name #params => #status_code
            })
        })
        .collect::<Result<Vec<_>, syn::Error>>()?;

    let expanded = quote! {
        impl ::actix_web_rest::RestError for #enum_name {
            fn status_code(&self) -> ::actix_web_rest::http::StatusCode {
                match self {
                    #(#match_arms),*
                }
            }
        }
    };

    Ok(expanded)
}

fn extract_status_code(variant: &Variant) -> Result<TokenStream, syn::Error> {
    let mut optional_token_stream = None;
    for attr in &variant.attrs {
        if attr.path().is_ident("rest") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("status_code") && meta.input.peek(token::Eq) {
                    let value = meta.value()?;
                    let path: Path = value.parse()?;
                    optional_token_stream = Some(quote! { #path });
                    return Ok(());
                }
                Err(syn::Error::new(
                    variant.span(),
                    "Unknown rest attribute format",
                ))
            })?;
        }
    }

    optional_token_stream.ok_or(syn::Error::new(
        variant.span(),
        "Missing `status_code` attribute on variant",
    ))
}

fn generate_response_error_impl(_input: &DeriveInput, name: &Ident) -> TokenStream {
    quote! {
        impl ::actix_web_rest::actix_web::ResponseError for #name {
            fn status_code(&self) -> ::actix_web_rest::http::StatusCode {
                ::actix_web_rest::RestError::status_code(self)
            }

            fn error_response(&self) -> ::actix_web_rest::actix_web::HttpResponse<::actix_web_rest::actix_web::body::BoxBody> {
                ::actix_web_rest::actix_web::HttpResponse::build(
                    ::actix_web_rest::actix_web::ResponseError::status_code(self)
                ).json(self)
            }
        }
    }
}
