use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_quote, FnArg, ImplItemMethod, ReturnType, Type};

pub fn expand_handler_context(ty: &Type, handler_context: &HandlerContext) -> TokenStream {
    let HandlerContext {
        message_type_name,
        method_name,
        result_type,
    } = handler_context;

    TokenStream::from(quote!(
        impl actix::Handler<#message_type_name> for #ty {
            type Result = #result_type;

            fn handle(self: &mut Self, msg: #message_type_name, ctx: &mut actix::Context<Self>) -> Self::Result {
                self.#method_name(msg, ctx)
            }
        }
    ))
}

#[derive(Clone)]
pub struct HandlerContext {
    pub(crate) message_type_name: Type,
    pub(crate) method_name: Ident,
    pub(crate) result_type: Type,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SignatureValidationError {
    WrongArity,
    UnexpectedArguments,
}

pub fn parse_handler_context(
    method: &ImplItemMethod,
) -> Result<HandlerContext, SignatureValidationError> {
    use SignatureValidationError::*;

    let signature = method.sig.clone();

    // Validate arity
    let arity = signature.inputs.len();
    if arity != 3 {
        return Err(WrongArity);
    }

    let method_name = signature.ident.clone();
    let message_arg = signature.inputs[1].clone();

    match message_arg {
        FnArg::Typed(message_arg) => {
            let message_type_name = *message_arg.ty;
            let result_type: Type = match signature.output {
                ReturnType::Default => parse_quote!(<#message_type_name as actix::Message>::Result),
                ReturnType::Type(_, r_type) => (*r_type).clone(),
            };

            Ok(HandlerContext {
                method_name,
                message_type_name,
                result_type,
            })
        }
        _ => Err(UnexpectedArguments),
    }
}

#[cfg(test)]
mod tests {
    use syn::{parse_quote, ImplItemMethod};

    use super::parse_handler_context;
    use crate::expand_method_handlers::SignatureValidationError;

    #[test]
    fn test_parse_handler_context() {
        let input: ImplItemMethod = parse_quote! {
            fn greet(&self, _message: Greeting, _ctx: &Example::Context) {}
        };
        let handler_context = parse_handler_context(&input);
        assert_eq!(handler_context.is_ok(), true);
        let handler_context = handler_context.unwrap_or_else(|_| panic!("Expected HandlerContext"));
        assert_eq!(
            format!("{:?}", handler_context.message_type_name),
            "Path(TypePath { qself: None, path: Path { leading_colon: None, segments: [PathSegment { ident: Ident(Greeting), arguments: None }] } })"
        );
        assert_eq!(format!("{:?}", handler_context.method_name), "Ident(greet)");
    }

    #[test]
    fn test_arity_failure() {
        let input: ImplItemMethod = parse_quote! {
            fn greet(&self, _message: Greeting) {}
        };
        let result = parse_handler_context(&input);
        assert_eq!(result.is_ok(), false);
        let err = result.err().unwrap();
        assert_eq!(err, SignatureValidationError::WrongArity);
    }
}
