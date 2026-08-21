use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::ext::IdentExt;
use syn::{
    Block, Data, DeriveInput, Fields, GenericArgument, Ident, Lifetime, Meta, PathSegment, Type,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Comma, PathSep},
};

use crate::{
    acceptor::common::ast::next_acceptors_trait_ast::PartialNextAcceptorsTraitImpl,
    common::{context::CodegenContext, syn::ast::tokens::PathSplitLastArgs},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct NextAcceptorOptions {
    once: bool,
    option_once: bool,
    mut_: bool,
    ref_: bool,
}

impl Parse for NextAcceptorOptions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut opts = Self::default();
        while !input.is_empty() {
            let ident: Ident = input.call(Ident::parse_any)?;
            match &*ident.to_string() {
                "once" => opts.once = true,
                "option_once" => opts.option_once = true,
                "mut" => opts.mut_ = true,
                "ref" => opts.ref_ = true,
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown option `{}`", other),
                    ));
                }
            }
            if input.peek(Comma) {
                let _ = input.parse::<Comma>();
            }
        }
        Ok(opts)
    }
}

fn option_inner_type(ty: &Type) -> Option<Type> {
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    if let Some(first) = args.args.first() {
                        if let GenericArgument::Type(inner) = first {
                            return Some(inner.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn expand(ctx: &CodegenContext, item: TokenStream) -> TokenStream {
    let input: DeriveInput = match parse2(item) {
        Ok(i) => i,
        Err(e) => return e.to_compile_error(),
    };

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => {
                return syn::Error::new(
                    s.struct_token.span(),
                    "NextAcceptors can only be derived for structs with named fields",
                )
                .to_compile_error();
            }
        },
        _ => {
            return syn::Error::new(
                input.ident.span(),
                "NextAcceptors can only be derived for structs",
            )
            .to_compile_error();
        }
    };

    let mut next_fields = Vec::new();
    let mut opts: Option<NextAcceptorOptions> = None;
    for field in fields.iter() {
        for attr in field
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("next_acceptor"))
        {
            let this_opts = match attr.meta.clone() {
                Meta::Path(_) => NextAcceptorOptions::default(),
                Meta::List(list) => match syn::parse2::<NextAcceptorOptions>(list.tokens) {
                    Ok(o) => o,
                    Err(e) => return e.to_compile_error(),
                },
                Meta::NameValue(_) => {
                    return syn::Error::new(attr.span(), "unsupported attribute format")
                        .to_compile_error();
                }
            };
            if let Some(existing) = &opts {
                if existing != &this_opts {
                    return syn::Error::new(attr.span(), "conflicting #[next_acceptor] options")
                        .to_compile_error();
                }
            } else {
                opts = Some(this_opts);
            }
            if let Some(id) = field.ident.clone() {
                next_fields.push((id, field.ty.clone()));
            }
        }
    }

    if next_fields.is_empty() {
        return syn::Error::new(input.ident.span(), "no field with #[next_acceptor] found")
            .to_compile_error();
    }

    let mut options = opts.unwrap_or_default();
    if !options.mut_ && !options.ref_ {
        options.ref_ = true;
    }
    if options.once && options.option_once {
        return syn::Error::new(
            input.ident.span(),
            "conflicting options: once and option_once",
        )
        .to_compile_error();
    }

    let iter_len = next_fields.len();
    if (options.once || options.option_once) && iter_len != 1 {
        return syn::Error::new(
            input.ident.span(),
            "options once/option_once require exactly one #[next_acceptor] field",
        )
        .to_compile_error();
    }

    let mut acceptor_type = next_fields[0].1.clone();
    if options.option_once {
        acceptor_type = match option_inner_type(&acceptor_type) {
            Some(t) => t,
            None => {
                return syn::Error::new(
                    acceptor_type.span(),
                    "field with option_once must be Option<T>",
                )
                .to_compile_error();
            }
        };
    } else if !next_fields.iter().all(|(_, ty)| *ty == acceptor_type) {
        return syn::Error::new(
            next_fields[0].1.span(),
            "all #[next_acceptor] fields must have the same type",
        )
        .to_compile_error();
    }

    let field_idents: Vec<Ident> = next_fields.into_iter().map(|(id, _)| id).collect();
    let iter_lifetime: Lifetime = syn::parse_quote!('a);

    let mut impls = Vec::new();

    if options.ref_ {
        let (iter_type, next_block): (Type, Block) = if options.once {
            let ident = &field_idents[0];
            (
                syn::parse_quote!(core::iter::Once<&'a #acceptor_type>),
                syn::parse_quote!({ core::iter::once(&self.#ident) }),
            )
        } else if options.option_once {
            let ident = &field_idents[0];
            (
                syn::parse_quote!(core::option::Iter<'a, #acceptor_type>),
                syn::parse_quote!({ self.#ident.iter() }),
            )
        } else {
            (
                syn::parse_quote!(core::array::IntoIter<&'a #acceptor_type, #iter_len>),
                syn::parse_quote!({ [#(&self.#field_idents),*].into_iter() }),
            )
        };

        let partial = PartialNextAcceptorsTraitImpl::from_types(
            acceptor_type.clone(),
            iter_lifetime.clone(),
            iter_type,
            false,
            next_block,
        );
        let self_ty_path = PathSplitLastArgs::from_parts(
            None,
            Punctuated::<PathSegment, PathSep>::new(),
            input.ident.clone(),
        );
        impls.push(partial.into_item_impl_from_path(ctx, self_ty_path, input.generics.clone()));
    }

    if options.mut_ {
        let (iter_type, next_block): (Type, Block) = if options.once {
            let ident = &field_idents[0];
            (
                syn::parse_quote!(core::iter::Once<&'a mut #acceptor_type>),
                syn::parse_quote!({ core::iter::once(&mut self.#ident) }),
            )
        } else if options.option_once {
            let ident = &field_idents[0];
            (
                syn::parse_quote!(core::option::IterMut<'a, #acceptor_type>),
                syn::parse_quote!({ self.#ident.iter_mut() }),
            )
        } else {
            (
                syn::parse_quote!(core::array::IntoIter<&'a mut #acceptor_type, #iter_len>),
                syn::parse_quote!({ [#(&mut self.#field_idents),*].into_iter() }),
            )
        };

        let partial = PartialNextAcceptorsTraitImpl::from_types(
            acceptor_type,
            iter_lifetime,
            iter_type,
            true,
            next_block,
        );
        let self_ty_path = PathSplitLastArgs::from_parts(
            None,
            Punctuated::<PathSegment, PathSep>::new(),
            input.ident.clone(),
        );
        impls.push(partial.into_item_impl_from_path(ctx, self_ty_path, input.generics.clone()));
    }

    let mut tokens = TokenStream::new();
    for item in impls {
        tokens.extend(item.into_token_stream());
    }
    tokens
}
