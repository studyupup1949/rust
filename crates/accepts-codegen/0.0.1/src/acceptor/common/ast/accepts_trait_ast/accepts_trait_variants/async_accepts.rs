use core::str::FromStr;

use syn::{
    FnArg, GenericParam, Generics, Ident, Lifetime, LifetimeParam, PredicateType, Receiver,
    ReturnType, Signature, Type, TypeImplTrait, TypeParamBound, WhereClause, WherePredicate,
    punctuated::Punctuated,
    token::{And, Gt, Lt, RArrow},
};

use crate::common::{
    context::CodegenContext,
    syn::ext::{
        GenericsConstructExt, IdentConstructExt, LifetimeConstructExt, PredicateTypeConstructExt,
        PunctuatedConstructExt, ReceiverConstructExt, SignatureConstructExt,
        TypeImplTraitConstructExt, WhereClauseConstructExt,
    },
};

use super::{
    AcceptsBuilder, AcceptsInfo,
    internal::{accept_receiver_ty, accept_value_fn_arg, future_type_param_bound},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AsyncAccepts;

impl AcceptsInfo for AsyncAccepts {
    fn accepts_name(&self) -> &'static str {
        "AsyncAccepts"
    }

    fn accept_fn_name(&self) -> &'static str {
        "accept_async"
    }

    fn accept_lifetime_name(&self) -> Option<&'static str> {
        Some("a")
    }

    fn accept_is_async(&self) -> bool {
        true
    }
}
impl AcceptsBuilder for AsyncAccepts {
    fn build_accept_signature(&self, _: &CodegenContext, accepts_t_type: Type) -> Signature {
        let lifetime = Lifetime::from_ident(Ident::from_str(self.accept_lifetime_name().unwrap()));

        let inputs = {
            let mut inputs = Punctuated::<FnArg, syn::token::Comma>::new();

            inputs.push(FnArg::Receiver(Receiver::from_ref_mut_colon_ty(
                Some((And::default(), Some(lifetime.clone()))),
                None,
                None,
                Box::new(accept_receiver_ty()),
            )));

            inputs.push(accept_value_fn_arg(accepts_t_type.clone()));

            inputs
        };

        let generics = {
            Generics::from_parts(
                Some(Lt::default()),
                Punctuated::from_value(GenericParam::Lifetime(LifetimeParam::new(
                    lifetime.clone(),
                ))),
                Some(Gt::default()),
                Some(WhereClause::from_predicates(Punctuated::from_value(
                    WherePredicate::Type(PredicateType::from_bounds(
                        accepts_t_type,
                        Punctuated::from_value(TypeParamBound::Lifetime(Lifetime::from_ident(
                            Ident::from_str(self.accept_lifetime_name().unwrap()),
                        ))),
                    )),
                ))),
            )
        };

        let output = {
            ReturnType::Type(
                RArrow::default(),
                Box::new(Type::ImplTrait(TypeImplTrait::from_bounds({
                    let mut bounds = Punctuated::new();

                    bounds.push(future_type_param_bound());

                    bounds.push(TypeParamBound::Lifetime(lifetime));

                    bounds
                }))),
            )
        };

        Signature::from_ident_generics_inputs_output(
            Ident::from_str(self.accept_fn_name()),
            generics,
            inputs,
            output,
        )
    }
}

impl FromStr for AsyncAccepts {
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
