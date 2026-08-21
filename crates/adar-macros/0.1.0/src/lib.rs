mod enum_trait_deref;
mod flags;
mod reflect;
mod state_machine;
use enum_trait_deref::*;
use flags::*;
use proc_macro::TokenStream;
use reflect::*;
use state_machine::*;
use syn::{parse::Nothing, parse_macro_input, DeriveInput, TypeTraitObject};

#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn FlagEnum(attr: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(attr as Nothing);
    let input = parse_macro_input!(input as DeriveInput);
    flag_enum_macro_inner(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn ReflectEnum(attr: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(attr as Nothing);
    let input = parse_macro_input!(input as DeriveInput);
    reflect_enum_macro_inner(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn EnumTraitDeref(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as TypeTraitObject);
    let input = parse_macro_input!(input as DeriveInput);
    enum_trait_deref_macro_inner(attr.into(), input, false)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn EnumTraitDerefMut(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as TypeTraitObject);
    let input = parse_macro_input!(input as DeriveInput);
    enum_trait_deref_macro_inner(attr.into(), input, true)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn StateEnum(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as StateMachineArgs);
    let input = parse_macro_input!(input as DeriveInput);
    state_enum_macro_inner(attr, input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
