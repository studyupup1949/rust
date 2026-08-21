use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Expr, Field, Fields, Lit, LitStr, Meta, Token, Type, parse_macro_input,
    punctuated::Punctuated,
};

// ── `#[aam(default = ...)]` handling ─────────────────────────────────────────
//
// Permitted forms:
//   #[aam(default)]            — use `<T as Default>::default()`
//   #[aam(default = "expr")]   — use the Rust expression parsed from the string
//
// When present, the field's "missing from input" branch yields the default
// value instead of a `NotFound` error (required fields) or `None`/`Vec::new()`
// (Option/Vec fields). The "present" branch still parses normally.
enum DefaultSpec {
    /// `#[aam(default)]` — `::<T as ::std::default::Default>::default()`.
    Standard,
    /// `#[aam(default = "src")]` — `src` parsed as a Rust expression.
    Expr(String),
}

fn get_aam_default(field: &Field) -> Option<DefaultSpec> {
    for attr in &field.attrs {
        if !attr.path().is_ident("aam") {
            continue;
        }
        let Meta::List(list) = &attr.meta else {
            continue;
        };
        let parsed: Punctuated<Meta, Token![,]> = list
            .parse_args_with(Punctuated::parse_terminated)
            .unwrap_or_default();
        for meta in &parsed {
            // `#[aam(default)]`
            if let Meta::Path(path) = meta
                && path.is_ident("default")
            {
                return Some(DefaultSpec::Standard);
            }
            // `#[aam(default = "expr")]`
            if let Meta::NameValue(nv) = meta
                && nv.path.is_ident("default")
                && let syn::Expr::Lit(lit) = &nv.value
                && let Lit::Str(s) = &lit.lit
            {
                return Some(DefaultSpec::Expr(s.value()));
            }
        }
    }
    None
}

/// Returns the path prefix for generated code (`::aam_rs` or `::aam_core`).
///
/// This lets the macros work both when invoked through the `aam-rs` facade and
/// when `aam-core` + `aam-derive` are used directly.
fn aam_root() -> proc_macro2::TokenStream {
    let found = proc_macro_crate::crate_name("aam-rs")
        .or_else(|_| proc_macro_crate::crate_name("aam-core"));

    let crate_name = match found {
        Ok(proc_macro_crate::FoundCrate::Name(name)) => name,
        Ok(proc_macro_crate::FoundCrate::Itself) => "aam_core".to_string(),
        Err(_) => "aam_core".to_string(),
    };

    let ident = proc_macro2::Ident::new(&crate_name, Span::call_site());
    quote!(::#ident)
}

/// Derive `FromAam` for a struct, deserializing from an inline-object AAM
/// string (`name = value`, `name = { ... }`, or `name = [ ... ]` separated by
/// newlines or commas).
///
/// # Field attributes
///
/// - `#[aam(rename = "key")]` — read the value from the AAM key `key` instead
///   of the field's Rust name (useful for keys that aren't valid Rust idents,
///   e.g. `source.dir`).
/// - `#[aam(default)]` — if the key is missing from the input, use
///   `<FieldType as Default>::default()` instead of returning a `NotFound`
///   error. Works for required, `Option`, and `Vec` fields.
/// - `#[aam(default = "expr")]` — if the key is missing, evaluate the Rust
///   expression `expr` (parsed from the string) instead of the type's
///   `Default`. Useful for non-`Default` defaults, e.g.
///   `#[aam(default = "42")]` for an `i32` field.
///
/// `Option<T>` fields default to `None` when missing; `Vec<T>` fields default
/// to `Vec::new()` when missing — unless an explicit `#[aam(default)]` /
/// `#[aam(default = "...")]` overrides it.
///
/// # Example
///
/// ```no_run
/// use aam_core::from_aam::FromAam;
/// use aam_derive::FromAam;
///
/// #[derive(FromAam, Default)]
/// struct Cfg {
///     // Required; `NotFound` if `host` is absent.
///     host: String,
///     // Optional; `None` when missing.
///     port: Option<u16>,
///     // Missing → `Default::default()` (i.e. 0).
///     #[aam(default)]
///     retries: u32,
///     // Missing → the literal expression `30`.
///     #[aam(default = "30")]
///     timeout: u32,
///     // Missing → empty `Vec`.
///     tags: Vec<String>,
/// }
///
/// let cfg = Cfg::from_aam_str("host = localhost\nport = 8080").unwrap();
/// assert_eq!(cfg.host, "localhost");
/// assert_eq!(cfg.port, Some(8080));
/// assert_eq!(cfg.retries, 0);
/// assert_eq!(cfg.timeout, 30);
/// assert!(cfg.tags.is_empty());
/// ```
#[proc_macro_derive(FromAam, attributes(aam))]
pub fn derive_from_aam(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(_) => {
                return syn::Error::new_spanned(&input, "FromAam does not support tuple structs")
                    .to_compile_error()
                    .into();
            }
            Fields::Unit => {
                return syn::Error::new_spanned(&input, "FromAam does not support unit structs")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "FromAam only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let field_parsers = fields.iter().map(generate_field_parser);

    let root = aam_root();
    let expanded = quote! {
        impl #root::from_aam::FromAam for #name {
            fn from_aam_str(value: &str) -> ::std::result::Result<Self, #root::error::AamlError> {
                let fields = #root::from_aam::parse_fields(value)?;
                Ok(Self {
                    #(#field_parsers)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_field_parser(field: &Field) -> proc_macro2::TokenStream {
    let field_ident = field.ident.as_ref().expect("named field");
    let ty = &field.ty;
    let root = aam_root();

    let aam_name = get_aam_name(field);
    let is_optional = is_option_type(ty);
    let default_spec = get_aam_default(field);

    let aam_name_lit = LitStr::new(&aam_name, field_ident.span());

    // Expression used when the field IS present in the input. Same for every
    // branch; only the "missing" arm differs based on type / `default`.
    let present_parse = if is_optional {
        let inner_ty = extract_option_inner(ty);
        quote! {
            Some(<#inner_ty as #root::from_aam::FromAam>::from_aam_str(v)?)
        }
    } else if is_vec_type(ty) {
        let inner_ty = extract_vec_inner(ty);
        quote! {
            <::std::vec::Vec<#inner_ty> as #root::from_aam::FromAam>::from_aam_str(v)?
        }
    } else {
        quote! {
            <#ty as #root::from_aam::FromAam>::from_aam_str(v)?
        }
    };

    // Expression used when the field is MISSING from the input.
    let missing_expr = if let Some(spec) = default_spec {
        match spec {
            DefaultSpec::Standard => quote! { ::std::default::Default::default() },
            DefaultSpec::Expr(src) => match syn::parse_str::<Expr>(&src) {
                Ok(expr) => quote! { #expr },
                Err(e) => {
                    let msg = format!("invalid `#[aam(default = ...)]` expression: {e}");
                    return syn::Error::new(field_ident.span(), msg).to_compile_error();
                }
            },
        }
    } else if is_optional {
        quote! { None }
    } else if is_vec_type(ty) {
        quote! { ::std::vec::Vec::new() }
    } else {
        quote! {
            return ::std::result::Result::Err(#root::error::AamlError::NotFound {
                key: #aam_name_lit.to_string(),
                context: "inline object fields".to_string(),
                diagnostics: None,
            })
        }
    };

    quote! {
        #field_ident: match fields.get(#aam_name_lit) {
            Some(v) => #present_parse,
            None => #missing_expr,
        },
    }
}

fn get_aam_name(field: &Field) -> String {
    let raw = field.ident.as_ref().unwrap().to_string();
    for attr in &field.attrs {
        if !attr.path().is_ident("aam") {
            continue;
        }
        if let Meta::List(list) = &attr.meta {
            let parsed: Punctuated<Meta, Token![,]> = list
                .parse_args_with(Punctuated::parse_terminated)
                .unwrap_or_default();
            for meta in &parsed {
                if let Meta::NameValue(nv) = meta
                    && nv.path.is_ident("rename")
                    && let syn::Expr::Lit(lit) = &nv.value
                    && let Lit::Str(s) = &lit.lit
                {
                    return s.value();
                }
            }
        }
    }
    raw.strip_prefix("r#").unwrap_or(&raw).to_string()
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
    {
        return seg.ident == "Option";
    }
    false
}

fn extract_type_inner<'a>(ty: &'a Type, target_ident: &str) -> &'a Type {
    if let Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
        && seg.ident == target_ident
        && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return inner;
    }
    ty
}

fn extract_option_inner(ty: &Type) -> &Type {
    extract_type_inner(ty, "Option")
}

fn is_vec_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(seg) = type_path.path.segments.last()
    {
        return seg.ident == "Vec";
    }
    false
}

fn extract_vec_inner(ty: &Type) -> &Type {
    extract_type_inner(ty, "Vec")
}

fn generate_schema_field_parser(
    field_name: &str,
    type_name: &str,
    optional: bool,
) -> proc_macro2::TokenStream {
    let root = aam_root();
    let field_ident = sanitize_ident(field_name);
    let aam_name_lit = LitStr::new(field_name, field_ident.span());

    // Inner type without the Option wrapper.
    let base_type_name = if optional {
        type_name.strip_suffix('*').unwrap_or(type_name)
    } else {
        type_name
    };

    let is_list = base_type_name.starts_with("list<") && base_type_name.ends_with('>');
    let inner_list_ty = if is_list {
        let inner = base_type_name
            .strip_prefix("list<")
            .and_then(|s| s.strip_suffix('>'))
            .unwrap_or(base_type_name)
            .trim();
        Some(aam_type_to_rust_type(inner, false))
    } else {
        None
    };

    let base_parser = if let Some(inner_ty) = inner_list_ty {
        quote! {
            <::std::vec::Vec<#inner_ty> as #root::from_aam::FromAam>::from_aam_str(v)?
        }
    } else {
        let rust_ty = aam_type_to_rust_type(base_type_name, false);
        quote! { <#rust_ty as #root::from_aam::FromAam>::from_aam_str(v)? }
    };

    let value_parser = if optional {
        quote! {
            match fields.get(#aam_name_lit) {
                Some(v) if !v.trim().is_empty() => Some(#base_parser),
                _ => None,
            }
        }
    } else if is_list {
        quote! {
            match fields.get(#aam_name_lit) {
                Some(v) => <::std::vec::Vec<_> as #root::from_aam::FromAam>::from_aam_str(v)?,
                None => ::std::vec::Vec::new(),
            }
        }
    } else {
        quote! {
            match fields.get(#aam_name_lit) {
                Some(v) => #base_parser,
                None => return ::std::result::Result::Err(#root::error::AamlError::NotFound {
                    key: #aam_name_lit.to_string(),
                    context: "inline object fields".to_string(),
                    diagnostics: None,
                }),
            }
        }
    };

    quote! { #field_ident: #value_parser, }
}

#[proc_macro]
pub fn schema_to_struct(input: TokenStream) -> TokenStream {
    let schema_str = parse_macro_input!(input as LitStr);
    let schema = schema_str.value();

    let parsed = match parse_schema_def(&schema) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into(),
    };

    let struct_name = format_ident!("{}", parsed.name);
    let struct_fields: Vec<_> = parsed
        .fields
        .iter()
        .map(|sf| {
            let field_ident = sanitize_ident(&sf.name);
            let rust_ty = aam_type_to_rust_type(&sf.type_name, sf.optional);
            quote! { pub #field_ident: #rust_ty }
        })
        .collect();

    let root = aam_root();
    let field_parsers: Vec<_> = parsed
        .fields
        .iter()
        .map(|sf| generate_schema_field_parser(&sf.name, &sf.type_name, sf.optional))
        .collect();

    let expanded = quote! {
        #[derive(Debug, Clone)]
        pub struct #struct_name {
            #(#struct_fields,)*
        }

        impl #root::from_aam::FromAam for #struct_name {
            fn from_aam_str(value: &str) -> ::std::result::Result<Self, #root::error::AamlError> {
                let fields = #root::from_aam::parse_fields(value)?;
                Ok(Self {
                    #(#field_parsers)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

fn sanitize_ident(name: &str) -> proc_macro2::Ident {
    if is_rust_keyword(name) {
        format_ident!("r#{}", name)
    } else {
        format_ident!("{}", name)
    }
}

fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "async"
            | "await"
            | "dyn"
    )
}

struct SchemaField {
    name: String,
    type_name: String,
    optional: bool,
}

struct ParsedSchema {
    name: String,
    fields: Vec<SchemaField>,
}

fn strip_schema_prefix(input: &str) -> Result<&str, syn::Error> {
    input
        .strip_prefix("@schema")
        .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "expected '@schema'"))
        .map(str::trim)
}

fn split_schema_name_body(input: &str) -> Result<(&str, &str), syn::Error> {
    input.split_once('{').ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected '{' after schema name",
        )
    })
}

fn extract_schema_body(body_part: &str) -> Result<&str, syn::Error> {
    body_part
        .rsplit_once('}')
        .ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "expected '}' to close schema",
            )
        })
        .map(|(body, _)| body)
}

fn parse_schema_field(
    token: &str,
    next_token: Option<&str>,
) -> Result<(SchemaField, usize), syn::Error> {
    let (field_raw, ty_part) = token.split_once(':').ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("bad field '{token}' — missing ':' separator"),
        )
    })?;

    let (ty, consumed) = if ty_part.is_empty() {
        (
            next_token.ok_or_else(|| {
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!("field '{field_raw}:' has no type specified"),
                )
            })?,
            2,
        )
    } else {
        (ty_part, 1)
    };

    let optional = field_raw.ends_with('*');
    let field_name = if optional {
        field_raw.trim_end_matches('*')
    } else {
        field_raw
    };

    if field_name.is_empty() || ty.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("bad field '{field_name}:{ty}' — empty name or type"),
        ));
    }

    Ok((
        SchemaField {
            name: field_name.to_string(),
            type_name: ty.to_string(),
            optional,
        },
        consumed,
    ))
}

fn parse_schema_def(input: &str) -> Result<ParsedSchema, syn::Error> {
    let input = input.trim();
    let after_prefix = strip_schema_prefix(input)?;
    let (name_part, body_part) = split_schema_name_body(after_prefix)?;
    let name = name_part.trim().to_string();

    if name.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "schema name is empty",
        ));
    }

    let body = extract_schema_body(body_part)?;
    let normalized = body.replace(',', " ");
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    let mut fields = Vec::new();

    let mut i = 0;
    while i < tokens.len() {
        let next = if i + 1 < tokens.len() {
            Some(tokens[i + 1])
        } else {
            None
        };
        let (field, consumed) = parse_schema_field(tokens[i], next)?;
        fields.push(field);
        i += consumed;
    }

    Ok(ParsedSchema { name, fields })
}

fn aam_type_to_rust_type(type_name: &str, optional: bool) -> proc_macro2::TokenStream {
    let base = match type_name {
        "i8" => quote! { i8 },
        "i16" => quote! { i16 },
        "i32" => quote! { i32 },
        "i64" => quote! { i64 },
        "u8" => quote! { u8 },
        "u16" => quote! { u16 },
        "u32" => quote! { u32 },
        "u64" => quote! { u64 },
        "f32" => quote! { f32 },
        "f64" => quote! { f64 },
        "string" => quote! { String },
        "bool" => quote! { bool },
        "color" => quote! { String },
        "math::vector2" | "math::vector3" | "math::vector4" | "math::quaternion"
        | "math::matrix3x3" | "math::matrix4x4" => quote! { String },
        "physics::kilogram" | "physics::meter" => quote! { f64 },
        "time::datetime" | "time::duration" | "time::year" | "time::day" | "time::hour"
        | "time::minute" => quote! { String },
        _ => {
            if let Some(inner) = type_name
                .strip_prefix("list<")
                .and_then(|s| s.strip_suffix('>'))
            {
                let inner_ty = aam_type_to_rust_type(inner.trim(), false);
                return if optional {
                    quote! { ::std::option::Option<::std::vec::Vec<#inner_ty>> }
                } else {
                    quote! { ::std::vec::Vec<#inner_ty> }
                };
            }
            let ident = format_ident!("{}", type_name);
            quote! { #ident }
        }
    };

    if optional {
        let ident = format_ident!("{}", "Option");
        quote! { #ident<#base> }
    } else {
        base
    }
}
