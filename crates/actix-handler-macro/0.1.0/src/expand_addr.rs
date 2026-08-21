use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{Path, PathSegment, Token, Type, TypePath};

use crate::expand_method_handlers::HandlerContext;
use crate::options::Options;

pub struct ImplContext {
    pub(crate) type_name: Type,
    pub(crate) handlers: Vec<HandlerContext>,
}

pub fn expand_addr_trait(options: Options, impl_context: ImplContext) -> TokenStream {
    let trait_type_name: Type =
        build_renamed_addr_type(&options.trait_name, &impl_context.type_name);
    let trait_block = expand_trait_declaration(
        &options,
        &impl_context,
        &impl_context.type_name,
        &trait_type_name,
    );
    let impl_block = expand_impl_declaration(
        &options,
        &impl_context,
        &impl_context.type_name,
        trait_type_name,
    );

    TokenStream::from(quote!(
        #trait_block

        #impl_block
    ))
}

fn expand_impl_declaration(
    options: &Options,
    impl_context: &ImplContext,
    type_name: &Type,
    trait_type_name: Type,
) -> TokenStream2 {
    let trait_impls = expand_trait_impls(&options, impl_context, &type_name);
    if options.no_trait_impl {
        quote!()
    } else {
        quote!(
            impl #trait_type_name for ::actix::Addr<#type_name> {
                #trait_impls
            }
        )
    }
}

fn expand_trait_declaration(
    options: &Options,
    impl_context: &ImplContext,
    type_name: &Type,
    trait_type_name: &Type,
) -> TokenStream2 {
    let trait_fns = expand_trait_methods(&options, &impl_context, &type_name);
    if options.no_trait_decl {
        quote!()
    } else {
        quote!(
            trait #trait_type_name {
                #trait_fns
            }
        )
    }
}

fn expand_trait_impls(
    options: &Options,
    impl_context: &ImplContext,
    type_name: &Type,
) -> TokenStream2 {
    impl_context
        .handlers
        .iter()
        .map(|handler_context| expand_addr_method(&options, type_name.clone(), handler_context))
        .fold(TokenStream2::new(), |mut m, i| {
            m.extend(i);
            m
        })
}

fn expand_trait_methods(
    options: &Options,
    impl_context: &ImplContext,
    type_name: &Type,
) -> TokenStream2 {
    impl_context
        .handlers
        .iter()
        .map(|handler_context| {
            let HandlerContext {
                method_name,
                message_type_name,
                ..
            } = handler_context;

            if options.use_recipient {
                quote!(
                    fn #method_name(
                        self: &Self,
                        msg: #message_type_name
                    ) -> ::actix::prelude::RecipientRequest<#message_type_name>;
                )
            } else {
                quote!(
                    fn #method_name(
                        self: &Self,
                        msg: #message_type_name
                    ) -> ::actix::prelude::Request<#type_name, #message_type_name>;
                )
            }
        })
        .fold(TokenStream2::new(), |mut m, i| {
            m.extend(i);
            m
        })
}

fn build_renamed_addr_type(trait_name: &Option<String>, type_name: &Type) -> Type {
    match type_name.clone() {
        Type::Path(path) => {
            let mut segments: Punctuated<PathSegment, Token![::]> = Punctuated::new();
            if let Some(type_segment) = path.path.segments.last() {
                let renamed_segment = PathSegment {
                    ident: trait_name
                        .as_ref()
                        .map(|tn| format_ident!("{}", tn))
                        .unwrap_or_else(|| format_ident!("{}Addr", type_segment.ident)),
                    arguments: type_segment.clone().arguments,
                };
                segments.push_value(renamed_segment);
            }
            Type::Path(TypePath {
                qself: path.qself,
                path: Path {
                    leading_colon: path.path.leading_colon,
                    segments,
                },
            })
        }
        ty => ty,
    }
}

fn expand_addr_method(
    options: &Options,
    type_name: Type,
    handler_context: &HandlerContext,
) -> TokenStream2 {
    let HandlerContext {
        method_name,
        message_type_name,
        ..
    } = handler_context;

    if options.use_recipient {
        quote!(
            fn #method_name(
                self: &Self,
                msg: #message_type_name
            ) -> ::actix::prelude::RecipientRequest<#message_type_name> {
                self.clone().recipient().send(msg)
            }
        )
    } else {
        quote!(
            fn #method_name(
                self: &Self,
                msg: #message_type_name
            ) -> ::actix::prelude::Request<#type_name, #message_type_name> {
                self.send(msg)
            }
        )
    }
}
