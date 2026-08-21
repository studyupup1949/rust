use proc_macro2::Span;
use syn::{
    Expr, LitStr,
    token::{Else, Eq},
};

pub trait LitStrConstructExt {
    fn from_value(str: &str) -> LitStr;
}

impl LitStrConstructExt for LitStr {
    fn from_value(value: &str) -> LitStr {
        LitStr::new(value, Span::call_site())
    }
}
