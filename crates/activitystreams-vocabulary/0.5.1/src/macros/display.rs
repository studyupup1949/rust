/// Helper macro to implement the [Display](core::fmt::Display) trait.
#[macro_export]
macro_rules! impl_display {
    ($ty:ident, json) => {
        #[allow(deprecated)]
        impl ::core::fmt::Display for $ty {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::serde_json::to_string(self)
                    .map_err(|_| ::core::fmt::Error)
                    .and_then(|s| write!(f, "{s}"))
            }
        }
    };
    ($ty:ident, str) => {
        #[allow(deprecated)]
        impl ::core::fmt::Display for $ty {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                write!(f, "{}", self.as_str())
            }
        }
    };
}
