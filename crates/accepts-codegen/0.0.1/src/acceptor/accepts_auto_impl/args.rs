use syn::{
    GenericArgument, Generics, ItemImpl, Meta, PathArguments, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
};

use crate::acceptor::common::ast::accepts_trait_ast::AcceptsInfo;

#[derive(Debug, Default, Clone)]
pub struct AsyncAutoImplArgs {
    pub outer_attrs: Punctuated<Meta, Comma>,
}
impl Parse for AsyncAutoImplArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            outer_attrs: Punctuated::<Meta, Comma>::parse_terminated(input)?,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct AutoPinDynAcceptsArgs {
    pub outer_attrs: Punctuated<Meta, Comma>,
}
impl Parse for AutoPinDynAcceptsArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            outer_attrs: Punctuated::<Meta, Comma>::parse_terminated(input)?,
        })
    }
}

pub struct AcceptsImplData {
    pub generics: Generics,
    pub accepts_t_type: Type,
    pub self_ty: Type,
}

impl AcceptsImplData {
    pub fn from_parts(generics: Generics, accepts_t_type: Type, self_ty: Type) -> Self {
        Self {
            generics,
            accepts_t_type,
            self_ty,
        }
    }

    pub fn from_item_impl<A: AcceptsInfo>(accepts: A, item_impl: ItemImpl) -> syn::Result<Self> {
        let trait_path = {
            if let Some((not, path, _)) = item_impl.trait_ {
                if not.is_some() {
                    Err(syn::Error::new(
                        not.span(),
                        "negative impl is not supported",
                    ))
                } else {
                    Ok(path)
                }
            } else {
                Err(syn::Error::new(
                    item_impl.span(),
                    "expected trait implementation",
                ))
            }
        }?;

        let span = trait_path.span();
        let accepts_segment = trait_path
            .segments
            .into_iter()
            .last()
            .ok_or_else(|| syn::Error::new(span, "expected trait path segment"))?;

        if accepts_segment.ident != accepts.accepts_name() {
            return Err(syn::Error::new(
                accepts_segment.span(),
                format!("expected {} trait", accepts.accepts_name()),
            ));
        };

        let accepts_t_type =
            if let PathArguments::AngleBracketed(mut angle) = accepts_segment.arguments {
                if angle.args.len() == 1 {
                    let value = angle.args.pop().unwrap().into_value();
                    if let GenericArgument::Type(accepts_t_type) = value {
                        Ok(accepts_t_type)
                    } else {
                        Err(syn::Error::new(
                            value.span(),
                            "expected type generic argument",
                        ))
                    }
                } else {
                    Err(syn::Error::new(
                        angle.span(),
                        "expected exactly one generic argument",
                    ))
                }
            } else {
                Err(syn::Error::new(
                    accepts_segment.arguments.span(),
                    "expected angled bracketed generic arguments",
                ))
            }?;

        //accepts_segment.arguments

        Ok(AcceptsImplData::from_parts(
            item_impl.generics,
            accepts_t_type,
            *item_impl.self_ty,
        ))
    }

    pub fn parse<A: AcceptsInfo>(accepts: A, input: ParseStream) -> syn::Result<Self> {
        let item_impl: ItemImpl = input.parse()?;

        Self::from_item_impl(accepts, item_impl)
    }
}
