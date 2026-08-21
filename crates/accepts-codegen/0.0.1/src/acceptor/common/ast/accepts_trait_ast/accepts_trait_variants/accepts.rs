use core::str::FromStr;

use syn::{
    FnArg, Ident, Receiver, ReturnType, Signature, Type, punctuated::Punctuated, token::And,
};

use crate::common::{
    context::CodegenContext,
    syn::ext::{IdentConstructExt, ReceiverConstructExt, SignatureConstructExt},
};

use super::{
    AcceptsBuilder, AcceptsInfo,
    internal::{accept_receiver_ty, accept_value_fn_arg},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Accepts;

impl AcceptsInfo for Accepts {
    fn accepts_name(&self) -> &'static str {
        "Accepts"
    }

    fn accept_fn_name(&self) -> &'static str {
        "accept"
    }

    fn accept_lifetime_name(&self) -> Option<&'static str> {
        None
    }

    fn accept_is_async(&self) -> bool {
        false
    }
}
impl AcceptsBuilder for Accepts {
    fn build_accept_signature(&self, _: &CodegenContext, accepts_t_type: Type) -> Signature {
        let inputs = {
            let mut inputs = Punctuated::<FnArg, syn::token::Comma>::new();

            inputs.push(FnArg::Receiver(Receiver::from_ref_mut_colon_ty(
                Some((And::default(), None)),
                None,
                None,
                Box::new(accept_receiver_ty()),
            )));

            inputs.push(accept_value_fn_arg(accepts_t_type));

            inputs
        };

        let output = { ReturnType::Default };

        Signature::from_ident_inputs_output(Ident::from_str("accept"), inputs, output)
    }
}

impl FromStr for Accepts {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let this = Self;
        if s == this.accepts_name() {
            Ok(this)
        } else {
            Err(())
        }
    }
}
