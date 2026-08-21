use std::collections::HashMap;

use proc_macro2::Span;
use quote::quote;
use syn::{
    Expr, FieldsNamed, FnArg, Ident, ImplItemFn, ReturnType, Signature, Type, Visibility,
    parse_quote, token::Pub,
};

pub struct NewFnOptions {
    pub vis: Visibility,
    pub name: Ident,
}

impl NewFnOptions {
    pub fn from_parts(vis: Visibility, name: Ident) -> Self {
        Self { vis, name }
    }
}

impl Default for NewFnOptions {
    fn default() -> Self {
        Self::from_parts(
            Visibility::Public(Pub::default()),
            Ident::new("new", Span::call_site()),
        )
    }
}

pub fn generate_new_impl_fn(
    fields: &FieldsNamed,
    auto_inits: &HashMap<Ident, Expr>,
    opts: NewFnOptions,
) -> ImplItemFn {
    let mut inputs: Vec<FnArg> = Vec::new();
    for field in &fields.named {
        let ident = field.ident.as_ref().expect("named field");
        if auto_inits.keys().any(|k| k == ident) {
            continue;
        }
        let ty: &Type = &field.ty;

        let arg: FnArg = parse_quote!(#ident: #ty);
        inputs.push(arg);
    }

    // 2) 初期化リスト
    let mut inits = Vec::new();
    for field in &fields.named {
        let ident = field.ident.as_ref().unwrap();
        if let Some(expr) = auto_inits.get(ident) {
            inits.push(quote!(#ident: #expr));
        } else {
            inits.push(quote!(#ident));
        }
    }

    let vis = opts.vis;
    let fn_name = opts.name;

    let sig: Signature = Signature {
        constness: None,
        asyncness: None,
        unsafety: None,
        abi: None,
        fn_token: Default::default(),
        ident: fn_name,
        generics: Default::default(),
        paren_token: Default::default(),
        inputs: inputs.into_iter().collect(),
        variadic: None,
        output: ReturnType::Type(Default::default(), Box::new(parse_quote!(Self))),
    };

    let block = parse_quote!({
        Self { #(#inits),* }
    });

    ImplItemFn {
        attrs: Vec::new(),
        vis,
        defaultness: None,
        sig,
        block,
    }
}
