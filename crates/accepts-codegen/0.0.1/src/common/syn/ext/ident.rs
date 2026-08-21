use proc_macro2::{Ident, Span};

pub trait IdentConstructExt {
    fn from_str(str: &str) -> Ident;
    fn with_suffix(base: &Ident, suffix: &str) -> Ident;
    fn with_prefix(prefix: &str, base: &Ident) -> Ident;
}

impl IdentConstructExt for Ident {
    /// Create a new [`Ident`] using [`Span::call_site`].
    fn from_str(str: &str) -> Ident {
        Ident::new(str, Span::call_site())
    }

    /// Create a new [`Ident`] by appending a suffix to an existing one.
    fn with_suffix(base: &Ident, suffix: &str) -> Ident {
        Ident::new(&format!("{}{}", base, suffix), base.span())
    }

    /// Create a new [`Ident`] by prepending a prefix to an existing one.
    fn with_prefix(prefix: &str, base: &Ident) -> Ident {
        Ident::new(&format!("{}{}", prefix, base), base.span())
    }
}
