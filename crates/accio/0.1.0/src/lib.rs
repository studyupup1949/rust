#![doc = include_str!("../README.md")]

#[doc(hidden)]
mod collect;

use crate::collect::{collect_all, fill_body, Args};
use proc_macro::TokenStream as TS;
use syn::parse_macro_input as pm;

/// Specifies that a block of code will be 'emitted'.
/// Without an `accio!` or an `accio_body!` invocation,
/// `accio_emit!` is no-op.
/// Any tokens passed to `accio_emit` are consumed.
#[proc_macro]
pub fn accio_emit(_: TS) -> TS {
    TS::new()
}

/// Collects the tokens emitted by `accio_emit!`
/// invocations with the same _scope_ (key)
/// and expands to all the tokens collected.
/// The order may not be preserved.
/// Invocations of this macro cannot be nested.
#[proc_macro]
pub fn accio(t: TS) -> TS {
    TS::from(collect_all(pm!(t as Args)))
}

/// Attribute macro variant of `accio`, for expanding
/// into item body blocks (e.g. struct fields, enum variants,
/// array elements etc). Replaces the first empty curly brace
/// (`{}`) or square bracket (`[]`) encountered.
#[proc_macro_attribute]
pub fn accio_body(a: TS, t: TS) -> TS {
    TS::from(fill_body(t, collect_all(pm!(a as Args))))
}
