use proc_macro::TokenStream;
use syn::{ImplItem, ImplItemMethod, ItemImpl, Type};

use crate::expand_addr::{expand_addr_trait, ImplContext};
use crate::expand_method_handlers::{
    expand_handler_context, parse_handler_context, HandlerContext, SignatureValidationError,
};
use crate::options::Options;
use crate::utils::compilation_error;

pub fn expand_item_impl(options: Options, item_impl: &mut ItemImpl) -> TokenStream {
    let ty = *item_impl.self_ty.clone();

    let handler_contexts: Vec<(
        &ImplItemMethod,
        Result<HandlerContext, SignatureValidationError>,
    )> = parse_method_handlers(item_impl);
    let handlers_output: TokenStream = expand_method_handlers(&ty, &handler_contexts);

    let impl_context = build_impl_context(ty, handler_contexts);
    let addr_output = expand_addr_trait(options, impl_context);

    let mut output = TokenStream::new();
    output.extend(handlers_output);
    output.extend(addr_output);
    output
}

fn build_impl_context(
    ty: Type,
    handler_contexts: Vec<(
        &ImplItemMethod,
        Result<HandlerContext, SignatureValidationError>,
    )>,
) -> ImplContext {
    ImplContext {
        type_name: ty,
        handlers: handler_contexts
            .iter()
            .filter_map(
                |(_method, handler_context_result)| match handler_context_result {
                    Ok(handler_context) => Some(handler_context.clone()),
                    _ => None,
                },
            )
            .collect(),
    }
}

fn expand_method_handlers(
    ty: &Type,
    handler_contexts: &[(
        &ImplItemMethod,
        Result<HandlerContext, SignatureValidationError>,
    )],
) -> TokenStream {
    handler_contexts
        .iter()
        .map(|(method, result)| match result {
            Ok(handler_context) => expand_handler_context(&ty, &handler_context),
            Err(err) => handle_signature_error(&method.sig.ident.to_string(), err.clone()),
        })
        .fold(TokenStream::new(), |mut m, i| {
            m.extend(i);
            m
        })
}

fn parse_method_handlers(
    item_impl: &mut ItemImpl,
) -> Vec<(
    &ImplItemMethod,
    Result<HandlerContext, SignatureValidationError>,
)> {
    item_impl
        .items
        .iter()
        .filter_map(|item| {
            if let ImplItem::Method(method) = item {
                Some(method)
            } else {
                None
            }
        })
        .map(|method| (method, parse_handler_context(method)))
        .collect()
}

fn handle_signature_error(method_name: &str, err: SignatureValidationError) -> TokenStream {
    use SignatureValidationError::*;
    match err {
        WrongArity => compilation_error(format!("Wrong arity for handler {}", method_name)),
        UnexpectedArguments => compilation_error(format!(
            "Unexpected argument types for handler {}",
            method_name
        )),
    }
}
