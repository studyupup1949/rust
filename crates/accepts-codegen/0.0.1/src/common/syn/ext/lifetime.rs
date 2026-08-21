use proc_macro2::Span;
use syn::{Ident, Lifetime};

pub trait LifetimeConstructExt {
    fn from_ident(ident: Ident) -> Lifetime;
}

impl LifetimeConstructExt for Lifetime {
    fn from_ident(ident: Ident) -> Lifetime {
        Lifetime {
            apostrophe: Span::call_site(),
            ident,
        }
    }
}
