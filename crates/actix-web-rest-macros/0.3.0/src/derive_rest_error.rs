use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse::discouraged::AnyDelimiter, spanned::Spanned, Data, DeriveInput, Fields, Ident};

use crate::args::RestErrorVariantArguments;

pub fn derive_rest_error_macro(input: DeriveInput) -> Result<TokenStream, syn::Error> {
    // Get the name of the struct or enum.
    let name = &input.ident;

    let enum_data = match input.data {
        Data::Enum(ref enum_data) => enum_data,
        _ => {
            return Err(syn::Error::new(
                name.span(),
                "RestError can only be derived for enums",
            ))
        }
    };

    // Generate RestError impl.
    let rest_error_impl = generate_rest_error_impl(&input, enum_data)?;
    // Generate actix_web::ResponseError impl.
    let response_error_impl = generate_response_error_impl(&input, name);

    // Generate serde::ser::Serialize impl.
    let serialize_impl = generate_serialize_impl(name.clone());

    // Generate utoipa::ToSchema impl.
    let to_schema_impl = generate_to_schema_impl(name.clone(), enum_data);

    // Merge generated code into one TokenStream.
    let mut expanded = TokenStream::new();
    expanded.extend(rest_error_impl);
    expanded.extend(response_error_impl);
    expanded.extend(serialize_impl);
    expanded.extend(to_schema_impl);

    // Return the generated implementation as a token stream.
    Ok(expanded)
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

            for attr in variant.attrs.iter() {
                if attr.path().is_ident("rest") {
                    let stream = attr.meta.clone().into_token_stream();
                    let RestErrorVariantArguments { status_code } = syn::parse::Parser::parse2(
                        |input: syn::parse::ParseStream| {
                            // Skip #[rest...]
                            let _: Ident = input.parse()?;

                            let (_, _, inner_input) = input.parse_any_delimiter()?;
                            inner_input.parse::<RestErrorVariantArguments>()
                        },
                        stream,
                    )?;

                    let params = match variant.fields {
                        Fields::Unit => quote! {},
                        Fields::Unnamed(..) => quote! { (..) },
                        Fields::Named(..) => quote! { {..} },
                    };

                    return Ok(quote_spanned! {variant.span()=>
                        Self::#variant_name #params => #status_code
                    });
                }
            }

            Err(syn::Error::new(
                variant.span(),
                "Missing rest attribute on variant",
            ))
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

fn generate_response_error_impl(_input: &DeriveInput, name: &Ident) -> TokenStream {
    quote! {
        impl ::actix_web::ResponseError for #name {
            fn status_code(&self) -> ::actix_web_rest::http::StatusCode {
                ::actix_web_rest::RestError::status_code(self)
            }

            fn error_response(&self) -> ::actix_web::HttpResponse<::actix_web::body::BoxBody> {
                ::actix_web::HttpResponse::build(
                    ::actix_web::ResponseError::status_code(self)
                ).json(self)
            }
        }
    }
}

fn generate_serialize_impl(name: Ident) -> TokenStream {
    quote! {
       impl ::serde::ser::Serialize for #name {
           fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
           where
               S: ::serde::ser::Serializer
           {
                use ::serde::ser::SerializeStruct;
                let mut struct_serializer = serializer.serialize_struct(stringify!(#name), 2)?;
                struct_serializer.serialize_field("error_code", self.as_ref())?;
                if self.as_ref() == "InternalError" {
                    struct_serializer.serialize_field(
                       "error_message",
                       &"internal server error, check server logs for more information",
                    )?;
                } else {
                    struct_serializer.serialize_field(
                       "error_message",
                       &self.to_string()
                    )?;
                }
                struct_serializer.end()
           }
        }
    }
}

fn generate_to_schema_impl(name: Ident, enum_data: &syn::DataEnum) -> TokenStream {
    let variant_names: Vec<String> = enum_data
        .variants
        .iter()
        .map(|variant| variant.ident.clone().to_string())
        .collect();

    let variants = quote! {
        vec![
            #(stringify!(#variant_names)),*
        ]
    };

    quote! {
        impl<'a> ::utoipa::ToSchema<'a> for #name {
            fn schema() -> (
                &'a str, ::utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
            ) {
                (
                    stringify!(#name),
                    ::utoipa::openapi::ObjectBuilder::new()
                        .property(
                            "error_code",
                            ::utoipa::openapi::ObjectBuilder::new()
                                .schema_type(::utoipa::openapi::SchemaType::String)
                                .enum_values::<Vec<&'static str>, &str>(Some(#variants)),
                        )
                        .required("error_code")
                        .property(
                            "error_message",
                            ::utoipa::openapi::ObjectBuilder::new()
                                .schema_type(::utoipa::openapi::SchemaType::String),
                        )
                        .required("error_message")
                        .into(),
                )
            }
        }
    }
}
