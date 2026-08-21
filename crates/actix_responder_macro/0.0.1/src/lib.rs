extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate proc_macro_error;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::str::FromStr;
use syn::{
    parse_macro_input, punctuated::Punctuated, token, AttrStyle, Attribute, AttributeArgs, Field,
    Fields, ItemStruct, Lit, Meta, NestedMeta, Path, PathSegment, Type, TypePath, VisPublic,
    Visibility,
};

fn parse_args(args: AttributeArgs) -> Option<(String, String)> {
    for nm in args {
        if let NestedMeta::Meta(meta) = nm {
            let has_meta = meta
                .path()
                .segments
                .iter()
                .any(|m| m.ident.to_string() == "meta_attr");

            if !has_meta {
                abort!(
                    meta,
                    "Invalid attribute argument. Only `meta_attr` is supported."
                );
            }

            if let Meta::NameValue(meta_name_val) = meta {
                if let Lit::Str(strlit) = meta_name_val.lit {
                    let val = strlit.value();

                    if val.is_empty() {
                        abort!(strlit, "empty meta_attr not allowed");
                    }

                    let end_ident = val
                        .chars()
                        .position(|ch| !ch.is_alphabetic())
                        .unwrap_or(val.len() - 1);

                    let ident = val[0..end_ident].chars().as_str().to_string();
                    let args = val[end_ident..val.len()].chars().as_str().to_string();

                    return Some((ident, args));
                }
            }
        }
    }
    return None;
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn actix_responder(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    if let Fields::Named(ref mut named) = input.fields {
        let user_struct_name = format_ident!("{}", input.ident.to_string());
        let meta_type_name = format_ident!("{}Metadata", input.ident.to_string());
        let meta_field_name = format_ident!(
            "{}_metadata",
            user_struct_name.to_string().as_str().to_lowercase()
        );

        let mut punc = Punctuated::new();

        punc.push_value(PathSegment {
            ident: meta_type_name.clone(),
            arguments: Default::default(),
        });

        let mut punc_attr = Punctuated::new();

        punc_attr.push_value(PathSegment {
            ident: format_ident!("serde"),
            arguments: Default::default(),
        });

        let mut attrs = vec![Attribute {
            pound_token: token::Pound::default(),
            style: AttrStyle::Outer,
            bracket_token: token::Bracket::default(),
            path: Path {
                leading_colon: None,
                segments: punc_attr,
            },
            tokens: TokenStream2::from_str("(skip)")
                .unwrap_or_else(|a| abort!(punc, format!("Lex error: {}", a))),
        }];

        let args = parse_macro_input!(args as AttributeArgs);
        if let Some((arg_ident, arg)) = parse_args(args) {
            let mut args_attr: Punctuated<PathSegment, token::Colon2> = Punctuated::new();

            args_attr.push_value(PathSegment {
                ident: format_ident!("{}", arg_ident),
                arguments: Default::default(),
            });

            let tokens = match arg.len() {
                0 => TokenStream2::default(),
                _ => TokenStream2::from_str(arg.as_str())
                    .unwrap_or_else(|a| abort!(arg, format!("Lex error: {}", a))),
            };

            attrs.push(Attribute {
                pound_token: token::Pound::default(),
                style: AttrStyle::Outer,
                bracket_token: token::Bracket::default(),
                path: Path {
                    leading_colon: None,
                    segments: args_attr,
                },
                tokens,
            });
        }

        named.named.push(Field {
            attrs,
            vis: Visibility::Public(VisPublic {
                pub_token: token::Pub::default(),
            }),
            ident: Some(meta_field_name.clone()),
            colon_token: None,
            ty: Type::Path(TypePath {
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments: punc,
                },
            }),
        });

        let meta_struct = quote!(
            #[derive(Default, Clone, Debug)]
            struct #meta_type_name {
                status_code: Option<actix_web::http::StatusCode>,
                content_type: Option<String>,
            }
        );

        let impl_responder = quote!(
            impl actix_web::Responder for #user_struct_name {
                type Error = actix_web::Error;
                type Future = futures::future::Ready<Result<actix_web::HttpResponse, Self::Error>>;

                fn respond_to(self, req: &actix_web::HttpRequest) -> Self::Future {
                    let body = serde_json::to_string(&self).unwrap();

                    return actix_web::HttpResponse::build(
                        self.#meta_field_name.status_code.unwrap_or(actix_web::http::StatusCode::OK)
                    ).content_type(
                        self.#meta_field_name.content_type.unwrap_or(String::new())
                    ).body(body).respond_to(req);
                }
            }
        );

        let res = TokenStream::from(quote!( #meta_struct #input #impl_responder ));
        return res;
    }

    abort!(input, "Tuple structs not supported")
}
