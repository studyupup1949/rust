//! Derive macro for the adapters library.
//!
//! Generates Serialize, Deserialize, Validate, SchemaProvider, and Adapter trait implementations.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Expr, ExprLit, Fields, GenericArgument, Lit, PathArguments, Type,
    parse_macro_input,
};

#[derive(Default, Debug)]
struct FieldAttrs {
    min_length: Option<usize>,
    max_length: Option<usize>,
    email: bool,
    url: bool,
    regex: Option<String>,
    min: Option<f64>,
    max: Option<f64>,
    strict: bool,
    default: Option<String>,
    alias: Option<String>,
    optional: bool,
    custom: Option<syn::Ident>,
    non_empty: bool,
    alphanumeric: bool,
    positive: bool,
    negative: bool,
    non_zero: bool,
}

fn parse_field_attrs(attrs: &[Attribute]) -> FieldAttrs {
    let mut out = FieldAttrs::default();
    for attr in attrs {
        if !attr.path().is_ident("schema") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("min_length") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;
                if let Lit::Int(i) = lit {
                    out.min_length = Some(i.base10_parse().unwrap_or(0));
                }
            } else if meta.path.is_ident("max_length") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;
                if let Lit::Int(i) = lit {
                    out.max_length = Some(i.base10_parse().unwrap_or(0));
                }
            } else if meta.path.is_ident("email") {
                out.email = true;
            } else if meta.path.is_ident("url") {
                out.url = true;
            } else if meta.path.is_ident("non_empty") {
                out.non_empty = true;
            } else if meta.path.is_ident("alphanumeric") {
                out.alphanumeric = true;
            } else if meta.path.is_ident("positive") {
                out.positive = true;
            } else if meta.path.is_ident("negative") {
                out.negative = true;
            } else if meta.path.is_ident("non_zero") {
                out.non_zero = true;
            } else if meta.path.is_ident("regex") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;
                if let Lit::Str(s) = lit {
                    out.regex = Some(s.value());
                }
            } else if meta.path.is_ident("min") {
                let value = meta.value()?;
                let expr: Expr = value.parse()?;
                out.min = extract_number(&expr);
            } else if meta.path.is_ident("max") {
                let value = meta.value()?;
                let expr: Expr = value.parse()?;
                out.max = extract_number(&expr);
            } else if meta.path.is_ident("strict") {
                out.strict = true;
            } else if meta.path.is_ident("default") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;
                out.default = Some(match &lit {
                    Lit::Str(s) => s.value(),
                    Lit::Int(i) => i.to_string(),
                    Lit::Float(f) => f.to_string(),
                    _ => String::new(),
                });
            } else if meta.path.is_ident("alias") {
                let value = meta.value()?;
                let lit: Lit = value.parse()?;
                if let Lit::Str(s) = lit {
                    out.alias = Some(s.value());
                }
            } else if meta.path.is_ident("optional") {
                out.optional = true;
            } else if meta.path.is_ident("custom") {
                let value = meta.value()?;
                let ident: syn::Ident = value.parse()?;
                out.custom = Some(ident);
            }
            Ok(())
        });
    }
    out
}

fn extract_number(expr: &Expr) -> Option<f64> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(i), ..
        }) => i.base10_parse::<f64>().ok(),
        Expr::Lit(ExprLit {
            lit: Lit::Float(f), ..
        }) => f.base10_parse::<f64>().ok(),
        Expr::Unary(u) => {
            if let syn::UnOp::Neg(_) = u.op {
                extract_number(&u.expr).map(|n| -n)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn extract_option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(tp) = ty else {
        return None;
    };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    let Some(GenericArgument::Type(inner)) = ab.args.first() else {
        return None;
    };
    Some(inner)
}

fn extract_vec_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(tp) = ty else {
        return None;
    };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Vec" {
        return None;
    }
    let PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    let Some(GenericArgument::Type(inner)) = ab.args.first() else {
        return None;
    };
    Some(inner)
}

fn type_ident_str(ty: &Type) -> Option<String> {
    if let Type::Path(tp) = ty {
        tp.path.segments.last().map(|s| s.ident.to_string())
    } else {
        None
    }
}

fn is_int_type(name: &str) -> bool {
    matches!(
        name,
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "usize"
    )
}

fn is_float_type(name: &str) -> bool {
    matches!(name, "f32" | "f64")
}

fn field_schema_expr(ty: &Type, attrs: &FieldAttrs) -> TokenStream2 {
    if let Some(inner) = extract_option_inner(ty) {
        let inner_schema = field_schema_expr_inner(inner, attrs, true);
        return inner_schema;
    }
    field_schema_expr_inner(ty, attrs, false)
}

fn field_schema_expr_inner(ty: &Type, attrs: &FieldAttrs, is_option: bool) -> TokenStream2 {
    if let Some(inner) = extract_vec_inner(ty) {
        let item_schema = field_schema_expr_inner(inner, &FieldAttrs::default(), false);
        let req = if is_option {
            quote! { .optional() }
        } else {
            quote! { .required() }
        };
        return quote! {
            ::adapters::ArraySchema::new(#item_schema) #req
        };
    }

    let name = type_ident_str(ty).unwrap_or_default();

    if name == "String" {
        let mut chain = quote! { ::adapters::StringSchema::new() };
        if let Some(n) = attrs.min_length {
            chain = quote! { #chain.min_length(#n) };
        }
        if let Some(n) = attrs.max_length {
            chain = quote! { #chain.max_length(#n) };
        }
        if attrs.non_empty {
            chain = quote! { #chain.non_empty() };
        }
        if attrs.alphanumeric {
            chain = quote! { #chain.alphanumeric() };
        }
        if attrs.email {
            chain = quote! { #chain.email() };
        }
        if attrs.url {
            chain = quote! { #chain.url() };
        }
        if let Some(ref pat) = attrs.regex {
            chain = quote! { #chain.regex(#pat) };
        }
        if attrs.strict {
            chain = quote! { #chain.strict() };
        }
        if let Some(ref def) = attrs.default {
            chain = quote! { #chain.default(#def) };
        }
        if let Some(ref alias) = attrs.alias {
            chain = quote! { #chain.alias(#alias) };
        }
        if is_option || attrs.optional {
            chain = quote! { #chain.optional() };
        } else {
            chain = quote! { #chain.required() };
        }
        return chain;
    }

    if is_int_type(&name) {
        let mut chain = quote! { ::adapters::IntegerSchema::new() };
        if let Some(n) = attrs.min {
            let n = n as i64;
            chain = quote! { #chain.min(#n) };
        }
        if let Some(n) = attrs.max {
            let n = n as i64;
            chain = quote! { #chain.max(#n) };
        }
        if attrs.positive {
            chain = quote! { #chain.positive() };
        }
        if attrs.negative {
            chain = quote! { #chain.negative() };
        }
        if attrs.non_zero {
            chain = quote! { #chain.non_zero() };
        }
        if attrs.strict {
            chain = quote! { #chain.strict() };
        }
        if let Some(d) = attrs
            .default
            .as_ref()
            .and_then(|def| def.parse::<i64>().ok())
        {
            chain = quote! { #chain.default(#d) };
        }
        if is_option || attrs.optional {
            chain = quote! { #chain.optional() };
        } else {
            chain = quote! { #chain.required() };
        }
        return chain;
    }

    if is_float_type(&name) {
        let mut chain = quote! { ::adapters::FloatSchema::new() };
        if let Some(n) = attrs.min {
            chain = quote! { #chain.min(#n) };
        }
        if let Some(n) = attrs.max {
            chain = quote! { #chain.max(#n) };
        }
        if attrs.positive {
            chain = quote! { #chain.positive() };
        }
        if attrs.negative {
            chain = quote! { #chain.negative() };
        }
        if attrs.non_zero {
            chain = quote! { #chain.non_zero() };
        }
        if attrs.strict {
            chain = quote! { #chain.strict() };
        }
        if let Some(d) = attrs
            .default
            .as_ref()
            .and_then(|def| def.parse::<f64>().ok())
        {
            chain = quote! { #chain.default(#d) };
        }
        if is_option || attrs.optional {
            chain = quote! { #chain.optional() };
        } else {
            chain = quote! { #chain.required() };
        }
        return chain;
    }

    if name == "bool" {
        let mut chain = quote! { ::adapters::BoolSchema::new() };
        if attrs.strict {
            chain = quote! { #chain.strict() };
        }
        if is_option || attrs.optional {
            chain = quote! { #chain.optional() };
        } else {
            chain = quote! { #chain.required() };
        }
        return chain;
    }

    if is_option || attrs.optional {
        quote! { <#ty as ::adapters::SchemaProvider>::schema().optional() }
    } else {
        quote! { <#ty as ::adapters::SchemaProvider>::schema().required() }
    }
}

fn gen_serialize(
    struct_name: &syn::Ident,
    fields: &[(syn::Ident, &Type, FieldAttrs)],
) -> TokenStream2 {
    let entries = fields.iter().map(|(fname, _ftype, attrs)| {
        let key = if let Some(ref alias) = attrs.alias {
            alias.clone()
        } else {
            fname.to_string()
        };
        quote! {
            map.insert(
                #key.to_string(),
                ::adapters::Serialize::serialize(&self.#fname),
            );
        }
    });
    quote! {
        impl ::adapters::Serialize for #struct_name {
            fn serialize(&self) -> ::adapters::Value {
                let mut map = ::std::collections::BTreeMap::new();
                #(#entries)*
                ::adapters::Value::Object(map)
            }
        }
    }
}

fn gen_deserialize(
    struct_name: &syn::Ident,
    fields: &[(syn::Ident, &Type, FieldAttrs)],
) -> TokenStream2 {
    let field_extractions = fields.iter().map(|(fname, ftype, attrs)| {
        let key = if let Some(ref alias) = attrs.alias {
            alias.clone()
        } else {
            fname.to_string()
        };
        let is_opt = extract_option_inner(ftype).is_some();
        let ty = *ftype;

        let default_fallback = if let Some(ref def) = attrs.default {
            let name = type_ident_str(ftype).unwrap_or_default();
            if is_int_type(&name) {
                if let Ok(n) = def.parse::<i64>() {
                    quote! { ::adapters::Value::Int(#n) }
                } else {
                    quote! { ::adapters::Value::Null }
                }
            } else if is_float_type(&name) {
                if let Ok(n) = def.parse::<f64>() {
                    quote! { ::adapters::Value::Float(#n) }
                } else {
                    quote! { ::adapters::Value::Null }
                }
            } else {
                quote! { ::adapters::Value::String(#def.to_string()) }
            }
        } else {
            quote! { ::adapters::Value::Null }
        };

        let has_default = attrs.default.is_some();

        if is_opt {
            quote! {
                let #fname: #ty = {
                    let raw = obj.get(#key).cloned().unwrap_or(::adapters::Value::Null);
                    ::adapters::Deserialize::deserialize(raw)?
                };
            }
        } else if has_default {
            quote! {
                let #fname: #ty = {
                    let raw = obj.get(#key).cloned()
                        .unwrap_or_else(|| #default_fallback);
                    ::adapters::Deserialize::deserialize(raw)?
                };
            }
        } else {
            quote! {
                let #fname: #ty = {
                    let raw = obj.get(#key).cloned().ok_or_else(|| {
                        ::adapters::error::DeserializationError::with_field(
                            format!("missing required field '{}'", #key),
                            #key,
                        )
                    })?;
                    ::adapters::Deserialize::deserialize(raw)?
                };
            }
        }
    });

    let field_names = fields.iter().map(|(fname, _, _)| fname);

    quote! {
        impl ::adapters::Deserialize for #struct_name {
            fn deserialize(value: ::adapters::Value) -> ::std::result::Result<Self, ::adapters::Error> {
                let obj = match value.as_object() {
                    Some(o) => o.clone(),
                    None => return ::std::result::Result::Err(
                        ::adapters::error::DeserializationError::new(
                            format!("expected object, got {}", value.type_name())
                        ).into()
                    ),
                };
                #(#field_extractions)*
                ::std::result::Result::Ok(#struct_name {
                    #(#field_names,)*
                })
            }
        }
    }
}

fn gen_validate(
    struct_name: &syn::Ident,
    fields: &[(syn::Ident, &Type, FieldAttrs)],
) -> TokenStream2 {
    let field_validations = fields.iter().map(|(fname, ftype, attrs)| {
        let fname_str = fname.to_string();
        let value_expr = quote! { ::adapters::Serialize::serialize(&self.#fname) };

        let mut validators: Vec<TokenStream2> = Vec::new();

        let is_opt = extract_option_inner(ftype).is_some();
        let type_name = type_ident_str(ftype).unwrap_or_default();
        let inner_type = extract_option_inner(ftype)
            .and_then(type_ident_str)
            .unwrap_or_else(|| type_name.clone());

        if !is_opt && !attrs.optional {
            validators.push(quote! {
                chain = chain.push_validator(::adapters::validator::RequiredValidator);
            });
        }

        if attrs.strict {
            let expected = if inner_type == "String" {
                "string"
            } else if is_int_type(&inner_type) {
                "int"
            } else if is_float_type(&inner_type) {
                "float"
            } else if inner_type == "bool" {
                "bool"
            } else {
                "object"
            };
            validators.push(quote! {
                chain = chain.push_validator(::adapters::validator::StrictTypeValidator { expected: #expected });
            });
        }

        if let Some(n) = attrs.min_length {
            validators.push(quote! {
                chain = chain.push_validator(::adapters::validator::MinLengthValidator(#n));
            });
        }
        if let Some(n) = attrs.max_length {
            validators.push(quote! {
                chain = chain.push_validator(::adapters::validator::MaxLengthValidator(#n));
            });
        }
        if attrs.email {
            validators.push(quote! {
                chain = chain.push_validator(::adapters::validator::EmailValidator);
            });
        }
        if attrs.url {
            validators.push(quote! {
                chain = chain.push_validator(::adapters::validator::UrlValidator);
            });
        }
        if attrs.non_empty {
            validators.push(quote! {
                chain = chain.push_validator(::adapters::validator::NonEmptyValidator);
            });
        }
        if attrs.alphanumeric {
            validators.push(quote! {
                chain = chain.push_validator(::adapters::validator::AlphanumericValidator);
            });
        }
        if attrs.positive {
            validators.push(quote! {
                chain = chain.push_validator(::adapters::validator::PositiveValidator);
            });
        }
        if attrs.negative {
            validators.push(quote! {
                chain = chain.push_validator(::adapters::validator::NegativeValidator);
            });
        }
        if attrs.non_zero {
            validators.push(quote! {
                chain = chain.push_validator(::adapters::validator::NonZeroValidator);
            });
        }
        if let Some(ref pat) = attrs.regex {
            validators.push(quote! {
                chain = chain.push_validator(::adapters::validator::RegexValidator(#pat.to_string()));
            });
        }

        if let Some(n) = attrs.min {
            if is_float_type(&inner_type) {
                validators.push(quote! {
                    chain = chain.push_validator(::adapters::validator::MinFloatValidator(#n));
                });
            } else {
                let n = n as i64;
                validators.push(quote! {
                    chain = chain.push_validator(::adapters::validator::MinIntValidator(#n));
                });
            }
        }
        if let Some(n) = attrs.max {
            if is_float_type(&inner_type) {
                validators.push(quote! {
                    chain = chain.push_validator(::adapters::validator::MaxFloatValidator(#n));
                });
            } else {
                let n = n as i64;
                validators.push(quote! {
                    chain = chain.push_validator(::adapters::validator::MaxIntValidator(#n));
                });
            }
        }

        let custom_call = if let Some(ref custom_fn) = attrs.custom {
            quote! {
                {
                    let val = #value_expr;
                    #custom_fn(&val, #fname_str)?;
                }
            }
        } else {
            quote! {}
        };

        quote! {
            {
                let val = #value_expr;
                if !(#is_opt && val.is_null()) {
                    let mut chain = ::adapters::validator::ValidatorChain::new();
                    #(#validators)*
                    chain.validate(&val, #fname_str)
                        .map_err(|e| e.errors.into_iter().next()
                            .map(::adapters::Error::Validation)
                            .unwrap_or_else(|| ::adapters::Error::Validation(
                                ::adapters::ValidationError::new(#fname_str, "validation failed", "unknown")
                            ))
                        )?;
                }
                #custom_call
            }
        }
    });

    quote! {
        impl ::adapters::Validate for #struct_name {
            fn validate(&self) -> ::std::result::Result<(), ::adapters::Error> {
                #(#field_validations)*
                ::std::result::Result::Ok(())
            }
        }
    }
}

fn gen_schema_provider(
    struct_name: &syn::Ident,
    fields: &[(syn::Ident, &Type, FieldAttrs)],
) -> TokenStream2 {
    let field_entries = fields.iter().map(|(fname, ftype, attrs)| {
        let fname_str = fname.to_string();
        let key = if let Some(ref alias) = attrs.alias {
            alias.clone()
        } else {
            fname_str.clone()
        };
        let schema_expr = field_schema_expr(ftype, attrs);
        quote! {
            schema = schema.field(#key, #schema_expr);
        }
    });

    quote! {
        impl ::adapters::SchemaProvider for #struct_name {
            fn schema() -> ::adapters::Schema {
                let mut schema = ::adapters::ObjectSchema::new();
                #(#field_entries)*
                ::adapters::Schema::Object(schema)
            }
        }

        impl ::adapters::Adapter for #struct_name {}
    }
}

/// Proc-macro derive entrypoint that configures validation and serialization bindings.
#[proc_macro_derive(Schema, attributes(schema))]
pub fn derive_schema(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let fields_data: Vec<(syn::Ident, &Type, FieldAttrs)> = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => named
                .named
                .iter()
                .map(|f| {
                    let ident = f.ident.clone().expect("named field");
                    let ty = &f.ty;
                    let attrs = parse_field_attrs(&f.attrs);
                    (ident, ty, attrs)
                })
                .collect(),
            _ => {
                return syn::Error::new_spanned(
                    struct_name,
                    "#[derive(Schema)] only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(struct_name, "#[derive(Schema)] only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let serialize_impl = gen_serialize(struct_name, &fields_data);
    let deserialize_impl = gen_deserialize(struct_name, &fields_data);
    let validate_impl = gen_validate(struct_name, &fields_data);
    let schema_provider_impl = gen_schema_provider(struct_name, &fields_data);

    let expanded = quote! {
        #serialize_impl
        #deserialize_impl
        #validate_impl
        #schema_provider_impl
    };

    TokenStream::from(expanded)
}
