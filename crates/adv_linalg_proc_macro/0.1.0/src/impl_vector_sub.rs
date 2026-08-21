use proc_macro2::Span;
use syn::parse2;
use syn::spanned::Spanned;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Bracket, Comma, Impl},
    Generics, Token, Type, Result, 
    bracketed, Error, LifetimeParam
};
use quote::quote;

use crate::attributes::AttrIdentFilter;
use crate::arg::ArgParser;
use crate::traits::Sub;

#[allow(unused)]
pub struct ImplVectorSubArg {
    pub impl_token: Impl,
    pub generics: Generics,
    pub(crate) contains_lhs: bool,
    pub trait_: Sub,
    pub for_: Token![for],
    pub bracket1: Bracket,
    pub lhs_tys: Punctuated<Type, Comma>,
    pub minus: Token![-],
    pub bracket2: Bracket,
    pub rhs_tys: Punctuated<Type, Comma>,
}
impl Parse for ImplVectorSubArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let lhs_tys_content;
        let rhs_tys_content;
        let mut output = Self {
            impl_token: input.parse()?,
            generics: input.parse()?,
            contains_lhs: false, // this will be checked before the end of this function
            trait_: input.parse()?,
            for_: input.parse()?,
            bracket1: bracketed!(lhs_tys_content in input),
            lhs_tys: Punctuated::parse_separated_nonempty(&lhs_tys_content)?,
            minus: input.parse()?,
            bracket2: bracketed!(rhs_tys_content in input),
            rhs_tys: Punctuated::parse_separated_nonempty(&rhs_tys_content)?,
        };

        output.contains_lhs = output.sanitize_generics()?;

        Ok(output)
    }
}
impl ImplVectorSubArg {
    fn sanitize_generics(&self) -> Result<bool> {
        let generics = self.generics.clone();

        let lifetimes = generics
            .lifetimes()
            .filter(|lifetime| 
                lifetime.lifetime.ident.to_string().as_str() == "rhs"
                || lifetime.lifetime.ident.to_string().as_str() == "lhs"
            ).collect::<Vec<&LifetimeParam>>();

    
        let rhs_check = lifetimes
            .iter()
            .filter_map(|&lifetime| 
                if lifetime.lifetime.ident.to_string().as_str() == "rhs" {
                    Some(lifetime)
                } else {
                    None
                }
            )
            .collect::<Vec<&LifetimeParam>>();

        let lhs_check = lifetimes
            .iter()
            .filter_map(|&lifetime| 
                if lifetime.lifetime.ident.to_string().as_str() == "lhs" {
                    Some(lifetime)
                } else {
                    None
                }
            )
            .collect::<Vec<&LifetimeParam>>();
        
        let mut error = Error::new(Span::call_site(), "Problem with generic arguments for implementation");
        if rhs_check.len() > 1 {
            let span = generics.lt_token.span().join(generics.gt_token.span()).unwrap();
            error.combine(Error::new(
                span,
                "Duplicate definition of required lifetime `'rhs`"
            ));
        }

        if lhs_check.len() > 1 {
            let span = generics.lt_token.span().join(generics.gt_token.span()).unwrap();
            return Err(Error::new(
                span,
                "Duplicate definition of required lifetime `'lhs`"
            ));
        }

        Ok(lhs_check.len() > 0)
    }

    pub fn mut_generics(&self) -> Generics {
        let mut mut_generics = self.generics.clone();
        if !self.contains_lhs {
            mut_generics.params.push(parse2(quote!{'lhs}).unwrap())
        }
        mut_generics
    }
}

pub struct FullArg<T: Parse> {
    pub mutability: AttrIdentFilter,
    pub mut_only: AttrIdentFilter,
    pub arg: T
}
impl<T: Parse> Parse for FullArg<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match AttrIdentFilter::parse_filter(&input, vec![]) {
            Ok(attr_filter) => Ok(Self{
                mutability: attr_filter.refilter(vec!["mut_left", "mut_right"]),
                mut_only: attr_filter.refilter(vec!["mut_only"]),
                arg: input.parse()?,
            }),
            Err(err) => Err(err)
        }
    }
}
impl<T: Parse> ArgParser for FullArg<T> {}

pub struct ImplVectorSub {
    pub args: Punctuated<FullArg<ImplVectorSubArg>, Token![;]>
}
impl Parse for ImplVectorSub {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            args: FullArg::parse_as_punctuated::<Token![;]>(input)?
        })
    }
}