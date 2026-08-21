use syn::{
    Result, bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Bracket, Comma, Impl},
    Generics, Token, Type};
use crate::{arg::ArgParser, attributes::AttrIdentFilter};
use crate::traits::Mul;

#[allow(unused)]
#[derive(Clone)]
pub struct ImplDotProductArg {
    pub impl_token: Impl,
    pub generics: Generics,
    pub trait_: Mul,
    pub for_: Token![for],
    pub bracket1: Bracket,
    pub lhs_tys: Punctuated<Type, Comma>,
    pub plus: Token![*],
    pub bracket2: Bracket,
    pub rhs_tys: Punctuated<Type, Comma>,
}
impl Parse for ImplDotProductArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let lhs_tys_content;
        let rhs_tys_content;
        let output = ImplDotProductArg {
            impl_token: input.parse()?,
            generics: input.parse()?,
            trait_: input.parse()?,
            for_: input.parse()?,
            bracket1: bracketed!(lhs_tys_content in input),
            lhs_tys: Punctuated::parse_separated_nonempty(&lhs_tys_content)?,
            plus: input.parse()?,
            bracket2: bracketed!(rhs_tys_content in input),
            rhs_tys: Punctuated::parse_separated_nonempty(&rhs_tys_content)?,
        };

        Ok(output)
    }
}

pub struct FullArg<T: Parse> {
    pub mutability: AttrIdentFilter,
    pub arg: T
}
impl<T: Parse> Parse for FullArg<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match AttrIdentFilter::parse_filter(&input, vec!["mut_left", "mut_right", "mut_both"]) {
            Ok(mutability) => Ok(Self{
                mutability,
                arg: input.parse()?,
            }),
            Err(err) => Err(err)
        }
    }
}
impl<T: Parse> ArgParser for FullArg<T> {}

pub struct ImplDotProduct {
    pub args: Punctuated<FullArg<ImplDotProductArg>, Token![;]>
}
impl Parse for ImplDotProduct {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            args: FullArg::parse_as_punctuated::<Token![;]>(input)?
        })
    }
}