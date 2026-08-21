extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate proc_macro_error;

use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use std::str::FromStr;
use syn::{
    parse_macro_input, punctuated::Punctuated, token, AttrStyle, Attribute, AttributeArgs, Field,
    Fields, ItemStruct, Lit, Meta, NestedMeta, Path, PathSegment, Type, TypePath, Visibility,
};

#[derive(Eq, PartialEq)]
enum AttrKind {
    Meta,
    Status,
    Content,
}

impl FromStr for AttrKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "meta_attr" => Ok(Self::Meta),
            "status_attr" => Ok(Self::Status),
            "content_attr" => Ok(Self::Content),
            _ => Err(()),
        }
    }
}

struct AttrType<T> {
    kind: AttrKind,
    attr: T,
}

impl From<AttrType<(String, String)>> for AttrType<Attribute> {
    fn from(attr: AttrType<(String, String)>) -> Self {
        let (ident, args) = attr.attr;

        let mut args_attr: Punctuated<PathSegment, token::Colon2> = Punctuated::new();

        args_attr.push_value(PathSegment {
            ident: format_ident!("{}", ident),
            arguments: Default::default(),
        });

        let tokens = match args.len() {
            0 => TokenStream2::default(),
            _ => TokenStream2::from_str(args.as_str())
                .unwrap_or_else(|a| abort!(args, format!("Lex error: {}", a))),
        };

        Self {
            kind: attr.kind,
            attr: Attribute {
                pound_token: token::Pound::default(),
                style: AttrStyle::Outer,
                bracket_token: token::Bracket::default(),
                path: Path {
                    leading_colon: None,
                    segments: args_attr,
                },
                tokens,
            },
        }
    }
}

fn parse_args(args: AttributeArgs) -> Vec<AttrType<(String, String)>> {
    let mut attributes = vec![];
    for nm in args {
        if let NestedMeta::Meta(meta) = nm {
            if let Meta::NameValue(meta_name_val) = meta.clone() {
                if let Lit::Str(strlit) = meta_name_val.lit {
                    let val = strlit.value();

                    if val.is_empty() {
                        abort!(strlit, "empty attribute not allowed");
                    }

                    let end_ident = val
                        .chars()
                        .position(|ch| !ch.is_alphabetic())
                        .unwrap_or(val.len() - 1);

                    let ident = val[0..end_ident].chars().as_str().to_string();
                    let args = val[end_ident..val.len()].chars().as_str().to_string();

                    let attr_kind = AttrKind::from_str(meta
                        .path()
                        .segments.first().unwrap_or_else(|| abort!(
                    meta,
                    "Invalid attribute argument. Only `meta_attr`, `status_attr`, `content_attr` is supported."
                )).ident.to_string().as_str()).unwrap_or_else(|_ | abort!(meta, "Invalid attribute argument. Only `meta_attr`, `status_attr`, `content_attr` is supported."));

                    attributes.push(AttrType {
                        kind: attr_kind,
                        attr: (ident, args),
                    });
                }
            }
        }
    }
    attributes
}

fn make_attrs(args: AttributeArgs) -> Vec<AttrType<Attribute>> {
    let mut punc_attr = Punctuated::new();

    punc_attr.push_value(PathSegment {
        ident: format_ident!("serde"),
        arguments: Default::default(),
    });

    let mut attrs = vec![AttrType {
        kind: AttrKind::Meta,
        attr: Attribute {
            pound_token: token::Pound::default(),
            style: AttrStyle::Outer,
            bracket_token: token::Bracket::default(),
            path: Path {
                leading_colon: None,
                segments: punc_attr.clone(),
            },
            tokens: TokenStream2::from_str("(skip)")
                .unwrap_or_else(|a| abort!(punc_attr, format!("Lex error: {}", a))),
        },
    }];

    let parsed_args = parse_args(args);

    for pa in parsed_args {
        let attr: AttrType<Attribute> = AttrType::from(pa);
        attrs.push(attr);
    }

    attrs
}

fn make_field(ident: Ident, ttype: Path, attrs: Vec<Attribute>) -> Field {
    Field {
        attrs,
        vis: Visibility::Inherited,
        ident: Some(ident),
        colon_token: None,
        ty: Type::Path(TypePath {
            qself: None,
            path: ttype,
        }),
    }
}

fn make_path(strs: &[&str]) -> Path {
    let mut punc: Punctuated<PathSegment, token::Colon2> = Punctuated::new();

    for i in 0..strs.len() {
        punc.push(PathSegment {
            ident: format_ident!("{}", strs[i]),
            arguments: Default::default(),
        });
    }

    Path {
        leading_colon: None,
        segments: punc,
    }
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn actix_responder(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);

    if let Fields::Named(ref mut named) = input.fields {
        let user_struct_name = input.ident.clone();
        let args = parse_macro_input!(args as AttributeArgs);
        let content_type_type_name = make_path(&["String"]);
        let content_type_field_name = format_ident!("content_type");

        let status_code_type_name = make_path(&["actix_web", "http", "StatusCode"]);
        let status_code_field_name = format_ident!("status_code");

        let field_attr = make_attrs(args.clone());
        let mut content_attr = vec![];
        let mut status_attr = vec![];

        for fa in field_attr {
            match fa.kind {
                AttrKind::Meta => {
                    content_attr.push(fa.attr.clone());
                    status_attr.push(fa.attr);
                }
                AttrKind::Content => content_attr.push(fa.attr.clone()),
                AttrKind::Status => status_attr.push(fa.attr.clone()),
            }
        }

        let content_field = make_field(
            content_type_field_name.clone(),
            content_type_type_name,
            content_attr,
        );

        let status_field = make_field(
            status_code_field_name.clone(),
            status_code_type_name,
            status_attr,
        );

        named.named.push(content_field);
        named.named.push(status_field);

        let impl_responder = quote!(
            impl actix_web::Responder for #user_struct_name {
                type Error = actix_web::Error;
                type Future = futures::future::Ready<Result<actix_web::HttpResponse, Self::Error>>;

                fn respond_to(self, req: &actix_web::HttpRequest) -> Self::Future {
                    let body = serde_json::to_string(&self).unwrap();

                    return actix_web::HttpResponse::build(
                        self.#status_code_field_name
                    ).content_type(
                        self.#content_type_field_name
                    ).body(body).respond_to(req);
                }
            }
        );

        let res = TokenStream::from(quote!( #input #impl_responder));
        return res;
    }

    abort!(input, "Tuple structs not supported")
}
